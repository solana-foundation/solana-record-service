use core::{str, mem::size_of};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::{Pubkey, try_find_program_address}};
use super::RecordAuthorityDelegate;
use crate::utils::{resize_account, ByteReader, ByteWriter};

/// Maximum size allowed for a record account
pub const MAX_RECORD_SIZE: usize = 1024 * 1024; // 1MB

const NAME_LEN_OFFSET: usize = size_of::<u8>() + size_of::<Pubkey>() * 2 + size_of::<bool>() * 2 + size_of::<u8>() + size_of::<i64>();

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
    /// Optional expiration timestamp
    pub expiry: Option<i64>,
    /// The record name/key
    pub name: &'info str,
    /// The record's data content
    pub data: &'info str,
}

impl<'info> Record<'info> {
    /// The discriminator byte used to identify this account type
    pub const DISCRIMINATOR: u8 = 2;
    
    /// Minimum size required for a valid record account
    pub const MINIMUM_CLASS_SIZE: usize = size_of::<u8>() * 3 
        + size_of::<bool>() * 2 
        + size_of::<Pubkey>() * 2 
        + size_of::<i64>();

    pub fn check_authority(data: &[u8], authority: &Pubkey) -> Result<(), ProgramError> {
        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority.ne(&data[33..65]) {
            return Err(ProgramError::MissingRequiredSignature)
        }

        Ok(())
    }

    pub fn check_authority_or_delegate(record: &AccountInfo, authority: &Pubkey, delegate: Option<&AccountInfo>) -> Result<(), ProgramError> {
        let data = record.try_borrow_data()?;

        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if authority is the owner
        if authority.eq(&data[33..65]) {
            return Ok(());
        }

        // If not owner, check delegate
        let has_authority_extension = data[66] == 1;
        if !has_authority_extension {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let delegate = delegate.ok_or(ProgramError::MissingRequiredSignature)?;
        let seeds = [b"authority", record.key().as_ref()];
        let (derived_delegate, _) = try_find_program_address(&seeds, &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?;

        if derived_delegate != *delegate.key() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let delegate_data = delegate.try_borrow_data()?;
        let extension = RecordAuthorityDelegate::from_bytes(&delegate_data)?;
        
        if extension.update_authority != *authority {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn update_is_frozen(record: &'info AccountInfo, is_frozen: bool) -> Result<(), ProgramError> {
        let mut data = record.try_borrow_mut_data()?;

        if data[67] == is_frozen as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        data[67] = is_frozen as u8;

        Ok(())
    }

    pub fn update_owner(record: &'info AccountInfo, new_owner: Pubkey) -> Result<(), ProgramError> {
        let mut data = record.try_borrow_mut_data()?;

        if data[67] == 1 as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        if new_owner.eq(&data[33..65]) {
            return Err(ProgramError::InvalidAccountData);
        }

        data[33..65].clone_from_slice(&new_owner);

        Ok(())
    }

    pub fn update_data(record: &'info AccountInfo, authority: &'info AccountInfo, data: &'info str) -> Result<(), ProgramError> {
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
            resize_account(
                record,
                authority,
                new_len,
                new_len < current_len
            )?;
        }

        // Update the data
        {
            let mut data_ref = record.try_borrow_mut_data()?;
            let data_buffer = unsafe {
                core::slice::from_raw_parts_mut(
                    data_ref.as_mut_ptr().add(offset),
                    data.len()
                )
            };
            data_buffer.clone_from_slice(data.as_bytes());
        }

        Ok(())
    }

    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        // Check account data has minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(data, Self::MINIMUM_CLASS_SIZE)?;

        // Deserialize discriminator
        let discriminator: u8 = data.read()?;

        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }   

        // Deserialize class
        let class: Pubkey = data.read()?;

        // Deserialize owner
        let owner: Pubkey = data.read()?;

        // Deserialize is_frozen
        let is_frozen: bool = data.read()?;

        // Deserialize has_authority_extension
        let has_authority_extension: bool = data.read()?;

        // Deserialize expiry
        let expiry: Option<i64> = data.read_optional()?;

        // Deserialize name
        let name: &'info str = data.read_str_with_length()?;

        // Deserialize data
        let data_content: &'info str = data.read_str(data.remaining_bytes())?;

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

        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Prevent reinitialization
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Create a ByteWriter
        let mut writer = ByteWriter::new_with_minimum_size(&mut data, required_space)?;

        // Write our discriminator
        writer.write(Self::DISCRIMINATOR)?;
        
        // Write our class
        writer.write(self.class)?;

        // Write our owner
        writer.write(self.owner)?;

        // Set is_frozen
        writer.write(self.is_frozen)?;

        // Set has_authority_extension
        writer.write(self.has_authority_extension)?;

        // Write expiry if present
        writer.write_optional(self.expiry)?;

        // Write name with length
        writer.write_str_with_length(self.name)?;

        // Write data
        writer.write_str(self.data)?;

        Ok(())
    }
}