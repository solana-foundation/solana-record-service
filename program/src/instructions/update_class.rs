use crate::constants::MAX_METADATA_LEN;
use crate::state::Class;
use crate::utils::{ByteReader, Context};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// UpdateClass instruction.
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

pub struct UpdateClassMetadata<'info> {
    accounts: UpdateClassAccounts,
    metadata: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassMetadata<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        let mut data = ByteReader::new(ctx.data);

        let metadata = data.read_str(data.remaining_bytes())?;

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

pub struct UpdateClassAuthority {
    accounts: UpdateClassAccounts,
    authority: Address,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassAuthority {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        let authority: Address = ByteReader::read_with_offset(ctx.data, 0)?;

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
