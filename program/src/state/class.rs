use crate::utils::{resize_account, ByteWriter};
use codama::CodamaAccount;
use core::{mem::size_of, str};
use pinocchio::{error::ProgramError, AccountView, Address};

const DISCRIMINATOR_OFFSET: usize = 0;
const AUTHORITY_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
pub const IS_PERMISSIONED_OFFSET: usize = AUTHORITY_OFFSET + size_of::<Address>();
const IS_FROZEN_OFFSET: usize = IS_PERMISSIONED_OFFSET + size_of::<bool>();
const NAME_LEN_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();

#[repr(C)]
#[derive(CodamaAccount)]
#[codama(field("discriminator", number(u8), value = 1))]
#[codama(discriminator(field = "discriminator"))]
#[codama(seed(type = string(utf8), value = "class"))]
#[codama(seed(name = "authority", type = public_key))]
#[codama(seed(name = "name", type = string(utf8)))]
pub struct Class<'info> {
    /// The authority that controls this class
    pub authority: Address,
    /// Whether creating records is permissioned or not
    pub is_permissioned: bool,
    /// Whether the class is frozen or not
    pub is_frozen: bool,
    /// Human-readable name for the class
    #[codama(type = string(utf8))]
    #[codama(size_prefix = number(u8))]
    pub name: &'info str,
    /// Optional metadata about the class
    #[codama(type = string(utf8))]
    pub metadata: &'info str,
}

impl<'info> Class<'info> {
    pub const DISCRIMINATOR: u8 = 1;
    pub const MINIMUM_CLASS_SIZE: usize =
        size_of::<u8>() + size_of::<Address>() + size_of::<bool>() * 2 + size_of::<u8>();

    /// Check if the program id and discriminator are valid
    #[inline(always)]
    pub fn check_program_id(class: &AccountView) -> Result<(), ProgramError> {
        if !class.owned_by(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn check_discriminator_unchecked(data: &[u8]) -> Result<(), ProgramError> {
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn check_authority_unchecked(data: &[u8], authority: &AccountView) -> Result<(), ProgramError> {
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Address>()].ne(authority.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    pub fn check_authority(class: &AccountView, authority: &AccountView) -> Result<(), ProgramError> {
        Self::check_program_id(class)?;

        let data = class.try_borrow()?;

        unsafe {
            Self::check_discriminator_unchecked(&data)?;
            Self::check_authority_unchecked(&data, authority)
        }
    }

    pub fn check_permission(class: &AccountView, authority: Option<&AccountView>) -> Result<(), ProgramError> {
        Self::check_program_id(class)?;

        let data = class.try_borrow()?;

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
    pub unsafe fn update_is_frozen_unchecked(class: &mut AccountView, is_frozen: bool) -> Result<(), ProgramError> {
        let mut data = class.try_borrow_mut()?;

        if data[IS_FROZEN_OFFSET] == is_frozen as u8 {
            return Ok(());
        }

        data[IS_FROZEN_OFFSET] = is_frozen as u8;

        Ok(())
    }

    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_authority_unchecked(class: &mut AccountView, authority: Address) -> Result<(), ProgramError> {
        let mut data = class.try_borrow_mut()?;

        data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + size_of::<Address>()].clone_from_slice(authority.as_ref());

        Ok(())
    }

    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_metadata_unchecked(
        class: &mut AccountView,
        payer: &mut AccountView,
        metadata: &str,
    ) -> Result<(), ProgramError> {
        let name_len = {
            let data_ref = class.try_borrow()?;
            data_ref[NAME_LEN_OFFSET] as usize
        };

        let offset = name_len + NAME_LEN_OFFSET + size_of::<u8>();
        let current_len = class.data_len();
        let new_len = offset + metadata.len();

        if new_len != current_len {
            resize_account(class, payer, new_len)?;
        }

        {
            let mut data_ref = class.try_borrow_mut()?;
            data_ref[offset..offset + metadata.len()].clone_from_slice(metadata.as_bytes());
        }

        Ok(())
    }

    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn initialize_unchecked(&self, account_info: &mut AccountView) -> Result<(), ProgramError> {
        let required_space = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();

        if required_space > account_info.data_len() {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut data = account_info.try_borrow_mut()?;

        if data[DISCRIMINATOR_OFFSET] != 0x00 {
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
