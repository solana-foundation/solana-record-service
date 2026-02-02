use core::mem::size_of;
use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{
    token2022::constants::TOKEN_2022_PROGRAM_ID,
    utils::{write_bytes, UNINIT_BYTE},
};

/// Updates the metadata for a Token-2022 mint.
///
/// ### Accounts:
/// 0. `[WRITE]` Metadata account
/// 1. `[SIGNER]` Update authority
///
/// ### Data: 0. `UpdateField`
///
/// pub struct UpdateField {
///    /// Field to update in the metadata (0 = Name, 1 = Symbol, 2 = Uri, 3 = Key(String))
///    pub field: Field,
///    /// Value to write for the field
///    pub value: String,
/// }
pub struct UpdateMetadata<'a> {
    /// Metadata Account [writable]
    pub metadata: &'a AccountInfo,
    /// Update Authority Account [signer]
    pub update_authority: &'a AccountInfo,
    /// URI to update the metadata to
    pub additional_metadata: &'a [u8],
}

const DISCRIMINATOR_OFFSET: usize = 0;
const FIELD_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<[u8; 8]>();
const ADDITIONAL_METADATA_LENGTH_OFFSET: usize = FIELD_OFFSET + size_of::<u8>();

impl UpdateMetadata<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        const DISCRIMINATOR: [u8; 8] = [0xdd, 0xe9, 0x31, 0x2d, 0xb5, 0xca, 0xdc, 0xc8];

        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.metadata.key()),
            AccountMeta::readonly_signer(self.update_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (8 bytes, [u8;8])
        // - [8]: field (u8)
        // - [9..13]: additional metadata length (u32)
        // - [13..13+additional_metadata.len()]: additional metadata bytes
        let instruction_data_size =
            DISCRIMINATOR.len() + size_of::<u8>() + self.additional_metadata.len();
        let mut instruction_data = [UNINIT_BYTE; 2_000];

        write_bytes(
            &mut instruction_data[DISCRIMINATOR_OFFSET..],
            &DISCRIMINATOR,
        );

        // Write field at offset [8]
        write_bytes(&mut instruction_data[FIELD_OFFSET..], &[3]);

        // Write additional metadata length at offset [9..13]
        write_bytes(
            &mut instruction_data[ADDITIONAL_METADATA_LENGTH_OFFSET..],
            self.additional_metadata,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data_size) },
        };

        invoke_signed(
            &instruction,
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
