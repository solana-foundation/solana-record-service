use crate::{state::Record, utils::Context};
use pinocchio::{error::ProgramError, AccountView, ProgramResult};

/// DeleteRecord instruction.
///
/// This function:
/// 1. Reallocates the record account data to 0 bytes
/// 2. Transfers the lamports from the record to the payer
/// 3. If the record has an authority delegate, it will close the delegate account
///    as well
///
/// # Accounts
/// 1. `authority` - The account that has permission to delete the record (must be a signer)
/// 2. `payer` - The account that will get refunded for the record account
/// 3. `record` - The record account to be deleted
/// 4. `class` - [optional] The class of the record to be deleted
/// 5. `token2022_program` - [optional] The token2022 program to be used to close the mint account
/// 6. `mint` - [optional] The mint of the record to be deleted
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. if the class is permissioned, the authority can be the permissioned authority
pub struct DeleteRecordAccounts {
    payer: AccountView,
    record: AccountView,
}

impl TryFrom<&[AccountView]> for DeleteRecordAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, payer, record, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the record owner or has a delegate
        Record::check_owner_or_delegate_or_deleted(record, rest.first(), authority, rest.last())?;

        Ok(Self { payer: *payer, record: *record })
    }
}

pub struct DeleteRecord {
    accounts: DeleteRecordAccounts,
}

impl<'info> TryFrom<Context<'info>> for DeleteRecord {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = DeleteRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self { accounts })
    }
}

impl DeleteRecord {
    pub fn process(ctx: Context<'_>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        // Safety: The account has already been validated
        unsafe {
            Record::delete_record_unchecked(&mut self.accounts.record, &mut self.accounts.payer)?;
        }

        Ok(())
    }
}
