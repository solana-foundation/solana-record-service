use borsh::de::BorshDeserialize;
use borsh::ser::BorshSerialize;
use core::str::FromStr;
use solana_account::{Account, WritableAccount};
use solana_program::program_error::ProgramError;

use kaigan::types::{RemainderStr, RemainderVec, U8PrefixString, U8PrefixVec};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use solana_pubkey::Pubkey;

use solana_record_service_client::{
    accounts::*,
    instructions::*,
    programs::SOLANA_RECORD_SERVICE_ID,
    types::{Metadata, MetadataAdditionalMetadata},
};

pub const AUTHORITY: Pubkey = Pubkey::new_from_array([0xaa; 32]);
pub const OWNER: Pubkey = Pubkey::new_from_array([0xbb; 32]);
pub const NEW_OWNER: Pubkey = Pubkey::new_from_array([0xcc; 32]);
pub const RANDOM_PUBKEY: Pubkey = Pubkey::new_from_array([0xdd; 32]);

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
]);

// Custom U32PrefixString type following kaigan pattern
#[derive(Clone, Eq, PartialEq)]
struct U32PrefixString(String);

impl std::ops::Deref for U32PrefixString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for U32PrefixString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl U32PrefixString {
    fn try_from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() < 4 {
            return Err("Slice too short");
        }
        let len = u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]) as usize;
        if slice.len() < 4 + len {
            return Err("Slice too short for string");
        }
        let string_data = &slice[4..4 + len];
        let string = String::from_utf8(string_data.to_vec()).map_err(|_| "Invalid utf8")?;
        Ok(Self(string))
    }
}

impl From<U32PrefixString> for String {
    fn from(u32_str: U32PrefixString) -> Self {
        u32_str.0
    }
}

/* Helpers */
fn make_u8prefix_string(s: &str) -> U8PrefixString {
    U8PrefixString::try_from_slice(&[&[s.len() as u8], s.as_bytes()].concat())
        .expect("Invalid name")
}

fn make_u8prefix_vec_u8(s: &[u8]) -> U8PrefixVec<u8> {
    U8PrefixVec::try_from_slice(&[&[s.len() as u8], s].concat()).expect("Invalid seed")
}

fn make_u32prefix_string(s: &str) -> String {
    let len = s.len() as u32;
    let len_bytes = len.to_le_bytes();
    let mut data = Vec::new();
    data.extend_from_slice(&len_bytes);
    data.extend_from_slice(s.as_bytes());
    U32PrefixString::try_from_slice(&data)
        .expect("Invalid name")
        .into()
}

fn make_remainder_vec(b: &[u8]) -> RemainderVec<u8> {
    RemainderVec::<u8>::try_from_slice(b).expect("Invalid slice")
}

fn make_remainder_str(s: &str) -> RemainderStr {
    RemainderStr::from_str(s).expect("Invalid metadata")
}

fn keyed_account_for_authority() -> (Pubkey, Account) {
    (
        AUTHORITY,
        Account::new(100_000_000_000u64, 0, &Pubkey::default()),
    )
}

fn keyed_account_for_random_authority() -> (Pubkey, Account) {
    (
        RANDOM_PUBKEY,
        Account::new(100_000_000_000u64, 0, &Pubkey::default()),
    )
}
fn keyed_account_for_owner() -> (Pubkey, Account) {
    (
        OWNER,
        Account::new(100_000_000_000u64, 0, &Pubkey::default()),
    )
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
    seed: &[u8],
    data: &[u8],
) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), seed.as_ref()],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let record_account_data = Record {
        discriminator: 2,
        class,
        owner_type,
        owner,
        is_frozen,
        expiry,
        seed: make_u8prefix_vec_u8(seed),
        data: RemainderVec::<u8>::try_from_slice(data).unwrap(),
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

/// Fake Metadata that has
/// - name: "test"
/// - symbol: "SRS"
/// - uri: "test"
/// - additional_metadata: []
const METADATA: &[u8; 27] = &[
    4, 0, 0, 0, 116, 101, 115, 116, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 0, 0,
    0, 0,
];

fn keyed_account_for_record_with_metadata(
    class: Pubkey,
    owner_type: u8,
    owner: Pubkey,
    is_frozen: bool,
    expiry: i64,
    name: &str,
    metadata: Option<&[u8]>,
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
        seed: make_u8prefix_vec_u8(name.as_bytes()),
        data: RemainderVec::<u8>::try_from_slice(metadata.unwrap_or(METADATA)).unwrap(),
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

/// Fake Metadata that has
/// - name: "test"
/// - symbol: "SRS"
/// - uri: "test"
/// - additional_metadata: [
///     { label: "test", value: "test" }
/// ]
const METADATA_WITH_ADDITIONAL_METADATA: &[u8] = &[
    4, 0, 0, 0, 116, 101, 115, 116, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 1, 0,
    0, 0, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 116, 101, 115, 116,
];

fn keyed_account_for_record_with_metadata_and_additional_metadata(
    class: Pubkey,
    owner_type: u8,
    owner: Pubkey,
    is_frozen: bool,
    expiry: i64,
    name: &str,
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
        seed: make_u8prefix_vec_u8(name.as_bytes()),
        data: RemainderVec::<u8>::try_from_slice(METADATA_WITH_ADDITIONAL_METADATA).unwrap(),
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

const METADATA_WITH_MULTIPLE_ADDITIONAL_METADATA: &[u8] = &[
    4, 0, 0, 0, 116, 101, 115, 116, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 3, 0,
    0, 0, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 115, 101,
    115, 116, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 114, 101, 115, 116, 4, 0, 0, 0, 116, 101,
    115, 116,
];

fn keyed_account_for_record_with_metadata_and_multiple_additional_metadata(
    class: Pubkey,
    owner_type: u8,
    owner: Pubkey,
    is_frozen: bool,
    expiry: i64,
    name: &str,
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
        seed: make_u8prefix_vec_u8(name.as_bytes()),
        data: RemainderVec::<u8>::try_from_slice(METADATA_WITH_MULTIPLE_ADDITIONAL_METADATA)
            .unwrap(),
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

const MINT_DATA_WITH_EXTENSIONS: &[u8] = &[
    1, 0, 0, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
    0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33, 142, 101,
    42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 1,
];
const MINT_CLOSE_AUTHORITY_EXTENSION: &[u8] = &[
    3, 0, 32, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127,
];
const MINT_PERMANENT_DELEGATE_EXTENSION: &[u8] = &[
    12, 0, 32, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127,
];
const MINT_METADATA_POINTER_EXTENSION: &[u8] = &[
    18, 0, 64, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31,
    190, 147, 58, 233, 60, 212, 133, 19, 33, 142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127,
];
const MINT_GROUP_MEMBER_POINTER_EXTENSION: &[u8] = &[
    22, 0, 64, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87,
    188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 44, 183, 51, 50, 60, 76,
    5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33, 142, 101, 42, 77, 206, 214, 6, 73, 4,
    96, 81, 27, 127,
];
const MINT_METADATA_EXTENSION: &[u8] = &[
    19, 0, 91, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31,
    190, 147, 58, 233, 60, 212, 133, 19, 33, 142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127,
    4, 0, 0, 0, 116, 101, 115, 116, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 0, 0,
    0, 0,
];
const MINT_GROUP_MEMBER_EXTENSION: &[u8] = &[
    23, 0, 72, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 52, 137, 177, 136, 59, 205, 145, 103,
    193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152,
    136, 141, 87, 92, 1, 0, 0, 0, 0, 0, 0, 0,
];

fn keyed_account_for_mint(record: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"mint", &record.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    // Base data (82) + 84 (padding + account_type) + Extensions (36 + 36 + 68) + Metadata (83 + name.len() + uri.len())
    let total_size = MINT_DATA_WITH_EXTENSIONS.len()
        + MINT_CLOSE_AUTHORITY_EXTENSION.len()
        + MINT_PERMANENT_DELEGATE_EXTENSION.len()
        + MINT_METADATA_POINTER_EXTENSION.len()
        + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()
        + MINT_METADATA_EXTENSION.len()
        + MINT_GROUP_MEMBER_EXTENSION.len();

    let mut mint_account_data = vec![0u8; total_size];

    // Mint Data
    mint_account_data[0..MINT_DATA_WITH_EXTENSIONS.len()]
        .copy_from_slice(MINT_DATA_WITH_EXTENSIONS);
    let mut offset = MINT_DATA_WITH_EXTENSIONS.len();
    // Close Authority Extension
    mint_account_data[offset..offset + MINT_CLOSE_AUTHORITY_EXTENSION.len()]
        .copy_from_slice(MINT_CLOSE_AUTHORITY_EXTENSION);
    offset += MINT_CLOSE_AUTHORITY_EXTENSION.len();
    // Permanent Delegate Extension
    mint_account_data[offset..offset + MINT_PERMANENT_DELEGATE_EXTENSION.len()]
        .copy_from_slice(MINT_PERMANENT_DELEGATE_EXTENSION);
    offset += MINT_PERMANENT_DELEGATE_EXTENSION.len();
    // Metadata Pointer Extension
    mint_account_data[offset..offset + MINT_METADATA_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_METADATA_POINTER_EXTENSION);
    offset += MINT_METADATA_POINTER_EXTENSION.len();
    // Group Pointer Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_POINTER_EXTENSION);
    offset += MINT_GROUP_MEMBER_POINTER_EXTENSION.len();
    // Metadata Extension
    mint_account_data[offset..offset + MINT_METADATA_EXTENSION.len()]
        .copy_from_slice(MINT_METADATA_EXTENSION);
    offset += MINT_METADATA_EXTENSION.len();
    // Group Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_EXTENSION);

    // Create the mint account
    let mut record_mint_account = Account::new(
        100_000_000u64,
        mint_account_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    record_mint_account
        .data_as_mut_slice()
        .copy_from_slice(&mint_account_data);

    (address, record_mint_account)
}

const MINT_METADATA_EXTENSION_UPDATED: &[u8] = &[
    19, 0, 92, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31,
    190, 147, 58, 233, 60, 212, 133, 19, 33, 142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127,
    5, 0, 0, 0, 116, 101, 115, 116, 50, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 0, 0,
    0, 0,
];
const MINT_GROUP_MEMBER_EXTENSION_UPDATED: &[u8] = &[
    23, 0, 72, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 52, 137, 177, 136, 59, 205, 145, 103,
    193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152,
    136, 141, 87, 92, 2, 0, 0, 0, 0, 0, 0, 0,
];

fn keyed_account_for_updated_mint(record: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"mint", &record.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    // Base data (82) + 84 (padding + account_type) + Extensions (36 + 36 + 68) + Metadata (83 + name.len() + uri.len())
    let total_size = MINT_DATA_WITH_EXTENSIONS.len()
        + MINT_CLOSE_AUTHORITY_EXTENSION.len()
        + MINT_PERMANENT_DELEGATE_EXTENSION.len()
        + MINT_METADATA_POINTER_EXTENSION.len()
        + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()
        + MINT_METADATA_EXTENSION_UPDATED.len()
        + MINT_GROUP_MEMBER_EXTENSION_UPDATED.len();

    let mut mint_account_data = vec![0u8; total_size];

    // Mint Data
    mint_account_data[0..MINT_DATA_WITH_EXTENSIONS.len()]
        .copy_from_slice(MINT_DATA_WITH_EXTENSIONS);
    let mut offset = MINT_DATA_WITH_EXTENSIONS.len();
    // Close Authority Extension
    mint_account_data[offset..offset + MINT_CLOSE_AUTHORITY_EXTENSION.len()]
        .copy_from_slice(MINT_CLOSE_AUTHORITY_EXTENSION);
    offset += MINT_CLOSE_AUTHORITY_EXTENSION.len();
    // Permanent Delegate Extension
    mint_account_data[offset..offset + MINT_PERMANENT_DELEGATE_EXTENSION.len()]
        .copy_from_slice(MINT_PERMANENT_DELEGATE_EXTENSION);
    offset += MINT_PERMANENT_DELEGATE_EXTENSION.len();
    // Metadata Pointer Extension
    mint_account_data[offset..offset + MINT_METADATA_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_METADATA_POINTER_EXTENSION);
    offset += MINT_METADATA_POINTER_EXTENSION.len();
    // Group Pointer Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_POINTER_EXTENSION);
    offset += MINT_GROUP_MEMBER_POINTER_EXTENSION.len();
    // Metadata Extension
    mint_account_data[offset..offset + MINT_METADATA_EXTENSION_UPDATED.len()]
        .copy_from_slice(MINT_METADATA_EXTENSION_UPDATED);
    offset += MINT_METADATA_EXTENSION_UPDATED.len();
    // Group Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_EXTENSION_UPDATED.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_EXTENSION_UPDATED);

    // Create the mint account
    let mut record_mint_account = Account::new(
        100_000_000u64,
        mint_account_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    record_mint_account
        .data_as_mut_slice()
        .copy_from_slice(&mint_account_data);

    (address, record_mint_account)
}

const MINT_METADATA_EXTENSIONE_WITH_ADDITIONAL_METADATA: &[u8; 111] = &[
    19, 0, 107, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19,
    33, 142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 44, 183, 51, 50, 60, 76, 5, 80, 101,
    31, 190, 147, 58, 233, 60, 212, 133, 19, 33, 142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27,
    127, 4, 0, 0, 0, 116, 101, 115, 116, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 1,
    0, 0, 0, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 116, 101, 115, 116,
];

fn keyed_account_for_mint_with_additional_metadata(record: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"mint", &record.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    let total_size = MINT_DATA_WITH_EXTENSIONS.len()
        + MINT_CLOSE_AUTHORITY_EXTENSION.len()
        + MINT_PERMANENT_DELEGATE_EXTENSION.len()
        + MINT_METADATA_POINTER_EXTENSION.len()
        + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()
        + MINT_METADATA_EXTENSIONE_WITH_ADDITIONAL_METADATA.len()
        + MINT_GROUP_MEMBER_EXTENSION.len();

    let mut mint_account_data = vec![0u8; total_size];

    // Mint Data
    mint_account_data[0..MINT_DATA_WITH_EXTENSIONS.len()]
        .copy_from_slice(MINT_DATA_WITH_EXTENSIONS);
    let mut offset = MINT_DATA_WITH_EXTENSIONS.len();
    // Close Authority Extension
    mint_account_data[offset..offset + MINT_CLOSE_AUTHORITY_EXTENSION.len()]
        .copy_from_slice(MINT_CLOSE_AUTHORITY_EXTENSION);
    offset += MINT_CLOSE_AUTHORITY_EXTENSION.len();
    // Permanent Delegate Extension
    mint_account_data[offset..offset + MINT_PERMANENT_DELEGATE_EXTENSION.len()]
        .copy_from_slice(MINT_PERMANENT_DELEGATE_EXTENSION);
    offset += MINT_PERMANENT_DELEGATE_EXTENSION.len();
    // Metadata Pointer Extension
    mint_account_data[offset..offset + MINT_METADATA_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_METADATA_POINTER_EXTENSION);
    offset += MINT_METADATA_POINTER_EXTENSION.len();
    // Group Pointer Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_POINTER_EXTENSION);
    offset += MINT_GROUP_MEMBER_POINTER_EXTENSION.len();
    // Metadata Extension
    mint_account_data[offset..offset + MINT_METADATA_EXTENSIONE_WITH_ADDITIONAL_METADATA.len()]
        .copy_from_slice(MINT_METADATA_EXTENSIONE_WITH_ADDITIONAL_METADATA);
    offset += MINT_METADATA_EXTENSIONE_WITH_ADDITIONAL_METADATA.len();
    // Group Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_EXTENSION);

    // Create the mint account
    let mut record_mint_account = Account::new(
        100_000_000u64,
        mint_account_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    record_mint_account
        .data_as_mut_slice()
        .copy_from_slice(&mint_account_data);

    (address, record_mint_account)
}

const MINT_METADATA_EXTENSIONE_WITH_MULTIPLE_ADDITIONAL_METADATA: &[u8; 143] = &[
    19, 0, 139, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19,
    33, 142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 44, 183, 51, 50, 60, 76, 5, 80, 101,
    31, 190, 147, 58, 233, 60, 212, 133, 19, 33, 142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27,
    127, 4, 0, 0, 0, 116, 101, 115, 116, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 3,
    0, 0, 0, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 115, 101,
    115, 116, 4, 0, 0, 0, 116, 101, 115, 116, 4, 0, 0, 0, 114, 101, 115, 116, 4, 0, 0, 0, 116, 101,
    115, 116,
];

fn keyed_account_for_mint_with_multiple_additional_metadata(record: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"mint", &record.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    let total_size = MINT_DATA_WITH_EXTENSIONS.len()
        + MINT_CLOSE_AUTHORITY_EXTENSION.len()
        + MINT_PERMANENT_DELEGATE_EXTENSION.len()
        + MINT_METADATA_POINTER_EXTENSION.len()
        + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()
        + MINT_METADATA_EXTENSIONE_WITH_MULTIPLE_ADDITIONAL_METADATA.len()
        + MINT_GROUP_MEMBER_EXTENSION.len();

    let mut mint_account_data = vec![0u8; total_size];

    // Mint Data
    mint_account_data[0..MINT_DATA_WITH_EXTENSIONS.len()]
        .copy_from_slice(MINT_DATA_WITH_EXTENSIONS);
    let mut offset = MINT_DATA_WITH_EXTENSIONS.len();
    // Close Authority Extension
    mint_account_data[offset..offset + MINT_CLOSE_AUTHORITY_EXTENSION.len()]
        .copy_from_slice(MINT_CLOSE_AUTHORITY_EXTENSION);
    offset += MINT_CLOSE_AUTHORITY_EXTENSION.len();
    // Permanent Delegate Extension
    mint_account_data[offset..offset + MINT_PERMANENT_DELEGATE_EXTENSION.len()]
        .copy_from_slice(MINT_PERMANENT_DELEGATE_EXTENSION);
    offset += MINT_PERMANENT_DELEGATE_EXTENSION.len();
    // Metadata Pointer Extension
    mint_account_data[offset..offset + MINT_METADATA_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_METADATA_POINTER_EXTENSION);
    offset += MINT_METADATA_POINTER_EXTENSION.len();
    // Group Pointer Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_POINTER_EXTENSION);
    offset += MINT_GROUP_MEMBER_POINTER_EXTENSION.len();
    // Metadata Extension
    mint_account_data
        [offset..offset + MINT_METADATA_EXTENSIONE_WITH_MULTIPLE_ADDITIONAL_METADATA.len()]
        .copy_from_slice(MINT_METADATA_EXTENSIONE_WITH_MULTIPLE_ADDITIONAL_METADATA);
    offset += MINT_METADATA_EXTENSIONE_WITH_MULTIPLE_ADDITIONAL_METADATA.len();
    // Group Extension
    mint_account_data[offset..offset + MINT_GROUP_MEMBER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_MEMBER_EXTENSION);

    // Create the mint account
    let mut record_mint_account = Account::new(
        100_000_000u64,
        mint_account_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    record_mint_account
        .data_as_mut_slice()
        .copy_from_slice(&mint_account_data);

    (address, record_mint_account)
}

const GROUP_MINT_DATA_WITH_EXTENSIONS: &[u8] = &[
    1, 0, 0, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188,
    182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188,
    182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];
const MINT_GROUP_POINTER_EXTENSION: &[u8] = &[
    20, 0, 64, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87,
    188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 52, 137, 177, 136, 59,
    205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211,
    23, 123, 152, 136, 141, 87, 92,
];
const MINT_GROUP_EXTENSION: &[u8] = &[
    21, 0, 80, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87,
    188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 52, 137, 177, 136, 59,
    205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211,
    23, 123, 152, 136, 141, 87, 92, 1, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0,
];

fn keyed_account_for_group(class: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"group", &class.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    let total_size = GROUP_MINT_DATA_WITH_EXTENSIONS.len()
        + MINT_GROUP_POINTER_EXTENSION.len()
        + MINT_GROUP_EXTENSION.len();

    let mut group_account_data = vec![0u8; total_size];

    // Mint Data
    group_account_data[0..GROUP_MINT_DATA_WITH_EXTENSIONS.len()]
        .copy_from_slice(GROUP_MINT_DATA_WITH_EXTENSIONS);
    let mut offset = GROUP_MINT_DATA_WITH_EXTENSIONS.len();
    // Group Pointer Extension
    group_account_data[offset..offset + MINT_GROUP_POINTER_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_POINTER_EXTENSION);
    offset += MINT_GROUP_POINTER_EXTENSION.len();
    // Group Extension
    group_account_data[offset..offset + MINT_GROUP_EXTENSION.len()]
        .copy_from_slice(MINT_GROUP_EXTENSION);

    let mut group_account = Account::new(
        100_000_000u64,
        group_account_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    group_account
        .data_as_mut_slice()
        .copy_from_slice(&group_account_data);

    (address, group_account)
}

fn keyed_account_for_token(owner: Pubkey, mint: Pubkey, is_frozen: bool) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(
        &[
            owner.as_ref(),
            mollusk_svm_programs_token::token2022::ID.as_ref(),
            mint.as_ref(),
        ],
        &mollusk_svm_programs_token::associated_token::ID,
    );

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
    let mut token_account = Account::new(
        100_000_000u64,
        token_account_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    token_account
        .data_as_mut_slice()
        .copy_from_slice(&token_account_data);

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
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();

    // Class Updated
    let (_, class_data_updated) = keyed_account_for_class(authority, false, false, "test", "test2");
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateClassMetadata {
        authority,
        payer,
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
            (payer, payer_data),
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
fn update_class_metadata_incorrect_authority() {
    // Authority
    let (random_authority, random_authority_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateClassMetadata {
        authority: random_authority,
        payer: random_authority,
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
        &[Check::err(ProgramError::MissingRequiredSignature)],
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
    let (record, record_data) =
        keyed_account_for_record(class, 0, owner, false, 0, b"test", b"test");
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
        seed: make_u8prefix_vec_u8(b"test"),
        data: make_remainder_vec(b"test"),
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
fn create_record_with_metadata() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record_with_metadata(class, 0, owner, false, 0, "test", None);
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateRecordTokenizable {
        owner,
        payer: owner,
        class,
        record,
        system_program,
        authority: None,
    }
    .instruction(CreateRecordTokenizableInstructionArgs {
        expiration: 0,
        seed: make_u8prefix_vec_u8(b"test"),
        metadata: Metadata {
            name: make_u32prefix_string("test"),
            symbol: make_u32prefix_string("SRS"),
            uri: make_u32prefix_string("test"),
            additional_metadata: vec![],
        },
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
fn create_record_with_metadata_and_additional_metadata() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record_with_metadata_and_additional_metadata(
        class, 0, owner, false, 0, "test",
    );
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = CreateRecordTokenizable {
        owner,
        payer: owner,
        class,
        record,
        system_program,
        authority: None,
    }
    .instruction(CreateRecordTokenizableInstructionArgs {
        expiration: 0,
        seed: make_u8prefix_vec_u8(b"test"),
        metadata: Metadata {
            name: make_u32prefix_string("test"),
            symbol: make_u32prefix_string("SRS"),
            uri: make_u32prefix_string("test"),
            additional_metadata: vec![MetadataAdditionalMetadata {
                label: make_u32prefix_string("test"),
                value: make_u32prefix_string("test"),
            }],
        },
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
    let (record, record_data) =
        keyed_account_for_record(class, 0, owner, false, 0, b"test", b"test");
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
        seed: make_u8prefix_vec_u8(b"test"),
        data: make_remainder_vec(b"test"),
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
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, owner, false, 0, b"test", b"test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, owner, false, 0, b"test", b"test2");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority: owner,
        payer,
        record,
        system_program,
        class: None,
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_vec(b"test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (payer, payer_data),
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
fn update_record_with_metadata() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, _) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, owner, false, 0, b"test", b"test");

    // New metadata
    let new_metadata = Metadata {
        name: make_u32prefix_string("test2"),
        symbol: make_u32prefix_string("SRS"),
        uri: make_u32prefix_string("test"),
        additional_metadata: vec![],
    };
    // Record updated
    let (_, record_data_updated) = keyed_account_for_record_with_metadata(
        class,
        0,
        owner,
        false,
        0,
        "test",
        Some(&new_metadata.try_to_vec().unwrap()),
    );

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecordTokenizable {
        authority: owner,
        payer,
        record,
        system_program,
        class: None,
    }
    .instruction(UpdateRecordTokenizableInstructionArgs {
        metadata: Metadata {
            name: make_u32prefix_string("test2"),
            symbol: make_u32prefix_string("SRS"),
            uri: make_u32prefix_string("test"),
            additional_metadata: vec![],
        },
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (payer, payer_data),
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
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test2");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority,
        payer,
        record,
        system_program,
        class: Some(class),
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_vec(b"test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (payer, payer_data),
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
fn update_record_with_delegate_not_permissioned() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, false, false, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority,
        payer,
        record,
        system_program,
        class: Some(class),
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_vec(b"test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (payer, payer_data),
            (record, record_data),
            (system_program, system_program_data),
            (class, class_data),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
/// Fails because class authority != from authority of the instruction
fn update_record_with_delegate_incorrect_authority() {
    // Authority
    let (random_authority, random_authority_data) = keyed_account_for_random_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, true, false, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority: random_authority,
        payer,
        record,
        system_program,
        class: Some(class),
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_vec(b"test2"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (random_authority, random_authority_data),
            (payer, payer_data),
            (record, record_data),
            (system_program, system_program_data),
            (class, class_data),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn transfer_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, owner, false, 0, b"test", b"test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, NEW_OWNER, false, 0, b"test", b"test");

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
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");
    // Record updated
    let (_, record_data_updated) =
        keyed_account_for_record(class, 0, NEW_OWNER, false, 0, b"test", b"test");

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
        &[
            (authority, authority_data),
            (record, record_data),
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
/// Fails because the record is frozen
fn fail_transfer_record_frozen() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");

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
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn delete_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");

    let instruction = DeleteRecord {
        authority: owner,
        payer,
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
        &[
            (owner, owner_data),
            (payer, payer_data),
            (record, record_data),
        ],
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
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");

    let instruction = DeleteRecord {
        authority,
        payer,
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
        &[
            (authority, authority_data),
            (payer, payer_data),
            (record, record_data),
            (class, class_data),
        ],
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
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");
    // Record frozen
    let (_, record_data_frozen) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");

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
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");
    // Record frozen
    let (_, record_data_frozen) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");

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
        &[
            (authority, authority_data),
            (record, record_data),
            (class, class_data),
        ],
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
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");
    // Record frozen
    let (_, record_data_frozen) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");

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
        &[
            (authority, authority_data),
            (record, record_data),
            (class, class_data),
        ],
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
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record_with_metadata(class, 0, owner, false, 0, "test", None);
    // Mint
    let (mint, mint_data) = keyed_account_for_mint(record);
    // Group
    let (group, group_data) = keyed_account_for_group(class);
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
        class,
        group,
        token_account,
        associated_token_program,
        token2022,
        system_program,
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
            (class, class_data),
            (group, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data.data).build(),
            Check::account(&group).data(&group_data.data).build(),
            Check::account(&token_account)
                .data(&token_account_data.data)
                .build(),
            Check::account(&group).rent_exempt().build(),
        ],
    );
}

#[test]
fn mint_record_token_with_additional_metadata() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record_with_metadata_and_additional_metadata(
        class, 0, owner, false, 0, "test",
    );
    // Mint
    let (mint, mint_data) = keyed_account_for_mint_with_additional_metadata(record);
    // Group
    let (group, group_data) = keyed_account_for_group(class);
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
        class,
        group,
        token_account,
        associated_token_program,
        token2022,
        system_program,
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
            (class, class_data),
            (group, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data.data).build(),
            Check::account(&group).data(&group_data.data).build(),
            Check::account(&token_account)
                .data(&token_account_data.data)
                .build(),
            Check::account(&mint).rent_exempt().build(),
        ],
    );
}

#[test]
fn mint_record_token_with_multiple_additional_metadata() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record_with_metadata_and_multiple_additional_metadata(
            class, 0, owner, false, 0, "test",
        );
    // Mint
    let (mint, mint_data) = keyed_account_for_mint_with_multiple_additional_metadata(record);
    // Group
    let (group, group_data) = keyed_account_for_group(class);
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
        class,
        group,
        token_account,
        associated_token_program,
        token2022,
        system_program,
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
            (class, class_data),
            (group, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data.data).build(),
            Check::account(&group).data(&group_data.data).build(),
            Check::account(&token_account)
                .data(&token_account_data.data)
                .build(),
            Check::account(&mint).rent_exempt().build(),
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
        keyed_account_for_record_with_metadata(class, 0, owner, false, 0, "test", None);
    // Mint
    let (mint, mint_data) = keyed_account_for_mint(record);
    // Group
    let (group, group_data) = keyed_account_for_group(class);
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);

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
        class,
        group,
        token_account,
        associated_token_program,
        token2022,
        system_program,
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
            (class, class_data),
            (group, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&mint).data(&mint_data.data).build(),
            Check::account(&group).data(&group_data.data).build(),
            Check::account(&token_account)
                .data(&token_account_data.data)
                .build(),
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
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
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
            Check::account(&token_account)
                .data(&token_account_data_updated.data)
                .build(),
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
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
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
            Check::account(&token_account)
                .data(&token_account_data_updated.data)
                .build(),
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
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);
    // New ATA
    let (new_token_account, new_token_account_data) =
        keyed_account_for_token(RANDOM_PUBKEY, mint, false);

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
        &[Check::success()],
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
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(OWNER, mint, false);
    // New ATA
    let (new_token_account, new_token_account_data) =
        keyed_account_for_token(RANDOM_PUBKEY, mint, false);

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
        &[Check::success()],
    );
}

#[test]
fn burn_tokenized_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = BurnTokenizedRecord {
        authority: owner,
        payer,
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
            (payer, payer_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (token2022, token2022_data),
        ],
        &[Check::success()],
    );
}

#[test]
fn burn_tokenized_record_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(OWNER, mint, false);

    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = BurnTokenizedRecord {
        authority,
        payer,
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
            (payer, payer_data),
            (record, record_data),
            (mint, mint_data),
            (token_account, token_account_data),
            (token2022, token2022_data),
            (class, class_data),
        ],
        &[Check::success()],
    );
}

#[test]
fn mint_and_burn_tokenized_record() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record_with_metadata(class, 0, owner, false, 0, "test", None);
    // Mint
    let (mint, _) = keyed_account_for_mint(record);
    // Group
    let (group, _) = keyed_account_for_group(class);
    // ATA
    let (token_account, _) = keyed_account_for_token(owner, mint, false);
    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();
    let (associated_token_program, associated_token_program_data) =
        mollusk_svm_programs_token::associated_token::keyed_account();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let mint_instruction = MintTokenizedRecord {
        owner,
        payer: owner,
        authority: owner,
        record,
        mint,
        class,
        group,
        token_account,
        associated_token_program,
        token2022,
        system_program,
    }
    .instruction();

    let burn_instruction = BurnTokenizedRecord {
        authority: owner,
        payer: owner,
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

    mollusk.process_and_validate_instruction_chain(
        &[
            (&mint_instruction, &[Check::success()]),
            (&burn_instruction, &[Check::success()]),
        ],
        &[
            (owner, owner_data),
            (record, record_data),
            (mint, Account::default()),
            (class, class_data),
            (group, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
    );
}

#[test]
fn mint_and_burn_tokenized_record_delegate() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record_with_metadata(class, 0, owner, false, 0, "test", None);
    // Mint
    let (mint, _) = keyed_account_for_mint(record);
    // Group
    let (group, _) = keyed_account_for_group(class);
    // ATA
    let (token_account, _) = keyed_account_for_token(owner, mint, false);

    let (associated_token_program, associated_token_program_data) =
        mollusk_svm_programs_token::associated_token::keyed_account();
    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let mint_instruction = MintTokenizedRecord {
        owner,
        payer: authority,
        authority,
        record,
        mint,
        class,
        group,
        token_account,
        associated_token_program,
        token2022,
        system_program,
    }
    .instruction();

    let burn_instruction = BurnTokenizedRecord {
        authority,
        payer: authority,
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

    mollusk.process_and_validate_instruction_chain(
        &[
            (&mint_instruction, &[Check::success()]),
            (&burn_instruction, &[Check::success()]),
        ],
        &[
            (owner, owner_data),
            (payer, payer_data),
            (authority, authority_data),
            (record, record_data),
            (mint, Account::default()),
            (class, class_data),
            (group, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
    );
}

#[test]
fn update_tokenized_record() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(authority, true, false, "test", "test");
    // Mint
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
    // ATA
    let (token_account, token_account_data) = keyed_account_for_token(OWNER, mint, false);
    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let burn_instruction = BurnTokenizedRecord {
        authority,
        payer,
        record,
        mint,
        token_account,
        token2022,
        class: Some(class),
    }
    .instruction();

    // Owner
    let (owner, owner_data) = keyed_account_for_owner();

    // New metadata
    let new_metadata = Metadata {
        name: make_u32prefix_string("test2"),
        symbol: make_u32prefix_string("SRS"),
        uri: make_u32prefix_string("test"),
        additional_metadata: vec![],
    };

    // Record updated
    let (_, record_data_updated) = keyed_account_for_record_with_metadata(
        class,
        0,
        owner,
        false,
        0,
        "test",
        Some(&new_metadata.try_to_vec().unwrap()),
    );

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let update_instruction = UpdateRecordTokenizable {
        authority: owner,
        payer,
        record,
        system_program,
        class: None,
    }
    .instruction(UpdateRecordTokenizableInstructionArgs {
        metadata: Metadata {
            name: make_u32prefix_string("test2"),
            symbol: make_u32prefix_string("SRS"),
            uri: make_u32prefix_string("test"),
            additional_metadata: vec![],
        },
    });

    // Group
    let (group, group_data) = keyed_account_for_group(class);
    // Associated Token Program
    let (associated_token_program, associated_token_program_data) =
        mollusk_svm_programs_token::associated_token::keyed_account();

    // New Mint 
    let (_, new_mint_data) = keyed_account_for_updated_mint(record);

    let mint_instruction = MintTokenizedRecord {
        owner,
        payer: authority,
        authority,
        record,
        mint,
        class,
        group,
        token_account,
        associated_token_program,
        token2022,
        system_program,
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction_chain(
        &[
            (&burn_instruction, &[Check::success()]),
            (
                &update_instruction, 
                &[
                    Check::success(),
                    Check::account(&record).data(&record_data_updated.data).build()
                ]
            ),
            (
                &mint_instruction, 
                &[
                    Check::success(),
                    Check::account(&mint).data(&new_mint_data.data).build()
                ]
            ),
        ],
        &[
            (owner, owner_data),
            (payer, payer_data),
            (authority, authority_data),
            (record, record_data),
            (mint, mint_data),
            (class, class_data),
            (group, group_data),
            (token_account, token_account_data),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
    );
}