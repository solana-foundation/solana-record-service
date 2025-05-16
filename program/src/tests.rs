use borsh::de::BorshDeserialize;
use borsh::ser::BorshSerialize;
use core::str::FromStr;

use kaigan::types::{RemainderStr, U8PrefixString};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use solana_sdk::{
    account::{Account, WritableAccount},
    pubkey::Pubkey,
};

use solana_record_service_sdk::{accounts::*, instructions::*, programs::SOLANA_RECORD_SERVICE_ID};

pub const AUTHORITY: Pubkey = Pubkey::new_from_array([0xaa; 32]);
pub const OWNER: Pubkey = Pubkey::new_from_array([0xbb; 32]);
pub const NEW_OWNER: Pubkey = Pubkey::new_from_array([0xcc; 32]);
pub const RANDOM_PUBKEY: Pubkey = Pubkey::new_from_array([0xdd; 32]);

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

fn keyed_account_for_record_mint(record: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"mint", &record.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    let record_mint_account = Account::new(100_000_000u64, 0, &Pubkey::from(crate::ID));
    (address, record_mint_account)
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
    let (mint, _mint_data) = keyed_account_for_record_mint(record);
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

    let instruction = MintRecordToken {
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
            // Check::account(&delegate).data(&[]).build(),
        ],
    );
}
