use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

use crate::{state::Credential, sdk::Context, utils::resize_account};

/// Represents the accounts required for updating a credential's authorized signers.
/// 
/// This instruction allows the credential authority or an authorized signer to modify
/// the list of authorized signers for a credential. The operation can both add and
/// remove signers in a single transaction.
/// 
/// # Accounts
/// 
/// * `authority` - The account that has permission to update the credential (must be a signer)
/// * `credential` - The credential account to be updated
/// * `system_program` - Required for account resizing operations
/// 
/// # Security
/// 
/// The authority must be:
/// 1. The credential's authority, or
/// 2. An authorized signer for the credential
/// 
/// # Validation
/// 
/// The instruction will fail if:
/// 1. The authority is not a signer
/// 2. The credential account is invalid
/// 3. The authority is not authorized to update the credential
/// 4. The instruction data is malformed
/// 5. The account cannot be resized to accommodate the new signers
pub struct UpdateCredentialAccounts<'info> {
    authority: &'info AccountInfo,
    credential: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateCredentialAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create UpdateCredentialAccounts from a slice of AccountInfo.
    /// 
    /// This function validates that the authority has permission to update
    /// the credential by checking the account signatures and credential validity.
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
    /// * `ProgramError::InvalidAccountOwner` - If credential is not owned by the program
    /// * `ProgramError::InvalidAccountData` - If credential validation fails
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, credential, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Credential::check(credential)?;

        // Load and validate authority
        let credential_data = credential.try_borrow_data()?;
        let credential_state = Credential::from_bytes(&credential_data)?;
        credential_state.validate_authority(authority)?;

        Ok(Self {
            authority,
            credential
        })
    }
}

/// Represents the UpdateCredential instruction with all its parameters.
/// 
/// This struct contains all the data needed to update a credential's
/// authorized signers list.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `signers` - The list of public keys to add or remove as authorized signers
/// 
/// # Implementation Notes
/// 
/// This instruction:
/// 1. Validates the authority and credential
/// 2. Modifies the authorized signers list in place
/// 3. Resizes the account if needed
/// 
/// The operation is atomic - either the entire update succeeds or fails.
pub struct UpdateCredential<'info> {
    accounts: UpdateCredentialAccounts<'info>,
    signers: &'info [Pubkey],
}

/// Minimum length of instruction data required for UpdateCredential
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for the number of signers
pub const UPDATE_CREDENTIAL_MIN_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for UpdateCredential<'info> {
    type Error = ProgramError;

    /// Attempts to create an UpdateCredential instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the list of signers.
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
        let accounts = UpdateCredentialAccounts::try_from(ctx.accounts)?;

        if ctx.data.len() < UPDATE_CREDENTIAL_MIN_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let num_signers = ctx.data[0] as usize;
        let signers_data = &ctx.data[UPDATE_CREDENTIAL_MIN_LENGTH..];

        // Validate signers data length
        if signers_data.len() != num_signers * size_of::<Pubkey>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Create a slice of Pubkeys from the instruction data
        let signers = unsafe {
            std::slice::from_raw_parts(
                signers_data.as_ptr() as *const Pubkey,
                num_signers
            )
        };

        // Validate no duplicate signers
        for i in 0..signers.len() {
            for j in (i + 1)..signers.len() {
                if signers[i] == signers[j] {
                    return Err(ProgramError::InvalidInstructionData);
                }
            }
        }

        Ok(Self { accounts, signers })
    }
}

impl<'info> UpdateCredential<'info> {
    /// Processes the UpdateCredential instruction.
    /// 
    /// This is the main entry point for the UpdateCredential instruction.
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
    /// * Various errors from credential updating and account resizing
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the UpdateCredential instruction.
    /// 
    /// This function performs the following steps:
    /// 1. Loads the current credential state
    /// 2. Modifies the authorized signers list in place
    /// 3. Resizes the account if needed
    /// 
    /// # State Changes
    /// 
    /// * Updates the credential's authorized_signers list
    /// * May resize the credential account if needed
    /// * Preserves all other credential fields
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
        // First, get the current credential data
        let credential_data = self.accounts.credential.try_borrow_data()?;
        let mut credential = Credential::from_bytes(&credential_data)?;

        // Save the current number of signers
        let initial_signers_len = credential.authorized_signers.len();

        // Add if the signer is not already in the list, remove if it is
        for signer in self.signers {
            credential.modify_signer(*signer)?;
        }

        // Get the new number of signers after all modifications
        let new_signers_len = credential.authorized_signers.len();

        // Calculate the actual size difference based on the real changes
        let size_diff = new_signers_len.saturating_sub(initial_signers_len) * size_of::<Pubkey>();
        let new_account_size = credential_data.len().saturating_add(size_diff);

        // Resize the account if needed, based on the actual changes
        resize_account(
            self.accounts.credential,
            self.accounts.authority,
            new_account_size,
            new_signers_len < initial_signers_len,
        )?;

        Ok(())
    }
}