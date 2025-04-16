use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{state::Class, sdk::Context, utils::resize_account};

/// # UpdateClass
/// 
/// Authority can update the metadata or permission of a class based on two 
/// different instructions.
/// 
/// Callers: D3, Ecosystem Partners
/// 
/// Parameters:
/// metadata: String for UpdateClassMetadata
/// is_frozen: bool for UpdateClassPermission
/// 
/// Accounts:
/// Authority (signer)
/// Class PDA
/// System Program

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

        if unsafe { class.owner().ne(&crate::ID) } {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if unsafe { class.borrow_data_unchecked() }[0].ne(&Class::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            authority,
            class
        })
    }
}

pub struct UpdateClassMetadata<'info> {
    accounts: UpdateClassAccounts<'info>,
    metadata: &'info str,
}

pub const UPDATE_CLASS_METADATA_MIN_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for UpdateClassMetadata<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        if ctx.data.len() < UPDATE_CLASS_METADATA_MIN_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let metadata = std::str::from_utf8(&ctx.data[UPDATE_CLASS_METADATA_MIN_LENGTH..]).map_err(|_| ProgramError::InvalidInstructionData)?;

        return Ok(UpdateClassMetadata { accounts, metadata });
    }
}

impl <'info> UpdateClassMetadata<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let class_data = self.accounts.class.try_borrow_data()?;
        let class = Class::from_bytes(&class_data)?;

        // Calculate new account size based on metadata length difference
        let current_metadata_len = class.metadata.len();
        let new_metadata_len = self.metadata.len();
        let size_diff = new_metadata_len.saturating_sub(current_metadata_len);
        let new_account_size = class_data.len().saturating_add(size_diff);

        // Resize the account if needed
        resize_account(
            self.accounts.class,
            self.accounts.authority,
            new_account_size,
            new_metadata_len < current_metadata_len,
        )?;

        // Update the metadata
        class.update_metadata(self.accounts.class, self.metadata)?;

        Ok(())
    }
}

const UPDATE_CLASS_PERMISSION_MIN_LENGTH: usize = size_of::<bool>();

pub struct UpdateClassPermission<'info> {
    accounts: UpdateClassAccounts<'info>,
    is_frozen: bool,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassPermission<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        if ctx.data.len() < UPDATE_CLASS_PERMISSION_MIN_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let is_frozen = ctx.data[UPDATE_CLASS_PERMISSION_MIN_LENGTH] == 1;

        return Ok(UpdateClassPermission { accounts, is_frozen });
    }
}

impl <'info> UpdateClassPermission<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let class_data = self.accounts.class.try_borrow_data()?;
        let class = Class::from_bytes(&class_data)?;

        // Update the permission
        class.update_is_frozen(self.accounts.class, self.is_frozen)?;

        Ok(())
    }
}