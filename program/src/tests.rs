use core::str::FromStr;
use std::io::Write;
use borsh::de::BorshDeserialize;
use borsh::ser::BorshSerialize;

use kaigan::types::{RemainderStr, U8PrefixString};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use solana_sdk::{
    account::{Account, WritableAccount}, pubkey::Pubkey, signature::Keypair, signer::Signer
};

use solana_record_service_sdk::{accounts::Class, instructions::{CreateClass, CreateClassInstructionArgs, UpdateClassFrozen, UpdateClassFrozenInstructionArgs, UpdateClassMetadata, UpdateClassMetadataInstructionArgs}, programs::SOLANA_RECORD_SERVICE_ID};

#[test]
fn create_class() {
    // Payer keypair
    let keypair = Keypair::new();
    let authority = keypair.pubkey();
    // Vault
    let (class, _bump) = Pubkey::find_program_address(&[b"class", &authority.as_ref(), b"test"], &SOLANA_RECORD_SERVICE_ID);
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateClass {
        authority,
        class,
        system_program,
    }.instruction(CreateClassInstructionArgs { 
        is_permissioned: false, 
        is_frozen: false,
        name: U8PrefixString::try_from_slice(b"\x04test").unwrap(),
        metadata: RemainderStr::from_str("test").unwrap()
    });

    println!("IX: {}", hex::encode(&instruction.data));

    let mollusk = Mollusk::new(&SOLANA_RECORD_SERVICE_ID, "../target/deploy/solana_record_service");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                authority,
                Account::new(100_000_000u64, 0, &Pubkey::default()),
            ),
            (
                class, 
                Account::new(0, 0, &Pubkey::default())
            ),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
        ]
    );
}

#[test]
fn update_class_metadata() {
    // Payer keypair
    let keypair = Keypair::new();
    let authority = keypair.pubkey();
    // Vault
    let (class, _bump) = Pubkey::find_program_address(&[b"class", &authority.as_ref(), b"test"], &SOLANA_RECORD_SERVICE_ID);
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateClassMetadata {
        authority,
        class,
        system_program,
    }.instruction(UpdateClassMetadataInstructionArgs { 
        metadata: RemainderStr::from_str("test2").unwrap()
    });

    println!("IX: {}", hex::encode(&instruction.data));
    
    let class_account_data = Class {
        discriminator: 1,
        authority,
        is_permissioned: false,
        is_frozen: false,
        name: U8PrefixString::try_from_slice(b"\x04test").unwrap(),
        metadata: RemainderStr::from_str("test").unwrap()
    }.try_to_vec().expect("Serialization error");

    println!("Class contents: {}", hex::encode(&class_account_data));

    let mut class_account = Account::new(100_000_000u64,  class_account_data.len(), &Pubkey::from(crate::ID));
    class_account.data_as_mut_slice().write(&class_account_data).expect("Failed to write account data");

    let mollusk = Mollusk::new(&SOLANA_RECORD_SERVICE_ID, "../target/deploy/solana_record_service");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                authority,
                Account::new(100_000_000u64, 0, &Pubkey::default()),
            ),
            (
                class, 
                class_account
            ),
            (system_program, system_program_data),
        ],
        &[
            Check::success()
        ]
    );
}

#[test]
fn update_class_frozen() {
    // Payer keypair
    let keypair = Keypair::new();
    let authority = keypair.pubkey();
    // Vault
    let (class, _bump) = Pubkey::find_program_address(&[b"class", &authority.as_ref(), b"test"], &SOLANA_RECORD_SERVICE_ID);

    let instruction = UpdateClassFrozen {
        authority,
        class,
    }.instruction(UpdateClassFrozenInstructionArgs { 
        is_frozen: true
    });

    println!("IX: {}", hex::encode(&instruction.data));
    
    let class_account_data = Class {
        discriminator: 1,
        authority,
        is_permissioned: false,
        is_frozen: false, 
        name: U8PrefixString::try_from_slice(b"\x04test").unwrap(),
        metadata: RemainderStr::from_str("test").unwrap()
    }.try_to_vec().expect("Serialization error");

    println!("Class contents: {}", hex::encode(&class_account_data));

    let mut class_account = Account::new(100_000_000u64,  class_account_data.len(), &Pubkey::from(crate::ID));
    class_account.data_as_mut_slice().write(&class_account_data).expect("Failed to write account data");

    let mollusk = Mollusk::new(&SOLANA_RECORD_SERVICE_ID, "../target/deploy/solana_record_service");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                authority,
                Account::new(100_000_000u64, 0, &Pubkey::default()),
            ),
            (
                class, 
                class_account
            )
        ],
        &[
            Check::success()
        ]
    );
}