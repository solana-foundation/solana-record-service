use core::{str, mem::size_of};

use pinocchio::{account_info::AccountInfo, log::sol_log_64, program_error::ProgramError, pubkey::Pubkey};

/// Represents a record that can be associated with a class.
/// The data layout is as follows:
/// - 1 byte: discriminator
/// - 32 bytes: class public key
/// - 32 bytes: owner public key
/// - 1 byte: is_frozen flag
/// - 1 byte: has_authority_extension flag
/// - 8 bytes: expiry timestamp (if has_authority_extension is true)
/// - 1 byte: name length
/// - N bytes: name string
/// - 1 byte: data length
/// - M bytes: data content
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
    /// This includes:
    /// - 1 byte for discriminator
    /// - 32 bytes for class
    /// - 32 bytes for owner
    /// - 1 byte for is_frozen
    /// - 1 byte for has_authority_extension
    /// - 8 bytes for expiry
    /// - 1 byte for name length
    /// - 1 byte for data length
    pub const MINIMUM_CLASS_SIZE: usize = 1 + 32 + 32 + 1 + 1 + 8 + 1 + 1;

    /// Deserializes a record from raw bytes
    /// 
    /// # Safety
    /// 
    /// This function performs unsafe operations to create slices from raw memory.
    /// The input data must be properly formatted and aligned.
    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::MINIMUM_CLASS_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let mut offset = 0;

        if data[offset] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        offset += size_of::<u8>();

        // Read class (32 bytes)
        let class: Pubkey = data[offset..offset + size_of::<Pubkey>()]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        offset += size_of::<Pubkey>();

        // Read owner (32 bytes)
        let owner: Pubkey = data[offset..offset + size_of::<Pubkey>()]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        offset += size_of::<Pubkey>();

        let is_frozen: bool = data[offset] == 1;
        offset += size_of::<u8>();

        let has_authority_extension: bool = data[offset] == 1;
        offset += size_of::<u8>();

        let has_expiry: bool = data[offset] == 1;
        offset += size_of::<u8>();

        let expiry: Option<i64> = if has_expiry {
            Some(i64::from_le_bytes(
                data[offset..offset + size_of::<i64>()]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ))
        } else {
            None
        };

        offset += size_of::<i64>();

        // Read name length (1 byte)
        let name_len = data[offset] as usize;

        offset += size_of::<u8>();

        // Check if we have enough data for the name
        if data.len() < offset + name_len {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Read name string
        let name: &'info str = str::from_utf8(&data[offset..offset + name_len])
            .map_err(|_| ProgramError::InvalidAccountData)?;
        
        offset += name_len;

        // Read data content
        let data_content: &'info str = str::from_utf8(&data[offset..])
            .map_err(|_| ProgramError::InvalidAccountData)?;

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

    /// Initializes a new record account with the given data
    /// 
    /// # Safety
    /// 
    /// The account must be properly allocated with enough space for all data.
    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        // Verify the account has enough space
        let required_size = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.data.len();

        if account_info.data_len() < required_size {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        let mut offset = 0;

        // Write discriminator
        data[offset] = Self::DISCRIMINATOR;
        
        offset += size_of::<u8>();

        // Write class
        data[offset..offset + size_of::<Pubkey>()].clone_from_slice(&self.class);

        offset += size_of::<Pubkey>();

        // Write owner
        data[offset..offset + size_of::<Pubkey>()].clone_from_slice(&self.owner);

        offset += size_of::<Pubkey>();

        // Write is_frozen
        data[offset] = self.is_frozen as u8;

        offset += size_of::<u8>();

        // Write has_authority_extension
        data[offset] = self.has_authority_extension as u8;

        offset += size_of::<u8>();

        // Write expiry if present
        if self.has_authority_extension {
            data[offset] = 1;

            offset += size_of::<u8>();

            data[offset..offset + size_of::<i64>()].clone_from_slice(&self.expiry.unwrap().to_le_bytes());
        } else {
            data[offset] = 0;

            offset += size_of::<u8>();

            data[offset..offset + size_of::<i64>()].fill(0);
        }

        offset += size_of::<i64>();

        // Write name length
        data[offset] = self.name.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;

        offset += size_of::<u8>();

        // Check if we have enough space for the name
        if data.len() < offset + self.name.len() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Write name
        data[offset..offset + self.name.len()].clone_from_slice(self.name.as_bytes());

        offset += self.name.len();

        // Write data
        data[offset..].clone_from_slice(self.data.as_bytes());

        Ok(())
    }

    pub fn update_data(&mut self, data: &'info str) -> Result<(), ProgramError> {
       self.data = data;

        Ok(())
    }

    pub fn update_owner(&mut self, new_owner: Pubkey) -> Result<(), ProgramError> {
        self.owner = new_owner;

        Ok(())
    }

    pub fn update_is_frozen(&mut self, is_frozen: bool) -> Result<(), ProgramError> {
        self.is_frozen = is_frozen;

        Ok(())
    }
}