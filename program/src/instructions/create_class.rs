use crate::constants::MAX_METADATA_LEN;

use core::mem::size_of;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::{Allocate, Assign, CreateAccount, Transfer};

use crate::{
    state::Class,
    utils::{ByteReader, Context},
};

/// CreateClass instruction.
///
/// # Accounts
/// 1. `authority` - The account that will own the class (must be a signer)
/// 2. `payer` - The account that will pay for the class account
/// 3. `class` - The new class account to be created
///
/// # Security
/// 1. The authority account must be a signer
pub struct CreateClassAccounts {
    authority: AccountView,
    payer: AccountView,
    class: AccountView,
}

impl TryFrom<&[AccountView]> for CreateClassAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, payer, class, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self { authority: *authority, payer: *payer, class: *class })
    }
}

const IS_PERMISSIONED_OFFSET: usize = 0;
const IS_FROZEN_OFFSET: usize = IS_PERMISSIONED_OFFSET + size_of::<bool>();
const NAME_LEN_OFFSET: usize = IS_FROZEN_OFFSET + size_of::<bool>();

pub struct CreateClass<'info> {
    accounts: CreateClassAccounts,
    is_permissioned: bool,
    is_frozen: bool,
    name: &'info str,
    metadata: &'info str,
}

impl<'info> TryFrom<Context<'info>> for CreateClass<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = CreateClassAccounts::try_from(ctx.accounts)?;

        let is_permissioned: bool = ByteReader::read_bool_with_offset(ctx.data, IS_PERMISSIONED_OFFSET)?;

        let is_frozen: bool = ByteReader::read_bool_with_offset(ctx.data, IS_FROZEN_OFFSET)?;

        let mut variable_data: ByteReader<'info> = ByteReader::new_with_offset(ctx.data, NAME_LEN_OFFSET);

        let name: &'info str = variable_data.read_str_with_length()?;

        let metadata: &'info str = variable_data.read_str(variable_data.remaining_bytes())?;

        if metadata.len() > MAX_METADATA_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(Self { accounts, is_permissioned, is_frozen, name, metadata })
    }
}

impl<'info> CreateClass<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        let space = Class::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.len();
        let rent = Rent::get()?.try_minimum_balance(space)?;
        let lamports = rent.saturating_sub(self.accounts.class.lamports());

        let seeds = [b"class", self.accounts.authority.address().as_ref(), self.name.as_bytes()];

        let bump: [u8; 1] =
            [Address::try_find_program_address(&seeds, &crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"class"),
            Seed::from(self.accounts.authority.address().as_ref()),
            Seed::from(self.name.as_bytes()),
            Seed::from(&bump),
        ];

        let signers = [Signer::from(&seeds)];

        if self.accounts.class.lamports() > 0 {
            Allocate { account: &self.accounts.class, space: space as u64 }.invoke_signed(&signers)?;

            Assign { account: &self.accounts.class, owner: &crate::ID }.invoke_signed(&signers)?;

            if self.accounts.class.lamports() < lamports {
                Transfer {
                    from: &self.accounts.payer,
                    to: &self.accounts.class,
                    lamports: lamports - self.accounts.class.lamports(),
                }
                .invoke()?;
            }
        } else {
            CreateAccount {
                from: &self.accounts.payer,
                to: &self.accounts.class,
                lamports,
                space: space as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&signers)?;
        }

        let class = Class {
            authority: *self.accounts.authority.address(),
            is_permissioned: self.is_permissioned,
            is_frozen: self.is_frozen,
            name: self.name,
            metadata: self.metadata,
        };

        unsafe { class.initialize_unchecked(&mut self.accounts.class) }
    }
}
