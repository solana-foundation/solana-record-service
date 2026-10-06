use helpers::{
    class_account, create_class, create_record, funded_keypair, instruction_error, lamports, program_test_context,
    send, TestContext,
};
use solana_address::Address;
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_record::{
    accounts::{Class, CLASS_DISCRIMINATOR},
    instructions::{FreezeClassBuilder, UpdateClassAuthorityBuilder, UpdateClassMetadataBuilder},
};
use solana_signer::Signer;

mod helpers;

fn update_metadata(
    ctx: &mut TestContext,
    authority: &Keypair,
    class: Address,
    metadata: &str,
) -> Result<(), solana_transaction_error::TransactionError> {
    let ix = UpdateClassMetadataBuilder::new()
        .authority(authority.pubkey())
        .payer(ctx.payer.pubkey())
        .class(class)
        .metadata(metadata.into())
        .instruction();
    send(ctx, &[ix], &[authority])
}

#[test]
fn create_class_writes_the_class_layout() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);

    let class = create_class(&mut ctx, &authority, "twitter", "handles", true, false);

    assert_eq!(class, Class::find_pda(&authority.pubkey(), "twitter".into()).0);
    let account = class_account(&ctx, &class);
    assert_eq!(account.discriminator, CLASS_DISCRIMINATOR);
    assert_eq!(account.authority, authority.pubkey());
    assert!(account.is_permissioned);
    assert!(!account.is_frozen);
    assert_eq!(&*account.name, "twitter");
    assert_eq!(&*account.metadata, "handles");
    assert_eq!(ctx.svm.get_account(&class).unwrap().data.len(), 1 + 32 + 2 + 1 + "twitter".len() + "handles".len());
}

#[test]
fn create_class_twice_fails() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    create_class(&mut ctx, &authority, "twitter", "", false, false);

    let ix = solana_record::instructions::CreateClassBuilder::new()
        .authority(authority.pubkey())
        .payer(ctx.payer.pubkey())
        .class(Class::find_pda(&authority.pubkey(), "twitter".into()).0)
        .is_permissioned(false)
        .is_frozen(false)
        .name("twitter".into())
        .metadata("again".into())
        .instruction();
    assert!(send(&mut ctx, &[ix], &[&authority]).is_err());
}

#[test]
fn update_class_metadata_resizes_and_refunds() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "twitter", "short", false, false);
    let rent_before = lamports(&ctx, &class);

    update_metadata(&mut ctx, &authority, class, "a much longer piece of class metadata").unwrap();
    assert_eq!(&*class_account(&ctx, &class).metadata, "a much longer piece of class metadata");
    let rent_grown = lamports(&ctx, &class);
    assert!(rent_grown > rent_before);

    let payer_before = lamports(&ctx, &ctx.payer.pubkey());
    update_metadata(&mut ctx, &authority, class, "").unwrap();
    assert_eq!(&*class_account(&ctx, &class).metadata, "");
    assert_eq!(&*class_account(&ctx, &class).name, "twitter");
    let refund = rent_grown - lamports(&ctx, &class);
    assert!(refund > 0);
    assert_eq!(lamports(&ctx, &ctx.payer.pubkey()), payer_before - 2 * 5_000 + refund);
}

#[test]
fn update_class_metadata_rejects_another_authority() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let impostor = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "twitter", "", false, false);

    let err = update_metadata(&mut ctx, &impostor, class, "hijacked").unwrap_err();

    assert_eq!(err, instruction_error(InstructionError::InvalidAccountData));
    assert_eq!(&*class_account(&ctx, &class).metadata, "");
}

#[test]
fn update_class_authority_hands_over_control() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let new_authority = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "twitter", "", false, false);

    let ix = UpdateClassAuthorityBuilder::new()
        .authority(authority.pubkey())
        .payer(ctx.payer.pubkey())
        .class(class)
        .new_authority(new_authority.pubkey())
        .instruction();
    send(&mut ctx, &[ix], &[&authority]).unwrap();

    assert_eq!(class_account(&ctx, &class).authority, new_authority.pubkey());
    assert!(update_metadata(&mut ctx, &authority, class, "old").is_err());
    update_metadata(&mut ctx, &new_authority, class, "new").unwrap();
}

#[test]
fn frozen_class_rejects_new_records() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let owner = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "twitter", "", false, false);

    let freeze = |ctx: &mut TestContext, is_frozen: bool| {
        let ix =
            FreezeClassBuilder::new().authority(authority.pubkey()).class(class).is_frozen(is_frozen).instruction();
        send(ctx, &[ix], &[&authority])
    };

    freeze(&mut ctx, true).unwrap();
    assert!(class_account(&ctx, &class).is_frozen);
    freeze(&mut ctx, true).unwrap();
    assert_eq!(
        create_record(&mut ctx, class, &owner, b"alice", b"data", None).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );

    freeze(&mut ctx, false).unwrap();
    create_record(&mut ctx, class, &owner, b"alice", b"data", None).unwrap();
}

#[test]
fn freeze_class_rejects_another_authority() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let impostor = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "twitter", "", false, false);

    let ix = FreezeClassBuilder::new().authority(impostor.pubkey()).class(class).is_frozen(true).instruction();

    assert_eq!(
        send(&mut ctx, &[ix], &[&impostor]).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
    assert!(!class_account(&ctx, &class).is_frozen);
}

#[test]
fn create_class_rejects_a_flag_byte_other_than_zero_or_one() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let mut ix = solana_record::instructions::CreateClassBuilder::new()
        .authority(authority.pubkey())
        .payer(ctx.payer.pubkey())
        .class(Class::find_pda(&authority.pubkey(), "twitter".into()).0)
        .is_permissioned(true)
        .is_frozen(false)
        .name("twitter".into())
        .metadata("".into())
        .instruction();
    ix.data[1] = 2;

    assert_eq!(
        send(&mut ctx, &[ix], &[&authority]).unwrap_err(),
        instruction_error(InstructionError::InvalidInstructionData)
    );
}

#[test]
fn class_metadata_is_capped_at_255_bytes() {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let ix = solana_record::instructions::CreateClassBuilder::new()
        .authority(authority.pubkey())
        .payer(ctx.payer.pubkey())
        .class(Class::find_pda(&authority.pubkey(), "twitter".into()).0)
        .is_permissioned(false)
        .is_frozen(false)
        .name("twitter".into())
        .metadata("m".repeat(256).as_str().into())
        .instruction();
    assert_eq!(send(&mut ctx, &[ix], &[&authority]).unwrap_err(), instruction_error(InstructionError::InvalidArgument));

    let class = create_class(&mut ctx, &authority, "twitter", &"m".repeat(255), false, false);
    assert_eq!(
        update_metadata(&mut ctx, &authority, class, &"m".repeat(256)).unwrap_err(),
        instruction_error(InstructionError::InvalidInstructionData)
    );
}
