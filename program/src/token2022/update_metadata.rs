use core::mem::size_of;
use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{
    constants::MAX_METADATA_LEN,
    token2022::constants::{TOKEN_2022_PROGRAM_ID, TOKEN_IS_FROZEN_FLAG},
    utils::{write_bytes, UNINIT_BYTE},
};

/// Accounts expected by this instruction:
///
///   0. `[w]` Metadata account
///   1. `[s]` Update authority
///
/// Data expected by this instruction:
///
///  0. `UpdateField`
/// pub struct UpdateField {
///     /// Field to update in the metadata (0 = Name, 1 = Symbol, 2 = Uri, 3 = Key(String))
///     pub field: Field,
///     /// Value to write for the field
///     pub value: String,
/// }
pub struct UpdateMetadata<'a> {
    /// Metadata Account [writable]
    pub metadata: &'a AccountInfo,
    /// Update Authority Account [signer]
    pub update_authority: &'a AccountInfo,
    /// URI to update the metadata to
    pub new_uri: &'a str,
}

const DISCRIMINATOR_OFFSET: usize = 0;
const FIELD_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u64>();
const NEW_URI_LENGTH_OFFSET: usize = FIELD_OFFSET + size_of::<u8>();
const NEW_URI_OFFSET: usize = NEW_URI_LENGTH_OFFSET + size_of::<u32>();

impl UpdateMetadata<'_> {
    pub const DISCRIMINATOR: [u8; 8] = [0xdd, 0xe9, 0x31, 0x2d, 0xb5, 0xca, 0xdc, 0xc8];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.metadata.key()),
            AccountMeta::readonly_signer(self.update_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (8 bytes, [u8;8])
        // - [8]: field (u8)
        // - [9..13]: new_uri length (u32)
        // - [13..13+new_uri.len()]: new_uri bytes
        let mut instruction_data =
            [UNINIT_BYTE; Self::DISCRIMINATOR.len() + size_of::<u32>() + MAX_METADATA_LEN];

        write_bytes(
            &mut instruction_data[DISCRIMINATOR_OFFSET..],
            &Self::DISCRIMINATOR,
        );

        // Write field at offset [8]
        write_bytes(
            &mut instruction_data[FIELD_OFFSET..],
            &[TOKEN_IS_FROZEN_FLAG],
        );

        // Write new_uri length at offset [9..13]
        write_bytes(
            &mut instruction_data[NEW_URI_LENGTH_OFFSET..],
            &(self.new_uri.len() as u32).to_le_bytes(),
        );

        // Write new_uri at offset [13]
        write_bytes(
            &mut instruction_data[NEW_URI_OFFSET..],
            self.new_uri.as_bytes(),
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe {
                from_raw_parts(
                    instruction_data.as_ptr() as _,
                    NEW_URI_OFFSET + self.new_uri.len(),
                )
            },
        };

        invoke_signed(&instruction, &[self.update_authority], signers)
    }
}
