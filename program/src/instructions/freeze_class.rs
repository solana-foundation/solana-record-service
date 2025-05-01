use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use crate::{ctx::Context, state::Class, utils::ByteReader};

/// FreezeClass instruction.
/// 
/// This function:
/// 1. Loads the current class state
/// 2. Updates the frozen status
/// 3. Saves the updated state
/// 
/// # Accounts
/// * `authority` - The account that has permission to freeze/unfreeze the class (must be a signer)
/// * `class` - The class account to be frozen/unfrozen
/// 
/// # Security
/// 
/// The authority must be the class owner to perform this operation.
/// This is a critical operation as it affects all records within the class.
pub struct FreezeClassAccounts<'info> {
    authority: &'info AccountInfo,
    class: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeClassAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, class] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            authority,
            class,
        })
    }
}

pub struct FreezeClass<'info> {
    accounts: FreezeClassAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeClass
pub const FREEZE_CLASS_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeClass<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = FreezeClassAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(ctx.data, FREEZE_CLASS_MIN_IX_LENGTH)?;

        // Deserialize `is_frozen`
        let is_frozen: bool = data.read()?;

        Ok(Self {
            accounts,
            is_frozen,
        })
    }
}

impl<'info> FreezeClass<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Freeze Class");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        Class::update_is_frozen(self.accounts.class, self.accounts.authority, self.is_frozen)
    }
}