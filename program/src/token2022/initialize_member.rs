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

/// Initializes a Member.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a member for.
///   1. `[]` The member account.
///   2. `[SIGNER]` The mint authority account.
///   3. `[WRITE]` The group account.
///   4. `[SIGNER]` The group update authority account.
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
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        const INITIALIZE_MEMBER_DISCRIMINATOR: [u8; 8] =
            [0x98, 0x20, 0xde, 0xb0, 0xdf, 0xed, 0x74, 0x86];

        // Account metadata
        let account_metas: [AccountMeta; 5] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.member.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly_signer(self.group_update_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (8 bytes, [u8;8])
        let mut instruction_data = [UNINIT_BYTE; 8];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &INITIALIZE_MEMBER_DISCRIMINATOR,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(
            &instruction,
            &[
                self.mint,
                self.member,
                self.mint_authority,
                self.group,
                self.group_update_authority,
            ],
            signers,
        )
    }
}
