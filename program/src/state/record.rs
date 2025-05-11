use super::RecordAuthorityDelegate;
use crate::{constants::{TOKEN_2022_MINT_LEN, TOKEN_2022_PROGRAM_ID, TOKEN_ACCOUNT_OWNER_OFFSET}, utils::{resize_account, ByteReader, ByteWriter}};
use core::{mem::size_of, str};
use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::{try_find_program_address, Pubkey},
};

/// Maximum size allowed for a record account
pub const MAX_RECORD_SIZE: usize = 1024 * 1024; // 1MB

const DISCRIMINATOR_OFFSET: usize = 0;
const CLASS_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
const OWNER_OFFSET: usize = CLASS_OFFSET + size_of::<Pubkey>();
const IS_FROZEN_OFFSET: usize = OWNER_OFFSET + size_of::<Pubkey>();
const HAS_AUTHORITY_DELEGATE_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();
const EXPIRY_OFFSET: usize = HAS_AUTHORITY_DELEGATE_OFFSET + size_of::<bool>();
const NAME_LEN_OFFSET: usize = EXPIRY_OFFSET + size_of::<i64>();
pub const NAME_OFFSET: usize = EXPIRY_OFFSET + size_of::<u8>();

#[repr(C)]
pub struct Record<'info> {
    /// The class this record belongs to
    pub class: Pubkey,
    /// The owner of this record
    pub owner: Pubkey,
    /// Whether the record is frozen
    pub is_frozen: bool,
    /// Flag indicating if authority delegate exists
    pub has_authority_delegate: bool,
    /// Optional expiration timestamp, if not set, the expiry is [0; 8]
    pub expiry: i64,
    /// The record name/key
    pub name: &'info str,
    /// The record's data content
    pub data: &'info str,
}

impl<'info> Record<'info> {
    /// The discriminator byte used to identify this account type
    pub const DISCRIMINATOR: u8 = 2;

    /// Minimum size required for a valid record account
    pub const MINIMUM_CLASS_SIZE: usize = size_of::<u8>()
        + size_of::<Pubkey>() * 2
        + size_of::<bool>() * 2
        + size_of::<i64>()
        + size_of::<u8>();

    /// Check if the program id and discriminator are valid
    #[inline(always)]
    fn check_program_id_and_discriminator(account_info: &AccountInfo) -> Result<(), ProgramError> {
        // Check Program ID
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        let data = account_info.try_borrow_data()?;

        // Check discriminator
        if data[DISCRIMINATOR_OFFSET].ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    /// Check if the owner authority is valid
    #[inline(always)]
    fn check_owner_authority(data: &[u8], authority: &Pubkey) -> Result<(), ProgramError> {
        // Check if the authority is the owner
        if authority.ne(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    /// DELEGATE TYPES
    pub const UPDATE_AUTHORITY_DELEGATION_TYPE: u8 = 1;
    pub const FREEZE_AUTHORITY_DELEGATION_TYPE: u8 = 2;
    pub const TRANSFER_AUTHORITY_DELEGATION_TYPE: u8 = 3;
    pub const BURN_AUTHORITY_DELEGATION_TYPE: u8 = 4;

    /// Validate the delegate
    #[inline(always)]
    fn validate_delegate(
        record: &AccountInfo,
        delegate: &AccountInfo,
        authority: &AccountInfo,
        delegation_type: u8,
    ) -> Result<(), ProgramError> {
        let seeds = [b"authority", record.key().as_ref()];
        let (derived_delegate, _) =
            try_find_program_address(&seeds, &crate::ID).ok_or(ProgramError::InvalidArgument)?;

        if derived_delegate != *delegate.key() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        match delegation_type {
            Self::UPDATE_AUTHORITY_DELEGATION_TYPE => {
                RecordAuthorityDelegate::check_update_authority(delegate, authority.key())?
            }
            Self::FREEZE_AUTHORITY_DELEGATION_TYPE => {
                RecordAuthorityDelegate::check_freeze_authority(delegate, authority.key())?
            }
            Self::TRANSFER_AUTHORITY_DELEGATION_TYPE => {
                RecordAuthorityDelegate::check_transfer_authority(delegate, authority.key())?
            }
            Self::BURN_AUTHORITY_DELEGATION_TYPE => {
                RecordAuthorityDelegate::check_burn_authority_and_close_delegate(
                    delegate, authority,
                )?;
            }
            _ => return Err(ProgramError::InvalidArgument),
        }

        Ok(())
    }

    #[inline(always)]
    pub fn check_authority(
        account_info: &AccountInfo,
        authority: &Pubkey,
    ) -> Result<(), ProgramError> {
        // Check the program id and the discriminator 
        Self::check_program_id_and_discriminator(account_info)?;

        let data = account_info.try_borrow_data()?;
        
        // Check the authority
        Self::check_owner_authority(&data, authority)
    }

    #[inline(always)]
    pub fn check_authority_or_delegate(
        record: &AccountInfo,
        authority: &AccountInfo,
        delegate: Option<&AccountInfo>,
        delegation_type: u8,
    ) -> Result<(), ProgramError> {
        // Check the program id and the discriminator 
        Self::check_program_id_and_discriminator(record)?;

        let data = record.try_borrow_data()?;

        // Check if the record is frozen and the delegation type is transfer
        if data[IS_FROZEN_OFFSET].eq(&1u8) && delegation_type == Self::TRANSFER_AUTHORITY_DELEGATION_TYPE {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the authority is the owner
        if authority.key().eq(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            if data[HAS_AUTHORITY_DELEGATE_OFFSET] == 1
                && delegation_type == Self::BURN_AUTHORITY_DELEGATION_TYPE
            {
                let delegate = delegate.ok_or(ProgramError::MissingRequiredSignature)?;
                RecordAuthorityDelegate::delete_record_delegate(delegate, authority)?;
            }
            return Ok(());
        }

        // Check if the record has an authority delegate
        if data[HAS_AUTHORITY_DELEGATE_OFFSET].ne(&1u8) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate the delegate
        let delegate = delegate.ok_or(ProgramError::MissingRequiredSignature)?;
        Self::validate_delegate(record, delegate, authority, delegation_type)
    }

    #[inline(always)]
    pub fn check_authority_or_delegate_tokenized(
        record: &AccountInfo,
        authority: &AccountInfo,
        mint: &AccountInfo,
        token_account: &AccountInfo,
        delegate: Option<&AccountInfo>,
        delegation_type: u8,
    ) -> Result<(), ProgramError> {
        Self::check_program_id_and_discriminator(record)?;

        if unsafe { mint.owner().ne(&TOKEN_2022_PROGRAM_ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        if mint.data_len() != TOKEN_2022_MINT_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let data = record.try_borrow_data()?;
        if mint.key().ne(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_data = token_account.try_borrow_data()?;
        if authority.key().eq(&token_data[TOKEN_ACCOUNT_OWNER_OFFSET..TOKEN_ACCOUNT_OWNER_OFFSET + size_of::<Pubkey>()]) {
            if data[HAS_AUTHORITY_DELEGATE_OFFSET] == 1
                && delegation_type == Self::BURN_AUTHORITY_DELEGATION_TYPE
            {
                let delegate = delegate.ok_or(ProgramError::MissingRequiredSignature)?;
                RecordAuthorityDelegate::delete_record_delegate(delegate, authority)?;
            }
            return Ok(());
        }

        if data[HAS_AUTHORITY_DELEGATE_OFFSET].ne(&1u8) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let delegate = delegate.ok_or(ProgramError::MissingRequiredSignature)?;
        Self::validate_delegate(record, delegate, authority, delegation_type)
    }

    #[inline(always)]
    pub unsafe fn update_is_frozen_unchecked(
        record: &'info AccountInfo,
        is_frozen: bool,
    ) -> Result<(), ProgramError> {
        let mut data = record.try_borrow_mut_data()?;
        if data[IS_FROZEN_OFFSET].eq(&(is_frozen as u8)) {
            return Err(ProgramError::InvalidAccountData);
        }
        data[IS_FROZEN_OFFSET] = is_frozen as u8;
        Ok(())
    }

    #[inline(always)]
    pub unsafe fn update_has_authority_extension_unchecked(
        record: &'info AccountInfo,
        has_authority_delegate: bool,
    ) -> Result<(), ProgramError> {
        let mut data = record.try_borrow_mut_data()?;
        if data[HAS_AUTHORITY_DELEGATE_OFFSET].eq(&(has_authority_delegate as u8)) {
            return Err(ProgramError::InvalidAccountData);
        }
        data[HAS_AUTHORITY_DELEGATE_OFFSET] = has_authority_delegate as u8;
        Ok(())
    }

    #[inline(always)]
    pub unsafe fn update_owner_unchecked(
        record: &'info AccountInfo,
        new_owner: Pubkey,
    ) -> Result<(), ProgramError> {
        let mut data = record.try_borrow_mut_data()?;
        if new_owner.eq(&data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::InvalidAccountData);
        }
        data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()].clone_from_slice(&new_owner);
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
    pub fn from_bytes(data: &'info [u8]) -> Result<Self, ProgramError> {
        let discriminator: u8 = ByteReader::read_with_offset(data, DISCRIMINATOR_OFFSET)?;
        if discriminator.ne(&Self::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        let class: Pubkey = ByteReader::read_with_offset(data, CLASS_OFFSET)?;
        let owner: Pubkey = ByteReader::read_with_offset(data, OWNER_OFFSET)?;
        let is_frozen: bool = ByteReader::read_with_offset(data, IS_FROZEN_OFFSET)?;
        let has_authority_delegate: bool =
            ByteReader::read_with_offset(data, HAS_AUTHORITY_DELEGATE_OFFSET)?;
        let expiry: i64 = ByteReader::read_with_offset(data, EXPIRY_OFFSET)?;

        let mut variable_data: ByteReader<'info> =
            ByteReader::new_with_offset(data, NAME_LEN_OFFSET);
        let name: &'info str = variable_data.read_str_with_length()?;
        let data_content: &'info str = variable_data.read_str(variable_data.remaining_bytes())?;

        Ok(Self {
            class,
            owner,
            is_frozen,
            has_authority_delegate,
            expiry,
            name,
            data: data_content,
        })
    }

    #[inline(always)]
    pub fn initialize(&self, account_info: &'info AccountInfo) -> Result<(), ProgramError> {
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
        ByteWriter::write_with_offset(&mut data, OWNER_OFFSET, self.owner)?;
        ByteWriter::write_with_offset(&mut data, IS_FROZEN_OFFSET, self.is_frozen)?;
        ByteWriter::write_with_offset(
            &mut data,
            HAS_AUTHORITY_DELEGATE_OFFSET,
            self.has_authority_delegate,
        )?;
        ByteWriter::write_with_offset(&mut data, EXPIRY_OFFSET, self.expiry)?;

        let mut variable_data = ByteWriter::new_with_offset(&mut data, NAME_LEN_OFFSET);
        variable_data.write_str_with_length(self.name)?;
        variable_data.write_str(self.data)?;

        Ok(())
    }
}
