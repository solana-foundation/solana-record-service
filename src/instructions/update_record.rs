use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{sdk::Context, state::{Record, RecordAuthorityExtension}, utils::resize_account};

/// # UpdateRecord
/// 
/// Updates the data content of an existing record. Performed by
/// the owner of the record or by the record_delegate.
/// 
/// Accounts:
/// 1. Owner                [signer, mut]
/// 2. record               [mut]
/// 3. record_delegate      [optional]
/// 4. record_delegate_authority [optional]
/// 
/// Parameters:
/// 1. data                 [str]
pub struct UpdateRecordAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateRecordAccounts<'info> {
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
                
                if record_authority_extension.update_authority != *authority.key() {
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

pub struct UpdateRecord<'info> {
    accounts: UpdateRecordAccounts<'info>,
    data: &'info str,
}

pub const UPDATE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for UpdateRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = UpdateRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length
        if ctx.data.len() < UPDATE_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let data_len = ctx.data[0] as usize;

        if ctx.data.len() < 1 + data_len {
            return Err(ProgramError::InvalidInstructionData);
        }

        let data = std::str::from_utf8(
            &ctx.data[1..1 + data_len]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            accounts,
            data
        })
    }
}

impl<'info> UpdateRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let record_data = self.accounts.record.try_borrow_data()?;
        let record = Record::from_bytes(&record_data)?;

        // Calculate new account size based on metadata length difference
        let current_data_len = record.data.len();
        let new_data_len = self.data.len();
        let size_diff = new_data_len.saturating_sub(current_data_len);
        let new_account_size = record_data.len().saturating_add(size_diff);

        // Resize the account if needed
        resize_account(
            self.accounts.record,
            self.accounts.authority,
            new_account_size,
            new_data_len < current_data_len,
        )?;

        // Update the data
        record.update_data(self.accounts.record, self.data)?;

        Ok(())
    }
}