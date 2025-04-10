use core::str;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
pub struct Class<'info> {
    pub authority: Pubkey,                  // The authority that controls this class
    pub is_frozen: bool,                    // Whether the class is frozen or not
    pub credential_account: Option<Pubkey>, // Associated credential (if permissioned)
    pub name: &'info str,                   // Human-readable name for the class
    pub metadata: &'info str,               // Optional metadata about the class
}

impl<'info> Class<'info> {
    pub const DISCRIMINATOR: u8 = 1;
    pub const MINIMUM_CLASS_SIZE: usize = 1 // discriminator
        + 32                                // authority
        + 1                                 // is_frozen
        + 1                                 // credential_account (option)
        + 32                                // credential_account (pubkey)
        + 1;                                // name_len

    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::MINIMUM_CLASS_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let authority: Pubkey = data[..32].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let is_frozen: bool = data[32] == 1;

        let credential_account: Option<Pubkey> = if data[33] == 0 {
            None
        } else {
            Some(data[34..66].try_into().map_err(|_| ProgramError::InvalidAccountData)?)
        };

        let mut offset = Self::MINIMUM_CLASS_SIZE;

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
            is_frozen,
            credential_account,
            name,
            metadata,
        })
    }

    pub fn update_is_frozen(&self, account_info: &'info AccountInfo, is_frozen: bool) -> Result<(), ProgramError> {
        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Write our discriminator
        if data[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update is_frozen
        data[33] = is_frozen as u8;

        Ok(())
    }

    pub fn update_metadata(&self, account_info: &'info AccountInfo, metadata: &'info str) -> Result<(), ProgramError> {
        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Write our discriminator
        if data[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update metadata
        let mut offset = Self::MINIMUM_CLASS_SIZE;

        let name_len: usize = data[offset] as usize;

        offset += 1;

        data[offset + name_len..].clone_from_slice(metadata.as_bytes());

        Ok(())
    }

    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Write our discriminator
        data[0] = Self::DISCRIMINATOR;
        
        // Write our authority
        data[1..33].clone_from_slice(&self.authority);

        // Set is_frozen to false
        data[33] = false as u8;

        // Set credential byte
        data[34] = self.credential_account.is_some() as u8;

        // Write credential if exists and update offset
        if let Some(credential) = self.credential_account {
            data[35..67].clone_from_slice(&credential);
        } else {
            data[35..67].clone_from_slice(&[0u8; 32]);
        }

        let mut offset = Self::MINIMUM_CLASS_SIZE;

        // Write the length of our name or error if overflowed
        data[offset] = self.name.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Add 1 to our offset
        offset += 1;

        // Write our name
        data[offset..offset + self.name.len()].clone_from_slice(self.name.as_bytes());

        // Add name length to our offset to write metadata
        offset += self.name.len();

        // Write metadata if exists
        data[offset..].clone_from_slice(self.metadata.as_bytes());

        Ok(())
    }
}