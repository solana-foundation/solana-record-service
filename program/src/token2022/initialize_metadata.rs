use core::mem::size_of;
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

/// Initializes metadata for a Token-2022 mint.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize metadata for.
///   1. `[]` The update authority account.
///   2. `[]` The mint account (same as account 0).
///   3. `[SIGNER]` The mint authority account.
pub struct InitializeMetadata<'a> {
    /// Metadata Account.
    pub metadata: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Update Authority Account.
    pub update_authority: &'a AccountInfo,
    /// Mint Authority Account.
    pub mint_authority: &'a AccountInfo,
    /// Metadata data (This is safe because if the data is invalid, the program will reject it)
    pub metadata_data: &'a [u8],
}

impl InitializeMetadata<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const METADATA_DATA_OFFSET: usize = Self::DISCRIMINATOR_OFFSET + size_of::<[u8; 8]>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        const INITIALIZE_METADATA_DISCRIMINATOR: [u8; 8] =
            [0xd2, 0xe1, 0x1e, 0xa2, 0x58, 0xb8, 0x4d, 0x8d];
        // Account metadata - Token-2022 metadata initialization expects:
        // 0. mint (writable)
        // 1. update_authority (readonly)
        // 2. mint (readonly, same as account 0)
        // 3. mint_authority (readonly, signer)
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.metadata.key()),
            AccountMeta::readonly(self.update_authority.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (8 bytes, [u8;8])
        // - [8..]: metadata data
        let instruction_data_size =
            INITIALIZE_METADATA_DISCRIMINATOR.len() + self.metadata_data.len();
        let mut instruction_data = [UNINIT_BYTE; 2_000];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &INITIALIZE_METADATA_DISCRIMINATOR,
        );

        write_bytes(
            &mut instruction_data[Self::METADATA_DATA_OFFSET..],
            self.metadata_data,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data_size) },
        };

        invoke_signed(
            &instruction,
            &[
                self.metadata,
                self.update_authority,
                self.mint,
                self.mint_authority,
            ],
            signers,
        )
    }
}
