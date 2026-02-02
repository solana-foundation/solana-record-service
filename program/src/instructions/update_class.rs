use crate::constants::MAX_METADATA_LEN;
use crate::state::Class;
use crate::utils::{ByteReader, Context};
use core::mem::size_of;
use pinocchio::pubkey::Pubkey;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// UpdateClass instruction.
///
/// This function:
/// 1. Loads the current class state
/// 2. Updates the metadata
/// 3. Saves the updated state
///
/// # Accounts
/// 1. `authority` - The account that has permission to update the class (must be a signer)
/// 2. `payer` - The account that will pay for the class account
/// 3. `class` - The class account to be updated
/// 4. `system_program` - Required for account resizing operations
///
/// # Security
/// 1. The authority must be a signer and should be the owner of the class
pub struct UpdateClassAccounts<'info> {
    payer: &'info AccountInfo,
    class: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateClassAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, payer, class, _system_program] = &accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Class::check_authority(class, authority)?;

        Ok(Self { payer, class })
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

impl<'info> UpdateClassMetadata<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Class Metadata");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        unsafe {
            Class::update_metadata_unchecked(
                self.accounts.class,
                self.accounts.payer,
                self.metadata,
            )
        }
    }
}

// UpdateClassAuthority
pub struct UpdateClassAuthority<'info> {
    accounts: UpdateClassAccounts<'info>,
    authority: Pubkey,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassAuthority<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < size_of::<Pubkey>() {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize authority
        let authority = ctx.data[0..size_of::<Pubkey>()]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(UpdateClassAuthority {
            accounts,
            authority,
        })
    }
}

impl<'info> UpdateClassAuthority<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Class Authority");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        unsafe { Class::update_authority_unchecked(self.accounts.class, self.authority) }
    }
}
