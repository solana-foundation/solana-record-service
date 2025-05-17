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

/// Initialize a new mint.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
pub struct InitializeMint2<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Decimals.
    pub decimals: u8,
    /// Mint Authority.
    pub mint_authority: &'a Pubkey,
    /// Freeze Authority.
    pub freeze_authority: Option<&'a Pubkey>,
}

impl InitializeMint2<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const DECIMALS_OFFSET: usize = Self::DISCRIMINATOR_OFFSET + size_of::<u8>();
    const MINT_AUTHORITY_OFFSET: usize = Self::DECIMALS_OFFSET + size_of::<u8>();
    const HAS_FREEZE_AUTHORITY_OFFSET: usize = Self::MINT_AUTHORITY_OFFSET + size_of::<Pubkey>();
    const FREEZE_AUTHORITY_OFFSET: usize = Self::HAS_FREEZE_AUTHORITY_OFFSET + size_of::<u8>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        const DISCRIMINATOR: u8 = 0x14;

        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: decimals (1 byte, u8)
        // -  [2..34]: mint_authority (32 bytes, Pubkey)
        // -  [34]: freeze_authority presence flag (1 byte, u8)
        // -  [35..67]: freeze_authority (optional, 32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 67];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &[DISCRIMINATOR],
        );

        write_bytes(
            &mut instruction_data[Self::DECIMALS_OFFSET..],
            &[self.decimals],
        );

        write_bytes(
            &mut instruction_data[Self::MINT_AUTHORITY_OFFSET..],
            self.mint_authority,
        );

        // Set COption & freeze_authority at offset [34..67]
        if let Some(freeze_auth) = self.freeze_authority {
            write_bytes(
                &mut instruction_data[Self::HAS_FREEZE_AUTHORITY_OFFSET..],
                &[1],
            );
            write_bytes(
                &mut instruction_data[Self::FREEZE_AUTHORITY_OFFSET..],
                freeze_auth,
            );
        } else {
            write_bytes(
                &mut instruction_data[Self::HAS_FREEZE_AUTHORITY_OFFSET..],
                &[0],
            );
        }

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
