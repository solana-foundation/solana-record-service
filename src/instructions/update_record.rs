use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, ProgramResult};

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
                
        // If authority is not the owner, we need to check the delegate
        if record_state.owner != *authority.key() {
            // Check if there is a delegate on the record
            if record_state.has_authority_extension {
                // Check if the record delegate passed in is the correct delegate
                if let Some(record_delegate) = rest.first() {
                    let seeds = [
                        b"authority",
                        record.key().as_ref(),
                    ];

                    let (derived_record_delegate, _) = try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?;
                    
                    if derived_record_delegate != *record_delegate.key() {
                        return Err(ProgramError::InvalidAccountData);
                    }

                    let delegate_data = record_delegate.try_borrow_data()?;
                    let record_authority_extension = RecordAuthorityExtension::from_bytes(&delegate_data)?;
                    
                    if record_authority_extension.update_authority != *authority.key() {
                        return Err(ProgramError::InvalidAccountData);
                    }
                } else {
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

        let data = std::str::from_utf8(
            &ctx.data[0..]
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
        // First get the current data length
        let record_data = self.accounts.record.try_borrow_data()?;
        let record = Record::from_bytes(&record_data)?;
        let current_data_len = record.data.len();
        drop(record_data);

        // Calculate new account size based on metadata length difference
        let new_data_len = self.data.len();
        let size_diff = new_data_len.saturating_sub(current_data_len);
        let new_account_size = self.accounts.record.data_len().saturating_add(size_diff);

        // Resize the account if needed
        resize_account(
            self.accounts.record,
            self.accounts.authority,
            new_account_size,
            new_data_len < current_data_len,
        )?;

        // Now update the data
        let record_data = self.accounts.record.try_borrow_mut_data()?;
        let mut record = Record::from_bytes(&record_data)?;
        record.update_data(self.data)?;

        Ok(())
    }
}