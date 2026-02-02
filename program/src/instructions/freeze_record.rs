use crate::{
    state::{Class, Record, CLASS_OFFSET},
    utils::{ByteReader, Context},
};
use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

/// FreezeRecord instruction.
///
/// This function:
/// 1. Loads the current record state
/// 2. Updates the frozen status
/// 3. Saves the updated state
///
/// # Accounts
/// 1. `authority` - The account that has permission to freeze/unfreeze the record (must be a signer)
/// 2. `record` - The record account to be frozen/unfrozen
/// 3. `class` - The class of the record to be frozen/unfrozen
///
/// # Security
/// The authority must be the class authority
pub struct FreezeRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeRecordAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, class] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the class authority
        Class::check_authority(class, authority)?;

        // Check if the Record is correct
        Record::check_program_id_and_discriminator(record)?;

        // Check if the class is the correct class
        if class.key().ne(&record.try_borrow_data()?[CLASS_OFFSET..CLASS_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self { record })
    }
}

const IS_FROZEN_OFFSET: usize = 0;

pub struct FreezeRecord<'info> {
    accounts: FreezeRecordAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeRecord
pub const FREEZE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = FreezeRecordAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < FREEZE_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize `is_frozen`
        let raw_is_frozen: u8 = ByteReader::read_with_offset(ctx.data, IS_FROZEN_OFFSET)?;
        let is_frozen = match raw_is_frozen {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        Ok(Self {
            accounts,
            is_frozen,
        })
    }
}

impl<'info> FreezeRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Freeze Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Update the record to be frozen [this is safe, check safety docs]
        unsafe {
            Record::update_is_frozen_unchecked(
                &mut self.accounts.record.try_borrow_mut_data()?,
                self.is_frozen,
            )
        }
    }
}
