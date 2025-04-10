use core::mem::size_of;
use pinocchio::{account_info::{self, AccountInfo}, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::try_find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{state::Class, sdk::Context};

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

        let metadata = str::from_utf8(&ctx.data[UPDATE_CLASS_METADATA_MIN_LENGTH..]).map_err(|_| ProgramError::InvalidInstructionData)?;

        return Ok(UpdateClassMetadata { accounts, metadata });
    }
}

impl <'info> UpdateClassMetadata<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = self.name.len() + self.metadata.len() + 3;
        let rent = Rent::get()?.minimum_balance(space);

        let lamports = rent.saturating_sub(self.accounts.class.lamports());

        let name_hash = solana_nostd_sha256::hash(self.name.as_bytes());

        let seeds = [
            b"class",
            self.accounts.authority.key().as_ref(),
            &name_hash,
        ];
            
        let bump: [u8;1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"class"),
            Seed::from(self.accounts.authority.key()),
            Seed::from(&name_hash),
            Seed::from(&bump)
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.authority,
            to: self.accounts.class,
            lamports,
            space: space as u64,
            owner: &crate::ID
        }.invoke_signed(
            &signers
        )?;

        let new_class = Class {
            authority: *self.accounts.authority.key(),
            is_frozen: false,
            credential_account: if self.is_permissioned {
               Some(*self.accounts.authority.key())
            } else {
                None
            },
            name: self.name,
            metadata: self.metadata
        };

        let mut data: account_info::RefMut<'_, [u8]> = self.accounts.class.try_borrow_mut_data()?;
        new_class.initialize(&mut data)
    }
}


pub struct UpdateClassPermission<'info> {
    accounts: UpdateClassAccounts<'info>,
    is_permissioned: bool,
}

pub const UPDATE_CLASS_PERMISSION_LENGTH: usize = size_of::<bool>();

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
