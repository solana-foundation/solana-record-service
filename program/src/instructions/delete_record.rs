use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use crate::{state::Record, utils::Context};

/// DeleteRecord instruction.
/// 
/// This function:
/// 1. Reallocates the record account to 0 bytes
/// 2. Transfers the lamports from the record to the authority
/// 3. Closes the record account
/// 
/// # Accounts
/// * `authority` - The account that has permission to delete the record (must be a signer)
/// * `record` - The record account to be deleted
/// 
/// # Optional Accounts
/// * `record_delegate` - Required if the authority is not the record owner
/// 
/// # Security
/// The authority must be either:
/// 1. The record owner, or
/// 2. A delegate with burn authority (requires record_delegate account)
pub struct DeleteRecordAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for DeleteRecordAccounts<'info> {
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
            authority,
            record,
        })
    }
}

pub struct DeleteRecord<'info> {
    accounts: DeleteRecordAccounts<'info>,
}

impl<'info> TryFrom<Context<'info>> for DeleteRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = DeleteRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self {
            accounts,
        })
    }
}

impl<'info> DeleteRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Delete Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Realloc the record account data to 0 bytes
        self.accounts.record.realloc(0, true)?;

        // Transfer the lamports from the record to the authority
        *self.accounts.authority.try_borrow_mut_lamports()? += *self.accounts.record.try_borrow_lamports()?;
        *self.accounts.record.try_borrow_mut_lamports()? = 0;

        // Close the record account
        self.accounts.record.close()?;

        Ok(())
    }
}