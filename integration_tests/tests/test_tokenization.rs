use helpers::{
    associated_token_address, create_associated_token_account, create_class, create_tokenizable_record, freeze_record,
    funded_keypair, group_pda, instruction_error, is_closed, metadata, mint_pda, mint_supply, mint_tokenized_record,
    program_test_context, record_account, record_metadata, send, token_account, Fixture,
};
use solana_address::Address;
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_record_service_client::{
    instructions::{
        BurnTokenizedRecordBuilder, DeleteRecordBuilder, FreezeTokenizedRecordBuilder, TransferTokenizedRecordBuilder,
        UpdateRecordTokenizableBuilder,
    },
    types::{Metadata, OwnerType},
};
use solana_signer::Signer;
use solana_transaction_error::TransactionError;
use spl_token_2022_interface::{
    extension::{
        group_member_pointer::GroupMemberPointer, group_pointer::GroupPointer, metadata_pointer::MetadataPointer,
        mint_close_authority::MintCloseAuthority, permanent_delegate::PermanentDelegate, BaseStateWithExtensions,
        StateWithExtensions,
    },
    instruction::burn_checked,
    state::{AccountState, Mint},
    ID as TOKEN_2022_PROGRAM_ID,
};
use spl_token_group_interface::state::{TokenGroup, TokenGroupMember};
use spl_token_metadata_interface::state::TokenMetadata;

mod helpers;

fn setup(is_permissioned: bool, metadata: Metadata) -> Fixture {
    let mut ctx = program_test_context();
    let authority = funded_keypair(&mut ctx);
    let owner = funded_keypair(&mut ctx);
    let class = create_class(&mut ctx, &authority, "badges", "", is_permissioned, false);
    let signer = is_permissioned.then_some(&authority);
    let record = create_tokenizable_record(&mut ctx, class, &owner, b"alice", metadata, signer);
    Fixture { ctx, authority, owner, class, record }
}

fn mint_record(f: &mut Fixture, signer: &Keypair) -> Result<(Address, Address), TransactionError> {
    let owner = f.owner.pubkey();
    mint_tokenized_record(&mut f.ctx, f.class, f.record, &owner, signer)
}

fn burn(f: &mut Fixture, signer: &Keypair, holder: &Address, with_class: bool) -> Result<(), TransactionError> {
    let class = with_class.then_some(f.class);
    let mint = mint_pda(&f.record);
    let ix = BurnTokenizedRecordBuilder::new()
        .authority(signer.pubkey())
        .payer(f.ctx.payer.pubkey())
        .mint(mint)
        .token_account(associated_token_address(holder, &mint))
        .record(f.record)
        .class(class)
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

fn transfer(f: &mut Fixture, signer: &Keypair, from: &Address, to: &Address, with_class: bool) -> Result<(), TransactionError> {
    let class = with_class.then_some(f.class);
    let mint = mint_pda(&f.record);
    let ix = TransferTokenizedRecordBuilder::new()
        .authority(signer.pubkey())
        .mint(mint)
        .token_account(associated_token_address(from, &mint))
        .new_token_account(associated_token_address(to, &mint))
        .record(f.record)
        .class(class)
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

fn freeze_token(f: &mut Fixture, signer: &Keypair, holder: &Address, is_frozen: bool) -> Result<(), TransactionError> {
    let mint = mint_pda(&f.record);
    let ix = FreezeTokenizedRecordBuilder::new()
        .authority(signer.pubkey())
        .mint(mint)
        .token_account(associated_token_address(holder, &mint))
        .record(f.record)
        .class(f.class)
        .is_frozen(is_frozen)
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

fn delete(f: &mut Fixture, signer: &Keypair, mint: Address) -> Result<(), TransactionError> {
    let ix = DeleteRecordBuilder::new()
        .authority(signer.pubkey())
        .payer(f.ctx.payer.pubkey())
        .record(f.record)
        .token2022_program(Some(TOKEN_2022_PROGRAM_ID))
        .mint(Some(mint))
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

#[test]
fn mint_tokenized_record_creates_the_group_mint_and_token() {
    let mut f = setup(false, metadata("Alice", &[("handle", "@alice"), ("verified", "true")]));
    let owner = f.owner.insecure_clone();

    let (mint, token) = mint_record(&mut f, &owner).unwrap();

    let record = record_account(&f.ctx, &f.record);
    assert_eq!(record.owner_type, OwnerType::Token);
    assert_eq!(record.owner, mint);

    let token = token_account(&f.ctx, &token);
    assert_eq!(token.mint, mint);
    assert_eq!(token.owner, owner.pubkey());
    assert_eq!(token.amount, 1);
    assert_eq!(token.state, AccountState::Initialized);

    let mint_data = f.ctx.svm.get_account(&mint).unwrap();
    assert_eq!(mint_data.owner, TOKEN_2022_PROGRAM_ID);
    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_data.data).unwrap();
    assert_eq!(mint_state.base.supply, 1);
    assert_eq!(mint_state.base.decimals, 0);
    assert_eq!(Option::<Address>::from(mint_state.base.mint_authority), Some(mint));
    assert_eq!(Option::<Address>::from(mint_state.base.freeze_authority), Some(mint));
    assert_eq!(Option::<Address>::from(mint_state.get_extension::<PermanentDelegate>().unwrap().delegate), Some(mint));
    assert_eq!(
        Option::<Address>::from(mint_state.get_extension::<MintCloseAuthority>().unwrap().close_authority),
        Some(mint)
    );
    assert_eq!(Option::<Address>::from(mint_state.get_extension::<MetadataPointer>().unwrap().metadata_address), Some(mint));
    let member_pointer = mint_state.get_extension::<GroupMemberPointer>().unwrap();
    assert_eq!(Option::<Address>::from(member_pointer.member_address), Some(mint));
    assert_eq!(Option::<Address>::from(member_pointer.authority), Some(group_pda(&f.class)));

    let token_metadata = mint_state.get_variable_len_extension::<TokenMetadata>().unwrap();
    assert_eq!(token_metadata.name, "Alice");
    assert_eq!(token_metadata.symbol, "SRS");
    assert_eq!(token_metadata.uri, "https://example.com/record.json");
    assert_eq!(Option::<Address>::from(token_metadata.update_authority), Some(mint));
    assert_eq!(
        token_metadata.additional_metadata,
        vec![("handle".to_string(), "@alice".to_string()), ("verified".to_string(), "true".to_string())]
    );

    let member = mint_state.get_extension::<TokenGroupMember>().unwrap();
    assert_eq!(member.group, group_pda(&f.class));
    assert_eq!(u64::from(member.member_number), 1);

    let group_data = f.ctx.svm.get_account(&group_pda(&f.class)).unwrap();
    let group_state = StateWithExtensions::<Mint>::unpack(&group_data.data).unwrap();
    assert_eq!(Option::<Address>::from(group_state.get_extension::<GroupPointer>().unwrap().group_address), Some(group_pda(&f.class)));
    let group = group_state.get_extension::<TokenGroup>().unwrap();
    assert_eq!(u64::from(group.size), 1);
    assert_eq!(u64::from(group.max_size), u64::MAX);
}

#[test]
fn second_record_joins_the_existing_group() {
    let mut f = setup(false, metadata("Alice", &[]));
    let owner = f.owner.insecure_clone();
    mint_record(&mut f, &owner).unwrap();

    let bob = funded_keypair(&mut f.ctx);
    let record = create_tokenizable_record(&mut f.ctx, f.class, &bob, b"bob", metadata("Bob", &[]), None);
    let (mint, _) = mint_tokenized_record(&mut f.ctx, f.class, record, &bob.pubkey(), &bob).unwrap();

    let mint_data = f.ctx.svm.get_account(&mint).unwrap();
    let member = *StateWithExtensions::<Mint>::unpack(&mint_data.data).unwrap().get_extension::<TokenGroupMember>().unwrap();
    assert_eq!(u64::from(member.member_number), 2);
}

#[test]
fn class_authority_mints_in_permissioned_class() {
    let mut f = setup(true, metadata("Alice", &[]));
    let authority = f.authority.insecure_clone();

    let (_, token) = mint_record(&mut f, &authority).unwrap();

    assert_eq!(token_account(&f.ctx, &token).owner, f.owner.pubkey());
}

#[test]
fn mint_rejects_a_stranger() {
    let mut f = setup(false, metadata("Alice", &[]));
    let stranger = funded_keypair(&mut f.ctx);

    assert_eq!(mint_record(&mut f, &stranger).unwrap_err(), instruction_error(InstructionError::InvalidAccountData));
}

#[test]
fn minting_a_frozen_record_freezes_the_token() {
    let mut f = setup(false, metadata("Alice", &[]));
    let authority = f.authority.insecure_clone();
    let owner = f.owner.insecure_clone();
    freeze_record(&mut f, &authority, true).unwrap();

    let (_, token) = mint_record(&mut f, &owner).unwrap();

    assert_eq!(token_account(&f.ctx, &token).state, AccountState::Frozen);
}

#[test]
fn class_authority_freezes_and_thaws_the_token() {
    let mut f = setup(false, metadata("Alice", &[]));
    let authority = f.authority.insecure_clone();
    let owner = f.owner.insecure_clone();
    let (_, token) = mint_record(&mut f, &owner).unwrap();

    freeze_token(&mut f, &authority, &owner.pubkey(), true).unwrap();
    freeze_token(&mut f, &authority, &owner.pubkey(), true).unwrap();
    assert_eq!(token_account(&f.ctx, &token).state, AccountState::Frozen);
    let recipient = Address::new_unique();
    create_associated_token_account(&mut f.ctx, &recipient, &mint_pda(&f.record));
    assert!(transfer(&mut f, &owner, &owner.pubkey(), &recipient, false).is_err());

    freeze_token(&mut f, &authority, &owner.pubkey(), false).unwrap();
    assert_eq!(token_account(&f.ctx, &token).state, AccountState::Initialized);
    transfer(&mut f, &owner, &owner.pubkey(), &recipient, false).unwrap();
}

#[test]
fn freeze_tokenized_record_rejects_the_holder() {
    let mut f = setup(false, metadata("Alice", &[]));
    let owner = f.owner.insecure_clone();
    mint_record(&mut f, &owner).unwrap();

    assert_eq!(
        freeze_token(&mut f, &owner, &owner.pubkey(), true).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
}

#[test]
fn holder_transfers_the_token() {
    let mut f = setup(false, metadata("Alice", &[]));
    let owner = f.owner.insecure_clone();
    let (mint, token) = mint_record(&mut f, &owner).unwrap();
    let recipient = Address::new_unique();
    let recipient_token = create_associated_token_account(&mut f.ctx, &recipient, &mint);

    transfer(&mut f, &owner, &owner.pubkey(), &recipient, false).unwrap();

    assert_eq!(token_account(&f.ctx, &token).amount, 0);
    assert_eq!(token_account(&f.ctx, &recipient_token).amount, 1);
    assert_eq!(record_account(&f.ctx, &f.record).owner, mint);
    assert!(transfer(&mut f, &owner, &owner.pubkey(), &recipient, false).is_err());
}

#[test]
fn class_authority_transfers_the_token_only_in_permissioned_class() {
    let mut f = setup(true, metadata("Alice", &[]));
    let authority = f.authority.insecure_clone();
    let (mint, _) = mint_record(&mut f, &authority).unwrap();
    let recipient = Address::new_unique();
    let recipient_token = create_associated_token_account(&mut f.ctx, &recipient, &mint);
    let holder = f.owner.pubkey();
    transfer(&mut f, &authority, &holder, &recipient, true).unwrap();
    assert_eq!(token_account(&f.ctx, &recipient_token).amount, 1);

    let mut f = setup(false, metadata("Alice", &[]));
    let authority = f.authority.insecure_clone();
    let owner = f.owner.insecure_clone();
    let (mint, _) = mint_record(&mut f, &owner).unwrap();
    create_associated_token_account(&mut f.ctx, &recipient, &mint);
    assert_eq!(
        transfer(&mut f, &authority, &owner.pubkey(), &recipient, true).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
}

#[test]
fn burn_hands_the_record_back_to_the_holder() {
    let mut f = setup(false, metadata("Alice", &[]));
    let owner = f.owner.insecure_clone();
    let (mint, token) = mint_record(&mut f, &owner).unwrap();
    let recipient = funded_keypair(&mut f.ctx);
    create_associated_token_account(&mut f.ctx, &recipient.pubkey(), &mint);
    transfer(&mut f, &owner, &owner.pubkey(), &recipient.pubkey(), false).unwrap();

    burn(&mut f, &recipient, &recipient.pubkey(), false).unwrap();

    let record = record_account(&f.ctx, &f.record);
    assert_eq!(record.owner_type, OwnerType::Pubkey);
    assert_eq!(record.owner, recipient.pubkey());
    assert!(!record.is_frozen);
    assert!(is_closed(&f.ctx, &mint));
    assert_eq!(token_account(&f.ctx, &token).amount, 0);
}

#[test]
fn class_authority_burns_in_permissioned_class() {
    let mut f = setup(true, metadata("Alice", &[]));
    let authority = f.authority.insecure_clone();
    mint_record(&mut f, &authority).unwrap();
    let holder = f.owner.pubkey();

    burn(&mut f, &authority, &holder, true).unwrap();

    assert_eq!(record_account(&f.ctx, &f.record).owner, holder);
}

#[test]
fn burn_rejects_a_stranger() {
    let mut f = setup(false, metadata("Alice", &[]));
    let owner = f.owner.insecure_clone();
    let (mint, _) = mint_record(&mut f, &owner).unwrap();
    let stranger = funded_keypair(&mut f.ctx);

    assert_eq!(
        burn(&mut f, &stranger, &owner.pubkey(), true).unwrap_err(),
        instruction_error(InstructionError::InvalidAccountData)
    );
    assert_eq!(mint_supply(&f.ctx, &mint), 1);
}

#[test]
fn burned_record_can_be_updated_and_minted_again() {
    let mut f = setup(true, metadata("Alice", &[]));
    let authority = f.authority.insecure_clone();
    mint_record(&mut f, &authority).unwrap();
    let holder = f.owner.pubkey();
    burn(&mut f, &authority, &holder, true).unwrap();

    let ix = UpdateRecordTokenizableBuilder::new()
        .authority(authority.pubkey())
        .payer(f.ctx.payer.pubkey())
        .record(f.record)
        .class(f.class)
        .metadata(metadata("Alice v2", &[("level", "2")]))
        .instruction();
    send(&mut f.ctx, &[ix], &[&authority]).unwrap();
    assert_eq!(record_metadata(&f.ctx, &f.record), metadata("Alice v2", &[("level", "2")]));

    let (mint, _) = mint_record(&mut f, &authority).unwrap();

    let mint_data = f.ctx.svm.get_account(&mint).unwrap();
    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_data.data).unwrap();
    let token_metadata = mint_state.get_variable_len_extension::<TokenMetadata>().unwrap();
    assert_eq!(token_metadata.name, "Alice v2");
    assert_eq!(token_metadata.additional_metadata, vec![("level".to_string(), "2".to_string())]);
}

#[test]
fn record_with_burned_token_can_be_deleted() {
    let mut f = setup(false, metadata("Alice", &[]));
    let owner = f.owner.insecure_clone();
    let (mint, token) = mint_record(&mut f, &owner).unwrap();
    let burn = burn_checked(&TOKEN_2022_PROGRAM_ID, &token, &mint, &owner.pubkey(), &[], 1, 0).unwrap();
    send(&mut f.ctx, &[burn], &[&owner]).unwrap();
    assert_eq!(mint_supply(&f.ctx, &mint), 0);

    delete(&mut f, &owner, mint).unwrap();

    assert!(is_closed(&f.ctx, &f.record));
    assert!(is_closed(&f.ctx, &mint));
}

#[test]
fn record_with_live_token_cannot_be_deleted() {
    let mut f = setup(false, metadata("Alice", &[]));
    let owner = f.owner.insecure_clone();
    let (mint, _) = mint_record(&mut f, &owner).unwrap();

    assert_eq!(delete(&mut f, &owner, mint).unwrap_err(), instruction_error(InstructionError::InvalidAccountData));
    assert!(!is_closed(&f.ctx, &f.record));
}
