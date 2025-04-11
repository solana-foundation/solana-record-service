use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{sdk::Context, state::{Record, RecordAuthorityExtension}, utils::resize_account};

/// # DeleteRecord
/// 
/// Marks a record as deleted and closes the account. Performed by
/// the authority of the record or by the record_delegate.
/// 
/// Accounts:
/// 1. Authority            [signer, mut]
/// 2. record               [mut]
/// 3. record_delegate      [optional]
/// 
/// Parameters:
/// 1. data                 [str]
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

        // Check if authority is the record owner
        let record_data = record.try_borrow_data()?;
        let record_state = Record::from_bytes(&record_data)?;
        
        let delegate = rest.first();
        
        // If authority is not the owner, we need to check the delegate
        if record_state.owner != *authority.key() {
            if let Some(record_delegate) = delegate {
                let delegate_data = record_delegate.try_borrow_data()?;
                let record_authority_extension = RecordAuthorityExtension::from_bytes(&delegate_data)?;
                
                if record_authority_extension.burn_authority != *authority.key() {
                    return Err(ProgramError::InvalidAccountData);
                }
            } else {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        Ok(Self {
            authority,
            record,
        })
    }
}

pub struct DeleteRecord<'info> {
    accounts: DeleteRecordAccounts<'info>,
}

pub const UPDATE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

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