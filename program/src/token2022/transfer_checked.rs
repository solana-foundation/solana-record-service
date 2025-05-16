use core::{mem::size_of, slice::from_raw_parts};

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

/// Initializes a Mint Close Authority.
///
/// ### Accounts:
///  0. `[writable]` The source account.
///  1. `[]` The token mint.
///  2. `[writable]` The destination account.
///  3. `[signer]` The source account's owner/delegate.
///
/// ### Data:
///  0. amount (u64)
///  1. decimals (u8)
pub struct TransferChecked<'a> {
    /// Mint Account.
    pub source: &'a AccountInfo,
    pub mint: &'a AccountInfo,
    pub destination: &'a AccountInfo,
    pub authority: &'a AccountInfo,
    pub amount: u64,
    pub decimals: u8,
}

const DISCRIMINATOR_OFFSET: usize = 0;
const AMOUNT_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
const DECIMALS_OFFSET: usize = AMOUNT_OFFSET + size_of::<u64>();

impl TransferChecked<'_> {
    const DISCRIMINATOR: u8 = 0x0c;
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::writable(self.destination.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]: decimals (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 10];

        write_bytes(&mut instruction_data, &[Self::DISCRIMINATOR]);

        write_bytes(
            &mut instruction_data[AMOUNT_OFFSET..],
            &self.amount.to_le_bytes(),
        );

        write_bytes(&mut instruction_data[DECIMALS_OFFSET..], &[self.decimals]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.authority], signers)
    }
}
