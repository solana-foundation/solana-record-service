use core::str;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

/// Represents a credential that can be used for authorization.
/// The data layout is as follows:
/// - 1 byte: discriminator
/// - 32 bytes: authority public key
/// - 1 byte: name length
/// - N bytes: name string
/// - 1 byte: number of authorized signers
/// - M * 32 bytes: authorized signer public keys
#[repr(C)]
pub struct Credential<'info> {
    /// The public key of the authority that controls this credential
    pub authority: Pubkey,
    /// Human-readable name for the credential
    pub name: &'info str,
    /// List of public keys that are authorized to sign for this credential
    pub authorized_signers: &'info [Pubkey],
}

impl<'info> Credential<'info> {
    /// The discriminator byte used to identify this account type
    pub const DISCRIMINATOR: u8 = 0;
    
    /// Minimum size required for a valid credential account
    /// This includes:
    /// - 1 byte for discriminator
    /// - 32 bytes for authority
    /// - 1 byte for name length
    /// - 1 byte for signers length
    pub const MINIMUM_CLASS_SIZE: usize = 1 + 32 + 1 + 1;

    /// Maximum number of authorized signers allowed
    pub const MAX_SIGNERS: usize = 16;

    /// Deserializes a credential from raw bytes
    /// 
    /// # Safety
    /// 
    /// This function performs unsafe operations to create slices from raw memory.
    /// The input data must be properly formatted and aligned.
    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::MINIMUM_CLASS_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Read authority (32 bytes)
        let authority: Pubkey = data[..32]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

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

        // Read number of signers (1 byte)
        let signers_len = data[offset] as usize;
        if signers_len > Self::MAX_SIGNERS {
            return Err(ProgramError::InvalidAccountData);
        }
        offset += 1;

        // Verify we have enough data for all signers
        if data.len() < offset + signers_len.checked_mul(32).unwrap() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Read authorized signers
        let authorized_signers: &'info [Pubkey] = unsafe {
            std::slice::from_raw_parts(
                data[offset..offset + signers_len.checked_mul(32).unwrap()].as_ptr() as *const Pubkey,
                signers_len
            )
        };

        Ok(Self {
            authority,
            name,
            authorized_signers,
        })
    }

    /// Toggles a signer's authorization status
    /// 
    /// If the signer is already authorized, they will be removed.
    /// If the signer is not authorized, they will be added.
    /// 
    /// # Safety
    /// 
    /// This function performs unsafe operations to modify the signers list in place.
    /// The caller must ensure the credential account has enough space for the operation.
    pub fn modify_signer(&mut self, signer: Pubkey) -> Result<(), ProgramError> {
        match self.authorized_signers.iter().position(|auth| auth == &signer) {
            Some(signer_index) => {
                // Remove the signer
                unsafe {
                    let src_ptr = self.authorized_signers.as_ptr().add((signer_index + 1) * 32) as *const u8;
                    let dst_ptr = self.authorized_signers.as_ptr().add(signer_index * 32) as *mut u8;
                    let bytes_to_copy = (self.authorized_signers.len() - signer_index - 1) * 32;
                    std::ptr::copy(src_ptr, dst_ptr, bytes_to_copy);
                    
                    // Decrement the length
                    let len_ptr = (self.authorized_signers.as_ptr() as *const u8).sub(1) as *mut u8;
                    *len_ptr -= 1;
                }
            }
            None => {
                // Check if we can add another signer
                if self.authorized_signers.len() >= Self::MAX_SIGNERS {
                    return Err(ProgramError::InvalidAccountData);
                }

                // Add the new signer to the list
                unsafe {
                    let new_signer_ptr = (self.authorized_signers.as_ptr() as *mut u8).add(self.authorized_signers.len() * 32);
                    std::ptr::copy_nonoverlapping(signer.as_ref().as_ptr(), new_signer_ptr, 32);
                    
                    // Increment the length
                    let len_ptr = (self.authorized_signers.as_ptr() as *const u8).sub(1) as *mut u8;
                    *len_ptr += 1;
                }
            }
        }

        Ok(())
    }

    /// Adds a new authorized signer to the credential
    /// 
    /// # Safety
    /// 
    /// This function performs unsafe operations to modify the signers list in place.
    /// The caller must ensure the credential account has enough space for the operation.
    pub fn add_signer(&mut self, signer: Pubkey) -> Result<(), ProgramError> {
        // Check if we can add another signer
        if self.authorized_signers.len() >= Self::MAX_SIGNERS {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the signer is already in the list
        if self.authorized_signers.iter().any(|auth| auth == &signer) {
            return Err(ProgramError::InvalidAccountData);
        }
        
        // Add the new signer to the list
        unsafe {
            let new_signer_ptr = (self.authorized_signers.as_ptr() as *mut u8).add(self.authorized_signers.len() * 32);
            std::ptr::copy_nonoverlapping(signer.as_ref().as_ptr(), new_signer_ptr, 32);
        }

        // Change the length of the slice
        unsafe {
            let len_ptr = (self.authorized_signers.as_ptr() as *const u8).sub(1) as *mut u8;
            *len_ptr += 1;
        }

        Ok(())
    }

    /// Removes an authorized signer from the credential
    /// 
    /// # Safety
    /// 
    /// This function performs unsafe operations to modify the signers list in place.
    pub fn remove_signer(&mut self, signer: Pubkey) -> Result<(), ProgramError> {
        // Check if the signer is in the list
        if !self.authorized_signers.iter().any(|auth| auth == &signer) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Remove the signer from the list
        unsafe {
            let signer_index = self.authorized_signers.iter().position(|auth| auth == &signer).unwrap();
            let src_ptr = self.authorized_signers.as_ptr().add((signer_index + 1) * 32) as *const u8;
            let dst_ptr = self.authorized_signers.as_ptr().add(signer_index * 32) as *mut u8;
            let bytes_to_copy = (self.authorized_signers.len() - signer_index - 1) * 32;
            std::ptr::copy(src_ptr, dst_ptr, bytes_to_copy);
        }
            
        // Decrement the length
        unsafe {
            let len_ptr = (self.authorized_signers.as_ptr() as *const u8).sub(1) as *mut u8;
            *len_ptr -= 1;
        }

        Ok(())
    }

    /// Initializes a new credential account with the given data
    /// 
    /// # Safety
    /// 
    /// The account must be properly allocated with enough space for all data.
    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        // Verify the account has enough space
        let required_size = Self::MINIMUM_CLASS_SIZE 
            + 1 // name length byte
            + self.name.len() // name bytes
            + 1 // signers length byte
            + self.authorized_signers.len() * 32; // signer public keys

        if account_info.data_len() < required_size {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Write discriminator
        data[0] = Self::DISCRIMINATOR;
        
        // Write authority
        data[1..33].clone_from_slice(&self.authority);

        // Write name length
        data[33] = self.name.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Write name
        let name_start = 34;
        data[name_start..name_start + self.name.len()].clone_from_slice(self.name.as_bytes());

        // Write number of signers
        let signers_len_pos = name_start + self.name.len();
        data[signers_len_pos] = self.authorized_signers.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;

        // Write signers
        let signers_start = signers_len_pos + 1;
        for (i, signer) in self.authorized_signers.iter().enumerate() {
            let start = signers_start + (i * 32);
            data[start..start + 32].clone_from_slice(signer.as_ref());
        }

        Ok(())
    }
}
