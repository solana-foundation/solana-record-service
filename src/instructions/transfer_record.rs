use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

use crate::{sdk::Context, state::{Record, RecordAuthorityExtension}};

/// # TransferRecord
/// 
/// Transfers the ownership of a record to a new owner. Performed by
/// the owner of the record or by the record_delegate.
/// 
/// Accounts:
/// 1. Authority            [signer, mut]
/// 2. record               [mut]
/// 3. record_delegate      [optional]
/// 
/// Parameters:
/// 1. new_owner            [Pubkey]
pub struct TransferRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for TransferRecordAccounts<'info> {
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
                
                if record_authority_extension.transfer_authority != *authority.key() {
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

pub struct TransferRecord<'info> {
    accounts: TransferRecordAccounts<'info>,
    new_owner: Pubkey,
}

pub const TRANSFER_RECORD_MIN_IX_LENGTH: usize = size_of::<Pubkey>();

impl<'info> TryFrom<Context<'info>> for TransferRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = TransferRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length
        if ctx.data.len() < TRANSFER_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let new_owner: Pubkey = ctx.data[0..32].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            accounts,
            new_owner
        })
    }
}

impl<'info> TransferRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let record_data = self.accounts.record.try_borrow_data()?;
        let mut record = Record::from_bytes(&record_data)?;

        record.update_owner(self.new_owner)?;

        Ok(())
    }
}