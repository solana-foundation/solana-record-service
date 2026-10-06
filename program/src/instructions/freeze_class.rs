use crate::{
    state::Class,
    utils::{ByteReader, Context},
};
use pinocchio::{error::ProgramError, AccountView, ProgramResult};

/// FreezeClass instruction.
///
/// # Accounts
/// 1. `authority` - The account that has permission to freeze/unfreeze the class (must be a signer)
/// 2. `class` - The class account to be frozen/unfrozen
///
/// # Security
/// 1. The authority account must be a signer and should be the owner of the class.
pub struct FreezeClassAccounts {
    class: AccountView,
}

impl TryFrom<&[AccountView]> for FreezeClassAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, class] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Class::check_authority(class, authority)?;

        Ok(Self { class: *class })
    }
}

const IS_FROZEN_OFFSET: usize = 0;
pub struct FreezeClass {
    accounts: FreezeClassAccounts,
    is_frozen: bool,
}

impl<'info> TryFrom<Context<'info>> for FreezeClass {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = FreezeClassAccounts::try_from(ctx.accounts)?;

        let is_frozen: bool = ByteReader::read_bool_with_offset(ctx.data, IS_FROZEN_OFFSET)?;

        Ok(Self { accounts, is_frozen })
    }
}

impl FreezeClass {
    pub fn process(ctx: Context<'_>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        unsafe { Class::update_is_frozen_unchecked(&mut self.accounts.class, self.is_frozen) }
    }
}
