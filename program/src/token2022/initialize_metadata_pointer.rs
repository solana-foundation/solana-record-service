use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, pubkey::Pubkey, ProgramResult
};

use crate::{constants::{TOKEN_2022_METADATA_POINTER_EXTENSION_IX, TOKEN_2022_METADATA_POINTER_INITIALIZE_IX, TOKEN_2022_PROGRAM_ID}, utils::{write_bytes, UNINIT_BYTE}};

/// Initializes a Metadata Pointer.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a close authority for.
pub struct InitializeMetadataPointer<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Authority Account.
    pub authority: &'a Pubkey,
    /// Metadata Address.
    pub metadata_address: &'a Pubkey
}

impl InitializeMetadataPointer<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [
            AccountMeta::writable(self.mint.key())
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: metadata pointer instruction discriminator (1 byte, u8)
        // -  [2..34]: metadataAuthority (32 bytes, Pubkey)
        // -  [34..66]: metadataAddress (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 66];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[TOKEN_2022_METADATA_POINTER_EXTENSION_IX]);
        // Set metadata pointer discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[1..2], &[TOKEN_2022_METADATA_POINTER_INITIALIZE_IX]);
        // Set metadata authority as [u8; 32] at offset [2..34]
        write_bytes(&mut instruction_data[2..34], self.authority);
        // Set metadata authority as [u8; 32] at offset [34..66]
        write_bytes(&mut instruction_data[34..], self.metadata_address);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 66) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}