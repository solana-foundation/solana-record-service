use core::{mem::size_of, str};

use pinocchio::{account_info::{AccountInfo, RefMut}, program_error::ProgramError, pubkey::Pubkey};

use crate::utils::{resize_account, ByteReader, ByteWriter};

#[repr(C)]
pub struct Class<'info> {
    /// The authority that controls this class
    pub authority: Pubkey,
    /// Whether creating records is permissioned or not
    pub is_permissioned: bool,
    /// Whether the class is frozen or not
    pub is_frozen: bool,
    /// Human-readable name for the class
    pub name: &'info str,
    /// Optional metadata about the class
    pub metadata: &'info str,
}

const NAME_LEN_OFFSET: usize = size_of::<u8>() + size_of::<Pubkey>() + size_of::<bool>() + size_of::<bool>();

impl<'info> Class<'info> {
    pub const DISCRIMINATOR: u8 = 1;
    pub const MINIMUM_CLASS_SIZE: usize = 1 // discriminator
        + 32                                // authority
        + 1                                 // is_permissionless
        + 1                                 // is_frozen
        + 1;                                // name_len  

    
    pub fn check_authority(data: &mut [u8], authority: &Pubkey) -> Result<(), ProgramError> {
        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority.ne(&data[1..33]) {
            return Err(ProgramError::MissingRequiredSignature)
        }

        Ok(())
    }

    pub fn check_permission(data: &[u8], authority: Option<&AccountInfo>) -> Result<(), ProgramError> {
        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check Permission
        if data[34] == 1 {
            match authority {
                Some(auth) => {
                    if auth.key().ne(&data[33..65]) {
                        return Err(ProgramError::MissingRequiredSignature);
                    }
                }
                None => return Err(ProgramError::MissingRequiredSignature),
            }
        }

        Ok(())
    }

    pub fn borrow_data_checked(account: &'info AccountInfo, authority: &Pubkey) -> Result<RefMut<'info, [u8]>, ProgramError> {
        let data_ref: RefMut<'info, [u8]> = account.try_borrow_mut_data().map_err(|_| ProgramError::InvalidAccountData)?;
    
        if data_ref[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
    
        if authority.ne(&data_ref[1..33]) {
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        Ok(data_ref)
    }

    pub fn update_is_permissioned(class: &'info AccountInfo, authority: &'info AccountInfo, is_permissioned: bool) -> Result<(), ProgramError> {
        let mut data_ref = Class::borrow_data_checked(class, authority.key())?;

        if data_ref[33] == is_permissioned as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update is_permissioned
        data_ref[33] = is_permissioned as u8;

        Ok(())
    }

    pub fn update_is_frozen(class: &'info AccountInfo, authority: &'info AccountInfo, is_frozen: bool) -> Result<(), ProgramError> {
        let mut data_ref = Class::borrow_data_checked(class, authority.key())?;

        if data_ref[34] == is_frozen as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update is_frozen
        data_ref[34] = is_frozen as u8;

        Ok(())
    }

    pub fn update_metadata(class: &'info AccountInfo, authority: &'info AccountInfo, metadata: &'info str) -> Result<(), ProgramError> {
        // Get our Class account
        let mut data_ref = Class::borrow_data_checked(class, authority.key())?;
        
        // Get metadata offset
        let offset = data_ref[NAME_LEN_OFFSET] as usize + NAME_LEN_OFFSET + size_of::<u8>();
        
        // Calculate current and len length of account
        let current_len = data_ref.len();
        let new_len = data_ref[NAME_LEN_OFFSET] as usize + NAME_LEN_OFFSET + size_of::<u8>() + metadata.len();

        // Resize Class account
        resize_account(
            class, 
            authority, 
            new_len, 
            new_len < current_len
        )?;

        // Create mutable metadata buffer
        let metadata_buffer = unsafe { 
            core::slice::from_raw_parts_mut(
            data_ref.as_mut_ptr().add(offset), 
                metadata.len()
            )
        };

        metadata_buffer.clone_from_slice(metadata.as_bytes());

        Ok(())
    }
    
    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        // Check ix data has minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(data, Self::MINIMUM_CLASS_SIZE)?;

        let discriminator: u8 = data.read()?;
        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize authority
        let authority: Pubkey = data.read()?;

        // Deserialize is_permissioned
        let is_permissioned: bool = data.read()?;

        // Deserialize is_frozen
        let is_frozen: bool = data.read()?;

        // Deserialize name
        let name = data.read_str_with_length()?;

        // Deserialize metadata
        let metadata = data.read_str(data.remaining_bytes())?;

        Ok(Self {
            authority,
            is_permissioned,
            is_frozen,
            name,
            metadata,
        })
    }

    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        // Calculate required space
        let required_space = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();

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
        
        // Write our authority
        writer.write(self.authority)?;

        // Set is_permissioned
        writer.write(self.is_permissioned)?;

        // Set is_frozen
        writer.write(self.is_frozen)?;

        // Write name with length
        writer.write_str_with_length(self.name)?;

        // Write metadata if exists
        if !self.metadata.is_empty() {
            writer.write_str(self.metadata)?;
        }

        Ok(())
    }
}