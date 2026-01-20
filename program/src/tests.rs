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
    types::{Metadata, AdditionalMetadata},
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
const MINT_DATA_WITH_EXTENSIONS_AND_NO_SUPPLY: &[u8] = &[
    1, 0, 0, 0, 44, 183, 51, 50, 60, 76, 5, 80, 101, 31, 190, 147, 58, 233, 60, 212, 133, 19, 33,
    142, 101, 42, 77, 206, 214, 6, 73, 4, 96, 81, 27, 127, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
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
    5, 0, 0, 0, 116, 101, 115, 116, 50, 3, 0, 0, 0, 83, 82, 83, 4, 0, 0, 0, 116, 101, 115, 116, 0,
    0, 0, 0,
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

const MINT_METADATA_EXTENSION_WITH_ADDITIONAL_METADATA: &[u8; 111] = &[
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
        + MINT_METADATA_EXTENSION_WITH_ADDITIONAL_METADATA.len()
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
    mint_account_data[offset..offset + MINT_METADATA_EXTENSION_WITH_ADDITIONAL_METADATA.len()]
        .copy_from_slice(MINT_METADATA_EXTENSION_WITH_ADDITIONAL_METADATA);
    offset += MINT_METADATA_EXTENSION_WITH_ADDITIONAL_METADATA.len();
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
    23, 123, 152, 136, 141, 87, 92, 1, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255,
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
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn update_class_authority() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // New Authority
    let (new_authority, _) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateClassAuthority { 
        authority: authority,
        payer: authority,
        class,
        system_program, 
    }.instruction(UpdateClassAuthorityInstructionArgs { new_authority });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // Class Updated
    let (_, class_data_updated) = keyed_account_for_class(new_authority, false, false, "test", "test");

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (class, class_data),
            (system_program, system_program_data),
        ],
        &[
            Check::success(),
            Check::account(&class).data(&class_data_updated.data).build(),
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
            additional_metadata: vec![AdditionalMetadata {
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
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
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
        class,
        system_program,
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
            (class, class_data),
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
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");

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
        OWNER,
        false,
        0,
        "test",
        Some(&new_metadata.try_to_vec().unwrap()),
    );

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecordTokenizable {
        authority,
        payer,
        record,
        class,
        system_program,
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
            (authority, authority_data),
            (payer, payer_data),
            (record, record_data),
            (class, class_data),
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
        class,
        system_program,
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
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn update_class_expiry() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class(AUTHORITY, true, false, "test", "test");
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");
    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecordExpiry {
        authority,
        payer,
        record,
        class,
        system_program,
    }
    .instruction(UpdateRecordExpiryInstructionArgs {
        expiry: 1000,
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // Record updated
    let (_, record_data_updated) = keyed_account_for_record(class, 0, OWNER, false, 1000, b"test", b"test");

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
        token2022_program: None,
        mint: None,
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
        token2022_program: None,
        mint: None,
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
fn delete_tokenized_record_with_no_supply() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Mint
    let (record, _bump) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    let (mint, mut mint_data) = keyed_account_for_mint(record);
    mint_data.data_as_mut_slice()[..MINT_DATA_WITH_EXTENSIONS_AND_NO_SUPPLY.len()].copy_from_slice(MINT_DATA_WITH_EXTENSIONS_AND_NO_SUPPLY);
    // Record
    let (_, record_data) =
        keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
    // Token2022 Program
    let (token2022_program, token2022_program_data) = mollusk_svm_programs_token::token2022::keyed_account();

    let instruction = DeleteRecord {
        authority,
        payer: authority,
        record,
        class: Some(class),
        token2022_program: Some(token2022_program),
        mint: Some(mint),
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (class, class_data),
            (mint, mint_data),
            (token2022_program, token2022_program_data),
        ],
        &[
            Check::success(),
            Check::account(&record).data(&[0xff]).build(),
            Check::account(&mint).data(&[]).build(),
        ],
    );
}

#[test]
fn freeze_record() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, false, 0, b"test", b"test");
    // Record frozen
    let (_, record_data_frozen) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");

    let instruction = FreezeRecord {
        authority,
        record,
        class,
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
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");
    // Record frozen
    let (_, record_data_frozen) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");

    let instruction = FreezeRecord {
        authority,
        record,
        class,
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

// `[1, 0, 0, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 20, 0, 64, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 21, 0, 80, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 1, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0]`,
// `[1, 0, 0, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 20, 0, 64, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 21, 0, 80, 0, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 52, 137, 177, 136, 59, 205, 145, 103, 193, 194, 30, 23, 233, 253, 189, 51, 87, 188, 182, 87, 172, 35, 137, 100, 211, 23, 123, 152, 136, 141, 87, 92, 1, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255]`

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
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
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
        class,
        token2022,
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
            (class, class_data),
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
        class,
        token2022,
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
        OWNER,
        false,
        0,
        "test",
        Some(&new_metadata.try_to_vec().unwrap()),
    );

    //System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let update_instruction = UpdateRecordTokenizable {
        authority,
        payer,
        record,
        class,
        system_program,
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

    // Owner
    let (owner, owner_data) = keyed_account_for_owner();

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
            (
                &burn_instruction, 
                &[Check::success()]
            ),
            (
                &update_instruction,
                &[
                    Check::success(),
                    Check::account(&record)
                        .data(&record_data_updated.data)
                        .build(),
                ],
            ),
            (
                &mint_instruction,
                &[
                    Check::success(),
                    Check::account(&mint).data(&new_mint_data.data).build(),
                ],
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

#[test]
fn test_create_record_rejects_non_canonical_pda() {
    let (class, class_data) = keyed_account_for_class_default();
    let (owner, owner_data) = keyed_account_for_owner();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    // Use an arbitrary keypair instead of the correct PDA
    let arbitrary_record_key = Pubkey::new_from_array([0xef; 32]);

    let seed = b"test";

    // Verify this is NOT the correct PDA
    let (correct_pda, _bump) = Pubkey::find_program_address(
        &[b"record", class.as_ref(), seed],
        &SOLANA_RECORD_SERVICE_ID,
    );
    assert_ne!(arbitrary_record_key, correct_pda, "Test setup error: arbitrary key should not match PDA");

    use solana_program::instruction::{AccountMeta, Instruction};

    let accounts = vec![
        AccountMeta::new_readonly(owner, true),    // owner
        AccountMeta::new(owner, true),             // payer
        AccountMeta::new(class, false),            // class
        AccountMeta::new(arbitrary_record_key, true), // record - arbitrary key instead of PDA
        AccountMeta::new_readonly(system_program, false), // system_program
    ];

    let expiration: i64 = 0;
    let data = b"test";

    // Build instruction data: discriminator (4) + expiry (i64) + seed_len (u8) + seed + data
    let mut instruction_data = vec![4u8]; // CreateRecord discriminator
    instruction_data.extend_from_slice(&expiration.to_le_bytes());
    instruction_data.push(seed.len() as u8);
    instruction_data.extend_from_slice(seed);
    instruction_data.extend_from_slice(data);

    let instruction = Instruction {
        program_id: SOLANA_RECORD_SERVICE_ID,
        accounts,
        data: instruction_data,
    };

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // The instruction should FAIL with InvalidSeeds because the record account
    // does not match the expected PDA
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (class, class_data),
            (arbitrary_record_key, Account::default()),
            (system_program, system_program_data),
        ],
        &[Check::err(ProgramError::InvalidSeeds)],
    );
}

#[test]
fn test_create_record_rejects_wrong_pda_different_seed() {
    // Test that using a PDA derived with a different seed fails

    let (class, class_data) = keyed_account_for_class_default();
    let (owner, owner_data) = keyed_account_for_owner();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    // Derive a PDA with a different seed
    let wrong_seed = b"wrong_seed";
    let correct_seed = b"test";

    let (wrong_pda, _bump) = Pubkey::find_program_address(
        &[b"record", class.as_ref(), wrong_seed],
        &SOLANA_RECORD_SERVICE_ID,
    );

    let (correct_pda, _bump) = Pubkey::find_program_address(
        &[b"record", class.as_ref(), correct_seed],
        &SOLANA_RECORD_SERVICE_ID,
    );

    assert_ne!(wrong_pda, correct_pda, "Test setup error: PDAs should differ");

    use solana_program::instruction::{AccountMeta, Instruction};

    let accounts = vec![
        AccountMeta::new_readonly(owner, true),
        AccountMeta::new(owner, true),
        AccountMeta::new(class, false),
        AccountMeta::new(wrong_pda, true), // Wrong PDA (derived with different seed)
        AccountMeta::new_readonly(system_program, false),
    ];

    let expiration: i64 = 0;
    let data = b"test";

    // Instruction data uses correct_seed, but account is wrong_pda
    let mut instruction_data = vec![4u8];
    instruction_data.extend_from_slice(&expiration.to_le_bytes());
    instruction_data.push(correct_seed.len() as u8);
    instruction_data.extend_from_slice(correct_seed);
    instruction_data.extend_from_slice(data);

    let instruction = Instruction {
        program_id: SOLANA_RECORD_SERVICE_ID,
        accounts,
        data: instruction_data,
    };

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // Should fail because the provided PDA doesn't match the expected PDA for the given seed
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (class, class_data),
            (wrong_pda, Account::default()),
            (system_program, system_program_data),
        ],
        &[Check::err(ProgramError::InvalidSeeds)],
    );
}

#[test]
fn test_create_class_rejects_non_canonical_pda() {
    let (authority, authority_data) = keyed_account_for_authority();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    // Use an arbitrary keypair instead of the correct PDA
    let arbitrary_class_key = Pubkey::new_from_array([0xef; 32]);

    let name = "test";

    // Verify this is NOT the correct PDA
    let (correct_pda, _bump) = Pubkey::find_program_address(
        &[b"class", authority.as_ref(), name.as_bytes()],
        &SOLANA_RECORD_SERVICE_ID,
    );
    assert_ne!(arbitrary_class_key, correct_pda, "Test setup error: arbitrary key should not match PDA");

    use solana_program::instruction::{AccountMeta, Instruction};

    let accounts = vec![
        AccountMeta::new(authority, true),         // authority
        AccountMeta::new(authority, true),         // payer
        AccountMeta::new(arbitrary_class_key, true), // class - arbitrary key instead of PDA
        AccountMeta::new_readonly(system_program, false), // system_program
    ];

    let metadata = "test";

    // Build instruction data: discriminator (0) + is_permissioned (bool) + is_frozen (bool) + name_len (u8) + name + metadata
    let mut instruction_data = vec![0u8]; // CreateClass discriminator
    instruction_data.push(0u8); // is_permissioned = false
    instruction_data.push(0u8); // is_frozen = false
    instruction_data.push(name.len() as u8);
    instruction_data.extend_from_slice(name.as_bytes());
    instruction_data.extend_from_slice(metadata.as_bytes());

    let instruction = Instruction {
        program_id: SOLANA_RECORD_SERVICE_ID,
        accounts,
        data: instruction_data,
    };

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // The instruction should FAIL with InvalidSeeds because the class account
    // does not match the expected PDA
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (arbitrary_class_key, Account::default()),
            (system_program, system_program_data),
        ],
        &[Check::err(ProgramError::InvalidSeeds)],
    );
}

#[test]
fn test_mint_tokenized_record_rejects_non_canonical_mint_pda() {
    let (owner, owner_data) = keyed_account_for_owner();
    let (class, class_data) = keyed_account_for_class_default();
    let (record, record_data) =
        keyed_account_for_record_with_metadata(class, 0, owner, false, 0, "test", None);

    // Use an arbitrary keypair instead of the correct mint PDA
    let arbitrary_mint_key = Pubkey::new_from_array([0xef; 32]);

    // Verify this is NOT the correct mint PDA
    let (correct_mint_pda, _bump) = Pubkey::find_program_address(
        &[b"mint", record.as_ref()],
        &SOLANA_RECORD_SERVICE_ID,
    );
    assert_ne!(arbitrary_mint_key, correct_mint_pda, "Test setup error: arbitrary key should not match mint PDA");

    // Group - use correct PDA
    let (group, _) = keyed_account_for_group(class);

    // ATA - need to derive based on the arbitrary mint
    let (token_account, _) = Pubkey::find_program_address(
        &[owner.as_ref(), TOKEN_2022_PROGRAM_ID.as_ref(), arbitrary_mint_key.as_ref()],
        &mollusk_svm_programs_token::associated_token::ID,
    );

    let (associated_token_program, associated_token_program_data) =
        mollusk_svm_programs_token::associated_token::keyed_account();
    let (token2022, token2022_data) = mollusk_svm_programs_token::token2022::keyed_account();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    use solana_program::instruction::{AccountMeta, Instruction};

    let accounts = vec![
        AccountMeta::new_readonly(owner, false),     // owner
        AccountMeta::new(owner, true),               // payer
        AccountMeta::new_readonly(owner, true),      // authority
        AccountMeta::new(record, false),             // record
        AccountMeta::new(arbitrary_mint_key, true),  // mint - arbitrary key instead of PDA
        AccountMeta::new(class, false),              // class
        AccountMeta::new(group, false),              // group
        AccountMeta::new(token_account, false),      // token_account
        AccountMeta::new_readonly(associated_token_program, false),
        AccountMeta::new_readonly(token2022, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    // Discriminator 10 = MintTokenizedRecord (no additional data needed)
    let instruction_data = vec![10u8];

    let instruction = Instruction {
        program_id: SOLANA_RECORD_SERVICE_ID,
        accounts,
        data: instruction_data,
    };

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    // The instruction should FAIL with InvalidSeeds because the mint account
    // does not match the expected PDA
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (record, record_data),
            (arbitrary_mint_key, Account::default()),
            (class, class_data),
            (group, Account::default()),
            (token_account, Account::default()),
            (associated_token_program, associated_token_program_data),
            (token2022, token2022_data),
            (system_program, system_program_data),
        ],
        &[Check::err(ProgramError::InvalidSeeds)],
    );
}

#[test]
fn test_create_record_with_prefunded_account_calculates_lamports_correctly() {
    let (class, class_data) = keyed_account_for_class_default();
    let (owner, owner_data) = keyed_account_for_owner();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let seed = b"prefunded-test";
    let data = b"test data";
    let expiry: i64 = 0;

    let (record, _) = Pubkey::find_program_address(
        &[b"record", class.as_ref(), seed],
        &SOLANA_RECORD_SERVICE_ID,
    );

    // Pre-fund the record PDA with some lamports (simulating a partially funded account)
    let prefund_amount = 50_000u64;
    let mut prefunded_record_account = Account::default();
    prefunded_record_account.lamports = prefund_amount;

    use solana_program::instruction::{AccountMeta, Instruction};

    let accounts = vec![
        AccountMeta::new_readonly(owner, true),    // owner
        AccountMeta::new(owner, true),             // payer
        AccountMeta::new(class, false),            // class
        AccountMeta::new(record, false),           // record - correct PDA
        AccountMeta::new_readonly(system_program, false), // system_program
    ];

    // Build instruction data: discriminator (4) + expiry (i64) + seed_len (u8) + seed + data
    let mut instruction_data = vec![4u8]; // CreateRecord discriminator
    instruction_data.extend_from_slice(&expiry.to_le_bytes());
    instruction_data.push(seed.len() as u8);
    instruction_data.extend_from_slice(seed);
    instruction_data.extend_from_slice(data);

    let instruction = Instruction {
        program_id: SOLANA_RECORD_SERVICE_ID,
        accounts,
        data: instruction_data,
    };

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // The instruction should succeed - the fix ensures the correct lamports delta is calculated
    let result = mollusk.process_instruction(
        &instruction,
        &[
            (owner, owner_data),
            (class, class_data),
            (record, prefunded_record_account),
            (system_program, system_program_data),
        ],
    );

    assert!(!result.program_result.is_err(), "Record creation with pre-funded account should succeed");

    // Verify the record was created with correct rent
    let record_account = result.get_account(&record).unwrap();
    assert!(record_account.lamports > prefund_amount, "Record should have more lamports than pre-funded amount");
}

#[test]
fn test_create_class_with_prefunded_account_calculates_lamports_correctly() {
    let name = "prefunded-class-test";
    let metadata = "test metadata";

    let (authority, authority_data) = keyed_account_for_authority();
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let (class, _) = Pubkey::find_program_address(
        &[b"class", authority.as_ref(), name.as_bytes()],
        &SOLANA_RECORD_SERVICE_ID,
    );

    // Pre-fund the class PDA with some lamports
    let prefund_amount = 100_000u64;
    let mut prefunded_class_account = Account::default();
    prefunded_class_account.lamports = prefund_amount;

    use solana_program::instruction::{AccountMeta, Instruction};

    let accounts = vec![
        AccountMeta::new_readonly(authority, true),  // authority
        AccountMeta::new(authority, true),           // payer
        AccountMeta::new(class, false),              // class - correct PDA
        AccountMeta::new_readonly(system_program, false), // system_program
    ];

    // Build instruction data: discriminator (0) + is_permissioned (bool) + is_frozen (bool) + name_len (u8) + name + metadata
    let mut instruction_data = vec![0u8]; // CreateClass discriminator
    instruction_data.push(0u8); // is_permissioned = false
    instruction_data.push(0u8); // is_frozen = false
    instruction_data.push(name.len() as u8);
    instruction_data.extend_from_slice(name.as_bytes());
    instruction_data.extend_from_slice(metadata.as_bytes());

    let instruction = Instruction {
        program_id: SOLANA_RECORD_SERVICE_ID,
        accounts,
        data: instruction_data,
    };

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // The instruction should succeed with the fix
    let result = mollusk.process_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (class, prefunded_class_account),
            (system_program, system_program_data),
        ],
    );

    assert!(!result.program_result.is_err(), "Class creation with pre-funded account should succeed");

    // Verify the class was created with correct rent
    let class_account = result.get_account(&class).unwrap();
    assert!(class_account.lamports > prefund_amount, "Class should have more lamports than pre-funded amount");
}

#[test]
fn test_delete_tokenized_record_requires_class_authority_after_external_burn() {
    // Use a random attacker as authority (not the class authority)
    let attacker = Pubkey::new_from_array([0xee; 32]);
    let attacker_data = Account::new(100_000_000_000u64, 0, &Pubkey::default());

    // Class
    let (class, class_data) = keyed_account_for_class_default();

    // Record - use b"test" seed to match keyed_account_for_mint
    let (record, _bump) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );

    // Mint (with 0 supply - externally burned)
    let (mint, mut mint_data) = keyed_account_for_mint(record);
    mint_data.data_as_mut_slice()[..MINT_DATA_WITH_EXTENSIONS_AND_NO_SUPPLY.len()]
        .copy_from_slice(MINT_DATA_WITH_EXTENSIONS_AND_NO_SUPPLY);

    // Record with Token owner type (owner_type = 1)
    let (_, record_data) = keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");

    // Token2022 Program
    let (token2022_program, token2022_program_data) = mollusk_svm_programs_token::token2022::keyed_account();

    // Build instruction as attacker (not class authority)
    let instruction = DeleteRecord {
        authority: attacker,
        payer: attacker,
        record,
        class: Some(class),
        token2022_program: Some(token2022_program),
        mint: Some(mint),
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    // Should fail because attacker is not the class authority
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (attacker, attacker_data),
            (record, record_data),
            (class, class_data),
            (token2022_program, token2022_program_data),
            (mint, mint_data),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn test_delete_tokenized_record_succeeds_for_class_authority_after_external_burn() {
    // This test verifies that the class authority CAN delete tokenized records after external burn.

    // Authority (class authority)
    let (authority, authority_data) = keyed_account_for_authority();

    // Class
    let (class, class_data) = keyed_account_for_class_default();

    // Record - use b"test" seed to match keyed_account_for_mint
    let (record, _bump) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );

    // Mint (with 0 supply - externally burned)
    let (mint, mut mint_data) = keyed_account_for_mint(record);
    mint_data.data_as_mut_slice()[..MINT_DATA_WITH_EXTENSIONS_AND_NO_SUPPLY.len()]
        .copy_from_slice(MINT_DATA_WITH_EXTENSIONS_AND_NO_SUPPLY);

    // Record with Token owner type (owner_type = 1)
    let (_, record_data) = keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");

    // Token2022 Program
    let (token2022_program, token2022_program_data) = mollusk_svm_programs_token::token2022::keyed_account();

    // Build instruction as class authority
    let instruction = DeleteRecord {
        authority,
        payer: authority,
        record,
        class: Some(class),
        token2022_program: Some(token2022_program),
        mint: Some(mint),
    }
    .instruction();

    let mut mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);

    // The instruction should succeed because authority IS the class authority
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (record, record_data),
            (class, class_data),
            (token2022_program, token2022_program_data),
            (mint, mint_data),
        ],
        &[
            Check::success(),
            Check::account(&record).data(&[0xff]).build(),
            Check::account(&mint).data(&[]).build(),
        ],
    );
}

#[test]
fn test_burn_frozen_tokenized_record_succeeds() {

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
    // Record - frozen (is_frozen = true)
    let (record, record_data) =
        keyed_account_for_record(class, 1, mint, true, 0, b"test", b"test");
    // ATA - frozen (is_frozen = true)
    let (token_account, token_account_data) = keyed_account_for_token(owner, mint, true);

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

/// Creates a fake Multisig account with 355 bytes (Token22 Multisig size)
/// This account could potentially pass the discriminator check at offset 165
/// if not properly validated
fn keyed_account_for_multisig_as_mint(record: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) =
        Pubkey::find_program_address(&[b"mint", &record.as_ref()], &SOLANA_RECORD_SERVICE_ID);

    // Multisig accounts have a fixed size of 355 bytes
    let mut multisig_data = vec![0u8; 355];
    // Set byte at offset 165 to match MINT_DISCRIMINATOR (0x01)
    multisig_data[165] = 0x01;

    let mut multisig_account = Account::new(
        100_000_000u64,
        multisig_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    multisig_account
        .data_as_mut_slice()
        .copy_from_slice(&multisig_data);

    (address, multisig_account)
}

fn keyed_account_for_multisig_as_token(owner: Pubkey, mint: Pubkey) -> (Pubkey, Account) {
    let (address, _bump) = Pubkey::find_program_address(
        &[
            owner.as_ref(),
            mollusk_svm_programs_token::token2022::ID.as_ref(),
            mint.as_ref(),
        ],
        &mollusk_svm_programs_token::associated_token::ID,
    );

    // Multisig accounts have a fixed size of 355 bytes
    let mut multisig_data = vec![0u8; 355];
    // Set byte at offset 165 to match TOKEN_ACCOUNT_DISCRIMINATOR (0x02)
    multisig_data[165] = 0x02;
    // Set the mint at offset 0
    multisig_data[0..32].copy_from_slice(&mint.to_bytes());
    // Set the owner at offset 32
    multisig_data[32..64].copy_from_slice(&owner.to_bytes());
    // Set amount to 1 at offset 64
    multisig_data[64..72].copy_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]);

    let mut multisig_account = Account::new(
        100_000_000u64,
        multisig_data.len(),
        &TOKEN_2022_PROGRAM_ID,
    );
    multisig_account
        .data_as_mut_slice()
        .copy_from_slice(&multisig_data);

    (address, multisig_account)
}

#[test]
fn test_multisig_account_rejected_as_mint() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record, record_data) = keyed_account_for_record(class, 1, RANDOM_PUBKEY, false, 0, b"test", b"test");
    // Fake mint using Multisig size (355 bytes)
    let (mint, mint_data) = keyed_account_for_multisig_as_mint(record);
    // Token account
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

    // Should fail because Multisig accounts are rejected
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
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn test_multisig_account_rejected_as_token_account() {
    // Owner
    let (owner, owner_data) = keyed_account_for_owner();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, _class_data) = keyed_account_for_class_default();
    // Record
    let (record_address, _) = Pubkey::find_program_address(
        &[b"record", &class.as_ref(), b"test"],
        &SOLANA_RECORD_SERVICE_ID,
    );
    // Mint
    let (mint, mint_data) = keyed_account_for_mint(record_address);
    // Record - owner_type = Token (1), owner = mint
    let (record, record_data) = keyed_account_for_record(class, 1, mint, false, 0, b"test", b"test");
    // Fake token account using Multisig size (355 bytes)
    let (token_account, token_account_data) = keyed_account_for_multisig_as_token(owner, mint);

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

    // Should fail because Multisig accounts are rejected
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
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn test_update_record_data_fails_when_frozen() {
    // Authority
    let (authority, authority_data) = keyed_account_for_authority();
    // Payer
    let (payer, payer_data) = keyed_account_for_random_authority();
    // Class
    let (class, class_data) = keyed_account_for_class_default();
    // Record - frozen (is_frozen = true)
    let (record, record_data) =
        keyed_account_for_record(class, 0, OWNER, true, 0, b"test", b"test");

    // System Program
    let (system_program, system_program_data) = keyed_account_for_system_program();

    let instruction = UpdateRecord {
        authority,
        payer,
        record,
        class,
        system_program,
    }
    .instruction(UpdateRecordInstructionArgs {
        data: make_remainder_vec(b"modified"),
    });

    let mollusk = Mollusk::new(
        &SOLANA_RECORD_SERVICE_ID,
        "../target/deploy/solana_record_service",
    );

    // Should fail because record is frozen
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (authority, authority_data),
            (payer, payer_data),
            (record, record_data),
            (class, class_data),
            (system_program, system_program_data),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}
