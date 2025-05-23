use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{constants::TOKEN_2022_PROGRAM_ID, utils::{write_bytes, UNINIT_BYTE}};

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub struct InitializeAccount3<'a> {
    /// New Account.
    pub account: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Owner of the new Account.
    pub owner: &'a Pubkey,
}

impl InitializeAccount3<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: owner (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 33];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[18]);
        // Set owner as [u8; 32] at offset [1..33]
        write_bytes(&mut instruction_data[1..], self.owner);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke_signed(&instruction, &[self.account, self.mint], signers)
    }
}
