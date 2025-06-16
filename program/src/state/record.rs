use crate::{
    token2022::{Mint, Token},
    utils::{resize_account, ByteReader, ByteWriter},
};
use core::{mem::size_of, str};
use pinocchio::{
    account_info::{AccountInfo, Ref, RefMut}, program_error::ProgramError, pubkey::Pubkey
};

use super::{Class, IS_PERMISSIONED_OFFSET};

/// Maximum size allowed for a record account
pub const MAX_RECORD_SIZE: usize = 1024 * 1024; // 1MB

/// Offsets
const DISCRIMINATOR_OFFSET: usize = 0;
const CLASS_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
const OWNER_TYPE_OFFSET: usize = CLASS_OFFSET + size_of::<Pubkey>();
pub const OWNER_OFFSET: usize = OWNER_TYPE_OFFSET + size_of::<u8>();
const IS_FROZEN_OFFSET: usize = OWNER_OFFSET + size_of::<Pubkey>();
const EXPIRY_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();
const NAME_LEN_OFFSET: usize = EXPIRY_OFFSET + size_of::<i64>();
pub const NAME_OFFSET: usize = NAME_LEN_OFFSET + size_of::<u8>();

#[repr(C)]
pub struct Record<'info> {
    /// The class this record belongs to
    pub class: Pubkey,
    /// The owner_type enum
    pub owner_type: OwnerType,
    /// The owner of this record
    pub owner: Pubkey,
    /// Whether the record is frozen
    pub is_frozen: bool,
    /// Optional expiration timestamp, if not set, the expiry is [0; 8]
    pub expiry: i64,
    /// The record name/key
    pub name: &'info str,
    /// The record's data content
    pub data: &'info str,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum OwnerType {
    /// The owner is a pubkey
    Pubkey,
    /// The owner is a token
    Token,
}

impl<'info> Record<'info> {
    /// The discriminator byte used to identify this account type
    pub const DISCRIMINATOR: u8 = 2;

    /// Minimum size required for a valid record account
    pub const MINIMUM_CLASS_SIZE: usize = size_of::<u8>()
        + size_of::<Pubkey>()
        + size_of::<u8>()
        + size_of::<Pubkey>()
        + size_of::<bool>()
        + size_of::<i64>()
        + size_of::<u8>();

    /// Check if the program id and discriminator are valid
    #[inline(always)]
    pub fn check_program_id_and_discriminator(
        account_info: &AccountInfo,
    ) -> Result<(), ProgramError> {
        // Check Program ID
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Check discriminator
        let data = account_info.try_borrow_data()?;
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    /// Check if the owner is valid
    #[inline(always)]
    pub unsafe fn check_owner_unchecked(
        data: &[u8],
        owner: &AccountInfo,
    ) -> Result<(), ProgramError> {
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if the authority is the owner
        if owner
            .key()
            .ne(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn check_owner(
        account_info: &AccountInfo,
        owner: &AccountInfo,
    ) -> Result<(), ProgramError> {
        // Check the program id and the discriminator
        Self::check_program_id_and_discriminator(account_info)?;

        let data = account_info.try_borrow_data()?;

        // Check the owner
        unsafe { Self::check_owner_unchecked(&data, owner) }
    }

    #[inline(always)]
    pub fn validate_delegate(
        class: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {
        Class::check_program_id(class)?;

        let class_data = class.try_borrow_data()?;

        if class_data[IS_PERMISSIONED_OFFSET].ne(&1u8) {
            return Err(ProgramError::InvalidAccountData);
        }

        unsafe {
            Class::check_discriminator_unchecked(&class_data)?;
            Class::check_authority_unchecked(&class_data, authority)
        }
    }

    #[inline(always)]
    pub fn check_owner_or_delegate(
        record: &AccountInfo,
        class: Option<&AccountInfo>,
        authority: &AccountInfo,
    ) -> Result<(), ProgramError> {        
        // Check the program id and the discriminator
        Self::check_program_id_and_discriminator(record)?;

        // Check if the authority is signer
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let data = record.try_borrow_data()?;

        // Check if the authority is the owner
        if authority
            .key()
            .eq(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()])
        {
            return Ok(());
        }

        // Check if the owner type is pubkey
        if data[OWNER_TYPE_OFFSET].ne(&(OwnerType::Pubkey as u8)) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate the delegate
        let class = class.ok_or(ProgramError::MissingRequiredSignature)?;
        Self::validate_delegate(&class, authority)
    }

    #[inline(always)]
    pub fn check_owner_or_delegate_tokenized(
        record: &AccountInfo,
        class: Option<&AccountInfo>,
        authority: &AccountInfo,
        mint: &AccountInfo,
        token_account: &AccountInfo,
    ) -> Result<(), ProgramError> {
        // Check the program id and the discriminator
        Self::check_program_id_and_discriminator(record)?;

        // Check if the authority is signer
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if the mint is owned by the token program
        Mint::check_program_id(mint)?;

        let mint_data = mint.try_borrow_data()?;

        // Check if the mint is the correct discriminator
        unsafe {
            Mint::check_discriminator_unchecked(&mint_data)?;
        }

        let record_data = record.try_borrow_data()?;
        
        // Check if the mint is the owner
        if mint
            .key()
            .ne(&record_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the token account is owned by the token program
        Token::check_program_id(token_account)?;

        let token_data = token_account.try_borrow_data()?;

        // Check if the token account is the correct discriminator
        unsafe {
            Token::check_discriminator_unchecked(&token_data)?;
        }

        // Check if the authority is the owner
        if authority
            .key()
            .eq(unsafe { &Token::get_owner_unchecked(&token_data)? })
        {
            return Ok(());
        }

        let class = class.ok_or(ProgramError::MissingRequiredSignature)?;
        Self::validate_delegate(&class, authority)
    }

    #[inline(always)]
    pub unsafe fn update_owner_type_unchecked(
        data: &mut RefMut<'info, [u8]>,
        owner_type: OwnerType,
    ) -> Result<(), ProgramError> {
        // Check if the owner_type is the same
        if data[OWNER_TYPE_OFFSET].eq(&(owner_type as u8)) {
            return Ok(());
        }

        // Update the owner_type
        data[OWNER_TYPE_OFFSET] = owner_type as u8;

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn update_is_frozen_unchecked(
        data: &mut RefMut<'info, [u8]>,
        is_frozen: bool,
    ) -> Result<(), ProgramError> {
        // Check if the is_frozen is the same
        if data[IS_FROZEN_OFFSET].eq(&(is_frozen as u8)) {
            return Ok(());
        }

        // Update the is_frozen
        data[IS_FROZEN_OFFSET] = is_frozen as u8;

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn update_owner_unchecked(
        data: &mut RefMut<'info, [u8]>,
        new_owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        // Check if the record is frozen
        if data[IS_FROZEN_OFFSET].eq(&1u8) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the new_owner is the same
        if new_owner.eq(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Ok(());
        }

        // Update the owner
        data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()].clone_from_slice(new_owner);

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn update_data_unchecked(
        record: &'info AccountInfo,
        authority: &'info AccountInfo,
        data: &'info str,
    ) -> Result<(), ProgramError> {
        let name_len = {
            let data_ref = record.try_borrow_data()?;
            data_ref[NAME_LEN_OFFSET] as usize
        };

        let offset = name_len + NAME_LEN_OFFSET + size_of::<u8>();
        let current_len = record.data_len();
        let new_len = offset + data.len();

        if new_len != current_len {
            resize_account(record, authority, new_len, new_len < current_len)?;
        }

        {
            let mut data_ref = record.try_borrow_mut_data()?;
            if data_ref[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
                return Err(ProgramError::InvalidAccountData);
            }
            let data_buffer = unsafe {
                core::slice::from_raw_parts_mut(data_ref.as_mut_ptr().add(offset), data.len())
            };
            data_buffer.clone_from_slice(data.as_bytes());
        }

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn delete_record_unchecked(
        record: &'info AccountInfo,
        authority: &'info AccountInfo,
    ) -> Result<(), ProgramError> {
        resize_account(record, authority, 1, true)?;
        {
            let mut data_ref = record.try_borrow_mut_data()?;
            data_ref[DISCRIMINATOR_OFFSET] = 0xff;
        }
        Ok(())
    }

    #[inline(always)]
    pub unsafe fn get_name_and_data_unchecked(
        data: &'info Ref<'info, [u8]>,
    ) -> Result<(&'info str, &'info str), ProgramError> {
        let record_data_offset = NAME_LEN_OFFSET + size_of::<u8>() + data[NAME_LEN_OFFSET] as usize;
        let record_name =
            str::from_utf8_unchecked(&data[NAME_LEN_OFFSET + size_of::<u8>()..record_data_offset]);
        let record_data = str::from_utf8_unchecked(&data[record_data_offset..]);
        Ok((record_name, record_data))
    }

    #[inline(always)]
    pub unsafe fn initialize_unchecked(
        &self,
        account_info: &'info AccountInfo,
    ) -> Result<(), ProgramError> {
        let required_space = Self::MINIMUM_CLASS_SIZE + self.name.len() + self.data.len();
        if account_info.data_len() < required_space {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut data = account_info.try_borrow_mut_data()?;
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;
        ByteWriter::write_with_offset(&mut data, CLASS_OFFSET, self.class)?;
        ByteWriter::write_with_offset(&mut data, OWNER_TYPE_OFFSET, self.owner_type)?;
        ByteWriter::write_with_offset(&mut data, OWNER_OFFSET, self.owner)?;
        ByteWriter::write_with_offset(&mut data, IS_FROZEN_OFFSET, self.is_frozen)?;
        ByteWriter::write_with_offset(&mut data, EXPIRY_OFFSET, self.expiry)?;

        let mut variable_data = ByteWriter::new_with_offset(&mut data, NAME_LEN_OFFSET);
        variable_data.write_str_with_length(self.name)?;
        variable_data.write_str(self.data)?;

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(data: &'info [u8]) -> Result<Self, ProgramError> {
        let mut variable = ByteReader::new_with_offset(data, NAME_LEN_OFFSET);

        Ok(Self {
            class: ByteReader::read_with_offset(data, CLASS_OFFSET)?,
            owner_type: ByteReader::read_with_offset(data, OWNER_TYPE_OFFSET)?,
            owner: ByteReader::read_with_offset(data, OWNER_OFFSET)?,
            is_frozen: ByteReader::read_with_offset(data, IS_FROZEN_OFFSET)?,
            expiry: ByteReader::read_with_offset(data, EXPIRY_OFFSET)?,
            name: variable.read_str_with_length()?,
            data: variable.read_str(variable.remaining_bytes())?,
        })
    }
}
