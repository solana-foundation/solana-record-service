#[cfg(not(feature="perf"))]
use pinocchio::log::sol_log;
use core::mem::size_of;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{state::Record, utils::{ByteReader, Context}};

/// FreezeRecord instruction.
/// 
/// This function:
/// 1. Loads the current record state
/// 2. Updates the frozen status
/// 3. Saves the updated state
/// 
/// # Accounts
/// * `authority` - The account that has permission to freeze/unfreeze the record (must be a signer)
/// * `record` - The record account to be frozen/unfrozen
/// 
/// # Optional Accounts
/// * `record_delegate` - Required if the authority is not the record owner
/// 
/// # Security
/// The authority must be either:
/// 1. The record owner, or
/// 2. A delegate with freeze authority (requires record_delegate account)
pub struct FreezeRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeRecordAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if authority is the record owner or has a delegate
        Record::check_authority_or_delegate(&record, authority.key(), rest.first())?;

        Ok(Self {
            record,
        })
    }
}

pub struct FreezeRecord<'info> {
    accounts: FreezeRecordAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeRecord
const FREEZE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {        
        // Deserialize our accounts array
        let accounts = FreezeRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(ctx.data, FREEZE_RECORD_MIN_IX_LENGTH)?;

        // Deserialize `is_frozen`
        let is_frozen: bool = data.read()?;

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
        Record::update_is_frozen(self.accounts.record, self.is_frozen)
    }
}