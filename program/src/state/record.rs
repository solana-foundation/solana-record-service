use crate::{
    token2022::{Mint, Token},
    utils::{resize_account, ByteWriter},
};
use core::{mem::size_of, str};
use pinocchio::{
    account::{Ref, RefMut},
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, Resize,
};
use pinocchio_token_2022::instructions::CloseAccount;

use super::{Class, IS_PERMISSIONED_OFFSET};

/// Offsets
const DISCRIMINATOR_OFFSET: usize = 0;
pub const CLASS_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
const OWNER_TYPE_OFFSET: usize = CLASS_OFFSET + size_of::<Address>();
pub const OWNER_OFFSET: usize = OWNER_TYPE_OFFSET + size_of::<u8>();
pub const IS_FROZEN_OFFSET: usize = OWNER_OFFSET + size_of::<Address>();
const EXPIRY_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();
const SEED_LEN_OFFSET: usize = EXPIRY_OFFSET + size_of::<i64>();
pub const SEED_OFFSET: usize = SEED_LEN_OFFSET + size_of::<u8>();

#[repr(C)]
pub struct Record<'info> {
    /// The class this record belongs to
    pub class: Address,
    /// The owner_type enum
    pub owner_type: OwnerType,
    /// The owner of this record
    pub owner: Address,
    /// Whether the record is frozen
    pub is_frozen: bool,
    /// Optional expiration timestamp, if not set, the expiry is [0; 8]
    pub expiry: i64,
    /// The record name/key
    pub seed: &'info [u8],
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
    pub const MINIMUM_RECORD_SIZE: usize = size_of::<u8>()
        + size_of::<Address>()
        + size_of::<u8>()
        + size_of::<Address>()
        + size_of::<bool>()
        + size_of::<i64>()
        + size_of::<u8>();

    /// Check if the program id and discriminator are valid
    #[inline(always)]
    pub fn check_program_id_and_discriminator(account_info: &AccountView) -> Result<(), ProgramError> {
        // Check Program ID
        if !account_info.owned_by(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Check discriminator
        let data = account_info.try_borrow()?;
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn validate_delegate(class: &AccountView, authority: &AccountView) -> Result<(), ProgramError> {
        Class::check_program_id(class)?;

        let class_data = class.try_borrow()?;

        if class_data[IS_PERMISSIONED_OFFSET].ne(&1u8) {
            return Err(ProgramError::InvalidAccountData);
        }

        unsafe {
            Class::check_discriminator_unchecked(&class_data)?;
            Class::check_authority_unchecked(&class_data, authority)
        }
    }

    #[inline(always)]
    pub fn check_owner_or_delegate_or_deleted(
        record: &AccountView,
        class: Option<&AccountView>,
        authority: &AccountView,
        mint: Option<&AccountView>,
    ) -> Result<(), ProgramError> {
        // Check the program id and the discriminator
        Self::check_program_id_and_discriminator(record)?;

        let data = record.try_borrow()?;

        // Check if the Mint has been burned without passing through the BurnTokenizedRecord instruction
        if data[OWNER_TYPE_OFFSET].eq(&(OwnerType::Token as u8)) {
            let mint = mint.ok_or(ProgramError::InvalidAccountData)?;

            if data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Address>()].ne(mint.address().as_array()) {
                return Err(ProgramError::InvalidAccountData);
            }

            if Mint::get_supply(mint)? != 0 {
                return Err(ProgramError::InvalidAccountData);
            }

            // Close the Mint and get back the rent
            let bump = [Address::try_find_program_address(&[b"mint", record.address().as_ref()], &crate::ID)
                .ok_or(ProgramError::InvalidArgument)?
                .1];

            let seeds = [Seed::from(b"mint"), Seed::from(record.address().as_ref()), Seed::from(&bump)];

            let signers = [Signer::from(&seeds)];

            // Close the mint account
            CloseAccount::new(mint, authority, mint).invoke_signed(&signers)?;

            return Ok(());
        }

        // Check if the authority is signer
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if the authority is the owner
        if data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Address>()].eq(authority.address().as_array()) {
            return Ok(());
        }

        // Validate the delegate
        let class = class.ok_or(ProgramError::InvalidAccountData)?;
        if data[CLASS_OFFSET..CLASS_OFFSET + size_of::<Address>()].ne(class.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Self::validate_delegate(class, authority)
    }

    #[inline(always)]
    pub fn check_owner_or_delegate(
        record: &AccountView,
        class: Option<&AccountView>,
        authority: &AccountView,
    ) -> Result<(), ProgramError> {
        // Check the program id and the discriminator
        Self::check_program_id_and_discriminator(record)?;

        // Check if the authority is signer
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let data = record.try_borrow()?;

        // Check if the authority is the owner
        if data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Address>()].eq(authority.address().as_array()) {
            return Ok(());
        }

        // Check if the owner type is pubkey
        if data[OWNER_TYPE_OFFSET].ne(&(OwnerType::Pubkey as u8)) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate the delegate
        let class = class.ok_or(ProgramError::MissingRequiredSignature)?;
        if data[CLASS_OFFSET..CLASS_OFFSET + size_of::<Address>()].ne(class.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Self::validate_delegate(class, authority)
    }

    #[inline(always)]
    pub fn check_owner_or_delegate_tokenized(
        record: &AccountView,
        class: Option<&AccountView>,
        authority: &AccountView,
        mint: &AccountView,
        token_account: &AccountView,
    ) -> Result<(), ProgramError> {
        // Check the program id and the discriminator
        Self::check_program_id_and_discriminator(record)?;

        // Check if the authority is signer
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if the mint is owned by the token program
        Mint::check_program_id(mint)?;

        let mint_data = mint.try_borrow()?;

        // Check if the mint is the correct discriminator
        unsafe {
            Mint::check_discriminator_unchecked(&mint_data)?;
        }

        let record_data = record.try_borrow()?;

        // Check if the mint is the owner
        if record_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Address>()].ne(mint.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the token account is owned by the token program
        Token::check_program_id(token_account)?;

        let token_data = token_account.try_borrow()?;

        // Check if the token account is the correct discriminator
        unsafe {
            Token::check_discriminator_unchecked(&token_data)?;
        }

        // Check if the authority is the owner
        if authority.address().eq(unsafe { &Token::get_owner_unchecked(&token_data)? }) {
            return Ok(());
        }

        // Validate the delegate
        let class = class.ok_or(ProgramError::InvalidAccountData)?;
        if record_data[CLASS_OFFSET..CLASS_OFFSET + size_of::<Address>()].ne(class.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Self::validate_delegate(class, authority)
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_owner_type_unchecked(
        data: &mut RefMut<'_, [u8]>,
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
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_is_frozen_unchecked(data: &mut RefMut<'_, [u8]>, is_frozen: bool) -> Result<(), ProgramError> {
        // Check if the is_frozen is the same
        if data[IS_FROZEN_OFFSET].eq(&(is_frozen as u8)) {
            return Ok(());
        }

        // Update the is_frozen
        data[IS_FROZEN_OFFSET] = is_frozen as u8;

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_owner_unchecked(data: &mut RefMut<'_, [u8]>, new_owner: &Address) -> Result<(), ProgramError> {
        // Check if the record is frozen
        if data[IS_FROZEN_OFFSET].eq(&1u8) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the new_owner is the same
        if data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Address>()].eq(new_owner.as_array()) {
            return Ok(());
        }

        // Update the owner
        data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Address>()].clone_from_slice(new_owner.as_ref());

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_expiry_unchecked(data: &mut RefMut<'_, [u8]>, new_expiry: i64) -> Result<(), ProgramError> {
        // Check if the record is frozen
        if data[IS_FROZEN_OFFSET].eq(&1u8) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update the expiry
        data[EXPIRY_OFFSET..EXPIRY_OFFSET + size_of::<i64>()].clone_from_slice(&new_expiry.to_le_bytes());

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn update_data_unchecked(
        record: &mut AccountView,
        payer: &mut AccountView,
        data: &str,
    ) -> Result<(), ProgramError> {
        let seed_len = {
            let data_ref = record.try_borrow()?;
            data_ref[SEED_LEN_OFFSET] as usize
        };

        let offset = seed_len + SEED_LEN_OFFSET + size_of::<u8>();
        let current_len = record.data_len();
        let new_len = offset + data.len();

        if new_len != current_len {
            resize_account(record, payer, new_len)?;
        }

        {
            let mut data_ref = record.try_borrow_mut()?;
            if data_ref[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
                return Err(ProgramError::InvalidAccountData);
            }
            data_ref[offset..offset + data.len()].clone_from_slice(data.as_bytes());
        }

        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn delete_record_unchecked(record: &mut AccountView, payer: &mut AccountView) -> Result<(), ProgramError> {
        // Resize to 0 bytes
        record.resize(0)?;
        // Transfer ALL lamports back to payer to fully close the account
        // This allows CreateAccount to work when re-creating the record
        let lamports = record.lamports();
        payer.set_lamports(payer.lamports().saturating_add(lamports));
        record.set_lamports(0);
        Ok(())
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn get_metadata_len_unchecked(data: &Ref<'_, [u8]>) -> Result<usize, ProgramError> {
        let mut offset = SEED_LEN_OFFSET + size_of::<u8>() + data[SEED_LEN_OFFSET] as usize;

        // Read seed_len and skip name
        let seed_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
        offset += size_of::<u32>() + seed_len;

        // Read ticker_len and skip ticker
        let ticker_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
        offset += size_of::<u32>() + ticker_len;

        // Read uri_len and skip uri
        let uri_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
        offset += size_of::<u32>() + uri_len;

        // Read additional_metadata_len and skip additional_metadata
        let additional_metadata_len =
            u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
        offset += size_of::<u32>();

        for _ in 0..additional_metadata_len {
            let key_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
            offset += size_of::<u32>() + key_len;
            let value_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
            offset += size_of::<u32>() + value_len;
        }

        Ok(offset - (SEED_LEN_OFFSET - size_of::<u8>() - data[SEED_LEN_OFFSET] as usize))
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn get_metadata_data_unchecked<'a>(
        data: &'a Ref<'_, [u8]>,
    ) -> Result<(&'a [u8], Option<&'a [u8]>), ProgramError> {
        let mut offset = SEED_LEN_OFFSET + size_of::<u8>() + data[SEED_LEN_OFFSET] as usize;

        // Read seed_len and skip seed
        let seed_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
        offset += size_of::<u32>() + seed_len;

        // Read ticker_len and skip ticker
        let ticker_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
        offset += size_of::<u32>() + ticker_len;

        // Read uri_len and skip uri
        let uri_len = u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) as usize;
        offset += size_of::<u32>() + uri_len;

        let metadata_data = &data[SEED_LEN_OFFSET + size_of::<u8>() + data[SEED_LEN_OFFSET] as usize..offset];

        let additional_metadata_data =
            if u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()) != 0 {
                Some(&data[offset..])
            } else {
                None
            };

        Ok((metadata_data, additional_metadata_data))
    }

    #[inline(always)]
    /// # Safety
    ///
    /// This function does not perform owner checks
    pub unsafe fn initialize_unchecked(&self, account_info: &mut AccountView) -> Result<(), ProgramError> {
        let required_space = Self::MINIMUM_RECORD_SIZE + self.seed.len() + self.data.len();
        if account_info.data_len() < required_space {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut data = account_info.try_borrow_mut()?;
        if data[DISCRIMINATOR_OFFSET] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;
        ByteWriter::write_with_offset(&mut data, CLASS_OFFSET, self.class)?;
        ByteWriter::write_with_offset(&mut data, OWNER_TYPE_OFFSET, self.owner_type)?;
        ByteWriter::write_with_offset(&mut data, OWNER_OFFSET, self.owner)?;
        ByteWriter::write_with_offset(&mut data, IS_FROZEN_OFFSET, self.is_frozen)?;
        ByteWriter::write_with_offset(&mut data, EXPIRY_OFFSET, self.expiry)?;

        let mut variable_data = ByteWriter::new_with_offset(&mut data, SEED_LEN_OFFSET);
        variable_data.write_bytes_with_length(self.seed)?;
        variable_data.write_str(self.data)?;

        Ok(())
    }
}
