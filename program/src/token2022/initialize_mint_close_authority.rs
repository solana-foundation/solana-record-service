use core::{mem::size_of, slice::from_raw_parts};

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    token2022::constants::TOKEN_2022_PROGRAM_ID,
    utils::{write_bytes, UNINIT_BYTE},
};

/// Initializes a Mint Close Authority.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a close authority for.
pub struct InitializeMintCloseAuthority<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    pub close_authority: &'a Pubkey,
}

impl InitializeMintCloseAuthority<'_> {
    const DISCRIMINATOR: u8 = 0x19;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const CLOSE_AUTHORITY_OFFSET: usize = Self::DISCRIMINATOR_OFFSET + size_of::<u16>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: closeAuthority (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 34];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &[Self::DISCRIMINATOR, 1],
        );

        write_bytes(
            &mut instruction_data[Self::CLOSE_AUTHORITY_OFFSET..],
            self.close_authority,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
