use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{sdk::Context, state::Record, utils::resize_account};

/// Represents the accounts required for updating a record's data content.
/// 
/// This instruction allows the record owner or an authorized delegate to modify
/// the data content of a record. The operation can update the record's data
/// while preserving all other record metadata.
/// 
/// # Accounts
/// 
/// * `authority` - The account that has permission to update the record (must be a signer)
/// * `record` - The record account to be updated
/// * `system_program` - Required for account resizing operations
/// * `record_delegate` - Optional account that has been delegated update authority
/// 
/// # Security
/// 
/// The authority must be:
/// 1. The record's owner, or
/// 2. An authorized delegate with update permissions
/// 
/// # Validation
/// 
/// The instruction will fail if:
/// 1. The authority is not a signer
/// 2. The record account is invalid
/// 3. The authority is not authorized to update the record
/// 4. The delegate account is invalid (if provided)
/// 5. The account cannot be resized to accommodate the new data
pub struct UpdateRecordAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateRecordAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create UpdateRecordAccounts from a slice of AccountInfo.
    /// 
    /// This function validates that the authority has permission to update
    /// the record by checking the account signatures and record validity.
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
    /// * `ProgramError::InvalidAccountData` - If record validation fails
    /// * `ProgramError::InvalidArgument` - If delegate derivation fails
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, _system_program, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Load and validate record
        let record_data = record.try_borrow_data()?;
        let record_state = Record::from_bytes(&record_data)?;
        
        // Validate authority
        let delegate = rest.first();
        record_state.validate_authority(authority, delegate)?;

        Ok(Self {   
            authority,
            record,
        })
    }
}

/// Represents the UpdateRecord instruction with all its parameters.
/// 
/// This struct contains all the data needed to update a record's content.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `data` - The new data content for the record
/// 
/// # Implementation Notes
/// 
/// This instruction:
/// 1. Validates the authority and record
/// 2. Updates the record's data content
/// 3. Resizes the account if needed
/// 
/// The operation is atomic - either the entire update succeeds or fails.
pub struct UpdateRecord<'info> {
    accounts: UpdateRecordAccounts<'info>,
    data: &'info str,
}

/// Minimum length of instruction data required for UpdateRecord
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the data length
pub const UPDATE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for UpdateRecord<'info> {
    type Error = ProgramError;

    /// Attempts to create an UpdateRecord instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the new record data.
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
    /// Processes the UpdateRecord instruction.
    /// 
    /// This is the main entry point for the UpdateRecord instruction.
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
    /// * Various errors from record updating and account resizing
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the UpdateRecord instruction.
    /// 
    /// This function performs the following steps:
    /// 1. Loads the current record state
    /// 2. Updates the record's data content
    /// 3. Resizes the account if needed
    /// 
    /// # State Changes
    /// 
    /// * Updates the record's data content
    /// * May resize the record account if needed
    /// * Preserves all other record fields
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::AccountDataTooSmall` - If account resizing fails
    pub fn execute(&self) -> ProgramResult {
        // Load current state
        let record_data = self.accounts.record.try_borrow_data()?;
        let record = Record::from_bytes(&record_data)?;
        
        // Validate new data size
        record.validate_data_size(self.data)?;
        
        // Calculate sizes
        let current_size = record_data.len();
        let new_size = record.calculate_required_size(self.data);
        
        // Resize if needed
        if current_size != new_size {
            resize_account(
                self.accounts.record,
                self.accounts.authority,
                new_size,
                new_size < current_size,
            )?;
        }
        
        // Update data
        let record_data = self.accounts.record.try_borrow_mut_data()?;
        let mut record = Record::from_bytes(&record_data)?;
        record.update_data(self.data)?;
        
        Ok(())
    }
}