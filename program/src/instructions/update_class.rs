use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use crate::{constants::MAX_METADATA_LEN, state::Class, utils::{ByteReader, Context}};

/// UpdateClass instruction.
/// 
/// This function:
/// 1. Loads the current class state
/// 2. Updates the metadata or frozen state
/// 3. Saves the updated state
/// 
/// # Accounts
/// * `authority` - The account that has permission to update the class (must be a signer)
/// * `class` - The class account to be updated
/// * `system_program` - Required for account resizing operations
/// 
/// # Security
/// The authority must be:
/// 1. The class owner, or
/// 2. An authorized delegate with update permissions
pub struct UpdateClassAccounts<'info> {
    authority: &'info AccountInfo,
    class: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateClassAccounts<'info> {
    type Error = ProgramError;

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

// UpdateClassMetadata
pub struct UpdateClassMetadata<'info> {
    accounts: UpdateClassAccounts<'info>,
    metadata: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassMetadata<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        // Create a byte reader
        let mut data = ByteReader::new(ctx.data);

        // Deserialize metadata
        let metadata = data.read_str(data.remaining_bytes())?;

        // Validate metadata length
        if metadata.len() > MAX_METADATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(UpdateClassMetadata { accounts, metadata })
    }
}

impl <'info> UpdateClassMetadata<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature="perf"))]
        sol_log("Update Class Metadata");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        Class::update_metadata(self.accounts.class, self.accounts.authority, self.metadata)
    }
}

// UpdateClassFrozen
pub struct UpdateClassFrozen<'info> {
    accounts: UpdateClassAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for UpdateClassPermission
pub const UPDATE_CLASS_PERMISSION_MIN_LENGTH: usize = size_of::<bool>();

impl<'info> TryFrom<Context<'info>> for UpdateClassFrozen<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        // Check instruction minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(ctx.data, UPDATE_CLASS_PERMISSION_MIN_LENGTH)?;

        // Deserialize is_frozen
        let is_frozen: bool = data.read()?;

        Ok(UpdateClassFrozen { accounts, is_frozen })
    }
}

impl <'info> UpdateClassFrozen<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature="perf"))]
        sol_log("Update Class Frozen");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        Class::update_is_frozen(self.accounts.class, self.accounts.authority, self.is_frozen)
    }
}