use std::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use crate::utils::{ByteReader, ByteWriter};

pub const DISCRIMINATOR_OFFSET: usize = 0;
pub const RECORD_OFFSET: usize = DISCRIMINATOR_OFFSET + size_of::<u8>();
pub const UPDATE_AUTHORITY_OFFSET: usize = RECORD_OFFSET + size_of::<Pubkey>();
pub const FREEZE_AUTHORITY_OFFSET: usize = UPDATE_AUTHORITY_OFFSET + size_of::<Pubkey>();
pub const TRANSFER_AUTHORITY_OFFSET: usize = FREEZE_AUTHORITY_OFFSET + size_of::<Pubkey>();
pub const BURN_AUTHORITY_OFFSET: usize = TRANSFER_AUTHORITY_OFFSET + size_of::<Pubkey>();
pub const AUTHORITY_PROGRAM_OFFSET: usize = BURN_AUTHORITY_OFFSET + size_of::<Pubkey>();

#[repr(C)]
pub struct RecordAuthorityDelegate {
    pub record: Pubkey,
    pub update_authority: Pubkey,
    pub freeze_authority: Pubkey,
    pub transfer_authority: Pubkey,
    pub burn_authority: Pubkey,
    pub authority_program: Pubkey, // Optional, if not set, the authority program is [0; 32]
}

impl RecordAuthorityDelegate {
    pub const DISCRIMINATOR: u8 = 3;
    pub const MINIMUM_RECORD_SIZE: usize = size_of::<u8>() + size_of::<Pubkey>() * 6;

    pub unsafe fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        let discriminator: u8 = ByteReader::read_with_offset(data, DISCRIMINATOR_OFFSET)?;

        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize record
        let record: Pubkey = ByteReader::read_with_offset(data, RECORD_OFFSET)?;

        // Deserialize update authority
        let update_authority: Pubkey = ByteReader::read_with_offset(data, UPDATE_AUTHORITY_OFFSET)?;

        // Deserialize freeze authority
        let freeze_authority: Pubkey = ByteReader::read_with_offset(data, FREEZE_AUTHORITY_OFFSET)?;

        // Deserialize transfer authority
        let transfer_authority: Pubkey = ByteReader::read_with_offset(data, TRANSFER_AUTHORITY_OFFSET)?;

        // Deserialize burn authority
        let burn_authority: Pubkey = ByteReader::read_with_offset(data, BURN_AUTHORITY_OFFSET)?;

        // Deserialize authority program
        let authority_program: Pubkey = ByteReader::read_with_offset(data, AUTHORITY_PROGRAM_OFFSET)?;

        Ok(Self {
            record,
            update_authority,
            freeze_authority,
            transfer_authority,
            burn_authority,
            authority_program,
        })
    }

    pub fn from_bytes_checked(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        #[cfg(not(feature = "perf"))]
        if account_info.data_len() < Self::MINIMUM_RECORD_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        unsafe { Self::from_bytes(account_info.try_borrow_data()?.as_ref()) }
    }

    pub unsafe fn initialize(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Prevent reinitialization
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Write our discriminator
        ByteWriter::write_with_offset(&mut data, DISCRIMINATOR_OFFSET, Self::DISCRIMINATOR)?;
        
        // Write our record
        ByteWriter::write_with_offset(&mut data, RECORD_OFFSET, self.record)?;

        // Write our update authority
        ByteWriter::write_with_offset(&mut data, UPDATE_AUTHORITY_OFFSET, self.update_authority)?;

        // Write our freeze authority
        ByteWriter::write_with_offset(&mut data, FREEZE_AUTHORITY_OFFSET, self.freeze_authority)?;

        // Write our transfer authority
        ByteWriter::write_with_offset(&mut data, TRANSFER_AUTHORITY_OFFSET, self.transfer_authority)?;

        // Write our burn authority
        ByteWriter::write_with_offset(&mut data, BURN_AUTHORITY_OFFSET, self.burn_authority)?;

        // Write our authority program
        ByteWriter::write_with_offset(&mut data, AUTHORITY_PROGRAM_OFFSET, self.authority_program)?;

        Ok(())
    }
    
    pub fn initialize_checked(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        #[cfg(not(feature = "perf"))]
        if unsafe { account_info.owner().ne(&crate::ID) } {
            return Err(ProgramError::IncorrectProgramId);
        }

        if account_info.data_len() < Self::MINIMUM_RECORD_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        unsafe { Self::initialize(self, account_info) }
    }
}