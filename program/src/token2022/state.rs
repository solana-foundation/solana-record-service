use crate::token2022::constants::{TOKEN_2022_PROGRAM_ID, TOKEN_IS_FROZEN_FLAG};
use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
const MINT_DISCRIMINATOR: u8 = 0x01;
const TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;

#[repr(C)]
pub struct Mint<'info> {
    pub raw_data: &'info [u8],
}

impl<'info> Mint<'info> {
    pub fn check_program_id(account_info: &AccountInfo) -> Result<(), ProgramError> {
        if unsafe { account_info.owner().ne(&TOKEN_2022_PROGRAM_ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    /// # Safety
    /// Token Program ID is not checked
    pub unsafe fn check_discriminator_unchecked(data: &[u8]) -> Result<(), ProgramError> {
        if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&MINT_DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    pub fn check_initialized(account_info: &AccountInfo) -> Result<bool, ProgramError> {
        if unsafe { account_info.owner().ne(&TOKEN_2022_PROGRAM_ID) } {
            return Ok(false);
        }

        let data = account_info.try_borrow_data()?;

        if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&MINT_DISCRIMINATOR) {
            return Ok(false);
        }

        Ok(true)
    }
}

const TOKEN_MINT_OFFSET: usize = 0;
const TOKEN_OWNER_OFFSET: usize = TOKEN_MINT_OFFSET + size_of::<Pubkey>();
const TOKEN_IS_FROZEN_OFFSET: usize =
    TOKEN_OWNER_OFFSET + size_of::<Pubkey>() + size_of::<u64>() + size_of::<u32>() + size_of::<Pubkey>();

#[repr(C)]
pub struct Token<'info> {
    pub raw_data: &'info [u8],
}

impl<'info> Token<'info> {
    pub fn check_program_id(account_info: &AccountInfo) -> Result<(), ProgramError> {
        if unsafe { account_info.owner().ne(&TOKEN_2022_PROGRAM_ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    /// # Safety
    /// Token Program ID is not checked
    pub unsafe fn check_discriminator_unchecked(data: &[u8]) -> Result<(), ProgramError> {
        if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&TOKEN_ACCOUNT_DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    /// # Safety
    /// Token Program ID is not checked
    pub unsafe fn get_mint_address_unchecked(data: &[u8]) -> Result<Pubkey, ProgramError> {
        Ok(
            data[TOKEN_MINT_OFFSET..TOKEN_MINT_OFFSET + size_of::<Pubkey>()]
                .try_into()
                .unwrap(),
        )
    }

    /// # Safety
    /// Token Program ID is not checked
    pub unsafe fn get_owner_unchecked(data: &[u8]) -> Result<Pubkey, ProgramError> {
        Ok(
            data[TOKEN_OWNER_OFFSET..TOKEN_OWNER_OFFSET + size_of::<Pubkey>()]
                .try_into()
                .unwrap(),
        )
    }

    /// # Safety
    /// Token Program ID is not checked
    pub unsafe fn get_is_frozen_unchecked(data: &[u8]) -> Result<bool, ProgramError> {
        Ok(data[TOKEN_IS_FROZEN_OFFSET].eq(&TOKEN_IS_FROZEN_FLAG))
    }
}
