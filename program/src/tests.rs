use borsh::de::BorshDeserialize;
use borsh::ser::BorshSerialize;
use core::str::FromStr;

use kaigan::types::{RemainderStr, U8PrefixString};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use solana_sdk::{
    account::{Account, WritableAccount},
    pubkey::Pubkey,
};

use solana_record_service_client::{accounts::*, instructions::*, programs::SOLANA_RECORD_SERVICE_ID};

pub const AUTHORITY: Pubkey = Pubkey::new_from_array([0xaa; 32]);
pub const OWNER: Pubkey = Pubkey::new_from_array([0xbb; 32]);
pub const NEW_OWNER: Pubkey = Pubkey::new_from_array([0xcc; 32]);
pub const RANDOM_PUBKEY: Pubkey = Pubkey::new_from_array([0xdd; 32]);

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
]);

/* Helpers */
fn make_u8prefix_string(s: &str) -> U8PrefixString {
    U8PrefixString::try_from_slice(&[&[s.len() as u8], s.as_bytes()].concat())
        .expect("Invalid name")
}

fn make_remainder_str(s: &str) -> RemainderStr {
    RemainderStr::from_str(s).expect("Invalid metadata")
}

fn keyed_account_for_authority() -> (Pubkey, Account) {
    (
        AUTHORITY,
        Account::new(100_000_000u64, 0, &Pubkey::default()),
    )
}

fn keyed_account_for_random_authority() -> (Pubkey, Account) {
    (
        RANDOM_PUBKEY,
        Account::new(100_000_000u64, 0, &Pubkey::default()),
    )
}
fn keyed_account_for_owner() -> (Pubkey, Account) {
    (OWNER, Account::new(100_000_000u64, 0, &Pubkey::default()))
}

fn keyed_account_for_class_default() -> (Pubkey, Account) {
    keyed_account_for_class(AUTHORITY, false, false, "test", "test")
}

fn keyed_account_for_class(
    authority: Pubkey,
    is_permissioned: bool,
    is_frozen: bool,
    name: &str,
    metadata: &str,
) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(
        &[b"class", &authority.as_ref(), name.as_ref()],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let class_account_data = Class {
        discriminator: 1,
        authority,
        is_permissioned,
        is_frozen,
        name: make_u8prefix_string(name),
        metadata: make_remainder_str(metadata),
    }
    .try_to_vec()
    .expect("Invalid class");

    let mut class_account = Account::new(
        100_000_000u64,
        class_account_data.len(),
        &Pubkey::from(crate::ID),
    );
    class_account
        .data_as_mut_slice()
        .clone_from_slice(&class_account_data);
    (address, class_account)
}

fn keyed_account_for_record(
    class: Pubkey,
    owner_type: u8,
    owner: Pubkey,
    is_frozen: bool,
    expiry: i64,
    name: &str,
    data: &str,
) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), name.as_ref()],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let record_account_data = Record {
        discriminator: 2,
        class,
        owner_type,
        owner,
        is_frozen,
        expiry,
        name: make_u8prefix_string(name),
        data: make_remainder_str(data),
    }
    .try_to_vec()
    .expect("Invalid record");

    let mut record_account = Account::new(
        100_000_000u64,
        record_account_data.len(),
        &Pubkey::from(crate::ID),
    );
    record_account
        .data_as_mut_slice()
        .clone_from_slice(&record_account_data);

    (address, record_account)
}

/// ..fixed_mint_data
/// 
/// 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
/// 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
/// 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
/// 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
/// 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
/// 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
/// 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // padding
/// 
/// 1, // account_type
/// 
/// 3, 0, // extension_type_1 -- mint_close_authority_extension
/// 32, 0, // extension_lenght_1
/// 44, 183, 51, 50, 60, 76, 5, 80, 
/// 101, 31, 190, 147, 58, 233, 60, 212, 
/// 133, 19, 33, 142, 101, 42, 77, 206, 
/// 214, 6, 73, 4, 96, 81, 27, 127, // extension_data_2
/// 
/// 12, 0, // extension_type_2 -- permanent_delegate_extension
/// 32, 0, // extension_lenght_2
/// 44, 183, 51, 50, 60, 76, 5, 80, 
/// 101, 31, 190, 147, 58, 233, 60, 212, 
/// 133, 19, 33, 142, 101, 42, 77, 206, 
/// 214, 6, 73, 4, 96, 81, 27, 127, // extension_data_1
/// 
/// 18, 0, // extension_type_3 -- metadata_pointer_extension
/// 64, 0, // extension_lenght_3
/// 
/// 44, 183, 51, 50, 60, 76, 5, 80, 
/// 101, 31, 190, 147, 58, 233, 60, 212, 
/// 133, 19, 33, 142, 101, 42, 77, 206, 
/// 214, 6, 73, 4, 96, 81, 27, 127, // update_authority
/// 
/// 44, 183, 51, 50, 60, 76, 5, 80, 
/// 101, 31, 190, 147, 58, 233, 60, 212, 
/// 133, 19, 33, 142, 101, 42, 77, 206, 
/// 214, 6, 73, 4, 96, 81, 27, 127, // mint
/// 
/// 4, 0, 0, 0, 116, 101, 115, 116, // name
/// 3, 0, 0, 0, 83, 82, 83, // symbol
/// 4, 0, 0, 0, 116, 101, 115, 116, // uri
/// 0, 0, 0, 0 // additional_metadata
fn keyed_account_for_mint(
    record: Pubkey,
    name: &str,
    uri: &str,
) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"mint", &record.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    // Base data (82) + 84 (padding + account_type) + Extensions (36 + 36 + 68) + Metadata (83 + name.len() + uri.len())
    let total_size = 82 + 84 + 36 + 36 + 68 + 87 + name.len() + uri.len();
    let mut mint_account_data = vec![0u8; total_size];

    // Mint authority
    mint_account_data[0..4].copy_from_slice(&[1, 0, 0, 0]);
    mint_account_data[4..36].copy_from_slice(&address.to_bytes());
    // Supply
    mint_account_data[36..44].copy_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]);
    // Decimals
    mint_account_data[44] = 0;
    // Is initialized
    mint_account_data[45] = 1;
    // Freeze authority
    mint_account_data[46..50].copy_from_slice(&[1, 0, 0, 0]);
    mint_account_data[50..82].copy_from_slice(&address.to_bytes());

    // Padding
    mint_account_data[82..165].fill(0);

    // Account type
    mint_account_data[165] = 1;

    // Extension 1 - MintCloseAuthorityExtension
    // Discriminator
    mint_account_data[166..168].copy_from_slice(&[3, 0]);
    // Size
    mint_account_data[168..170].copy_from_slice(&[32, 0]);
    // Mint close authority
    mint_account_data[170..202].copy_from_slice(&address.to_bytes());

    // Extension 2 - PermanentDelegateExtension
    // Discriminator
    mint_account_data[202..204].copy_from_slice(&[12, 0]);
    // Size
    mint_account_data[204..206].copy_from_slice(&[32, 0]);
    // Permanent delegate Authority
    mint_account_data[206..238].copy_from_slice(&address.to_bytes());

    // Extension 3 - MetadataPointerExtension
    // Discriminator
    mint_account_data[238..240].copy_from_slice(&[18, 0]);
    // Size
    mint_account_data[240..242].copy_from_slice(&[64, 0]);
    // Metadata pointer Authority
    mint_account_data[242..274].copy_from_slice(&address.to_bytes());
    // Metadata pointer Address
    mint_account_data[274..306].copy_from_slice(&address.to_bytes());

    // Extension 4 - Metadata
    // Discriminator
    mint_account_data[306..308].copy_from_slice(&[19, 0]);
    // Size (32 + 32 + 4 + name.len() + 4 + 3 + 4 + uri.len() + 4)
    mint_account_data[308..310].copy_from_slice(&((83 + name.len() + uri.len()) as u16).to_le_bytes());
    // Update authority
    mint_account_data[310..342].copy_from_slice(&address.to_bytes());
    // Mint
    mint_account_data[342..374].copy_from_slice(&address.to_bytes());
    // Name length
    mint_account_data[374..378].copy_from_slice(&(name.len() as u32).to_le_bytes());
    // Name
    mint_account_data[378..378 + name.len()].copy_from_slice(name.as_bytes());
    // Symbol length
    mint_account_data[378 + name.len()..378 + name.len() + 4].copy_from_slice(&[3, 0, 0, 0]);
    // Symbol
    mint_account_data[378 + name.len() + 4..378 + name.len() + 7].copy_from_slice(b"SRS");
    // URI length
    mint_account_data[378 + name.len() + 7..378 + name.len() + 11].copy_from_slice(&(uri.len() as u32).to_le_bytes());
    // URI
    mint_account_data[378 + name.len() + 11..378 + name.len() + 11 + uri.len()].copy_from_slice(uri.as_bytes());
    // Additional metadata
    mint_account_data[378 + name.len() + 11 + uri.len()..378 + name.len() + 11 + uri.len() + 4].fill(0);

    // Create the mint account
    let mut record_mint_account = Account::new(100_000_000u64, mint_account_data.len(), &TOKEN_2022_PROGRAM_ID);
    record_mint_account.data_as_mut_slice().copy_from_slice(&mint_account_data);

    (address, record_mint_account)
}

fn keyed_account_for_token(
    owner: Pubkey,
    mint: Pubkey,
    is_frozen: bool,
) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[
            owner.as_ref(),
            mollusk_svm_programs_token::token2022::ID.as_ref(),
            mint.as_ref()
        ], &mollusk_svm_programs_token::associated_token::ID);

    let mut token_account_data = [0u8; 170];
    // Mint
    token_account_data[0..32].copy_from_slice(&mint.to_bytes());
    // Owner
    token_account_data[32..64].copy_from_slice(&owner.to_bytes());
    // Amount
    token_account_data[64..72].copy_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]);
    // Delegate
    token_account_data[72..108].copy_from_slice(&[0; 36]);
    // State
    token_account_data[108] = if is_frozen { 2 } else { 1 };
    // IsNative
    token_account_data[109..121].copy_from_slice(&[0; 12]);
    token_account_data[121..129].copy_from_slice(&[0; 8]);
    token_account_data[129..165].copy_from_slice(&[0; 36]);
    // Account type
    token_account_data[165] = 2;
    // Extension
    token_account_data[166..170].copy_from_slice(&[7, 0, 0, 0]);


    // Create the mint account
    let mut token_account = Account::new(100_000_000u64, token_account_data.len(), &TOKEN_2022_PROGRAM_ID);
    token_account.data_as_mut_slice().copy_from_slice(&token_account_data);

    (address, token_account)
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
        payer: authority,
        class,
        system_program,
    }
    .instruction(CreateClassInstructionArgs {
        is_permissioned: false,
        is_frozen: false,
        name: make_u8prefix_string("test"),
        metadata: make_remainder_str("test"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (class, Account::default()),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&class).data(&class_data.data).build(),
        ],
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
    }
    .instruction(UpdateClassMetadataInstructionArgs {
        metadata: RemainderStr::from_str("test2").unwrap(),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (class, class_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&class)
                .data(&class_data_updated.data)
                .build(),
        ],
    );
}

#[test]
/// Fails because the class_authority != authority of the instruction
fn fail_update_class_metadata() {
    // Authority
    let (random_authority, random_authority_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Class Updated
    let (_, class_data_updated) = keyed_account_for_class(AUTHORITY, false, false, "test", "test2");
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateClassMetadata {
        authority: random_authority,
        class,
        system_program,
    }
    .instruction(UpdateClassMetadataInstructionArgs {
        metadata: RemainderStr::from_str("test2").unwrap(),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (random_authority, random_authority_data),
            (class, class_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&class)
                .data(&class_data_updated.data)
                .build(),
        ],
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

    let instruction = FreezeClass { authority, class }
        .instruction(FreezeClassInstructionArgs { is_frozen: true });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(authority, authority_data), (class, class_data)],
        &[
            Check::success(),
            Check::account(&class).data(&class_data_frozen.data).build(),
        ],
    );
}

#[test]
fn update_class_frozen_already_frozen() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, false, true, "test", "test");
    // Class frozen
    let (_, class_data_frozen) = keyed_account_for_class(authority, false, true, "test", "test");

    let instruction = FreezeClass { authority, class }
        .instruction(FreezeClassInstructionArgs { is_frozen: true });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(authority, authority_data), (class, class_data)],
        &[
            Check::success(),
            Check::account(&class).data(&class_data_frozen.data).build(),
        ],
    );
}

#[test]
fn create_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, owner, false, 0, "test", "test");
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateRecord {
        owner,
        payer: owner,
        class,
        record,
        system_program,
        authority: None,
    }
    .instruction(CreateRecordInstructionArgs {
        expiration: 0,
        name: make_u8prefix_string("test"),
        data: make_remainder_str("test"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (class, class_data),
            (record, Account::default()),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&record).data(&record_data.data).build(),
        ],
    );
}

#[test]
fn create_permissioned_record() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, true, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, owner, false, 0, "test", "test");
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateRecord {
        owner,
        payer: authority,
        class,
        record,
        system_program,
        authority: Some(authority),
    }
    .instruction(CreateRecordInstructionArgs {
        expiration: 0,
        name: make_u8prefix_string("test"),
        data: make_remainder_str("test"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (class, class_data),
            (record, Account::default()),
            (system_program, system_program_data),
            (authority, authority_data),
        ],
        &[
            Check::success(),
            Check::account(&record).data(&record_data.data).build(),
        ],
    );
}

#[test]
fn update_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, owner, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, owner, false, 0, "test", "test2");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority: owner,
        record,
        system_program,
        class: None,
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_str("test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (record, record_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_updated.data)
                .build(),
        ],
    );
}

#[test]
fn update_record_with_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test2");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority,
        record,
        system_program,
        class: Some(class),
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_str("test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (system_program, system_program_data),
            (class, class_data),
        ],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_updated.data)
                .build(),
        ],
    );
}

#[test]
/// Fails because the class is not permissioned
fn fail_update_record_with_delegate_1() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, false, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test2");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority,
        record,
        system_program,
        class: Some(class),
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_str("test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (system_program, system_program_data),
            (class, class_data),
        ],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_updated.data)
                .build(),
        ],
    );
}

#[test]
/// Fails because class authority != from authority of the instruction
fn fail_update_record_with_delegate_2() {
    // Authority
    let (random_authority, random_authority_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, true, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test2");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority: random_authority,
        record,
        system_program,
        class: Some(class),
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_str("test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (random_authority, random_authority_data),
            (record, record_data),
            (system_program, system_program_data),
            (class, class_data),
        ],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_updated.data)
                .build(),
        ],
    );
}

#[test]
fn transfer_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, owner, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, NEW_OWNER, false, 0, "test", "test");

    let instruction = TransferRecord {
        authority: owner,
        record,
        class: None,
    }
    .instruction(TransferRecordInstructionArgs {
        new_owner: Pubkey::new_from_array([0xcc; 32]),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(owner, owner_data), (record, record_data)],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_updated.data)
                .build(),
        ],
    );
}

#[test]
fn transfer_record_with_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, NEW_OWNER, false, 0, "test", "test");

    let instruction = TransferRecord {
        authority,
        record,
        class: Some(class),
    }
    .instruction(TransferRecordInstructionArgs {
        new_owner: Pubkey::new_from_array([0xcc; 32]),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(authority, authority_data), (record, record_data), (class, class_data)],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_updated.data)
                .build(),
        ],
    );
}

#[test]
/// Fails because the record is frozen
fn fail_transfer_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, true, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, NEW_OWNER, true, 0, "test", "test");

    let instruction = TransferRecord {
        authority: owner,
        record,
        class: None,
    }
    .instruction(TransferRecordInstructionArgs {
        new_owner: Pubkey::new_from_array([0xcc; 32]),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(owner, owner_data), (record, record_data)],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_updated.data)
                .build(),
        ],
    );
}

#[test]
fn delete_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");

    let instruction = DeleteRecord {
        authority: owner,
        record,
        class: None,
    }
    .instruction();

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(owner, owner_data), (record, record_data)],
        &[
            Check::success(),
            Check::account(&record).data(&[0xff]).build(),
        ],
    );
}

#[test]
fn delete_record_with_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");

    let instruction = DeleteRecord {
        authority,
        record,
        class: Some(class),
    }
    .instruction();

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(authority, authority_data), (record, record_data), (class, class_data)],
        &[
            Check::success(),
            Check::account(&record).data(&[0xff]).build(),
        ],
    );
}

#[test]
fn freeze_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");
    // Record frozen
    let (_, record_data_frozen) = keyed_account_for_record(class, 0, OWNER, true, 0, "test", "test");

    let instruction = FreezeRecord {
        authority: owner,
        record,
        class: None,
    }
    .instruction(FreezeRecordInstructionArgs { is_frozen: true });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(owner, owner_data), (record, record_data)],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_frozen.data)
                .build(),
        ],
    );
}

#[test]
fn freeze_record_with_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, false, 0, "test", "test");
    // Record frozen
    let (_, record_data_frozen) = keyed_account_for_record(class, 0, OWNER, true, 0, "test", "test");

    let instruction = FreezeRecord {
        authority,
        record,
        class: Some(class),
    }
    .instruction(FreezeRecordInstructionArgs { is_frozen: true });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(authority, authority_data), (record, record_data), (class, class_data)],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_frozen.data)
                .build(),
        ],
    );
}

#[test]
fn freeze_record_already_frozen() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) = keyed_account_for_record(class, 0, OWNER, true, 0, "test", "test");
    // Record frozen
    let (_, record_data_frozen) = keyed_account_for_record(class, 0, OWNER, true, 0, "test", "test");

    let instruction = FreezeRecord {
        authority,
        record,
        class: Some(class),
    }
    .instruction(FreezeRecordInstructionArgs { is_frozen: true });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(authority, authority_data), (record, record_data), (class, class_data)],
        &[
            Check::success(),
            Check::account(&record)
                .data(&record_data_frozen.data)
                .build(),
        ],
    );
}

#[test]
fn mint_record_token() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, owner, false, 0, "test", "test");
    // Mint
    let (mint, mint_data) = keyed_account_for_mint(record, "test", "test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);

    let (associated_token_program, associated_token_program_data) =
        mollusk_svm_programs_token::associated_token::keyed_account();
    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = MintTokenizedRecord {
        owner,
        payer: owner,
        authority: owner,
        record,
        mint,
        token_account,
        associated_token_program,
        token2022,
        system_program,
        class: None,
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (record, record_data),
            (mint, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data.data).build(),
            Check::account(&token_account).data(&token_account_data.data).build(),
        ],
    );
}

#[test]
fn mint_record_token_with_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, owner, false, 0, "test", "test");
    // Mint
    let (mint, mint_data) = keyed_account_for_mint(record, "test", "test");
    // ATA
    let (token_account, _) = Pubkey::find_program_address(
        &[
            owner.as_ref(),
            mollusk_svm_programs_token::token2022::ID.as_ref(),
            mint.as_ref(),
        ],
        &mollusk_svm_programs_token::associated_token::ID,
    );

    let (associated_token_program, associated_token_program_data) =
        mollusk_svm_programs_token::associated_token::keyed_account();
    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = MintTokenizedRecord {
        owner,
        payer: authority,
        authority,
        record,
        mint,
        token_account,
        associated_token_program,
        token2022,
        system_program,
        class: Some(class),
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (authority, authority_data),
            (record, record_data),
            (mint, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
            (class, class_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data.data).build(),
        ],
    );
}

#[test]
fn update_tokenized_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Mint updated
    let (_, mint_data_updated) = keyed_account_for_mint(record_address, "test", "test2");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test2");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = UpdateTokenizedRecord {
        authority: owner,
        record,
        mint,
        token_account,
        token2022,
        class: None,
    }
    .instruction(UpdateTokenizedRecordInstructionArgs {
        new_data: make_remainder_str("test2"),
    });

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (token2022, token2022_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data_updated.data).build(),
            Check::account(&record).data(&record_data_updated.data).build(),
        ],
    );
}

#[test]
fn update_tokenized_record_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Mint updated
    let (_, mint_data_updated) = keyed_account_for_mint(record_address, "test", "test2");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test2");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(OWNER, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = UpdateTokenizedRecord {
        authority,
        record,
        mint,
        token_account,
        token2022,
        class: Some(class),
    }
    .instruction(UpdateTokenizedRecordInstructionArgs {
        new_data: make_remainder_str("test2"),
    });

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (token2022, token2022_data),
            (class, class_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data_updated.data).build(),
            Check::account(&record).data(&record_data_updated.data).build(),
        ],
    );
}

#[test]
fn freeze_tokenized_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);
    // ATA updated
    let (_, token_account_data_updated) = keyed_account_for_token(owner, mint, true);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = FreezeTokenizedRecord {
        authority: owner,
        record,
        mint,
        token_account,
        token2022,
        class: None,
    }
    .instruction(FreezeTokenizedRecordInstructionArgs { is_frozen: true });

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (token2022, token2022_data),
        ],
        &[
            Check::success(),
            Check::account(&token_account).data(&token_account_data_updated.data).build(),
        ],
    );
}

#[test]
fn freeze_tokenized_record_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(OWNER, mint, false);
    // ATA updated
    let (_, token_account_data_updated) = keyed_account_for_token(OWNER, mint, true);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = FreezeTokenizedRecord {
        authority,
        record,
        mint,
        token_account,
        token2022,
        class: Some(class),
    }
    .instruction(FreezeTokenizedRecordInstructionArgs { is_frozen: true });

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (token2022, token2022_data),
            (class, class_data),
        ],
        &[
            Check::success(),
            Check::account(&token_account).data(&token_account_data_updated.data).build(),
        ],
    );
}

#[test]
fn transfer_tokenized_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);
    // New ATA
    let (new_token_account, new_token_account_data) = keyed_account_for_token(RANDOM_PUBKEY, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = TransferTokenizedRecord {
        authority: owner,
        record,
        mint,
        token_account,
        new_token_account,
        token2022,
        class: None,
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (new_token_account, new_token_account_data),
            (token2022, token2022_data),
        ],
        &[
            Check::success(),
        ],
    );
}

#[test]
fn transfer_tokenized_record_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(OWNER, mint, false);
    // New ATA
    let (new_token_account, new_token_account_data) = keyed_account_for_token(RANDOM_PUBKEY, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = TransferTokenizedRecord {
        authority,
        record,
        mint,
        token_account,
        new_token_account,
        token2022,
        class: Some(class),
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (new_token_account, new_token_account_data),
            (token2022, token2022_data),
            (class, class_data),
        ],
        &[
            Check::success(),
        ],
    );
}

#[test]
fn burn_tokenized_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);
    // New ATA
    let (new_token_account, new_token_account_data) = keyed_account_for_token(RANDOM_PUBKEY, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = BurnTokenizedRecord {
        authority: owner,
        record,
        mint,
        token_account,
        token2022,
        class: None,
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (new_token_account, new_token_account_data),
            (token2022, token2022_data),
        ],
        &[
            Check::success(),
        ],
    );
}

#[test]
fn burn_tokenized_record_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, "test", "test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(OWNER, mint, false);
    // New ATA
    let (new_token_account, new_token_account_data) = keyed_account_for_token(RANDOM_PUBKEY, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = BurnTokenizedRecord {
        authority,
        record,
        mint,
        token_account,
        token2022,
        class: Some(class),
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (new_token_account, new_token_account_data),
            (token2022, token2022_data),
            (class, class_data),
        ],
        &[
            Check::success(),
        ],
    );
}