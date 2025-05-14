use crate::{
    state::Record,
    utils::{ByteReader, Context},
};
use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

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
/// 3. `class` - [remaining accounts] Required if the authority is not the record owner
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. if the class is permissioned, the authority must be the permissioned authority
pub struct FreezeRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeRecordAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the record owner or has a delegate
        Record::check_owner_or_delegate(
            record,
            rest.first(),
            authority,
        )?;

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
        let is_frozen: bool = ByteReader::read_with_offset(ctx.data, IS_FROZEN_OFFSET)?;

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
        unsafe { Record::update_is_frozen_unchecked(self.accounts.record.try_borrow_mut_data()?, self.is_frozen) }
    }
}
