use crate::utils::{resize_account, ByteReader, ByteWriter};
use core::{mem::size_of, str};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

const DISCRIMINATOR_OFFSET: usize = 0;
const AUTHORITY_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
const IS_PERMISSIONED_OFFSET: usize = AUTHORITY_OFFSET + size_of::<Pubkey>();
const IS_FROZEN_OFFSET: usize = IS_PERMISSIONED_OFFSET + size_of::<bool>();
const NAME_LEN_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();

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

impl<'info> Class<'info> {
    pub const DISCRIMINATOR: u8 = 1;
    pub const MINIMUM_CLASS_SIZE: usize =
        size_of::<u8>() + size_of::<Pubkey>() + size_of::<bool>() * 2 + size_of::<u8>();

    /// Perform a check to ensure that the class is valid (correct program id and discriminator)
    #[inline(always)]
    fn validate_program_and_discriminator(class: &AccountInfo) -> Result<(), ProgramError> {
        // Check Program ID
        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Check Discriminator
        let data = class.try_borrow_data()?;
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    /// Perform a check to ensure that the authority is valid
    #[inline(always)]
    fn validate_authority(data: &[u8], authority: &Pubkey) -> Result<(), ProgramError> {
        // Check Authority
        if authority.ne(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        Ok(())
    }

    #[inline(always)]
    fn validate_authority_account(authority: &AccountInfo, data: &[u8]) -> Result<(), ProgramError> {
        // Check Authority
        if authority.key().ne(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if the authority is a signer
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }

    pub fn check_authority(class: &AccountInfo, authority: &Pubkey) -> Result<(), ProgramError> {
        // Check Program ID and Discriminator
        Self::validate_program_and_discriminator(class)?;

        let data = class.try_borrow_data()?;

        // Check Authority
        Self::validate_authority(&data, authority)
    }

    pub fn check_permission(
        class: &AccountInfo,
        authority: Option<&AccountInfo>,
    ) -> Result<(), ProgramError> {
        // Check Program ID and Discriminator
        Self::validate_program_and_discriminator(class)?;
        
        let data = class.try_borrow_data()?;

        // Check if the class is permissioned
        if data[IS_PERMISSIONED_OFFSET] == 1 {
            match authority {
                Some(auth) => Self::validate_authority_account(auth, &data)?,
                None => return Err(ProgramError::MissingRequiredSignature),
            }
        }

        // Check if the class is frozen
        if data[IS_FROZEN_OFFSET] == 1 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    pub fn update_is_permissioned(
        class: &'info AccountInfo,
        authority: &'info AccountInfo,
        is_permissioned: bool,
    ) -> Result<(), ProgramError> {
        // Check Program ID and Discriminator
        Self::validate_program_and_discriminator(class)?;

        let mut data = class.try_borrow_mut_data()?;
        
        // Check Authority
        Self::validate_authority_account(authority, &data)?;

        // Check if the class is_permissioned field is different from the is_permissioned argument
        if data[IS_PERMISSIONED_OFFSET] == is_permissioned as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update the is_permissioned field
        data[IS_PERMISSIONED_OFFSET] = is_permissioned as u8;

        Ok(())
    }

    pub fn update_is_frozen(
        class: &'info AccountInfo,
        authority: &'info AccountInfo,
        is_frozen: bool,
    ) -> Result<(), ProgramError> {
        // Check Program ID and Discriminator
        Self::validate_program_and_discriminator(class)?;
        
        let mut data = class.try_borrow_mut_data()?;
        
        // Check Authority
        Self::validate_authority_account(authority, &data)?;

        // Check if the class is_frozen field is different from the is_frozen argument
        if data[IS_FROZEN_OFFSET] == is_frozen as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update the is_frozen field
        data[IS_FROZEN_OFFSET] = is_frozen as u8;

        Ok(())
    }

    pub fn update_metadata(
        class: &'info AccountInfo,
        authority: &'info AccountInfo,
        metadata: &'info str,
    ) -> Result<(), ProgramError> {
        // Check Program ID and Discriminator
        Self::validate_program_and_discriminator(class)?;
        
        // Resize
        let name_len = {
            let data_ref = class.try_borrow_data()?;
            data_ref[NAME_LEN_OFFSET] as usize
        };

        let offset = name_len + NAME_LEN_OFFSET + size_of::<u8>();
        let current_len = class.data_len();
        let new_len = offset + metadata.len();

        if new_len != current_len {
            resize_account(class, authority, new_len, new_len < current_len)?;
        }

        {
            let mut data_ref = class.try_borrow_mut_data()?;

            // Check Authority
            Self::validate_authority_account(authority, &data_ref)?;

            // Update Metadata
            let metadata_buffer = unsafe {
                core::slice::from_raw_parts_mut(data_ref.as_mut_ptr().add(offset), metadata.len())
            };
            metadata_buffer.clone_from_slice(metadata.as_bytes());
        }

        Ok(())
    }

    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        let discriminator: u8 = ByteReader::read_with_offset(data, DISCRIMINATOR_OFFSET)?;
        if discriminator.ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Read Authority
        let authority: Pubkey = ByteReader::read_with_offset(data, AUTHORITY_OFFSET)?;

        // Read is_permissioned
        let is_permissioned: bool = ByteReader::read_with_offset(data, IS_PERMISSIONED_OFFSET)?;

        // Read is_frozen
        let is_frozen: bool = ByteReader::read_with_offset(data, IS_FROZEN_OFFSET)?;

        let mut variable_data: ByteReader<'info> = ByteReader::new_with_offset(data, NAME_LEN_OFFSET);
        
        // Read Name
        let name: &'info str = variable_data.read_str_with_length()?;

        // Read Metadata
        let metadata: &'info str = variable_data.read_str(variable_data.remaining_bytes())?;

        Ok(Self {
            authority,
            is_permissioned,
            is_frozen,
            name,
            metadata,
        })
    }

    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
        let required_space = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();

        if required_space > account_info.data_len() {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut data = account_info.try_borrow_mut_data()?;

        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;
        ByteWriter::write_with_offset(&mut data, AUTHORITY_OFFSET, self.authority)?;
        ByteWriter::write_with_offset(&mut data, IS_PERMISSIONED_OFFSET, self.is_permissioned)?;
        ByteWriter::write_with_offset(&mut data, IS_FROZEN_OFFSET, self.is_frozen)?;

        let mut variable_data = ByteWriter::new_with_offset(&mut data, NAME_LEN_OFFSET);
        variable_data.write_str_with_length(self.name)?;

        if !self.metadata.is_empty() {
            variable_data.write_str(self.metadata)?;
        }

        Ok(())
    }
}
