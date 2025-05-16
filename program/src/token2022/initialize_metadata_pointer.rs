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

/// Initializes a Metadata Pointer.
///
/// ### Accounts:
///   0. `[WRITE]`  The mint account to initialize a close authority for.
pub struct InitializeMetadataPointer<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Authority Account.
    pub authority: &'a Pubkey,
    /// Metadata Address.
    pub metadata_address: &'a Pubkey,
}

impl InitializeMetadataPointer<'_> {
    const METADATA_POINTER_DISCRIMINATOR: u8 = 0x27;
    const METADATA_POINTER_INITIALIZE_DISCRIMINATOR: u8 = 0x00;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const METADATA_POINTER_DISCRIMINATOR_OFFSET: usize =
        Self::DISCRIMINATOR_OFFSET + size_of::<u8>();
    const METADATA_AUTHORITY_OFFSET: usize =
        Self::METADATA_POINTER_DISCRIMINATOR_OFFSET + size_of::<u8>();
    const METADATA_ADDRESS_OFFSET: usize = Self::METADATA_AUTHORITY_OFFSET + size_of::<Pubkey>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: metadata pointer instruction discriminator (1 byte, u8)
        // -  [2..34]: metadataAuthority (32 bytes, Pubkey)
        // -  [34..66]: metadataAddress (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 66];

        write_bytes(
            &mut instruction_data,
            &[Self::METADATA_POINTER_DISCRIMINATOR],
        );

        write_bytes(
            &mut instruction_data[Self::METADATA_POINTER_DISCRIMINATOR_OFFSET..],
            &[Self::METADATA_POINTER_INITIALIZE_DISCRIMINATOR],
        );

        write_bytes(
            &mut instruction_data[Self::METADATA_AUTHORITY_OFFSET..],
            self.authority,
        );

        write_bytes(
            &mut instruction_data[Self::METADATA_ADDRESS_OFFSET..],
            self.metadata_address,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
