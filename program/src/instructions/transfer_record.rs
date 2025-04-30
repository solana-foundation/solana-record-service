use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, ProgramResult};

use crate::{ctx::Context, state::{Record, RecordAuthorityExtension}};

/// Represents the accounts required for transferring a record.
/// 
/// This instruction allows the record owner or a delegate with transfer authority
/// to transfer ownership of a record to a new owner. This is a critical operation
/// that changes the fundamental ownership of the record.
/// 
/// # Accounts
/// 
/// * `authority` - The account that has permission to transfer the record (must be a signer)
/// * `record` - The record account to be transferred
/// 
/// # Optional Accounts
/// 
/// * `record_delegate` - Required if the authority is not the record owner
/// 
/// # Security
/// 
/// The authority must be either:
/// 1. The record owner, or
/// 2. A delegate with transfer authority (requires record_delegate account)
/// 
/// The instruction will fail if:
/// 1. The record is frozen (frozen records cannot be transferred)
/// 2. The authority is not the record owner or a valid delegate
/// 3. The new owner is the same as the current owner
/// 
/// # Implementation Notes
/// 
/// This instruction is atomic - either the entire transfer succeeds or fails.
/// No partial state changes are possible.
pub struct TransferRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TransferRecordAccounts<'info> {
    /// Validates that the authority has permission to transfer the record.
    /// 
    /// # Arguments
    /// 
    /// * `authority` - The account attempting to transfer the record
    /// * `record` - The record account being transferred
    /// * `rest` - Optional accounts (used for delegate validation)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If authority is valid
    /// * `Err(ProgramError)` - If authority validation fails
    fn validate_authority(
        authority: &AccountInfo,
        record: &AccountInfo,
        rest: &[AccountInfo],
    ) -> ProgramResult {
        // Check if authority is the record owner
        let record_data = record.try_borrow_data()?;
        let record_state = Record::from_bytes(&record_data)?;

        if record_state.owner == *authority.key() {
            return Ok(());
        }

        // If not owner, check delegate
        if !record_state.has_authority_extension {
            return Err(ProgramError::InvalidAccountData);
        }

        let record_delegate = rest.first().ok_or(ProgramError::InvalidAccountData)?;
        
        let seeds = [
            b"authority",
            record.key().as_ref(),
        ];

        let (derived_record_delegate, _) = try_find_program_address(&seeds, &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?;

        if derived_record_delegate != *record_delegate.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        let delegate_data = record_delegate.try_borrow_data()?;
        let record_authority_extension = RecordAuthorityExtension::from_bytes(&delegate_data)?;

        if record_authority_extension.transfer_authority != *authority.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl<'info> TryFrom<&'info [AccountInfo]> for TransferRecordAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a TransferRecordAccounts from a slice of AccountInfo.
    /// 
    /// This function validates that the authority has permission to transfer
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

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Self::validate_authority(authority, record, rest)?;

        Ok(Self {
            record,
        })
    }
}

/// Represents the TransferRecord instruction with all its parameters.
/// 
/// This struct contains all the data needed to transfer a record,
/// including the accounts and the new owner's public key.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `new_owner` - The public key of the new owner
/// 
/// # Validation
/// 
/// The instruction will fail if:
/// 1. The record is frozen
/// 2. The new owner is the same as the current owner
pub struct TransferRecord<'info> {
    accounts: TransferRecordAccounts<'info>,
    new_owner: Pubkey,
}

/// Minimum length of instruction data required for TransferRecord
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 32 bytes for the new owner's public key
pub const TRANSFER_RECORD_MIN_IX_LENGTH: usize = size_of::<Pubkey>();

impl<'info> TryFrom<Context<'info>> for TransferRecord<'info> {
    type Error = ProgramError;

    /// Attempts to create a TransferRecord instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the new owner's public key and performing several
    /// validation checks:
    /// 1. Verifies the record is not frozen (frozen records cannot be transferred)
    /// 2. Ensures the new owner is different from the current owner
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The Context containing accounts and instruction data
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - If the instruction data is valid and all checks pass
    /// * `Err(ProgramError)` - If the data is invalid or any check fails
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidInstructionData` - If data format is invalid
    /// * `ProgramError::InvalidAccountData` - If:
    ///     - Record is frozen
    ///     - New owner is the same as current owner
    ///     - Record state is invalid
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = TransferRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length
        if ctx.data.len() < TRANSFER_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let new_owner: Pubkey = ctx.data[0..32].try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        let record_data = accounts.record.try_borrow_data()?;
        let record = Record::from_bytes(&record_data)?;

        if record.is_frozen {
            return Err(ProgramError::InvalidAccountData);
        }

        if record.owner == new_owner {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            accounts,
            new_owner
        })
    }
}

impl<'info> TransferRecord<'info> {
    /// Processes the TransferRecord instruction.
    /// 
    /// This is the main entry point for the TransferRecord instruction.
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
    /// * `ProgramError::InvalidAccountData` - If:
    ///     - Record is frozen
    ///     - New owner is same as current owner
    ///     - Authority validation fails
    /// * `ProgramError::InvalidInstructionData` - If instruction data is malformed
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the TransferRecord instruction.
    /// 
    /// This function performs the following steps atomically:
    /// 1. Loads the current record state
    /// 2. Updates the owner to the new owner
    /// 3. Saves the updated state
    /// 
    /// # State Changes
    /// 
    /// * Updates the record's owner field to the new owner
    /// * Preserves all other record fields (including frozen state)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidAccountData` - If record state is invalid
    /// * `ProgramError::AccountDataTooSmall` - If record account is too small
    pub fn execute(&self) -> ProgramResult {
        let record_data = self.accounts.record.try_borrow_mut_data()?;
        let mut record = Record::from_bytes(&record_data)?;

        record.update_owner(self.new_owner)?;

        Ok(())
    }
}