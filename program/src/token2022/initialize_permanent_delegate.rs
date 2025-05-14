use core::slice::from_raw_parts;

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
///   0. `[WRITE]`  The mint account to initialize a permanent delegate for.
pub struct InitializePermanentDelegate<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    pub delegate: &'a Pubkey,
}

impl InitializePermanentDelegate<'_> {
    const DISCRIMINATOR: u8 = 0x23;
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const DELEGATE_OFFSET: usize = Self::DISCRIMINATOR_OFFSET + size_of::<u8>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: delegate (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 33];

        // Set discriminator as u8 at offset [0]
        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &[Self::DISCRIMINATOR],
        );
        // Set delegate as [u8; 32] at offset [1..33]
        write_bytes(&mut instruction_data[Self::DELEGATE_OFFSET..33], self.delegate);

        let instruction: Instruction<'_, '_, '_, '_> = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
