use crate::utils::{ByteReader, ByteWriter};
use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
pub struct RecordAuthorityDelegate {
    pub record: Pubkey,
    pub update_authority: Pubkey,
    pub freeze_authority: Pubkey,
    pub transfer_authority: Pubkey,
    pub burn_authority: Pubkey,
    pub authority_program: Option<Pubkey>,
}

impl RecordAuthorityDelegate {
    pub const DISCRIMINATOR: u8 = 3;
    pub const MINIMUM_RECORD_SIZE: usize =
        size_of::<u8>() + size_of::<Pubkey>() * 6 + size_of::<u8>();

    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        // Check account data has minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(data, Self::MINIMUM_RECORD_SIZE)?;

        let discriminator: u8 = data.read()?;

        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize record
        let record: Pubkey = data.read()?;

        // Deserialize update authority
        let update_authority: Pubkey = data.read()?;

        // Deserialize freeze authority
        let freeze_authority: Pubkey = data.read()?;

        // Deserialize transfer authority
        let transfer_authority: Pubkey = data.read()?;

        // Deserialize burn authority
        let burn_authority: Pubkey = data.read()?;

        // Deserialize authority program
        let authority_program: Option<Pubkey> = data.read_optional()?;

        Ok(Self {
            record,
            update_authority,
            freeze_authority,
            transfer_authority,
            burn_authority,
            authority_program,
        })
    }

    pub fn initialize(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Prevent reinitialization
        if data[0] != 0x00 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Create a ByteWriter
        let mut writer = ByteWriter::new_with_minimum_size(&mut data, Self::MINIMUM_RECORD_SIZE)?;

        // Write our discriminator
        writer.write(Self::DISCRIMINATOR)?;

        // Write our record
        writer.write(self.record)?;

        // Write our update authority
        writer.write(self.update_authority)?;

        // Write our freeze authority
        writer.write(self.freeze_authority)?;

        // Write our transfer authority
        writer.write(self.transfer_authority)?;

        // Write our burn authority
        writer.write(self.burn_authority)?;

        // Write our authority program
        writer.write_optional(self.authority_program)?;

        Ok(())
    }
}
