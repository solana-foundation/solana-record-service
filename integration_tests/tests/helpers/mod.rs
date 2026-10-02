#![allow(dead_code)]

use borsh::BorshDeserialize;
use litesvm::LiteSVM;
use solana_address::Address;
use solana_instruction::Instruction;
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_program_pack::Pack;
use solana_record_service_client::{
    accounts::{Class, Record},
    instructions::{
        CreateClassBuilder, CreateRecordBuilder, CreateRecordTokenizableBuilder, FreezeRecordBuilder,
        MintTokenizedRecordBuilder,
    },
    programs::SOLANA_RECORD_SERVICE_ID,
    types::{AdditionalMetadata, Metadata},
};
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;
use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account as create_associated_token_account_ix,
};
use spl_token_2022_interface::{
    extension::StateWithExtensions,
    state::{Account, Mint},
    ID as TOKEN_2022_PROGRAM_ID,
};

pub struct TestContext {
    pub svm: LiteSVM,
    pub payer: Keypair,
}

pub struct Fixture {
    pub ctx: TestContext,
    pub authority: Keypair,
    pub owner: Keypair,
    pub class: Address,
    pub record: Address,
}

/// A LiteSVM instance with the program loaded and a funded payer.
pub fn program_test_context() -> TestContext {
    let mut svm = LiteSVM::new();
    let program_path = std::path::Path::new(&std::env::var("SBF_OUT_DIR").expect("SBF_OUT_DIR")).join("srs.so");
    svm.add_program_from_file(SOLANA_RECORD_SERVICE_ID, program_path).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
    TestContext { svm, payer }
}

pub fn funded_keypair(ctx: &mut TestContext) -> Keypair {
    let keypair = Keypair::new();
    ctx.svm.airdrop(&keypair.pubkey(), 10_000_000_000).unwrap();
    keypair
}

pub fn send(ctx: &mut TestContext, ixs: &[Instruction], signers: &[&Keypair]) -> Result<(), TransactionError> {
    let mut all_signers = vec![&ctx.payer];
    all_signers.extend_from_slice(signers);
    let transaction =
        Transaction::new_signed_with_payer(ixs, Some(&ctx.payer.pubkey()), &all_signers, ctx.svm.latest_blockhash());
    let result = ctx.svm.send_transaction(transaction).map(|_| ()).map_err(|e| e.err);
    ctx.svm.expire_blockhash();
    result
}

pub fn instruction_error(error: InstructionError) -> TransactionError {
    TransactionError::InstructionError(0, error)
}

pub fn mint_pda(record: &Address) -> Address {
    Address::find_program_address(&[b"mint", record.as_ref()], &SOLANA_RECORD_SERVICE_ID).0
}

pub fn group_pda(class: &Address) -> Address {
    Address::find_program_address(&[b"group", class.as_ref()], &SOLANA_RECORD_SERVICE_ID).0
}

pub fn associated_token_address(owner: &Address, mint: &Address) -> Address {
    get_associated_token_address_with_program_id(owner, mint, &TOKEN_2022_PROGRAM_ID)
}

pub fn create_associated_token_account(ctx: &mut TestContext, owner: &Address, mint: &Address) -> Address {
    let ix = create_associated_token_account_ix(&ctx.payer.pubkey(), owner, mint, &TOKEN_2022_PROGRAM_ID);
    send(ctx, &[ix], &[]).unwrap();
    associated_token_address(owner, mint)
}

pub fn create_class(
    ctx: &mut TestContext,
    authority: &Keypair,
    name: &str,
    metadata: &str,
    is_permissioned: bool,
    is_frozen: bool,
) -> Address {
    let class = Class::find_pda(&authority.pubkey(), name.into()).0;
    let ix = CreateClassBuilder::new()
        .authority(authority.pubkey())
        .payer(ctx.payer.pubkey())
        .class(class)
        .is_permissioned(is_permissioned)
        .is_frozen(is_frozen)
        .name(name.into())
        .metadata(metadata.into())
        .instruction();
    send(ctx, &[ix], &[authority]).unwrap();
    class
}

pub fn create_record_ix(
    ctx: &TestContext,
    class: Address,
    owner: &Keypair,
    seed: &[u8],
    data: &[u8],
    authority: Option<&Keypair>,
) -> Instruction {
    CreateRecordBuilder::new()
        .owner(owner.pubkey())
        .payer(ctx.payer.pubkey())
        .class(class)
        .record(Record::find_pda(&class, seed.into()).0)
        .authority(authority.map(|a| a.pubkey()))
        .expiration(0)
        .seed(seed.into())
        .data(data.into())
        .instruction()
}

pub fn create_record(
    ctx: &mut TestContext,
    class: Address,
    owner: &Keypair,
    seed: &[u8],
    data: &[u8],
    authority: Option<&Keypair>,
) -> Result<Address, TransactionError> {
    let ix = create_record_ix(ctx, class, owner, seed, data, authority);
    let mut signers = vec![owner];
    signers.extend(authority);
    send(ctx, &[ix], &signers)?;
    Ok(Record::find_pda(&class, seed.into()).0)
}

pub fn metadata(name: &str, additional: &[(&str, &str)]) -> Metadata {
    Metadata {
        name: name.to_string(),
        symbol: "SRS".to_string(),
        uri: "https://example.com/record.json".to_string(),
        additional_metadata: additional
            .iter()
            .map(|(label, value)| AdditionalMetadata { label: label.to_string(), value: value.to_string() })
            .collect(),
    }
}

pub fn create_tokenizable_record(
    ctx: &mut TestContext,
    class: Address,
    owner: &Keypair,
    seed: &[u8],
    metadata: Metadata,
    authority: Option<&Keypair>,
) -> Address {
    let record = Record::find_pda(&class, seed.into()).0;
    let ix = CreateRecordTokenizableBuilder::new()
        .owner(owner.pubkey())
        .payer(ctx.payer.pubkey())
        .class(class)
        .record(record)
        .authority(authority.map(|a| a.pubkey()))
        .expiration(0)
        .seed(seed.into())
        .metadata(metadata)
        .instruction();
    let mut signers = vec![owner];
    signers.extend(authority);
    send(ctx, &[ix], &signers).unwrap();
    record
}

pub fn mint_tokenized_record_ix(
    ctx: &TestContext,
    class: Address,
    record: Address,
    owner: &Address,
    authority: &Keypair,
) -> Instruction {
    let mint = mint_pda(&record);
    MintTokenizedRecordBuilder::new()
        .owner(*owner)
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .record(record)
        .mint(mint)
        .class(class)
        .group(group_pda(&class))
        .token_account(associated_token_address(owner, &mint))
        .instruction()
}

/// Mints the record token to `owner` and returns the mint and the owner's token account.
pub fn mint_tokenized_record(
    ctx: &mut TestContext,
    class: Address,
    record: Address,
    owner: &Address,
    authority: &Keypair,
) -> Result<(Address, Address), TransactionError> {
    let ix = mint_tokenized_record_ix(ctx, class, record, owner, authority);
    send(ctx, &[ix], &[authority])?;
    let mint = mint_pda(&record);
    Ok((mint, associated_token_address(owner, &mint)))
}

pub fn freeze_record(f: &mut Fixture, signer: &Keypair, is_frozen: bool) -> Result<(), TransactionError> {
    let ix = FreezeRecordBuilder::new()
        .authority(signer.pubkey())
        .record(f.record)
        .class(f.class)
        .is_frozen(is_frozen)
        .instruction();
    send(&mut f.ctx, &[ix], &[signer])
}

pub fn class_account(ctx: &TestContext, class: &Address) -> Class {
    Class::from_bytes(&ctx.svm.get_account(class).unwrap().data).unwrap()
}

pub fn record_account(ctx: &TestContext, record: &Address) -> Record {
    Record::from_bytes(&ctx.svm.get_account(record).unwrap().data).unwrap()
}

pub fn token_account(ctx: &TestContext, token_account: &Address) -> Account {
    Account::unpack(&ctx.svm.get_account(token_account).unwrap().data[..Account::LEN]).unwrap()
}

pub fn mint_supply(ctx: &TestContext, mint: &Address) -> u64 {
    StateWithExtensions::<Mint>::unpack(&ctx.svm.get_account(mint).unwrap().data).unwrap().base.supply
}

pub fn is_closed(ctx: &TestContext, address: &Address) -> bool {
    ctx.svm.get_account(address).is_none_or(|account| account.lamports == 0 && account.data.is_empty())
}

pub fn lamports(ctx: &TestContext, address: &Address) -> u64 {
    ctx.svm.get_account(address).map_or(0, |account| account.lamports)
}

pub fn record_metadata(ctx: &TestContext, record: &Address) -> Metadata {
    Metadata::try_from_slice(&record_account(ctx, record).data).unwrap()
}
