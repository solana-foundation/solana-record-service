extern crate alloc;

use alloc::{string::String, vec::Vec};
use codama::CodamaInstructions;
use pinocchio::Address;

/// Instructions for the Solana Record Service. The program dispatches on the
/// discriminator in `lib.rs`; this enum only describes the wire format for the IDL.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum SolanaRecordServiceInstruction {
    /// Create a new class.
    #[codama(account(name = "authority", signer, docs = "Authority used to create a new class"))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will pay for the class account"
    ))]
    #[codama(account(
        name = "class",
        writable,
        default_value = pda("class", [account("authority"), argument("name")]),
        docs = "New class account to be initialized"
    ))]
    #[codama(account(
        name = "system_program",
        default_value = program("system"),
        docs = "System Program used to open our new class account"
    ))]
    CreateClass {
        is_permissioned: bool,
        is_frozen: bool,
        #[codama(type = string(utf8))]
        #[codama(size_prefix = number(u8))]
        name: String,
        #[codama(type = string(utf8))]
        metadata: String,
    } = 0,

    /// Replace the metadata of a class.
    #[codama(account(name = "authority", signer, writable, docs = "Authority used to update a class"))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will pay or get refunded for the class update"
    ))]
    #[codama(account(name = "class", writable, docs = "Class account to be updated"))]
    #[codama(account(
        name = "system_program",
        default_value = program("system"),
        docs = "System Program used to extend our class account"
    ))]
    UpdateClassMetadata {
        #[codama(type = string(utf8))]
        metadata: String,
    } = 1,

    /// Hand a class over to a new authority.
    #[codama(account(name = "authority", signer, writable, docs = "Authority used to update a class"))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will pay or get refunded for the class update"
    ))]
    #[codama(account(name = "class", writable, docs = "Class account to be updated"))]
    #[codama(account(
        name = "system_program",
        default_value = program("system"),
        docs = "System Program used to extend our class account"
    ))]
    UpdateClassAuthority { new_authority: Address } = 2,

    /// Freeze or thaw a class.
    #[codama(account(name = "authority", signer, writable, docs = "Authority used to freeze/thaw a class"))]
    #[codama(account(name = "class", writable, docs = "Class account to be frozen/thawed"))]
    FreezeClass { is_frozen: bool } = 3,

    /// Create a record in a class. `createRecordTokenizable` shares this discriminator.
    #[codama(account(name = "owner", signer, docs = "Owner of the new record"))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will pay for the record account"
    ))]
    #[codama(account(name = "class", writable, docs = "Class account for the record to be created"))]
    #[codama(account(
        name = "record",
        writable,
        default_value = pda("record", [account("class"), argument("seed")]),
        docs = "Record account to be created"
    ))]
    #[codama(account(
        name = "system_program",
        default_value = program("system"),
        docs = "System Program used to create our record account"
    ))]
    #[codama(account(name = "authority", optional, signer, docs = "Optional authority for permissioned classes"))]
    CreateRecord {
        expiration: i64,
        #[codama(type = bytes)]
        #[codama(size_prefix = number(u8))]
        seed: Vec<u8>,
        #[codama(type = bytes)]
        data: Vec<u8>,
    } = 4,

    /// Replace the data of a record. `updateRecordTokenizable` shares this discriminator.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Record owner or class authority for permissioned classes"
    ))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will pay or get refunded for the record update"
    ))]
    #[codama(account(name = "record", writable, docs = "Record account to be updated"))]
    #[codama(account(name = "class", docs = "Class account of the record"))]
    #[codama(account(
        name = "system_program",
        default_value = program("system"),
        docs = "System Program used to extend our record account"
    ))]
    UpdateRecord {
        #[codama(type = bytes)]
        data: Vec<u8>,
    } = 5,

    /// Set the expiry of a record.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Record owner or class authority for permissioned classes"
    ))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will pay or get refunded for the record update"
    ))]
    #[codama(account(name = "record", writable, docs = "Record account to be updated"))]
    #[codama(account(name = "class", docs = "Class account of the record"))]
    #[codama(account(
        name = "system_program",
        default_value = program("system"),
        docs = "System Program used to extend our record account"
    ))]
    UpdateRecordExpiry { expiry: i64 } = 6,

    /// Transfer a record to a new owner.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Record owner or class authority for permissioned classes"
    ))]
    #[codama(account(name = "record", writable, docs = "Record account to be updated"))]
    #[codama(account(name = "class", optional, docs = "Class account of the record"))]
    TransferRecord { new_owner: Address } = 7,

    /// Delete a record and refund its rent.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Record owner or class authority for permissioned classes"
    ))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will get refunded for the record deletion"
    ))]
    #[codama(account(name = "record", writable, docs = "Record account to be deleted"))]
    #[codama(account(name = "class", optional, docs = "Class account of the record"))]
    #[codama(account(
        name = "token2022_program",
        optional,
        docs = "Token2022 Program used to close the mint account"
    ))]
    #[codama(account(name = "mint", optional, writable, docs = "Mint account for the tokenized record"))]
    DeleteRecord {} = 8,

    /// Freeze or thaw a record.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Record owner or class authority for permissioned classes"
    ))]
    #[codama(account(name = "record", writable, docs = "Record account to be updated"))]
    #[codama(account(name = "class", docs = "Class account of the record"))]
    FreezeRecord { is_frozen: bool } = 9,

    /// Mint a Token-2022 token that represents ownership of a record.
    #[codama(account(name = "owner", docs = "Record owner"))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will pay for the mint account"
    ))]
    #[codama(account(name = "authority", signer, docs = "Record owner or class authority for permissioned classes"))]
    #[codama(account(name = "record", writable, docs = "Record account associated with the tokenized record"))]
    #[codama(account(
        name = "mint",
        writable,
        default_value = pda("mint", [account("record")]),
        docs = "Mint account for the tokenized record"
    ))]
    #[codama(account(name = "class", docs = "Class account of the record"))]
    #[codama(account(
        name = "group",
        writable,
        default_value = pda("group", [account("class")]),
        docs = "Group account for the tokenized record"
    ))]
    #[codama(account(name = "token_account", writable, docs = "Token Account for the tokenized record"))]
    #[codama(account(
        name = "associated_token_program",
        default_value = program("associated-token"),
        docs = "Associated Token Program used to create our token"
    ))]
    #[codama(account(
        name = "token2022",
        default_value = program("token-2022"),
        docs = "Token2022 Program used to create our token"
    ))]
    #[codama(account(
        name = "system_program",
        default_value = program("system"),
        docs = "System Program used to create our token"
    ))]
    MintTokenizedRecord {} = 10,

    /// Freeze or thaw the token of a tokenized record.
    #[codama(account(name = "authority", signer, docs = "Record owner or class authority for permissioned classes"))]
    #[codama(account(name = "mint", docs = "Mint account for the tokenized record"))]
    #[codama(account(name = "token_account", writable, docs = "Token Account for the tokenized record"))]
    #[codama(account(name = "record", docs = "Record account associated with the tokenized record"))]
    #[codama(account(name = "class", docs = "Class account of the record"))]
    #[codama(account(
        name = "token2022",
        default_value = program("token-2022"),
        docs = "Token2022 Program used to freeze/unfreeze the tokenized record"
    ))]
    FreezeTokenizedRecord { is_frozen: bool } = 11,

    /// Transfer the token of a tokenized record.
    #[codama(account(name = "authority", signer, docs = "Record owner or class authority for permissioned classes"))]
    #[codama(account(name = "mint", docs = "Mint account for the tokenized record"))]
    #[codama(account(name = "token_account", writable, docs = "Token Account for the tokenized record"))]
    #[codama(account(name = "new_token_account", writable, docs = "New Token Account for the tokenized record"))]
    #[codama(account(name = "record", docs = "Record account associated with the tokenized record"))]
    #[codama(account(
        name = "token2022",
        default_value = program("token-2022"),
        docs = "Token2022 Program used to transfer the tokenized record"
    ))]
    #[codama(account(name = "class", optional, docs = "Class account of the record"))]
    TransferTokenizedRecord {} = 12,

    /// Burn the token of a tokenized record and hand the record back to the token owner.
    #[codama(account(
        name = "authority",
        signer,
        writable,
        docs = "Record owner or class authority for permissioned classes"
    ))]
    #[codama(account(
        name = "payer",
        signer,
        writable,
        default_value = payer,
        docs = "Account that will get refunded for the tokenized record burn"
    ))]
    #[codama(account(name = "mint", writable, docs = "Mint account for the tokenized record"))]
    #[codama(account(name = "token_account", writable, docs = "Token Account for the tokenized record"))]
    #[codama(account(name = "record", writable, docs = "Record account associated with the tokenized record"))]
    #[codama(account(
        name = "token2022",
        default_value = program("token-2022"),
        docs = "Token2022 Program used to burn the tokenized record"
    ))]
    #[codama(account(name = "class", optional, docs = "Class account of the record"))]
    BurnTokenizedRecord {} = 13,
}

pub mod create_class;
pub use create_class::CreateClass;

pub mod update_class;
pub use update_class::UpdateClassAuthority;
pub use update_class::UpdateClassMetadata;

pub mod freeze_class;
pub use freeze_class::FreezeClass;

pub mod create_record;
pub use create_record::CreateRecord;

pub mod update_record;
pub use update_record::UpdateRecordData;
pub use update_record::UpdateRecordExpiry;

pub mod transfer_record;
pub use transfer_record::TransferRecord;

pub mod freeze_record;
pub use freeze_record::FreezeRecord;

pub mod delete_record;
pub use delete_record::DeleteRecord;

pub mod mint_tokenized_record;
pub use mint_tokenized_record::*;

pub mod transfer_tokenized_record;
pub use transfer_tokenized_record::*;

pub mod freeze_tokenized_record;
pub use freeze_tokenized_record::*;

pub mod burn_tokenized_record;
pub use burn_tokenized_record::*;
