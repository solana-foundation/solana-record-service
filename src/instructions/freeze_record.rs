use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{sdk::Context, state::{Record, RecordAuthorityExtension}};

/// # FreezeRecord
/// 
/// Freezes or unfreezes a record to prevent modifications. Performed by
/// the owner of the record or by the record_delegate.
/// 
/// Accounts:
/// 1. Authority             [signer, mut]
/// 2. record               [mut]
/// 3. record_delegate      [optional]
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

        // Check if authority is the record owner
        let record_data = record.try_borrow_data()?;
        let record_state = Record::from_bytes(&record_data)?;
        
        let delegate = rest.first();
        
        // If authority is not the owner, we need to check the delegate
        if record_state.owner != *authority.key() {
            if let Some(record_delegate) = delegate {
                let delegate_data = record_delegate.try_borrow_data()?;
                let record_authority_extension = RecordAuthorityExtension::from_bytes(&delegate_data)?;
                
                if record_authority_extension.freeze_authority != *authority.key() {
                    return Err(ProgramError::InvalidAccountData);
                }
            } else {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        Ok(Self {
            record,
        })
    }
}

pub struct FreezeRecord<'info> {
    accounts: FreezeRecordAccounts<'info>,
}

impl<'info> TryFrom<Context<'info>> for FreezeRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = FreezeRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self {
            accounts,
        })
    }
}

impl<'info> FreezeRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let record_data = self.accounts.record.try_borrow_data()?;
        let record = Record::from_bytes(&record_data)?;

        // Update the is_frozen
        record.update_is_frozen(self.accounts.record)?;

        Ok(())
    }
}