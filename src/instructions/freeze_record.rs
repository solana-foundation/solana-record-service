use core::mem::size_of;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::try_find_program_address, ProgramResult};

use crate::{sdk::Context, state::{Record, RecordAuthorityExtension}};

/// Represents the accounts required for freezing or unfreezing a record.
/// 
/// This instruction allows the record owner or a delegate with freeze authority
/// to prevent or allow modifications to a record. When a record is frozen,
/// no modifications can be made to it.
/// 
/// # Accounts
/// 
/// * `authority` - The account that has permission to freeze/unfreeze the record (must be a signer)
/// * `record` - The record account to be frozen/unfrozen
/// 
/// # Optional Accounts
/// 
/// * `record_delegate` - Required if the authority is not the record owner
/// 
/// # Security
/// 
/// The authority must be either:
/// 1. The record owner, or
/// 2. A delegate with freeze authority (requires record_delegate account)
/// 
/// # Validation
/// 
/// The instruction will fail if:
/// 1. The authority is not the record owner or a valid delegate
/// 2. The record is already in the requested frozen state (prevents redundant operations)
pub struct FreezeRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeRecordAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a FreezeRecordAccounts from a slice of AccountInfo.
    /// 
    /// This function validates that the authority has permission to freeze/unfreeze
    /// the record by checking either direct ownership or delegate authority.
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
    /// * `ProgramError::InvalidArgument` - If delegate PDA derivation fails
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

                    if record_authority_extension.freeze_authority != *authority.key() {
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
            record,
        })
    }
}

/// Represents the FreezeRecord instruction with all its parameters.
/// 
/// This struct contains all the data needed to freeze or unfreeze a record,
/// including the accounts and the desired frozen state.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `is_frozen` - Whether to freeze (true) or unfreeze (false) the record
/// 
/// # Validation
/// 
/// The instruction will fail if the record is already in the requested frozen state.
/// This prevents redundant operations and potential confusion about the record's state.
pub struct FreezeRecord<'info> {
    accounts: FreezeRecordAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeRecord
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the is_frozen flag
const FREEZE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeRecord<'info> {
    type Error = ProgramError;

    /// Attempts to create a FreezeRecord instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the is_frozen flag and checking the current frozen state.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The Context containing accounts and instruction data
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - If the instruction data is valid
    /// * `Err(ProgramError)` - If the data is invalid
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidInstructionData` - If data format is invalid
    /// * `ProgramError::InvalidAccountData` - If record is already in the requested frozen state
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {        
        // Deserialize our accounts array
        let accounts = FreezeRecordAccounts::try_from(ctx.accounts)?;

        if ctx.data.len() < FREEZE_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let is_frozen = ctx.data[0] == 1;

        let record_data = accounts.record.try_borrow_data()?;
        let record = Record::from_bytes(&record_data)?;

        if record.is_frozen == is_frozen {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            accounts,
            is_frozen,
        })
    }
}

impl<'info> FreezeRecord<'info> {
    /// Processes the FreezeRecord instruction.
    /// 
    /// This is the main entry point for the FreezeRecord instruction.
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
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidAccountData` - If record is already in the requested frozen state
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the FreezeRecord instruction.
    /// 
    /// This function:
    /// 1. Loads the current record state
    /// 2. Updates the frozen status
    /// 3. Saves the updated state
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * Various errors from state loading and updating
    pub fn execute(&self) -> ProgramResult {
        let record_data = self.accounts.record.try_borrow_data()?;
        let mut record = Record::from_bytes(&record_data)?;

        // Update the is_frozen
        record.update_is_frozen(self.is_frozen)?;

        Ok(())
    }
}