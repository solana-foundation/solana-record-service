use super::RecordAuthorityDelegate;
use crate::utils::{resize_account, ByteReader, ByteWriter};
use core::{mem::size_of, str};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{try_find_program_address, Pubkey},
};

/// Maximum size allowed for a record account
pub const MAX_RECORD_SIZE: usize = 1024 * 1024; // 1MB

const DISCRIMINATOR_OFFSET: usize = 0;
const CLASS_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
const OWNER_OFFSET: usize = CLASS_OFFSET + size_of::<Pubkey>();
const IS_FROZEN_OFFSET: usize = OWNER_OFFSET + size_of::<Pubkey>();
const HAS_AUTHORITY_EXTENSION_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();
const EXPIRY_OFFSET: usize = HAS_AUTHORITY_EXTENSION_OFFSET + size_of::<bool>();
const NAME_LEN_OFFSET: usize = EXPIRY_OFFSET + size_of::<i64>();

#[repr(C)]
pub struct Record<'info> {
    /// The class this record belongs to
    pub class: Pubkey,
    /// The owner of this record
    pub owner: Pubkey,
    /// Whether the record is frozen
    pub is_frozen: bool,
    /// Flag indicating if authority extension exists
    pub has_authority_extension: bool,
    /// Optional expiration timestamp, if not set, the expiry is [0; 8]
    pub expiry: i64,
    /// The record name/key
    pub name: &'info str,
    /// The record's data content
    pub data: &'info str,
}

impl<'info> Record<'info> {
    /// The discriminator byte used to identify this account type
    pub const DISCRIMINATOR: u8 = 2;

    /// Minimum size required for a valid record account
    pub const MINIMUM_CLASS_SIZE: usize = size_of::<u8>()
        + size_of::<Pubkey>() * 2
        + size_of::<bool>() * 2
        + size_of::<i64>()
        + size_of::<u8>();

    pub fn check_authority(
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
        if authority.ne(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn check_authority_or_delegate(
        record: &AccountInfo,
        authority: &Pubkey,
        delegate: Option<&AccountInfo>,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { record.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = record.try_borrow_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if authority is the owner
        if authority.eq(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Ok(());
        }

        // If not owner, check delegate
        let has_authority_extension = data[HAS_AUTHORITY_EXTENSION_OFFSET] == 1;
        if !has_authority_extension {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let delegate = delegate.ok_or(ProgramError::MissingRequiredSignature)?;
        let seeds = [b"authority", record.key().as_ref()];
        let (derived_delegate, _) =
            try_find_program_address(&seeds, &crate::ID).ok_or(ProgramError::InvalidArgument)?;

        if derived_delegate != *delegate.key() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let extension = RecordAuthorityDelegate::from_bytes_checked(delegate)?;

        if extension.update_authority != *authority {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn update_is_frozen(
        record: &'info AccountInfo,
        is_frozen: bool,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { record.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let mut data = record.try_borrow_mut_data()?;

        // Check if frozen is the same
        if data[IS_FROZEN_OFFSET] == is_frozen as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update the frozen status
        data[IS_FROZEN_OFFSET] = is_frozen as u8;

        Ok(())
    }

    pub fn update_owner(record: &'info AccountInfo, new_owner: Pubkey) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { record.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let mut data = record.try_borrow_mut_data()?;

        // Check if frozen
        if data[IS_FROZEN_OFFSET] == 1u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if new owner is the same as the current owner
        if new_owner.eq(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update the owner
        data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()].clone_from_slice(&new_owner);

        Ok(())
    }

    pub fn update_data(
        record: &'info AccountInfo,
        authority: &'info AccountInfo,
        data: &'info str,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { record.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the name length
        let name_len = {
            let data_ref = record.try_borrow_data()?;
            data_ref[NAME_LEN_OFFSET] as usize
        };

        // Calculate the new size
        let offset = name_len + NAME_LEN_OFFSET + size_of::<u8>();
        let current_len = record.data_len();
        let new_len = offset + data.len();

        // Check if we need to resize, if so, resize the account
        if new_len != current_len {
            resize_account(record, authority, new_len, new_len < current_len)?;
        }

        // Update the data
        {
            let mut data_ref = record.try_borrow_mut_data()?;

            // Check the discriminator
            if data_ref[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
                return Err(ProgramError::InvalidAccountData);
            }

            // Update the data
            let data_buffer = unsafe {
                core::slice::from_raw_parts_mut(data_ref.as_mut_ptr().add(offset), data.len())
            };
            data_buffer.clone_from_slice(data.as_bytes());
        }

        Ok(())
    }

    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        // Check discriminator
        let discriminator: u8 = ByteReader::read_with_offset(data, DISCRIMINATOR_OFFSET)?;

        // Check discriminator
        if discriminator.ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize class
        let class: Pubkey = ByteReader::read_with_offset(data, CLASS_OFFSET)?;

        // Deserialize owner
        let owner: Pubkey = ByteReader::read_with_offset(data, OWNER_OFFSET)?;

        // Deserialize is_frozen
        let is_frozen: bool = ByteReader::read_with_offset(data, IS_FROZEN_OFFSET)?;

        // Deserialize has_authority_extension
        let has_authority_extension: bool =
            ByteReader::read_with_offset(data, HAS_AUTHORITY_EXTENSION_OFFSET)?;

        // Deserialize expiry
        let expiry: i64 = ByteReader::read_with_offset(data, EXPIRY_OFFSET)?;

        // Deserialize variable length data
        let mut variable_data: ByteReader<'info> =
            ByteReader::new_with_offset(data, NAME_LEN_OFFSET);

        // Deserialize name
        let name: &'info str = variable_data.read_str_with_length()?;

        // Deserialize data
        let data_content: &'info str = variable_data.read_str(variable_data.remaining_bytes())?;

        Ok(Self {
            class,
            owner,
            is_frozen,
            has_authority_extension,
            expiry,
            name,
            data: data_content,
        })
    }

    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        // Calculate required space
        let required_space = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.data.len();

        if account_info.data_len() < required_space {
            return Err(ProgramError::InvalidAccountData);
        }

        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Prevent reinitialization
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Write our discriminator
        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;

        // Write our class
        ByteWriter::write_with_offset(&mut data, CLASS_OFFSET, self.class)?;

        // Write our owner
        ByteWriter::write_with_offset(&mut data, OWNER_OFFSET, self.owner)?;

        // Set is_frozen
        ByteWriter::write_with_offset(&mut data, IS_FROZEN_OFFSET, self.is_frozen)?;

        // Set has_authority_extension
        ByteWriter::write_with_offset(
            &mut data,
            HAS_AUTHORITY_EXTENSION_OFFSET,
            self.has_authority_extension,
        )?;

        // Write expiry if present
        ByteWriter::write_with_offset(&mut data, EXPIRY_OFFSET, self.expiry)?;

        // Write variable length data
        let mut variable_data = ByteWriter::new_with_offset(&mut data, NAME_LEN_OFFSET);

        // Write name with length
        variable_data.write_str_with_length(self.name)?;

        // Write data
        variable_data.write_str(self.data)?;

        Ok(())
    }
}
