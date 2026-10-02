use crate::constants::MAX_METADATA_LEN;
use crate::state::Class;
use crate::utils::{ByteReader, Context};
use core::mem::size_of;
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

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
pub struct UpdateClassAccounts {
    payer: AccountView,
    class: AccountView,
}

impl TryFrom<&[AccountView]> for UpdateClassAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, payer, class, _system_program] = &accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Class::check_authority(class, authority)?;

        Ok(Self { payer: *payer, class: *class })
    }
}

// UpdateClassMetadata
pub struct UpdateClassMetadata<'info> {
    accounts: UpdateClassAccounts,
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
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        unsafe { Class::update_metadata_unchecked(&mut self.accounts.class, &mut self.accounts.payer, self.metadata) }
    }
}

// UpdateClassAuthority
pub struct UpdateClassAuthority {
    accounts: UpdateClassAccounts,
    authority: Address,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassAuthority {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < size_of::<Address>() {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize authority
        let authority: [u8; 32] =
            ctx.data[0..size_of::<Address>()].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let authority = Address::new_from_array(authority);

        Ok(UpdateClassAuthority { accounts, authority })
    }
}

impl UpdateClassAuthority {
    pub fn process(ctx: Context<'_>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        unsafe { Class::update_authority_unchecked(&mut self.accounts.class, self.authority) }
    }
}
