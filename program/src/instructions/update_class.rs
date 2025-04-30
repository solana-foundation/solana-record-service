use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{constants::MAX_METADATA_LEN, ctx::Context, state::Class, utils::resize_account};

/// Represents the accounts required for updating a class.
/// 
/// This instruction allows authorized users to update either the metadata
/// or permissions of a class. The authority must be the class owner or
/// an authorized delegate.
/// 
/// # Accounts
/// 
/// * `authority` - The account that has permission to update the class (must be a signer)
/// * `class` - The class account to be updated
/// * `system_program` - Required for account resizing operations
/// 
/// # Security
/// 
/// The authority must be:
/// 1. The class owner, or
/// 2. An authorized delegate with update permissions
/// 
/// # Validation
/// 
/// The instruction will fail if:
/// 1. The authority is not a signer
/// 2. The class account is invalid
/// 3. The authority is not authorized to update the class
pub struct UpdateClassAccounts<'info> {
    authority: &'info AccountInfo,
    class: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateClassAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create UpdateClassAccounts from a slice of AccountInfo.
    /// 
    /// This function validates that the authority has permission to update
    /// the class by checking the account signatures and class validity.
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
    /// * `ProgramError::InvalidAccountData` - If class validation fails
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, class, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            authority,
            class
        })
    }
}

/// Represents the UpdateClassMetadata instruction with all its parameters.
/// 
/// This struct contains all the data needed to update a class's metadata,
/// including the accounts and the new metadata string.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `metadata` - The new metadata string to set
/// 
/// # Implementation Notes
/// 
/// This instruction:
/// 1. Validates the authority and class
/// 2. Resizes the account if the new metadata is larger
/// 3. Updates the metadata in the class
/// 
/// The operation is atomic - either the entire update succeeds or fails.
pub struct UpdateClassMetadata<'info> {
    accounts: UpdateClassAccounts<'info>,
    metadata: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassMetadata<'info> {
    type Error = ProgramError;

    /// Attempts to create an UpdateClassMetadata instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the metadata string.
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
    /// * `ProgramError::InvalidAccountData` - If metadata is not valid UTF-8
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        let metadata = core::str::from_utf8(&ctx.data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        // Validate metadata length
        if metadata.len() > MAX_METADATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(UpdateClassMetadata { accounts, metadata })
    }
}

impl <'info> UpdateClassMetadata<'info> {
    /// Processes the UpdateClassMetadata instruction.
    /// 
    /// This is the main entry point for the UpdateClassMetadata instruction.
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
    /// * Various errors from account resizing and metadata updating
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the UpdateClassMetadata instruction.
    /// 
    /// This function performs the following steps:
    /// 1. Gets the current metadata length
    /// 2. Calculates the new account size needed
    /// 3. Resizes the account if necessary
    /// 4. Updates the metadata
    /// 
    /// # State Changes
    /// 
    /// * Updates the class's metadata field
    /// * May resize the class account if needed
    /// * Preserves all other class fields
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::AccountDataTooSmall` - If account resizing fails
    /// * `ProgramError::InvalidAccountData` - If metadata update fails
    pub fn execute(&self) -> ProgramResult {
        Class::update_metadata(self.accounts.class, self.accounts.authority, self.metadata)
    }
}

/// Represents the UpdateClassPermission instruction with all its parameters.
/// 
/// This struct contains all the data needed to update a class's permissions,
/// specifically the frozen state of the class.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `is_frozen` - The new frozen state to set
/// 
/// # Implementation Notes
/// 
/// This instruction:
/// 1. Validates the authority and class
/// 2. Updates the frozen state
/// 
/// The operation is atomic - either the entire update succeeds or fails.
pub struct UpdateClassPermission<'info> {
    accounts: UpdateClassAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for UpdateClassPermission
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the boolean frozen state
pub const UPDATE_CLASS_PERMISSION_MIN_LENGTH: usize = size_of::<bool>();

impl<'info> TryFrom<Context<'info>> for UpdateClassPermission<'info> {
    type Error = ProgramError;

    /// Attempts to create an UpdateClassPermission instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the frozen state boolean.
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
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        if ctx.data.len() < UPDATE_CLASS_PERMISSION_MIN_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let is_frozen = ctx.data[0] == 1;

        // Check if the frozen state would actually change
        let class_data = accounts.class.try_borrow_data()?;
        let class = Class::from_bytes(&class_data)?;

        if class.is_frozen == is_frozen {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(UpdateClassPermission { accounts, is_frozen })
    }
}

impl <'info> UpdateClassPermission<'info> {
    /// Processes the UpdateClassPermission instruction.
    /// 
    /// This is the main entry point for the UpdateClassPermission instruction.
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
    /// * Various errors from permission updating
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the UpdateClassPermission instruction.
    /// 
    /// This function:
    /// 1. Loads the current class state
    /// 2. Updates the frozen state
    /// 
    /// # State Changes
    /// 
    /// * Updates the class's is_frozen field
    /// * Preserves all other class fields
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidAccountData` - If permission update fails
    pub fn execute(&self) -> ProgramResult {
        let class_data = self.accounts.class.try_borrow_mut_data()?;
        let mut class = Class::from_bytes(&class_data)?;

        class.update_is_frozen(self.is_frozen)?;

        Ok(())
    }
}