use core::mem::size_of;
use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{
    constants::{
        MAX_METADATA_LEN, MAX_NAME_LEN, SRS_TICKER, TOKEN_2022_METADATA_POINTER_EXTENSION_IX,
        TOKEN_2022_PROGRAM_ID,
    },
    utils::{write_bytes, UNINIT_BYTE},
};

/// Initializes a Mint Close Authority.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a close authority for.
pub struct InitializeMetadata<'a> {
    /// Metadata Account.
    pub metadata: &'a AccountInfo,
    /// Update Authority Account.
    pub update_authority: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Mint Authority Account.
    pub mint_authority: &'a AccountInfo,
    /// Token name.
    pub name: &'a str,
    /// Token symbol.
    pub symbol: &'a str,
    /// Token metadata URI.
    pub uri: &'a str,
}

impl InitializeMetadata<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.update_authority.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (1 byte, u8)
        // - [1..5]: name length (u32)
        // - [5..5+name.len()]: name bytes
        // - [..]: symbol length (u32)
        // - [..]: symbol bytes
        // - [..]: uri length (u32)
        // - [..]: uri bytes
        let mut instruction_data = [UNINIT_BYTE;
            size_of::<u8>()
                + size_of::<u32>() * 3
                + MAX_NAME_LEN
                + SRS_TICKER.len()
                + MAX_METADATA_LEN];

        // Write discriminator as u8 at offset [0]
        write_bytes(
            &mut instruction_data,
            &[TOKEN_2022_METADATA_POINTER_EXTENSION_IX],
        );
        // Write name length at offset [1]
        write_bytes(
            &mut instruction_data[1..1 + size_of::<u32>()],
            &(self.name.len() as u32).to_le_bytes(),
        );

        // Switch to dynamic offsets
        let mut offset = size_of::<u8>() + size_of::<u32>() + self.name.len();

        // Write name at offset [5]
        write_bytes(&mut instruction_data[5..offset], self.name.as_bytes());
        offset += self.name.len();

        // Write symbol length
        write_bytes(
            &mut instruction_data[offset..offset + size_of::<u32>()],
            &(self.symbol.len() as u32).to_le_bytes(),
        );
        offset += size_of::<u32>();

        // Write symbol
        write_bytes(
            &mut instruction_data[offset..offset + self.symbol.len()],
            self.symbol.as_bytes(),
        );
        offset += self.symbol.len();

        // Write URI length
        write_bytes(
            &mut instruction_data[offset..offset + size_of::<u32>()],
            &(self.uri.len() as u32).to_le_bytes(),
        );
        offset += size_of::<u32>();

        // Write URI
        write_bytes(
            &mut instruction_data[offset..offset + self.uri.len()],
            self.uri.as_bytes(),
        );
        offset += self.uri.len();

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, offset) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
