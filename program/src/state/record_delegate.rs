use crate::{
    constants::CLOSE_ACCOUNT_DISCRIMINATOR,
    utils::{resize_account, ByteReader, ByteWriter},
};
use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

pub const DISCRIMINATOR_OFFSET: usize = 0;
pub const RECORD_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
pub const UPDATE_AUTHORITY_OFFSET: usize = RECORD_OFFSET + size_of::<Pubkey>();
pub const FREEZE_AUTHORITY_OFFSET: usize = UPDATE_AUTHORITY_OFFSET + size_of::<Pubkey>();
pub const TRANSFER_AUTHORITY_OFFSET: usize = FREEZE_AUTHORITY_OFFSET + size_of::<Pubkey>();
pub const BURN_AUTHORITY_OFFSET: usize = TRANSFER_AUTHORITY_OFFSET + size_of::<Pubkey>();
pub const AUTHORITY_PROGRAM_OFFSET: usize = BURN_AUTHORITY_OFFSET + size_of::<Pubkey>();

#[repr(C)]
pub struct RecordAuthorityDelegate {
    pub record: Pubkey,
    pub update_authority: Pubkey,
    pub freeze_authority: Pubkey,
    pub transfer_authority: Pubkey,
    pub burn_authority: Pubkey,
    pub authority_program: Pubkey, // Optional, if not set, the authority program is [0; 32]
}

impl RecordAuthorityDelegate {
    pub const DISCRIMINATOR: u8 = 3;
    pub const MINIMUM_RECORD_SIZE: usize = size_of::<u8>() + size_of::<Pubkey>() * 6;

    /// # Safety
    ///
    /// This function uses static offsets to validate the account data.
    #[inline(always)]
    unsafe fn validate_account(account_info: &AccountInfo) -> Result<(), ProgramError> {
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }
        Ok(())
    }

    /// # Safety
    ///
    /// This function uses static offsets to validate the account data and discriminator.
    #[inline(always)]
    unsafe fn validate_account_and_discriminator(account_info: &AccountInfo) -> Result<(), ProgramError> {
        Self::validate_account(account_info)?;
        let data = account_info.try_borrow_data()?;
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    /// # Safety
    ///
    /// This function uses static offsets to check authority at a specific offset.
    #[inline(always)]
    unsafe fn check_authority_at_offset(
        account_info: &AccountInfo,
        authority: &Pubkey,
        offset: usize,
    ) -> Result<(), ProgramError> {
        Self::validate_account_and_discriminator(account_info)?;
        let data = account_info.try_borrow_data()?;
        if authority.ne(&data[offset..offset + size_of::<Pubkey>()]) {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn check_update_authority(
        account_info: &AccountInfo,
        authority: &Pubkey,
    ) -> Result<(), ProgramError> {
        unsafe { Self::check_authority_at_offset(account_info, authority, UPDATE_AUTHORITY_OFFSET) }
    }

    #[inline(always)]
    pub fn check_freeze_authority(
        account_info: &AccountInfo,
        authority: &Pubkey,
    ) -> Result<(), ProgramError> {
        unsafe { Self::check_authority_at_offset(account_info, authority, FREEZE_AUTHORITY_OFFSET) }
    }

    #[inline(always)]
    pub fn check_transfer_authority(
        account_info: &AccountInfo,
        authority: &Pubkey,
    ) -> Result<(), ProgramError> {
        unsafe { Self::check_authority_at_offset(account_info, authority, TRANSFER_AUTHORITY_OFFSET) }
    }

    #[inline(always)]
    pub fn check_burn_authority_and_close_delegate(
        account_info: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        unsafe {
            Self::check_authority_at_offset(account_info, authority.key(), BURN_AUTHORITY_OFFSET)?;
            Self::delete_record_delegate_unchecked(account_info, authority)
        }
    }

    /// # Safety
    ///
    /// This function uses static offsets to set the first byte of this account to the CLOSE_ACCOUNT_DISCRIMINATOR.
    #[inline(always)]
    pub unsafe fn delete_record_delegate_unchecked(
        record_delegate: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        // Update the Discriminator to CLOSE_ACCOUNT_DISCRIMINATOR to prevent reinitialization
        {
            let mut data_ref = record_delegate.try_borrow_mut_data()?;
            data_ref[DISCRIMINATOR_OFFSET] = CLOSE_ACCOUNT_DISCRIMINATOR;
        }

        // Resize the account to 1 byte
        resize_account(record_delegate, authority, 0, true)
    }

    #[inline(always)]
    pub fn delete_record_delegate(
        record_delegate: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        unsafe {
            Self::validate_account_and_discriminator(record_delegate)?;
            Self::delete_record_delegate_unchecked(record_delegate, authority)
        }
    }

    #[inline(always)]
    pub fn update(
        record_delegate: &AccountInfo,
        update_authority: Pubkey,
        freeze_authority: Pubkey,
        transfer_authority: Pubkey,
        burn_authority: Pubkey,
        authority_program: Pubkey,
    ) -> Result<(), ProgramError> {
        unsafe {
            Self::validate_account_and_discriminator(record_delegate)?;
            let mut data = record_delegate.try_borrow_mut_data()?;

            ByteWriter::write_with_offset(&mut data, UPDATE_AUTHORITY_OFFSET, update_authority)?;
            ByteWriter::write_with_offset(&mut data, FREEZE_AUTHORITY_OFFSET, freeze_authority)?;
            ByteWriter::write_with_offset(&mut data, TRANSFER_AUTHORITY_OFFSET, transfer_authority)?;
            ByteWriter::write_with_offset(&mut data, BURN_AUTHORITY_OFFSET, burn_authority)?;
            ByteWriter::write_with_offset(&mut data, AUTHORITY_PROGRAM_OFFSET, authority_program)?;

            Ok(())
        }
    }

    /// # Safety
    ///
    /// This function uses static offsets to deserialize the statically sized portions of this account.
    #[inline(always)]
    pub unsafe fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        let discriminator: u8 = ByteReader::read_with_offset(data, DISCRIMINATOR_OFFSET)?;

        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            record: ByteReader::read_with_offset(data, RECORD_OFFSET)?,
            update_authority: ByteReader::read_with_offset(data, UPDATE_AUTHORITY_OFFSET)?,
            freeze_authority: ByteReader::read_with_offset(data, FREEZE_AUTHORITY_OFFSET)?,
            transfer_authority: ByteReader::read_with_offset(data, TRANSFER_AUTHORITY_OFFSET)?,
            burn_authority: ByteReader::read_with_offset(data, BURN_AUTHORITY_OFFSET)?,
            authority_program: ByteReader::read_with_offset(data, AUTHORITY_PROGRAM_OFFSET)?,
        })
    }

    #[inline(always)]
    pub fn from_bytes_checked(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        unsafe {
            Self::validate_account(account_info)?;

            #[cfg(not(feature = "perf"))]
            if account_info.data_len() < Self::MINIMUM_RECORD_SIZE {
                return Err(ProgramError::InvalidAccountData);
            }

            Self::from_bytes(account_info.try_borrow_data()?.as_ref())
        }
    }

    /// # Safety
    ///
    /// This function uses static offsets to serialize the statically sized portions of this account.
    #[inline(always)]
    pub unsafe fn initialize(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        let mut data = account_info.try_borrow_mut_data()?;

        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;
        ByteWriter::write_with_offset(&mut data, RECORD_OFFSET, self.record)?;
        ByteWriter::write_with_offset(&mut data, UPDATE_AUTHORITY_OFFSET, self.update_authority)?;
        ByteWriter::write_with_offset(&mut data, FREEZE_AUTHORITY_OFFSET, self.freeze_authority)?;
        ByteWriter::write_with_offset(&mut data, TRANSFER_AUTHORITY_OFFSET, self.transfer_authority)?;
        ByteWriter::write_with_offset(&mut data, BURN_AUTHORITY_OFFSET, self.burn_authority)?;
        ByteWriter::write_with_offset(&mut data, AUTHORITY_PROGRAM_OFFSET, self.authority_program)?;

        Ok(())
    }

    #[inline(always)]
    pub fn initialize_checked(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        unsafe {
            Self::validate_account(account_info)?;

            if account_info.data_len() < Self::MINIMUM_RECORD_SIZE {
                return Err(ProgramError::InvalidAccountData);
            }

            Self::initialize(self, account_info)
        }
    }
}
