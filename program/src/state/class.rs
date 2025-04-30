use core::{mem::size_of, str};

use pinocchio::{account_info::{AccountInfo, RefMut}, log::sol_log, program_error::ProgramError, pubkey::Pubkey};

use crate::utils::resize_account;

#[repr(C)]
pub struct Class<'info> {
    pub authority: Pubkey,                  // The authority that controls this class
    pub is_permissioned: bool,              // Whether creating records is permissioned or not
    pub is_frozen: bool,                    // Whether the class is frozen or not
    pub name: &'info str,                   // Human-readable name for the class
    pub metadata: &'info str,               // Optional metadata about the class
}

pub const NAME_LEN_OFFSET: usize = size_of::<u8>() + size_of::<Pubkey>() + size_of::<bool>() + size_of::<bool>();

impl<'info> Class<'info> {
    pub const DISCRIMINATOR: u8 = 1;
    pub const MINIMUM_CLASS_SIZE: usize = 1 // discriminator
        + 32                                // authority
        + 1                                 // is_permissionless
        + 1                                 // is_frozen
        + 1;                                // name_len  

    /// Validates a class account.
    /// 
    /// This method performs basic account validation:
    /// 1. Verifies the account is owned by the program
    /// 2. Verifies the account has the correct discriminator
    /// 
    /// # Arguments
    /// 
    /// * `account_info` - The account info to validate
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If all checks pass
    /// * `Err(ProgramError)` - If any check fails
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::IllegalOwner` - If account is not owned by the program
    /// * `ProgramError::InvalidAccountData` - If discriminator is incorrect
    pub fn check(data: &mut [u8], authority: &Pubkey) -> Result<(), ProgramError> {
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
    
    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::MINIMUM_CLASS_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        if data[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        let authority: Pubkey = data[1..33].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let is_permissioned: bool = data[33] == 1;
        
        let is_frozen: bool = data[34] == 1;

        let mut offset = size_of::<u8>() + size_of::<Pubkey>() + size_of::<bool>() + size_of::<u8>() + size_of::<Pubkey>();

        let name_len = data[offset] as usize;
        
        offset += 1;

        if data.len() < offset + name_len {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let name: &'info str = str::from_utf8(
            &data[offset..offset + name_len]
        ).map_err(|_| ProgramError::InvalidAccountData)?;

        offset += name_len;

        let metadata: &'info str = str::from_utf8(
            &data[offset..]
        ).map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(Self {
            authority,
            is_permissioned,
            is_frozen,
            name,
            metadata,
        })
    }

    pub fn update_is_permissioned(&mut self, is_permissioned: bool) -> Result<(), ProgramError> {
        // Update is_permissioned
        self.is_permissioned = is_permissioned;

        Ok(())
    }

    pub fn update_is_frozen(&mut self, is_frozen: bool) -> Result<(), ProgramError> {
        // Update is_frozen
        self.is_frozen = is_frozen;

        Ok(())
    }

    pub fn update_metadata_old(&mut self, metadata: &'info str) -> Result<(), ProgramError> {
        // Update metadata
        self.metadata = metadata;
        Ok(())
    }

    pub fn update_metadata(account: &'info AccountInfo, authority: &'info AccountInfo, metadata: &'info str) -> Result<(), ProgramError> {
        // Get our Class account
        let mut data_ref = Class::borrow_data_checked(account, authority.key())?;

        // Get metadata offset
        let offset = data_ref[NAME_LEN_OFFSET] as usize + NAME_LEN_OFFSET + size_of::<u8>();

        // Calculate current and len length of account
        let current_len = data_ref.len();
        let new_len = data_ref[NAME_LEN_OFFSET] as usize + NAME_LEN_OFFSET + size_of::<u8>() + metadata.len();

        // Resize Class account
        resize_account(
            account, 
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

    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        // Calculate required space
        let required_space = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();
        
        // Verify account has enough space
        if account_info.data_len() != required_space {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Prevent reinitialization
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Write our discriminator
        data[0] = Self::DISCRIMINATOR;
        
        // Write our authority
        data[1..33].clone_from_slice(&self.authority);

        // Set is_permissioned to false
        data[33] = self.is_permissioned as u8;

        // Set is_frozen to false
        data[34] = self.is_frozen as u8;

        // Write the length of our name or error if overflowed
        data[35] = self.name.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Write our name
        data[36..36 + self.name.len()].clone_from_slice(self.name.as_bytes());

        sol_log("Got here");
        // Add name length to our offset to write metadata
        if !self.metadata.is_empty() {
            // Write metadata if exists
            data[36 + self.name.len()..].clone_from_slice(self.metadata.as_bytes());
        }

        Ok(())
    }

    /// Validates that an account has authority over this class.
    /// 
    /// # Arguments
    /// 
    /// * `authority` - The account info to validate as the authority
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the account is the authority
    /// * `Err(ProgramError)` - If the account is not the authority
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidAccountData` - If the account is not the authority
    pub fn validate_authority(&self, authority: &AccountInfo) -> Result<(), ProgramError> {
        if self.authority != *authority.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}