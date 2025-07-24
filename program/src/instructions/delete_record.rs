use crate::{constants::ONE_LAMPORT_RENT, state::Record, utils::Context};
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// DeleteRecord instruction.
///
/// This function:
/// 1. Reallocates the record account data to 1 byte, 0xff to counter
///    reinitialization attacks
/// 2. Transfers the lamports from the record to the authority
/// 3. If the record has an authority delegate, it will close the delegate account
///    as well
///
/// # Accounts
/// 1. `authority` - The account that has permission to delete the record (must be a signer)
/// 2. `payer` - The account that will pay for the record account
/// 3. `record` - The record account to be deleted
/// 4. `class` - [optional] The class of the record to be deleted
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. if the class is permissioned, the authority can be the permissioned authority
pub struct DeleteRecordAccounts<'info> {
    _authority: &'info AccountInfo,
    payer: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for DeleteRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [_authority, payer, record, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the record owner or has a delegate
        Record::check_owner_or_delegate(record, rest.first(), _authority)?;

        Ok(Self {
            _authority,
            payer,
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

        Ok(Self { accounts })
    }
}

impl<'info> DeleteRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Delete Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Safety: The account has already been validated
        unsafe {
            Record::delete_record_unchecked(self.accounts.record, self.accounts.payer)?;

            // Refund the payer of all the lamports
            self.accounts
                .payer
                .borrow_mut_lamports_unchecked()
                .checked_add(*self.accounts.record.borrow_lamports_unchecked())
                .and_then(|x| x.checked_sub(ONE_LAMPORT_RENT))
                .ok_or(ProgramError::InvalidAccountData)?;

            *self.accounts.record.borrow_mut_lamports_unchecked() = ONE_LAMPORT_RENT;
        }

        Ok(())
    }
}
