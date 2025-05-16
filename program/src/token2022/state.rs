use crate::token2022::constants::{
    TOKEN_2022_MINT_LEN, TOKEN_2022_PROGRAM_ID, TOKEN_IS_FROZEN_FLAG,
};
use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
#[repr(C)]
pub struct Mint<'info> {
    pub raw_data: &'info [u8],
}

impl<'info> Mint<'info> {
    pub fn check_program_id(account_info: &AccountInfo) -> Result<(), ProgramError> {
        if account_info.key() != &TOKEN_2022_PROGRAM_ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    pub unsafe fn check_discriminator_unchecked(data: &[u8]) -> Result<(), ProgramError> {
        if data.len().ne(&TOKEN_2022_MINT_LEN) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

const TOKEN_MINT_OFFSET: usize = 0;
const TOKEN_OWNER_OFFSET: usize = TOKEN_MINT_OFFSET + size_of::<Pubkey>();
const TOKEN_IS_FROZEN_OFFSET: usize =
    TOKEN_OWNER_OFFSET + size_of::<u64>() + size_of::<u32>() + size_of::<Pubkey>();

#[repr(C)]
pub struct Token<'info> {
    pub raw_data: &'info [u8],
}

impl<'info> Token<'info> {
    pub fn check_program_id(account_info: &AccountInfo) -> Result<(), ProgramError> {
        if account_info.key() != &TOKEN_2022_PROGRAM_ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    pub unsafe fn check_discriminator_unchecked(data: &[u8]) -> Result<(), ProgramError> {
        if data.len().eq(&TOKEN_2022_MINT_LEN) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    pub unsafe fn get_mint_address_unchecked(data: &[u8]) -> Result<Pubkey, ProgramError> {
        Ok(
            data[TOKEN_MINT_OFFSET..TOKEN_MINT_OFFSET + size_of::<Pubkey>()]
                .try_into()
                .unwrap(),
        )
    }

    pub unsafe fn get_owner_unchecked(data: &[u8]) -> Result<Pubkey, ProgramError> {
        Ok(
            data[TOKEN_OWNER_OFFSET..TOKEN_OWNER_OFFSET + size_of::<Pubkey>()]
                .try_into()
                .unwrap(),
        )
    }

    pub unsafe fn get_is_frozen_unchecked(data: &[u8]) -> Result<bool, ProgramError> {
        Ok(data[TOKEN_IS_FROZEN_OFFSET].eq(&TOKEN_IS_FROZEN_FLAG))
    }
}
