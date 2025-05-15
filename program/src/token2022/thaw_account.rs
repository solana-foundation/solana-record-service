use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::token2022::constants::TOKEN_2022_PROGRAM_ID;

/// Thaw a Frozen account using the Mint's freeze_authority
///
/// ### Accounts:
///   0. `[WRITE]` The account to thaw.
///   1. `[]` The token mint.
///   2. `[SIGNER]` The mint freeze authority.
pub struct ThawAccount<'a> {
    /// Token Account to thaw.
    pub account: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Mint Freeze Authority Account
    pub freeze_authority: &'a AccountInfo,
}

impl ThawAccount<'_> {
    const DISCRIMINATOR: u8 = 0x0B;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.freeze_authority.key()),
        ];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[Self::DISCRIMINATOR],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.mint, self.freeze_authority],
            signers,
        )
    }
}