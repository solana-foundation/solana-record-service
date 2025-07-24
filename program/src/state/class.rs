use crate::utils::{resize_account, ByteWriter};
use core::{mem::size_of, str};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

const DISCRIMINATOR_OFFSET: usize = 0;
const AUTHORITY_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
pub const IS_PERMISSIONED_OFFSET: usize = AUTHORITY_OFFSET + size_of::<Pubkey>();
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

    /// Check if the program id and discriminator are valid
    #[inline(always)]
    pub fn check_program_id(class: &AccountInfo) -> Result<(), ProgramError> {
        // Check Program ID
        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn check_discriminator_unchecked(data: &[u8]) -> Result<(), ProgramError> {
        if data[0].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn check_authority_unchecked(
        data: &[u8],
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if authority
            .key()
            .ne(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn check_authority(
        class: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        Self::check_program_id(class)?;

        let data = class.try_borrow_data()?;

        unsafe {
            Self::check_discriminator_unchecked(&data)?;
            Self::check_authority_unchecked(&data, authority)
        }
    }

    pub fn check_permission(
        class: &AccountInfo,
        authority: Option<&AccountInfo>,
    ) -> Result<(), ProgramError> {
        Self::check_program_id(class)?;

        let data = class.try_borrow_data()?;

        unsafe { Self::check_discriminator_unchecked(&data)? }

        if data[IS_PERMISSIONED_OFFSET] == 1 {
            let authority = authority.ok_or(ProgramError::InvalidAccountData)?;
            unsafe { Self::check_authority_unchecked(&data, authority) }?;
        }

        if data[IS_FROZEN_OFFSET] == 1 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_is_permissioned_unchecked(
        data: &'info mut [u8],
        authority: &'info AccountInfo,
        is_permissioned: bool,
    ) -> Result<(), ProgramError> {
        unsafe {
            Self::check_authority_unchecked(data, authority)?;
        }

        if data[IS_PERMISSIONED_OFFSET] == is_permissioned as u8 {
            return Ok(());
        }

        data[IS_PERMISSIONED_OFFSET] = is_permissioned as u8;

        Ok(())
    }

    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_is_frozen_unchecked(
        class: &'info AccountInfo,
        authority: &'info AccountInfo,
        is_frozen: bool,
    ) -> Result<(), ProgramError> {
        let mut data = class.try_borrow_mut_data()?;

        unsafe {
            Self::check_authority_unchecked(&data, authority)?;
        }

        if data[IS_FROZEN_OFFSET] == is_frozen as u8 {
            return Ok(());
        }

        data[IS_FROZEN_OFFSET] = is_frozen as u8;

        Ok(())
    }

    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_metadata_unchecked(
        class: &'info AccountInfo,
        payer: &'info AccountInfo,
        metadata: &'info str,
    ) -> Result<(), ProgramError> {
        let name_len = {
            let data_ref = class.try_borrow_data()?;
            data_ref[NAME_LEN_OFFSET] as usize
        };

        let offset = name_len + NAME_LEN_OFFSET + size_of::<u8>();
        let current_len = class.data_len();
        let new_len = offset + metadata.len();

        if new_len != current_len {
            resize_account(class, payer, new_len, new_len < current_len)?;
        }

        {
            let mut data_ref = class.try_borrow_mut_data()?;

            let metadata_buffer = unsafe {
                core::slice::from_raw_parts_mut(data_ref.as_mut_ptr().add(offset), metadata.len())
            };
            metadata_buffer.clone_from_slice(metadata.as_bytes());
        }

        Ok(())
    }

    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn initialize_unchecked(
        &self,
        account_info: &'info AccountInfo,
    ) -> Result<(), ProgramError> {
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
