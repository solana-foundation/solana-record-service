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

/// Initialize a new group pointer.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
pub struct InitializeGroupPointer<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Authority Account.
    pub authority: &'a Pubkey,
    /// Group Address.
    pub group_address: &'a Pubkey,
}
impl InitializeGroupPointer<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    const DISCRIMINATOR_OFFSET: usize = 0;
    const GROUP_POINTER_DISCRIMINATOR_OFFSET: usize = Self::DISCRIMINATOR_OFFSET + size_of::<u8>();
    const GROUP_AUTHORITY_OFFSET: usize =
        Self::GROUP_POINTER_DISCRIMINATOR_OFFSET + size_of::<u8>();
    const GROUP_ADDRESS_OFFSET: usize = Self::GROUP_AUTHORITY_OFFSET + size_of::<Pubkey>();

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        const GROUP_POINTER_DISCRIMINATOR: u8 = 0x28;
        const GROUP_POINTER_INITIALIZE_DISCRIMINATOR: u8 = 0x00;

        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: group pointer instruction discriminator (1 byte, u8)
        // -  [2..34]: groupAuthority (32 bytes, Pubkey)
        // -  [34..66]: groupAddress (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 66];

        write_bytes(
            &mut instruction_data[Self::DISCRIMINATOR_OFFSET..],
            &[GROUP_POINTER_DISCRIMINATOR],
        );

        write_bytes(
            &mut instruction_data[Self::GROUP_POINTER_DISCRIMINATOR_OFFSET..],
            &[GROUP_POINTER_INITIALIZE_DISCRIMINATOR],
        );

        write_bytes(
            &mut instruction_data[Self::GROUP_AUTHORITY_OFFSET..],
            self.authority,
        );

        write_bytes(
            &mut instruction_data[Self::GROUP_ADDRESS_OFFSET..],
            self.group_address,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
