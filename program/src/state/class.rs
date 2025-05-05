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

    pub fn check_authority(class: &AccountInfo, authority: &Pubkey) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = class.try_borrow_data()?;

        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority.ne(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn check_permission(
        class: &AccountInfo,
        authority: Option<&AccountInfo>,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let data = class.try_borrow_data()?;

        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check Permission
        if data[IS_PERMISSIONED_OFFSET] == 1 {
            match authority {
                Some(auth) => {
                    if auth
                        .key()
                        .ne(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()])
                    {
                        return Err(ProgramError::MissingRequiredSignature);
                    }
                }
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
        // Check program id
        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let mut data = class.try_borrow_mut_data()?;

        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority
            .key()
            .ne(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Update is_permissioned
        if data[IS_PERMISSIONED_OFFSET] == is_permissioned as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update is_permissioned
        data[IS_PERMISSIONED_OFFSET] = is_permissioned as u8;

        Ok(())
    }

    pub fn update_is_frozen(
        class: &'info AccountInfo,
        authority: &'info AccountInfo,
        is_frozen: bool,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the account data
        let mut data = class.try_borrow_mut_data()?;

        // Check discriminator
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check authority
        if authority
            .key()
            .ne(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if data[IS_FROZEN_OFFSET] == is_frozen as u8 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update is_frozen
        data[IS_FROZEN_OFFSET] = is_frozen as u8;

        Ok(())
    }

    pub fn update_metadata(
        class: &'info AccountInfo,
        authority: &'info AccountInfo,
        metadata: &'info str,
    ) -> Result<(), ProgramError> {
        // Check program id
        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the name length
        let name_len = {
            let data_ref = class.try_borrow_data()?;
            data_ref[NAME_LEN_OFFSET] as usize
        };

        // Calculate the new size
        let offset = name_len + NAME_LEN_OFFSET + size_of::<u8>();
        let current_len = class.data_len();
        let new_len = offset + metadata.len();

        // Check if we need to resize, if so, resize the account
        if new_len != current_len {
            resize_account(class, authority, new_len, new_len < current_len)?;
        }

        // Update metadata
        {
            let mut data_ref = class.try_borrow_mut_data()?;

            // Check discriminator
            if data_ref[0].ne(&Self::DISCRIMINATOR) {
                return Err(ProgramError::InvalidAccountData);
            }

            // Check authority
            if authority
                .key()
                .ne(&data_ref[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()])
            {
                return Err(ProgramError::MissingRequiredSignature);
            }

            let metadata_buffer = unsafe {
                core::slice::from_raw_parts_mut(data_ref.as_mut_ptr().add(offset), metadata.len())
            };
            metadata_buffer.clone_from_slice(metadata.as_bytes());
        }

        Ok(())
    }

    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        // Check discriminator
        let discriminator: u8 = ByteReader::read_with_offset(data, DISCRIMINATOR_OFFSET)?;

        // Check discriminator
        if discriminator.ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize authority
        let authority: Pubkey = ByteReader::read_with_offset(data, AUTHORITY_OFFSET)?;

        // Deserialize is_permissioned
        let is_permissioned: bool = ByteReader::read_with_offset(data, IS_PERMISSIONED_OFFSET)?;

        // Deserialize is_frozen
        let is_frozen: bool = ByteReader::read_with_offset(data, IS_FROZEN_OFFSET)?;

        // Deserialize Variable Length Data
        let mut variable_data: ByteReader<'info> =
            ByteReader::new_with_offset(data, NAME_LEN_OFFSET);

        // Deserialize name
        let name: &'info str = variable_data.read_str_with_length()?;

        // Deserialize metadata
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
        // Calculate required space
        let required_space = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();

        if required_space > account_info.data_len() {
            return Err(ProgramError::InvalidAccountData);
        }

        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Prevent reinitialization
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Write discriminator
        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;

        // Write authority
        ByteWriter::write_with_offset(&mut data, AUTHORITY_OFFSET, self.authority)?;

        // Write is_permissioned
        ByteWriter::write_with_offset(&mut data, IS_PERMISSIONED_OFFSET, self.is_permissioned)?;

        // Write is_frozen
        ByteWriter::write_with_offset(&mut data, IS_FROZEN_OFFSET, self.is_frozen)?;

        // Write variable length data
        let mut variable_data = ByteWriter::new_with_offset(&mut data, NAME_LEN_OFFSET);

        // Write name with length
        variable_data.write_str_with_length(self.name)?;

        // Write data
        if !self.metadata.is_empty() {
            variable_data.write_str(self.metadata)?;
        }

        Ok(())
    }
}
