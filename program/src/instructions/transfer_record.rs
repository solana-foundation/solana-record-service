use crate::{
    state::Record,
    utils::{ByteReader, Context},
};
use core::mem::size_of;
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

/// TransferRecord instruction.
///
/// This function:
/// 1. Loads the current record state
/// 2. Updates the owner to the new owner
/// 3. Saves the updated state
///
/// # Accounts
/// 1. `authority` - The account that has permission to transfer the record (must be a signer)
/// 2. `record` - The record account to be transferred
/// 3. `record_delegate` - [remaining accounts] Required if the authority is not the record owner
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. A delegate with transfer authority
/// 2. The record must not be frozen
pub struct TransferRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for TransferRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        
        Record::check_owner_or_delegate(
            record,
            authority,
            rest.first(),
            Record::TRANSFER_AUTHORITY_DELEGATION_TYPE,
        )?;

        Ok(Self { record })
    }
}

const NEW_OWNER_OFFSET: usize = 0;

pub struct TransferRecord<'info> {
    accounts: TransferRecordAccounts<'info>,
    new_owner: Pubkey,
}

/// Minimum length of instruction data required for TransferRecord
pub const TRANSFER_RECORD_MIN_IX_LENGTH: usize = size_of::<Pubkey>();

impl<'info> TryFrom<Context<'info>> for TransferRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = TransferRecordAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < TRANSFER_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize new owner
        let new_owner: Pubkey = ByteReader::read_with_offset(ctx.data, NEW_OWNER_OFFSET)?;

        Ok(Self {
            accounts,
            new_owner,
        })
    }
}

impl<'info> TransferRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Transfer Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Update the record to be transferred [this is safe, check safety docs]
        unsafe { Record::update_owner_unchecked(self.accounts.record.try_borrow_mut_data()?, self.new_owner) }
    }
}
