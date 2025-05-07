use crate::{constants::CLOSE_ACCOUNT_DISCRIMINATOR, utils::{resize_account, ByteReader, ByteWriter}};
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

    pub fn check_update_authority(
        account_info: &AccountInfo,
        authority: &Pubkey,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = account_info.try_borrow_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority
            .ne(&data[UPDATE_AUTHORITY_OFFSET..UPDATE_AUTHORITY_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn check_freeze_authority(
        account_info: &AccountInfo,
        authority: &Pubkey,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = account_info.try_borrow_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority
            .ne(&data[FREEZE_AUTHORITY_OFFSET..FREEZE_AUTHORITY_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn check_transfer_authority(
        account_info: &AccountInfo,
        authority: &Pubkey,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = account_info.try_borrow_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority
            .ne(&data[TRANSFER_AUTHORITY_OFFSET..TRANSFER_AUTHORITY_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn check_burn_authority_and_close_delegate(
        account_info: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = account_info.try_borrow_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority
            .key()
            .ne(&data[BURN_AUTHORITY_OFFSET..BURN_AUTHORITY_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Close Delegate
        unsafe { Self::delete_record_delegate_unchecked(account_info, authority)? };

        Ok(())
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

    pub fn delete_record_delegate(
        record_delegate: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { record_delegate.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = record_delegate.try_borrow_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // TODO: Try to get rid of this drop
        drop(data);

        unsafe { Self::delete_record_delegate_unchecked(record_delegate, authority)? };

        Ok(())
    }

    pub fn update(
        record_delegate: &AccountInfo,
        update_authority: Pubkey,
        freeze_authority: Pubkey,
        transfer_authority: Pubkey,
        burn_authority: Pubkey,
        authority_program: Pubkey
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { record_delegate.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let mut data = record_delegate.try_borrow_mut_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update the update authority
        ByteWriter::write_with_offset(&mut data, UPDATE_AUTHORITY_OFFSET, update_authority)?;

        // Update the freeze authority
        ByteWriter::write_with_offset(&mut data, FREEZE_AUTHORITY_OFFSET, freeze_authority)?;

        // Update the transfer authority
        ByteWriter::write_with_offset(&mut data, TRANSFER_AUTHORITY_OFFSET, transfer_authority)?;

        // Update the burn authority
        ByteWriter::write_with_offset(&mut data, BURN_AUTHORITY_OFFSET, burn_authority)?;

        // Update the burn authority
        ByteWriter::write_with_offset(&mut data, AUTHORITY_PROGRAM_OFFSET, authority_program)?;

        Ok(())
    }

    /// # Safety
    ///
    /// This function uses static offsets to deserialize the statically sized portions of this account.
    pub unsafe fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        let discriminator: u8 = ByteReader::read_with_offset(data, DISCRIMINATOR_OFFSET)?;

        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize record
        let record: Pubkey = ByteReader::read_with_offset(data, RECORD_OFFSET)?;

        // Deserialize update authority
        let update_authority: Pubkey = ByteReader::read_with_offset(data, UPDATE_AUTHORITY_OFFSET)?;

        // Deserialize freeze authority
        let freeze_authority: Pubkey = ByteReader::read_with_offset(data, FREEZE_AUTHORITY_OFFSET)?;

        // Deserialize transfer authority
        let transfer_authority: Pubkey =
            ByteReader::read_with_offset(data, TRANSFER_AUTHORITY_OFFSET)?;

        // Deserialize burn authority
        let burn_authority: Pubkey = ByteReader::read_with_offset(data, BURN_AUTHORITY_OFFSET)?;

        // Deserialize authority program
        let authority_program: Pubkey =
            ByteReader::read_with_offset(data, AUTHORITY_PROGRAM_OFFSET)?;

        Ok(Self {
            record,
            update_authority,
            freeze_authority,
            transfer_authority,
            burn_authority,
            authority_program,
        })
    }

    pub fn from_bytes_checked(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        #[cfg(not(feature = "perf"))]
        if account_info.data_len() < Self::MINIMUM_RECORD_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        unsafe { Self::from_bytes(account_info.try_borrow_data()?.as_ref()) }
    }

    /// # Safety
    ///
    /// This function uses static offsets to serialize the statically sized portions of this account.
    pub unsafe fn initialize(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Prevent reinitialization
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Write our discriminator
        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;

        // Write our record
        ByteWriter::write_with_offset(&mut data, RECORD_OFFSET, self.record)?;

        // Write our update authority
        ByteWriter::write_with_offset(&mut data, UPDATE_AUTHORITY_OFFSET, self.update_authority)?;

        // Write our freeze authority
        ByteWriter::write_with_offset(&mut data, FREEZE_AUTHORITY_OFFSET, self.freeze_authority)?;

        // Write our transfer authority
        ByteWriter::write_with_offset(
            &mut data,
            TRANSFER_AUTHORITY_OFFSET,
            self.transfer_authority,
        )?;

        // Write our burn authority
        ByteWriter::write_with_offset(&mut data, BURN_AUTHORITY_OFFSET, self.burn_authority)?;

        // Write our authority program
        ByteWriter::write_with_offset(&mut data, AUTHORITY_PROGRAM_OFFSET, self.authority_program)?;

        Ok(())
    }

    pub fn initialize_checked(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        #[cfg(not(feature = "perf"))]
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        if account_info.data_len() < Self::MINIMUM_RECORD_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        unsafe { Self::initialize(self, account_info) }
    }
}
