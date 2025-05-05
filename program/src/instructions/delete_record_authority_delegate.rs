use crate::{state::{Record, RecordAuthorityDelegate}, utils::Context};
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// DeleteRecordAuthorityDelegate instruction.
///
/// This function:
/// 1. Reallocates the record account data to 1 byte, 0xff to counter
/// reinitialization attacks
/// 2. Transfers the lamports from the record to the authority
/// 3. Updates the record to point out that it does not have an authority delegate
///
/// # Accounts
/// 1. `owner` - The current owner of the record (must be a signer)
/// 2. `record` - The record account that will be associated with the delegate
/// 3. `delegate` - The delegate account to be deleted
///
/// # Security
/// 1. The owner account must be a signer and must match the current owner of the record.
pub struct DeleteRecordAuthorityDelegateAccounts<'info> {
    owner: &'info AccountInfo,
    delegate: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for DeleteRecordAuthorityDelegateAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, record, delegate] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check owner
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check record authority
        Record::check_authority(record, owner.key())?;

        Ok(Self {
            owner,
            delegate,
        })
    }
}

pub struct DeleteRecordAuthorityDelegate<'info> {
    accounts: DeleteRecordAuthorityDelegateAccounts<'info>,
}

impl<'info> TryFrom<Context<'info>> for DeleteRecordAuthorityDelegate<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = DeleteRecordAuthorityDelegateAccounts::try_from(ctx.accounts)?;

        Ok(Self { accounts })
    }
}

impl<'info> DeleteRecordAuthorityDelegate<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Delete Record Authority Delegate");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        RecordAuthorityDelegate::delete_record_delegate(self.accounts.delegate, self.accounts.owner)
    }
}
