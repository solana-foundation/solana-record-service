#[cfg(not(feature = "perf"))]
use crate::constants::{MAX_METADATA_LEN, MAX_NAME_LEN};
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;

use core::mem::size_of;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::try_find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    state::Class,
    utils::{ByteReader, Context},
};

/// CreateClass instruction.
///
/// This function:
/// 1. Calculates required account space and rent
/// 2. Derives the PDA for the class account
/// 3. Creates the new account
/// 4. Transfers the minimum rent needed to make the account rent-exempt
/// 5. Initializes the class data
///
/// # Accounts
/// 1. `authority` - The account that will own the class (must be a signer)
/// 2. `payer` - The account that will pay for the class account
/// 3. `class` - The new class account to be created
///
/// # Security
/// 1. The authority account must be a signer
pub struct CreateClassAccounts<'info> {
    authority: &'info AccountInfo,
    payer: &'info AccountInfo,
    class: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateClassAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, payer, class, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Authority Check
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            authority,
            payer,
            class,
        })
    }
}

const IS_PERMISSIONED_OFFSET: usize = 0;
const IS_FROZEN_OFFSET: usize = IS_PERMISSIONED_OFFSET + size_of::<bool>();
const NAME_LEN_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();

pub struct CreateClass<'info> {
    accounts: CreateClassAccounts<'info>,
    is_permissioned: bool,
    is_frozen: bool,
    name: &'info str,
    metadata: &'info str,
}

/// Minimum length of instruction data required for CreateClass
pub const CREATE_CLASS_MIN_IX_LENGTH: usize = size_of::<bool>() * 2 + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateClass<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateClassAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < CREATE_CLASS_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize `is_permissioned`
        let is_permissioned: bool = ByteReader::read_with_offset(ctx.data, IS_PERMISSIONED_OFFSET)?;

        // Deserialize `is_frozen`
        let is_frozen: bool = ByteReader::read_with_offset(ctx.data, IS_FROZEN_OFFSET)?;

        // Read the variable length data
        let mut variable_data: ByteReader<'info> =
            ByteReader::new_with_offset(ctx.data, NAME_LEN_OFFSET);

        // Read the name
        let name: &'info str = variable_data.read_str_with_length()?;

        #[cfg(not(feature = "perf"))]
        if name.len() > MAX_NAME_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        // Read the remaining data as metadata
        let metadata: &'info str = variable_data.read_str(variable_data.remaining_bytes())?;

        #[cfg(not(feature = "perf"))]
        if metadata.len() > MAX_METADATA_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(Self {
            accounts,
            is_permissioned,
            is_frozen,
            name,
            metadata,
        })
    }
}

impl<'info> CreateClass<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Create Class");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = Class::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.class.lamports());

        let seeds = [
            b"class",
            self.accounts.authority.key().as_ref(),
            self.name.as_bytes(),
        ];

        let bump: [u8; 1] = [try_find_program_address(&seeds, &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?
            .1];

        let seeds = [
            Seed::from(b"class"),
            Seed::from(self.accounts.authority.key()),
            Seed::from(self.name.as_bytes()),
            Seed::from(&bump),
        ];

        let signers = [Signer::from(&seeds)];

        // Create the account with our program as owner
        CreateAccount {
            from: self.accounts.payer,
            to: self.accounts.class,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signers)?;

        let class = Class {
            authority: *self.accounts.authority.key(),
            is_permissioned: self.is_permissioned,
            is_frozen: self.is_frozen,
            name: self.name,
            metadata: self.metadata,
        };

        unsafe { class.initialize_unchecked(self.accounts.class) }
    }
}
