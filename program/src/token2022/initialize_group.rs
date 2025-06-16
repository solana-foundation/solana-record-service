use core::mem::size_of;
use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, log::sol_log_data, program::invoke_signed, pubkey::Pubkey, ProgramResult
};

use crate::{
    token2022::constants::TOKEN_2022_PROGRAM_ID,
    utils::{write_bytes, UNINIT_BYTE},
};

/// Initializes a Mint Close Authority.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a close authority for.
pub struct InitializeGroup<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Mint Authority Account.
    pub mint_authority: &'a AccountInfo,
    /// Update Authority Account.
    pub update_authority: &'a Pubkey,
    /// Max Size
    pub max_size: u64,
}

impl InitializeGroup<'_> {
    pub const DISCRIMINATOR: [u8; 8] = [0x79, 0x71, 0x6c, 0x27, 0x36, 0x33, 0x00, 0x04];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const UPDATE_AUTHORITY_OFFSET: usize = Self::DISCRIMINATOR_OFFSET;
    const MAX_SIZE_OFFSET: usize = Self::UPDATE_AUTHORITY_OFFSET + size_of::<Pubkey>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (8 bytes, [u8;8])
        // - [8..40]: updateAuthority (32 bytes, Pubkey)
        // - [40..48]: maxSize (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 48];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &Self::DISCRIMINATOR,
        );

        write_bytes(
            &mut instruction_data[Self::UPDATE_AUTHORITY_OFFSET..],
            self.update_authority,
        );

        write_bytes(
            &mut instruction_data[Self::MAX_SIZE_OFFSET..],
            &self.max_size.to_le_bytes(),
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        sol_log_data(&[instruction.data]);

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
