use core::mem::size_of;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{ctx::Context, state::Class};

/// Represents the accounts required for freezing or unfreezing a class.
/// 
/// This instruction allows the class authority to prevent or allow modifications
/// to a class and its associated records. When a class is frozen, no modifications
/// can be made to it or its records.
/// 
/// # Accounts
/// 
/// * `authority` - The account that has permission to freeze/unfreeze the class (must be a signer)
/// * `class` - The class account to be frozen/unfrozen
/// 
/// # Security
/// 
/// The authority must be the class owner to perform this operation.
/// This is a critical operation as it affects all records within the class.
/// 
/// # Validation
/// 
/// The instruction will fail if:
/// 1. The authority is not the class owner
/// 2. The class is already in the requested frozen state (prevents redundant operations)
pub struct FreezeClassAccounts<'info> {
    class: &'info AccountInfo, // Note for @dean: since we're going to deserialize this multiple times, do you think we can just pass the Class struct instead?
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeClassAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a FreezeClassAccounts from a slice of AccountInfo.
    /// 
    /// This function validates that the authority has permission to freeze/unfreeze
    /// the class by checking ownership.
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
    /// * `ProgramError::InvalidAccountData` - If authority is not the class owner
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, class] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let class_data = class.try_borrow_data()?;
        let class_state = Class::from_bytes(&class_data)?;

        if class_state.authority != *authority.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            class,
        })
    }
}

/// Represents the FreezeClass instruction with all its parameters.
/// 
/// This struct contains all the data needed to freeze or unfreeze a class,
/// including the accounts and the desired frozen state.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `is_frozen` - Whether to freeze (true) or unfreeze (false) the class
/// 
/// # Validation
/// 
/// The instruction will fail if the class is already in the requested frozen state.
/// This prevents redundant operations and potential confusion about the class's state.
pub struct FreezeClass<'info> {
    accounts: FreezeClassAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeClass
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the is_frozen flag
pub const FREEZE_CLASS_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeClass<'info> {
    type Error = ProgramError;

    /// Attempts to create a FreezeClass instruction from a Context.
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
    /// * `ProgramError::InvalidAccountData` - If class is already in the requested frozen state
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = FreezeClassAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length
        if ctx.data.len() < FREEZE_CLASS_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let is_frozen = ctx.data[0] != 0;

        let class_data = accounts.class.try_borrow_mut_data()?;
        let class = Class::from_bytes(&class_data)?;

        if class.is_frozen == is_frozen {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            accounts,
            is_frozen,
        })
    }
}

impl<'info> FreezeClass<'info> {
    /// Processes the FreezeClass instruction.
    /// 
    /// This is the main entry point for the FreezeClass instruction.
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
    /// * `ProgramError::InvalidAccountData` - If class is already in the requested frozen state
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the FreezeClass instruction.
    /// 
    /// This function:
    /// 1. Loads the current class state
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
        let class_data = self.accounts.class.try_borrow_mut_data()?;
        let mut class = Class::from_bytes(&class_data)?;
        
        // Update the is_frozen state
        class.update_is_frozen(self.is_frozen)?;
        
        Ok(())
    }
}