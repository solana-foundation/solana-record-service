use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{ctx::Context, state::Credential};

/// Represents the accounts required for creating a new credential.
/// 
/// A credential account identifies authorities who can manage a class.
/// This struct encapsulates all the accounts needed for the CreateCredential instruction.
/// 
/// # Accounts
/// 
/// * `authority` - The account that will own the credential (must be a signer)
/// * `credential` - The new credential account to be created
/// 
/// # Security
/// 
/// The authority account must be a signer to prevent unauthorized credential creation.
/// The credential account will be owned by the program and can only be modified
/// through program instructions.
pub struct CreateCredentialAccounts<'info> {
    authority: &'info AccountInfo,
    credential: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateCredentialAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateCredentialAccounts from a slice of AccountInfo.
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
        let [authority, credential, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            authority,
            credential
        })
    }
}

/// Represents the CreateCredential instruction with all its parameters.
/// 
/// This struct contains all the data needed to create a new credential, including
/// the accounts, name, and authorized signers.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `name` - The name of the credential
/// * `authorized_signers` - A list of public keys that are authorized to use this credential
pub struct CreateCredential<'info> {
    accounts: CreateCredentialAccounts<'info>,
    name: &'info str,
    authorized_signers: &'info [Pubkey],
}

/// Minimum length of instruction data required for CreateCredential
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the name length
/// * 1 byte for the authorized signers length
pub const CREATE_CREDENTIAL_MIN_IX_LENGTH: usize = size_of::<u8>() + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateCredential<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateCredential instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the name and authorized signers list.
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
    /// * `ProgramError::InvalidArgument` - If authorized signers length exceeds maximum
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateCredentialAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length of at least 5 for boolean and length bytes
        if ctx.data.len() < CREATE_CREDENTIAL_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        let name_len = ctx.data[0] as usize;

        let name = core::str::from_utf8(
            &ctx.data[1..1 + name_len]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        let authorized_signers_len = ctx.data[1 + name_len] as usize;

        if authorized_signers_len > 16 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let authorized_signers = if authorized_signers_len > 0 {
            unsafe {
                core::slice::from_raw_parts(
                    ctx.data[1 + name_len..1 + name_len + authorized_signers_len * 32].as_ptr() as *const Pubkey,
                    authorized_signers_len
                )
            }
        } else {
            &[]
        };

        Ok(Self {
            accounts,
            name,
            authorized_signers
        })
    }
}

impl <'info> CreateCredential<'info> {
    /// Processes the CreateCredential instruction.
    /// 
    /// This is the main entry point for the CreateCredential instruction.
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

    /// Executes the CreateCredential instruction.
    /// 
    /// This function:
    /// 1. Calculates required account space and rent
    /// 2. Derives the PDA for the credential account
    /// 3. Creates the new account
    /// 4. Initializes the credential data
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
        let space = Credential::MINIMUM_CLASS_SIZE + self.name.len() + self.authorized_signers.len() * 32;
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.credential.lamports());

        let seeds = [
            b"credential",
            self.accounts.authority.key().as_ref(),
            self.name.as_bytes(),
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"credential"),
            Seed::from(self.accounts.authority.key()),
            Seed::from(self.name.as_bytes()),
            Seed::from(&bump)
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.authority,
            to: self.accounts.credential,
            lamports,
            space: space as u64,
            owner: &crate::ID
        }.invoke_signed(
            &signers
        )?;

        let credential = Credential {
            authority: *self.accounts.authority.key(),
            name: self.name,
            authorized_signers: self.authorized_signers
        };

        credential.initialize(&self.accounts.credential)?;

        Ok(())
    }
}