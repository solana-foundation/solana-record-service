use crate::token2022::constants::TOKEN_2022_PROGRAM_ID;
use core::mem::size_of;
use pinocchio::{error::ProgramError, AccountView, Address};

const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
const TOKEN_IS_FROZEN_FLAG: u8 = 2;
const MINT_DISCRIMINATOR: u8 = 0x01;
const TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;
const TOKEN_ACCOUNT_SUPPLY_OFFSET: usize = 36;

#[repr(C)]
pub struct Mint<'info> {
    pub raw_data: &'info [u8],
}

impl<'info> Mint<'info> {
    pub fn check_program_id(account_info: &AccountView) -> Result<(), ProgramError> {
        if !account_info.owned_by(&TOKEN_2022_PROGRAM_ID) {
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

    pub fn check_discriminator(account_info: &AccountView) -> Result<bool, ProgramError> {
        if !account_info.owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Ok(false);
        }

        let data = account_info.try_borrow()?;

        if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&MINT_DISCRIMINATOR) {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn get_supply(account_info: &AccountView) -> Result<u64, ProgramError> {
        if !account_info.owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountData);
        }

        let data = account_info.try_borrow()?;

        Ok(
            u64::from_le_bytes(
                data[TOKEN_ACCOUNT_SUPPLY_OFFSET..TOKEN_ACCOUNT_SUPPLY_OFFSET + size_of::<u64>()]
                    .try_into()
                    .unwrap()
            )
        )
    }
}

const TOKEN_MINT_OFFSET: usize = 0;
const TOKEN_OWNER_OFFSET: usize = TOKEN_MINT_OFFSET + size_of::<Address>();
const TOKEN_IS_FROZEN_OFFSET: usize = TOKEN_OWNER_OFFSET
    + size_of::<Address>()
    + size_of::<u64>()
    + size_of::<u32>()
    + size_of::<Address>();

#[repr(C)]
pub struct Token<'info> {
    pub raw_data: &'info [u8],
}

impl<'info> Token<'info> {
    pub fn check_program_id(account_info: &AccountView) -> Result<(), ProgramError> {
        if !account_info.owned_by(&TOKEN_2022_PROGRAM_ID) {
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
    pub unsafe fn get_owner_unchecked(data: &[u8]) -> Result<Address, ProgramError> {
        let owner: [u8; 32] = data[TOKEN_OWNER_OFFSET..TOKEN_OWNER_OFFSET + size_of::<Address>()].try_into().unwrap();
        Ok(Address::new_from_array(owner))
    }

    /// # Safety
    /// Token Program ID is not checked
    pub unsafe fn get_is_frozen_unchecked(data: &[u8]) -> Result<bool, ProgramError> {
        Ok(data[TOKEN_IS_FROZEN_OFFSET].eq(&TOKEN_IS_FROZEN_FLAG))
    }
}
