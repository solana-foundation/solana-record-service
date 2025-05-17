use core::mem::size_of;
use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    log::sol_log_data,
    program::invoke_signed,
    ProgramResult,
};

use crate::{
    constants::{MAX_METADATA_LEN, MAX_NAME_LEN, SRS_TICKER},
    token2022::constants::TOKEN_2022_PROGRAM_ID,
    utils::{write_bytes, UNINIT_BYTE},
};

/// Initializes a Mint Close Authority.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a close authority for.
pub struct InitializeMetadata<'a> {
    /// Metadata Account.
    pub metadata: &'a AccountInfo,
    /// Update Authority Account.
    pub update_authority: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Mint Authority Account.
    pub mint_authority: &'a AccountInfo,
    /// Token name.
    pub name: &'a str,
    /// Token symbol.
    pub symbol: &'a str,
    /// Token metadata URI.
    pub uri: &'a str,
}

pub struct Metadata<'a> {
    pub name: &'a str,
    pub symbol: &'a str,
    pub uri: &'a str,
    pub additional_metadata: &'a [&'a [u8]],
}

#[allow(clippy::len_without_is_empty)]
impl Metadata<'_> {
    pub const FIXED_HEADER_LEN: usize = size_of::<u16>() * 2 + // instruction_id_len + data_len
        size_of::<u32>() * 4; // name_len_size, symbol_len_size, uri_len_size, additional_metadata_len_size

    pub fn len(&self) -> u64 {
        Self::FIXED_HEADER_LEN as u64
            + self.name.len() as u64
            + self.symbol.len() as u64
            + self.uri.len() as u64
            + self
                .additional_metadata
                .iter()
                .map(|entry| size_of::<u32>() as u64 + entry.len() as u64)
                .sum::<u64>()
    }
}

impl InitializeMetadata<'_> {
    pub const DISCRIMINATOR: [u8; 8] = [0xd2, 0xe1, 0x1e, 0xa2, 0x58, 0xb8, 0x4d, 0x8d];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const NAME_LENGTH_OFFSET: usize = Self::DISCRIMINATOR_OFFSET + size_of::<u64>();
    const NAME_OFFSET: usize = Self::NAME_LENGTH_OFFSET + size_of::<u32>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.update_authority.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // instruction data
        // - [0]: instruction discriminator (8 bytes, [u8;8])
        // - [8..12]: name length (u32)
        // - [12..12+name.len()]: name bytes
        // - [..]: symbol length (u32)
        // - [..]: symbol bytes
        // - [..]: uri length (u32)
        // - [..]: uri bytes
        let mut instruction_data = [UNINIT_BYTE;
            Self::DISCRIMINATOR.len()
                + size_of::<u32>() * 3
                + MAX_NAME_LEN
                + SRS_TICKER.len()
                + MAX_METADATA_LEN];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &Self::DISCRIMINATOR,
        );

        write_bytes(
            &mut instruction_data[Self::NAME_LENGTH_OFFSET..],
            &(self.name.len() as u32).to_le_bytes(),
        );

        // Switch to dynamic offsets
        let mut offset = Self::NAME_OFFSET + self.name.len();

        write_bytes(
            &mut instruction_data[Self::NAME_OFFSET..],
            self.name.as_bytes(),
        );

        // Write symbol length
        write_bytes(
            &mut instruction_data[offset..],
            &(self.symbol.len() as u32).to_le_bytes(),
        );
        offset += size_of::<u32>();

        // Write symbol
        write_bytes(&mut instruction_data[offset..], self.symbol.as_bytes());
        offset += self.symbol.len();

        // Write URI length
        write_bytes(
            &mut instruction_data[offset..],
            &(self.uri.len() as u32).to_le_bytes(),
        );
        offset += size_of::<u32>();

        // Write URI
        write_bytes(&mut instruction_data[offset..], self.uri.as_bytes());

        offset += self.uri.len();

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, offset) },
        };

        sol_log_data(&[instruction.data]);

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
