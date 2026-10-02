use helpers::{
    create_class, create_record, create_record_ix, freeze_record, funded_keypair, instruction_error, is_closed,
    lamports, program_test_context, record_account, send, Fixture,
};
use solana_address::Address;
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_record_service_client::{
    accounts::{Record, RECORD_DISCRIMINATOR},
    instructions::{DeleteRecordBuilder, TransferRecordBuilder, UpdateRecordBuilder, UpdateRecordExpiryBuilder},
    types::OwnerType,
};
use solana_signer::Signer;
use solana_transaction_error::TransactionError;

mod helpers;

fn setup(is_permissioned: bool) -> Fixture {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let owner = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "twitter", "", is_permissioned, false);
    let signer = is_permissioned.then_some(&authority);
    let record = create_record(&mut ctx, class, &owner, b"alice", b"@alice", signer).unwrap();
    Fixture { ctx, authority, owner, class, record }
}

fn update_data(f: &mut Fixture, signer: &Keypair, data: &[u8]) -> Result<(), TransactionError> {
    let ix = UpdateRecordBuilder::new()
        .authority(signer.pubkey())
        .payer(f.ctx.payer.pubkey())
        .record(f.record)
        .class(f.class)
        .data(data.into())
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

fn transfer(f: &mut Fixture, signer: &Keypair, with_class: bool, new_owner: Address) -> Result<(), TransactionError> {
    let class = with_class.then_some(f.class);
    let ix = TransferRecordBuilder::new()
        .authority(signer.pubkey())
        .record(f.record)
        .class(class)
        .new_owner(new_owner)
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

fn delete(f: &mut Fixture, signer: &Keypair, with_class: bool) -> Result<(), TransactionError> {
    let class = with_class.then_some(f.class);
    let ix = DeleteRecordBuilder::new()
        .authority(signer.pubkey())
        .payer(f.ctx.payer.pubkey())
        .record(f.record)
        .class(class)
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

#[test]
fn create_record_writes_the_record_layout() {
    let f = setup(false);

    assert_eq!(f.record, Record::find_pda(&f.class, b"alice".into()).0);
    let record = record_account(&f.ctx, &f.record);
    assert_eq!(record.discriminator, RECORD_DISCRIMINATOR);
    assert_eq!(record.class, f.class);
    assert_eq!(record.owner_type, OwnerType::Pubkey);
    assert_eq!(record.owner, f.owner.pubkey());
    assert!(!record.is_frozen);
    assert_eq!(record.expiry, 0);
    assert_eq!(&*record.seed, b"alice");
    assert_eq!(&*record.data, b"@alice");
}

#[test]
fn permissioned_class_requires_the_class_authority_to_create() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let owner = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "twitter", "", true, false);

    assert_eq!(
        create_record(&mut ctx, class, &owner, b"alice", b"@alice", None).unwrap_err(),
        instruction_error(InstructionError::MissingRequiredSignature)
    );
    let impostor = funded_keypair(&mut ctx);
    assert_eq!(
        create_record(&mut ctx, class, &owner, b"alice", b"@alice", Some(&impostor)).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );

    create_record(&mut ctx, class, &owner, b"alice", b"@alice", Some(&authority)).unwrap();
}

#[test]
fn create_record_twice_fails() {
    let mut f = setup(false);
    let owner = f.owner.insecure_clone();
    let ix = create_record_ix(&f.ctx, f.class, &owner, b"alice", b"again", None);

    assert!(send(&mut f.ctx, &[ix], &[&owner]).is_err());
}

#[test]
fn class_authority_updates_record_data() {
    let mut f = setup(true);
    let authority = f.authority.insecure_clone();

    update_data(&mut f, &authority, b"a considerably longer handle").unwrap();
    assert_eq!(&*record_account(&f.ctx, &f.record).data, b"a considerably longer handle");

    update_data(&mut f, &authority, b"").unwrap();
    assert_eq!(&*record_account(&f.ctx, &f.record).data, b"");
    assert_eq!(&*record_account(&f.ctx, &f.record).seed, b"alice");
}

#[test]
fn update_record_rejects_anyone_but_the_class_authority() {
    let mut f = setup(false);
    let owner = f.owner.insecure_clone();

    assert_eq!(
        update_data(&mut f, &owner, b"@mallory").unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
    assert_eq!(&*record_account(&f.ctx, &f.record).data, b"@alice");
}

#[test]
fn update_record_rejects_a_record_from_another_class() {
    let mut f = setup(false);
    let authority = f.authority.insecure_clone();
    let other_class = create_class(&mut f.ctx, &authority, "github", "", false, false);
    let ix = UpdateRecordBuilder::new()
        .authority(authority.pubkey())
        .payer(f.ctx.payer.pubkey())
        .record(f.record)
        .class(other_class)
        .data(b"moved".into())
        .instruction();

    assert_eq!(
        send(&mut f.ctx, &[ix], &[&authority]).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
}

#[test]
fn update_record_expiry_is_blocked_while_frozen() {
    let mut f = setup(false);
    let authority = f.authority.insecure_clone();
    let set_expiry = |f: &mut Fixture, expiry: i64| {
        let ix = UpdateRecordExpiryBuilder::new()
            .authority(authority.pubkey())
            .payer(f.ctx.payer.pubkey())
            .record(f.record)
            .class(f.class)
            .expiry(expiry)
            .instruction();
        send(&mut f.ctx, &[ix], &[&authority])
    };

    set_expiry(&mut f, 1_900_000_000).unwrap();
    assert_eq!(record_account(&f.ctx, &f.record).expiry, 1_900_000_000);

    freeze_record(&mut f, &authority, true).unwrap();
    assert_eq!(set_expiry(&mut f, 0).unwrap_err(), instruction_error(InstructionError::InvalidAccountData));
    assert_eq!(record_account(&f.ctx, &f.record).expiry, 1_900_000_000);
}

#[test]
fn owner_transfers_record() {
    let mut f = setup(false);
    let owner = f.owner.insecure_clone();
    let new_owner = Address::new_unique();

    transfer(&mut f, &owner, false, new_owner).unwrap();

    assert_eq!(record_account(&f.ctx, &f.record).owner, new_owner);
    assert!(transfer(&mut f, &owner, false, owner.pubkey()).is_err());
}

#[test]
fn class_authority_transfers_record_only_in_permissioned_class() {
    let mut f = setup(true);
    let authority = f.authority.insecure_clone();
    let new_owner = Address::new_unique();
    transfer(&mut f, &authority, true, new_owner).unwrap();
    assert_eq!(record_account(&f.ctx, &f.record).owner, new_owner);

    let mut f = setup(false);
    let authority = f.authority.insecure_clone();
    assert_eq!(
        transfer(&mut f, &authority, true, new_owner).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
}

#[test]
fn frozen_record_cannot_be_transferred() {
    let mut f = setup(false);
    let authority = f.authority.insecure_clone();
    let owner = f.owner.insecure_clone();

    freeze_record(&mut f, &authority, true).unwrap();
    freeze_record(&mut f, &authority, true).unwrap();
    assert!(record_account(&f.ctx, &f.record).is_frozen);
    assert_eq!(
        transfer(&mut f, &owner, false, Address::new_unique()).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );

    freeze_record(&mut f, &authority, false).unwrap();
    transfer(&mut f, &owner, false, Address::new_unique()).unwrap();
}

#[test]
fn freeze_record_rejects_the_owner() {
    let mut f = setup(false);
    let owner = f.owner.insecure_clone();

    assert_eq!(
        freeze_record(&mut f, &owner, true).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
}

#[test]
fn owner_deletes_record_and_payer_gets_the_rent() {
    let mut f = setup(false);
    let owner = f.owner.insecure_clone();
    let rent = lamports(&f.ctx, &f.record);
    let payer_before = lamports(&f.ctx, &f.ctx.payer.pubkey());

    delete(&mut f, &owner, false).unwrap();

    assert!(is_closed(&f.ctx, &f.record));
    assert_eq!(lamports(&f.ctx, &f.ctx.payer.pubkey()), payer_before - 2 * 5_000 + rent);
    create_record(&mut f.ctx, f.class, &owner, b"alice", b"@alice-again", None).unwrap();
    assert_eq!(&*record_account(&f.ctx, &f.record).data, b"@alice-again");
}

#[test]
fn class_authority_deletes_record_in_permissioned_class() {
    let mut f = setup(true);
    let authority = f.authority.insecure_clone();

    delete(&mut f, &authority, true).unwrap();

    assert!(is_closed(&f.ctx, &f.record));
}

#[test]
fn delete_record_rejects_a_stranger() {
    let mut f = setup(false);
    let stranger = funded_keypair(&mut f.ctx);

    assert_eq!(delete(&mut f, &stranger, true).unwrap_err(), instruction_error(InstructionError::InvalidAccountData));
    assert!(!is_closed(&f.ctx, &f.record));
}
