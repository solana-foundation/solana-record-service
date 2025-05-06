use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, pubkey::Pubkey, ProgramResult
};

use crate::{constants::{TOKEN_2022_INITIALIZE_PERMANENT_DELEGATE_IX, TOKEN_2022_PROGRAM_ID}, utils::{write_bytes, UNINIT_BYTE}};

/// Initializes a Mint Close Authority.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a permanent delegate for.
pub struct InitializePermanentDelegate<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    pub delegate: &'a Pubkey
}

impl InitializePermanentDelegate<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [
            AccountMeta::writable(self.mint.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: delegate (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 33];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[TOKEN_2022_INITIALIZE_PERMANENT_DELEGATE_IX]);
        // Set delegate as [u8; 32] at offset [1..33]
        write_bytes(&mut instruction_data[1..], self.delegate);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}