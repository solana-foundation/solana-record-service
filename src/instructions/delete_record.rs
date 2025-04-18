use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::try_find_program_address, ProgramResult};

use crate::{sdk::Context, state::{Record, RecordAuthorityExtension}};

/// Represents the accounts required for deleting a record.
/// 
/// This instruction allows the record owner or a delegate with burn authority
/// to delete a record and close its account. The record's lamports are
/// transferred back to the authority.
/// 
/// # Accounts
/// 
/// * `authority` - The account that has permission to delete the record (must be a signer)
/// * `record` - The record account to be deleted
/// 
/// # Optional Accounts
/// 
/// * `record_delegate` - Required if the authority is not the record owner
/// 
/// # Security
/// 
/// The authority must be either:
/// 1. The record owner, or
/// 2. A delegate with burn authority (requires record_delegate account)
pub struct DeleteRecordAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for DeleteRecordAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a DeleteRecordAccounts from a slice of AccountInfo.
    /// 
    /// This function validates that the authority has permission to delete the record
    /// by checking either direct ownership or delegate authority.
    /// 
    /// # Arguments
    /// 
    /// * `accounts` - A slice of AccountInfo containing the required accounts
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - If all required accounts are present and valid
    /// * `Err(ProgramError)` - If accounts are missing or invalid
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::NotEnoughAccountKeys` - If insufficient accounts are provided
    /// * `ProgramError::MissingRequiredSignature` - If authority is not a signer
    /// * `ProgramError::InvalidAccountData` - If authority validation fails
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
                    
                    if record_authority_extension.burn_authority != *authority.key() {
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

/// Represents the DeleteRecord instruction.
/// 
/// This struct contains all the data needed to delete a record,
/// including the accounts and validation logic.
pub struct DeleteRecord<'info> {
    accounts: DeleteRecordAccounts<'info>,
}

/// Minimum length of instruction data required for DeleteRecord
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the data length
pub const UPDATE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for DeleteRecord<'info> {
    type Error = ProgramError;

    /// Attempts to create a DeleteRecord instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The Context containing accounts and instruction data
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - If the instruction data is valid
    /// * `Err(ProgramError)` - If the data is invalid
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = DeleteRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self {
            accounts,
        })
    }
}

impl<'info> DeleteRecord<'info> {
    /// Processes the DeleteRecord instruction.
    /// 
    /// This is the main entry point for the DeleteRecord instruction.
    /// It validates the instruction and executes it if valid.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The Context containing accounts and instruction data
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the instruction executed successfully
    /// * `Err(ProgramError)` - If execution failed
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the DeleteRecord instruction.
    /// 
    /// This function:
    /// 1. Reallocates the record account to 0 bytes
    /// 2. Transfers the lamports from the record to the authority
    /// 3. Closes the record account
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * Various errors from account reallocation and closure
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