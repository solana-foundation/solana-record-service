use core::str::FromStr;
use borsh::de::BorshDeserialize;
use borsh::ser::BorshSerialize;

use kaigan::types::{RemainderStr, U8PrefixString};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use solana_program::example_mocks::solana_keypair::Keypair;
use solana_sdk::{account::{Account, WritableAccount}, pubkey::Pubkey};

use solana_record_service_sdk::{accounts::{Class, Record}, instructions::{CreateClass, CreateClassInstructionArgs, CreateRecord, CreateRecordInstructionArgs, FreezeClass, FreezeClassInstructionArgs, TransferRecord, TransferRecordInstructionArgs, UpdateClassMetadata, UpdateClassMetadataInstructionArgs, UpdateRecord, UpdateRecordInstructionArgs}, programs::SOLANA_RECORD_SERVICE_ID};

pub const AUTHORITY: Pubkey = Pubkey::new_from_array([0xaa; 32]);
pub const OWNER: Pubkey = Pubkey::new_from_array([0xbb; 32]);
pub const NEW_OWNER: Pubkey = Pubkey::new_from_array([0xcc; 32]);

/* Helpers */
fn make_u8prefix_string(s: &str) -> U8PrefixString {
    U8PrefixString::try_from_slice(&[&[s.len() as u8], s.as_bytes()].concat()).expect("Invalid name")
}

fn make_remainder_str(s: &str) -> RemainderStr {
    RemainderStr::from_str(s).expect("Invalid metadata")
}

fn keyed_account_for_authority() -> (Pubkey, Account) {
    (AUTHORITY, Account::new(100_000_000u64,  0, &Pubkey::default()))
}

fn keyed_account_for_owner() -> (Pubkey, Account) {
    (OWNER, Account::new(100_000_000u64,  0, &Pubkey::default()))
}

fn keyed_account_for_class_default() -> (Pubkey, Account) {
    keyed_account_for_class(AUTHORITY, false, false, "test", "test")
}

fn keyed_account_for_class(authority: Pubkey, is_permissioned: bool, is_frozen: bool, name: &str, metadata: &str, ) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(&[b"class", &authority.as_ref(), name.as_ref()], &SOLANA_RECORD_SERVICE_ID);
    let class_account_data = Class {
        discriminator: 1,
        authority,
        is_permissioned,
        is_frozen,
        name: make_u8prefix_string(name),
        metadata: make_remainder_str(metadata),
    }.try_to_vec().expect("Invalid class");

    let mut class_account = Account::new(100_000_000u64,  class_account_data.len(), &Pubkey::from(crate::ID));
    class_account.data_as_mut_slice().clone_from_slice(&class_account_data);
    (address, class_account)
}

fn keyed_account_for_record(class: Pubkey, owner: Pubkey, is_frozen: bool, expiry: i64, name: &str, data: &str, ) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(&[b"record", &class.as_ref(), name.as_ref()], &SOLANA_RECORD_SERVICE_ID);
    let record_account_data = Record {
        discriminator: 2,
        class,
        owner,
        is_frozen,
        has_authority_extension: expiry > 0,
        expiry,
        name: make_u8prefix_string(name),
        data: make_remainder_str(data),
    }.try_to_vec().expect("Invalid record");

    let mut record_account = Account::new(100_000_000u64,  record_account_data.len(), &Pubkey::from(crate::ID));
    record_account.data_as_mut_slice().clone_from_slice(&record_account_data);

    (address, record_account)
}

/* Tests */

#[test]
fn create_class() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateClass {
        authority,
        class,
        system_program,
    }.instruction(CreateClassInstructionArgs { 
        is_permissioned: false, 
        is_frozen: false,
        name: make_u8prefix_string("test"),
        metadata: make_remainder_str("test")
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
            Check::account(&class).data(&class_data.data).build()
        ]
    );
}

#[test]
fn update_class_metadata() {

    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    
    // Class Updated
    let (_, class_data_updated) = keyed_account_for_class(authority, false, false, "test", "test2");
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
            Check::success(),
            Check::account(&class).data(&class_data_updated.data).build()
        ]
    );
}

#[test]
fn update_class_frozen() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Class frozen
    let (_, class_data_frozen) = keyed_account_for_class(authority, false, true, "test", "test");
    
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
            Check::success(),
            Check::account(&class).data(&class_data_frozen.data).build()
        ]
    );
}

#[test]
fn create_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, owner, false, 0, "test", "test");
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateRecord {
        owner,
        class,
        record,
        system_program,
        authority: None
    }.instruction(CreateRecordInstructionArgs { 
        expiration: 0,
        name: make_u8prefix_string("test"),
        data: make_remainder_str("test")
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
            Check::success(),
            Check::account(&record).data(&record_data.data).build(),
        ]
    );
}

#[test]
fn update_record() {
    // Owner
    let (authority, authority_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, OWNER, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) = keyed_account_for_record(class, OWNER, false, 0, "test", "test2");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority,
        record,
        system_program,
        delegate: None
    }.instruction(UpdateRecordInstructionArgs { 
        data: make_remainder_str("test2")
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
                record, 
                record_data
            ),
            (
                system_program,
                system_program_data
            )
            
        ],
        &[
            Check::success(),
            Check::account(&record).data(&record_data_updated.data).build()
        ]
    );
}

#[test]
fn transfer_record() {
    // Owner
    let (authority, authority_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, OWNER, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) = keyed_account_for_record(class, NEW_OWNER, false, 0, "test", "test");

    let instruction = TransferRecord {
        authority,
        record,
        delegate: None
    }.instruction(TransferRecordInstructionArgs { 
        new_owner: Pubkey::new_from_array([0xcc; 32])
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
                record, 
                record_data
            )
        ],
        &[
            Check::success(),
            Check::account(&record).data(&record_data_updated.data).build()
        ]
    );
}