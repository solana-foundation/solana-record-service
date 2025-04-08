use core::str;

use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
pub struct Class<'info> {
    pub authority: Pubkey,                  // The authority that controls this class
    pub is_frozen: bool,                    // Whether the class is frozen or not
    pub credential_account: Option<Pubkey>, // Associated credential (if permissioned)
    pub name: &'info str,                   // Human-readable name for the class
    pub metadata: &'info str,               // Optional metadata about the class
}

impl<'info> Class<'info> {
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

    pub fn freeze(&mut self)

    pub fn initialize(&self, buffer: &'info mut [u8]) -> Result<(), ProgramError> {
        buffer[..32].clone_from_slice(&self.authority);
        buffer[32] = 0;
        let mut offset = match self.credential_account {
            Some(credential) => {
                buffer[32] = self.credential_account.is_some() as u8;
                buffer[33..65].clone_from_slice(&credential);
                65
            },
            None => {
                buffer[32] = 0;
                33
            }
        };
        buffer[offset] = self.name.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;
        offset += 1;
        buffer[offset..offset + self.name.len()].clone_from_slice(self.name.as_bytes());
        offset += self.name.len();
        buffer[offset..].clone_from_slice(self.metadata.as_bytes());
        Ok(())
    }
}