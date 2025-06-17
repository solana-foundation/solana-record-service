use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, log::sol_log_data, program::invoke_signed, ProgramResult
};

use crate::{
    token2022::constants::TOKEN_2022_PROGRAM_ID,
    utils::{write_bytes, UNINIT_BYTE},
};

/// Initializes a Mint Close Authority.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a close authority for.
pub struct InitializeMember<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Member Account
    pub member: &'a AccountInfo,
    /// Mint Authority Account.
    pub mint_authority: &'a AccountInfo,
    /// Update Authority Account.
    pub group: &'a AccountInfo,
    /// Member Address.
    pub group_update_authority: &'a AccountInfo,
}



impl InitializeMember<'_> {
    pub const DISCRIMINATOR: [u8; 8] = [0x98, 0x20, 0xde, 0xb0, 0xdf, 0xed, 0x74, 0x86];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 5] = [
            AccountMeta::writable_signer(self.mint.key()),
            AccountMeta::writable_signer(self.member.key()),
            AccountMeta::writable_signer(self.mint_authority.key()),
            AccountMeta::writable_signer(self.group.key()),
            AccountMeta::writable_signer(self.group_update_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (8 bytes, [u8;8])
        let mut instruction_data = [UNINIT_BYTE; 8];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &Self::DISCRIMINATOR,
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
