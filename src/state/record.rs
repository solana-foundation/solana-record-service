use core::str;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

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

        // Read class (32 bytes)
        let class: Pubkey = data[..32]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        // Read owner (32 bytes)
        let owner: Pubkey = data[32..64]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let is_frozen: bool = data[64] == 1;
        let has_authority_extension: bool = data[65] == 1;

        let expiry: Option<i64> = if has_authority_extension {
            Some(i64::from_le_bytes(
                data[66..74]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ))
        } else {
            None
        };

        let mut offset = Self::MINIMUM_CLASS_SIZE;

        // Read name length (1 byte)
        let name_len = data[offset] as usize;

        offset += 1;

        // Verify we have enough data for the name
        if data.len() < offset + name_len {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Read name string
        let name: &'info str = str::from_utf8(&data[offset..offset + name_len])
            .map_err(|_| ProgramError::InvalidAccountData)?;
        offset += name_len;

        // Read data length (1 byte)
        let data_len = data[offset] as usize;

        offset += 1;

        // Verify we have enough data for the content
        if data.len() < offset + data_len {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Read data content
        let data_content: &'info str = str::from_utf8(&data[offset..offset + data_len])
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
        let required_size = Self::MINIMUM_CLASS_SIZE
            + 1 // name_len
            + self.name.len() // name bytes
            + 1 // data_len
            + self.data.len(); // data bytes

        if account_info.data_len() < required_size {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Write discriminator
        data[0] = Self::DISCRIMINATOR;

        // Write class
        data[1..33].clone_from_slice(&self.class);

        // Write owner
        data[33..65].clone_from_slice(&self.owner);

        // Write is_frozen
        data[65] = self.is_frozen as u8;

        // Write has_authority_extension
        data[66] = self.has_authority_extension as u8;

        // Write expiry if present
        if self.has_authority_extension {
            data[67..75].clone_from_slice(&self.expiry.unwrap().to_le_bytes());
        } else {
            data[67..75].fill(0);
        }

        // Write name length
        data[75] = self.name.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Write name
        let name_start = 76;
        data[name_start..name_start + self.name.len()].clone_from_slice(self.name.as_bytes());

        // Write data length
        let data_len_pos = name_start + self.name.len();
        data[data_len_pos] = self.data.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Write data content
        let data_start = data_len_pos + 1;
        data[data_start..data_start + self.data.len()].clone_from_slice(self.data.as_bytes());

        Ok(())
    }
}