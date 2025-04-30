use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, log::sol_log, program_error::ProgramError, pubkey::try_find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{ctx::Context, state::Class};

/// Represents the accounts required for creating a new class.
/// 
/// A class defines a namespace for records (e.g., TLD class, Twitter handles class).
/// This struct encapsulates all the accounts needed for the CreateClass instruction.
/// 
/// # Accounts
/// 
/// * `authority` - The account that will own the class (must be a signer)
/// * `class` - The new class account to be created
/// * `credential` - Optional credential account (required if class is permissioned)
/// 
/// # Security
/// 
/// The authority account must be a signer to prevent unauthorized class creation.
/// For permissioned classes, the credential account must be provided and the authority
/// must be either the credential's owner or an authorized signer.
pub struct CreateClassAccounts<'info> {
    authority: &'info AccountInfo,
    class: &'info AccountInfo,
    // system_program: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateClassAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateClassAccounts from a slice of AccountInfo.
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

/// Represents the CreateClass instruction with all its parameters.
/// 
/// This struct contains all the data needed to create a new class, including
/// the accounts, permission settings, and metadata.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `is_permissioned` - Whether the class requires credentials for access
/// * `name` - The name of the class
/// * `metadata` - Optional metadata associated with the class
pub struct CreateClass<'info> {
    accounts: CreateClassAccounts<'info>,
    is_permissioned: bool,
    is_frozen: bool,
    name: &'info str,
    metadata: &'info str,
}

/// Minimum length of instruction data required for CreateClass
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the is_permissioned flag
/// * 1 byte for the is_frozen flag
/// * 1 byte for the name length
pub const CREATE_CLASS_MIN_IX_LENGTH: usize = size_of::<bool>() // is_permissioned
    + size_of::<bool>() // is_frozen
    + size_of::<u8>(); // name_len

impl<'info> TryFrom<Context<'info>> for CreateClass<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateClass instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including checking permissions and parsing the name and metadata.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The Context containing accounts and instruction data
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - If the instruction data is valid
    /// * `Err(ProgramError)` - If the data is invalid or permissions are incorrect
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidInstructionData` - If data format is invalid
    /// * `ProgramError::InvalidAccountData` - If credential permissions are invalid
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateClassAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length of at least 2 for boolean and length byte
        if ctx.data.len() < CREATE_CLASS_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Deserialize `is_permissioned`
        let is_permissioned = ctx.data[0] == 1;

        // Deserialize `is_permissioned`
        let is_frozen = ctx.data[1] == 1;

        // In order to ensure our IX data contains a human-readable `name``, we must deserialize and perform a UTF-8 check.
        // Otherwise, we could skip deserialization and directly copy the remaining IX data to our `Class` account.
        let name_len = ctx.data[2] as usize;

        // Try to deserialize `name` from our IX data
        let name = core::str::from_utf8(
            &ctx.data[CREATE_CLASS_MIN_IX_LENGTH..CREATE_CLASS_MIN_IX_LENGTH + name_len]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        // Check if we have `metadata` and perform a UTF-8 check against it.
        let metadata = core::str::from_utf8(
            &ctx.data[CREATE_CLASS_MIN_IX_LENGTH + name_len..]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            accounts,
            is_permissioned,
            is_frozen,
            name,
            metadata,
        })
    }
}

impl <'info> CreateClass<'info> {
    /// Processes the CreateClass instruction.
    /// 
    /// This is the main entry point for the CreateClass instruction.
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
        sol_log("Create Class");
        Self::try_from(ctx)?.execute()
    }

    /// Executes the CreateClass instruction.
    /// 
    /// This function:
    /// 1. Calculates required account space and rent
    /// 2. Derives the PDA for the class account
    /// 3. Creates the new account
    /// 4. Initializes the class data
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidArgument` - If PDA derivation fails
    /// * Various other errors from account creation and initialization
    pub fn execute(&self) -> ProgramResult {
        let space = Class::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.class.lamports());

        let seeds = [
            b"class",
            self.accounts.authority.key().as_ref(),
            &self.name.as_bytes(),
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"class"),
            Seed::from(self.accounts.authority.key()),
            Seed::from(self.name.as_bytes()),
            Seed::from(&bump)
        ];

        let signers = [Signer::from(&seeds)];

        // Create the account with our program as owner
        CreateAccount {
            from: self.accounts.authority,
            to: self.accounts.class,
            lamports,
            space: space as u64,
            owner: &crate::ID
        }.invoke_signed(
            &signers
        )?;
        
        let class = Class {
            authority: *self.accounts.authority.key(),
            is_permissioned: false,
            is_frozen: false,
            name: self.name,
            metadata: self.metadata
        };

        class.initialize(self.accounts.class)
    }
} 