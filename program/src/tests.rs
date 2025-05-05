use core::str::FromStr;
use borsh::de::BorshDeserialize;
use borsh::ser::BorshSerialize;

use kaigan::types::{RemainderStr, U8PrefixString};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use solana_sdk::{account::{Account, ReadableAccount, WritableAccount}, pubkey::Pubkey};

use solana_record_service_sdk::{accounts::{Class, Record}, instructions::{CreateClass, CreateClassInstructionArgs, CreateRecord, CreateRecordInstructionArgs, FreezeClass, FreezeClassInstructionArgs, UpdateClassMetadata, UpdateClassMetadataInstructionArgs}, programs::SOLANA_RECORD_SERVICE_ID};

pub const AUTHORITY: Pubkey = Pubkey::new_from_array([0xaa; 32]);
pub const OWNER: Pubkey = Pubkey::new_from_array([0xbb; 32]);

fn keyed_account_for_authority() -> (Pubkey, Account) {
    (AUTHORITY, Account::new(100_000_000u64,  0, &Pubkey::default()))
}

fn keyed_account_for_class(authority: Pubkey, is_permissioned: bool, is_frozen: bool, name: &str, metadata: &str, ) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(&[b"class", &authority.as_ref(), name.as_ref()], &SOLANA_RECORD_SERVICE_ID);
    let class_account_data = Class {
        discriminator: 1,
        authority,
        is_permissioned,
        is_frozen,
        name: U8PrefixString::try_from_slice(&[
            &[name.len() as u8][..],
            &name.as_bytes()
        ].concat()).expect("Invalid name"),
        metadata: RemainderStr::from_str(metadata).expect("Invalid metadata"),
    }.try_to_vec().expect("Invalid class");

    let mut class_account = Account::new(100_000_000u64,  class_account_data.len(), &Pubkey::from(crate::ID));
    class_account.data_as_mut_slice().clone_from_slice(&class_account_data);
    (address, class_account)
}

fn keyed_account_for_record(class: Pubkey, is_frozen: bool, expiry: i64, name: &str, metadata: &str, ) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(&[b"record", &class.as_ref(), name.as_ref()], &SOLANA_RECORD_SERVICE_ID);
    let record_account_data = Record {
        discriminator: 2,
        class,
        owner: OWNER,
        is_frozen,
        has_authority_extension:  expiry != 0,
        expiry,
        name: U8PrefixString::try_from_slice(&[
            &[name.len() as u8][..],
            &name.as_bytes()
        ].concat()).expect("Invalid name"),
        metadata: RemainderStr::from_str(metadata).expect("Invalid metadata"),
    }.try_to_vec().expect("Invalid record");

    let mut record_account = Account::new(100_000_000u64,  record_account_data.len(), &Pubkey::from(crate::ID));
    record_account.data_as_mut_slice().clone_from_slice(&record_account_data);

    (address, record_account)
}

#[test]
fn create_class() {
    let is_permissioned = false;
    let is_frozen = false;
    let name = "test";
    let metadata = "test";

    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, is_permissioned, is_frozen, name, metadata);

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateClass {
        authority: AUTHORITY,
        class,
        system_program,
    }.instruction(CreateClassInstructionArgs { 
        is_permissioned, 
        is_frozen,
        name: U8PrefixString::try_from_slice(&[
            &[name.len() as u8][..],
            &name.as_bytes()
        ].concat()).expect("Invalid name or name length"),
        metadata: RemainderStr::from_str(metadata).expect("Invalid metadata")
    });

    let mollusk = Mollusk::new(&SOLANA_RECORD_SERVICE_ID, "../target/deploy/solana_record_service");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                authority,
                authority_data
            ),
            (
                class,
                Account::default()
            ),
            (
                system_program, 
                system_program_data
            ),
        ],
        &[
            Check::success(),
            Check::account(&class).data(&class_data.data()).build()
        ]
    );
}

#[test]
fn update_class_metadata() {
    let is_permissioned = false;
    let is_frozen = false;
    let name = "test";
    let metadata = "test";
    
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, is_permissioned, is_frozen, name, metadata);
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateClassMetadata {
        authority,
        class,
        system_program,
    }.instruction(UpdateClassMetadataInstructionArgs { 
        metadata: RemainderStr::from_str("test2").unwrap()
    });
    
    let mollusk = Mollusk::new(&SOLANA_RECORD_SERVICE_ID, "../target/deploy/solana_record_service");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                authority,
                authority_data,
            ),
            (
                class, 
                class_data,
            ),
            (
                system_program, 
                system_program_data
            ),
        ],
        &[
            Check::success()
        ]
    );
}

#[test]
fn update_class_frozen() {
    let is_permissioned = false;
    let is_frozen = false;
    let name = "test";
    let metadata = "test";
    
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, is_permissioned, is_frozen, name, metadata);

    let instruction = FreezeClass {
        authority,
        class,
    }.instruction(FreezeClassInstructionArgs { 
        is_frozen: true
    });

    let mollusk = Mollusk::new(&SOLANA_RECORD_SERVICE_ID, "../target/deploy/solana_record_service");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                authority,
                authority_data,
            ),
            (
                class, 
                class_data
            )
        ],
        &[
            Check::success()
        ]
    );
}

#[test]
fn create_record() {
    let is_permissioned = false;
    let is_frozen = false;
    let name = "test";
    let metadata = "test";
    
    // Owner
    let (owner, owner_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, is_permissioned, is_frozen, name, metadata);
    // Record
    let (record, _bump) = keyed_account_for_record(class, false, 0, "test", "test");
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateRecord {
        owner,
        class,
        record,
        system_program,
    }.instruction(CreateRecordInstructionArgs { 
        expiration: 0,
        name: U8PrefixString::try_from_slice(b"\x04test").unwrap(),
        data: RemainderStr::from_str("test").unwrap()
    });

    let mollusk = Mollusk::new(&SOLANA_RECORD_SERVICE_ID, "../target/deploy/solana_record_service");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                owner,
                owner_data,
            ),
            (
                class,
                class_data,
            ),
            (
                record, 
                Account::default()
            ),
            (
                system_program,
                system_program_data
            )
            
        ],
        &[
            Check::success()
        ]
    );
}