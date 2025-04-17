use core::{mem::size_of, str};

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use crate::state::Credential;

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

    pub fn check(account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IllegalOwner);
        }

        if unsafe { account_info.borrow_data_unchecked() }[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::MINIMUM_CLASS_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        if data[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        let authority: Pubkey = data[1..33].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let is_frozen: bool = data[33] == 1;

        let credential_account: Option<Pubkey> = if data[34] == 0 {
            None
        } else {
            Some(data[35..67].try_into().map_err(|_| ProgramError::InvalidAccountData)?)
        };

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
            is_frozen,
            credential_account,
            name,
            metadata,
        })
    }

    pub fn update_is_frozen(&mut self, is_frozen: bool) -> Result<(), ProgramError> {
        // Update is_frozen
        self.is_frozen = is_frozen;

        Ok(())
    }

    pub fn update_metadata(&mut self, metadata: &'info str) -> Result<(), ProgramError> {
        // Update metadata
        self.metadata = metadata;

        Ok(())
    }

    /// Validates that a credential account and its authority are properly configured
    /// 
    /// # Arguments
    /// 
    /// * `credential_account` - The account info for the credential
    /// * `credential_authority` - The account info for the credential authority
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the validation passes, otherwise returns an appropriate `ProgramError`
    pub fn validate_credential(
        &self,
        credential_account: &'info AccountInfo,
        credential_authority: &'info AccountInfo,
    ) -> Result<(), ProgramError> {
        // Verify the credential account matches the one in the class
        if credential_account.key().ne(&self.credential_account.unwrap()) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Verify the credential authority is a signer
        if !credential_authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify the credential authority is authorized
        let credential_borrowed_data = credential_account.try_borrow_data()?;
        let credential_data = Credential::from_bytes(credential_borrowed_data.as_ref())?;
        
        // Check if the credential authority is either the credential's authority OR in the authorized signers list
        if credential_authority.key().ne(&credential_data.authority) && !credential_data.authorized_signers.contains(credential_authority.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

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

        let mut offset = size_of::<u8>() + size_of::<Pubkey>() + size_of::<bool>() + size_of::<u8>() + size_of::<Pubkey>();

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