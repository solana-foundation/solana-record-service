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
    pub fn load(&self, buffer: &'info [u8]) -> Result<Self, ProgramError> {
        let authority: Pubkey = buffer[..32].try_into().map_err(|_| ProgramError::InvalidAccountData)?;
        let is_frozen: bool = buffer[32] == 1;
        let (mut offset, credential_account): (usize, Option<Pubkey>) = if buffer[33] == 0 {
            (34, None)
        } else {
            (66, Some(buffer[34..66].try_into().map_err(|_| ProgramError::InvalidAccountData)?))
        };
        let name_len = buffer[offset] as usize;
        offset+=1;
        let name: &'info str = str::from_utf8(&buffer[offset..offset+name_len]).map_err(|_| ProgramError::InvalidAccountData)?;
        let metadata = str::from_utf8(&buffer[offset+name_len..]).map_err(|_| ProgramError::InvalidAccountData)?;
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

    pub fn update_metadata(&self, account_info: &'info AccountInfo, is_frozen: bool, metadata: &'info str) -> Result<(), ProgramError> {
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
        let mut offset = if let Some(credential) = self.credential_account {
            data[35..67].clone_from_slice(&credential);
            67
        } else {
            35
        };

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