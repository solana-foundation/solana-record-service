use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::try_find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{accounts::Class, sdk::Context};

/// # CreateClass
/// 
/// Create a new namespace for records 
/// 
/// Callers: D3, Ecosystem Partners
/// 
/// Parameters:
/// is_permissioned: bool 
/// name: String 
/// metadata: Option<String>
/// 
/// Accounts:
/// Authority (signer)
/// Class PDA
/// System Program
pub struct CreateClass<'info> {
    accounts: CreateClassAccounts<'info>,
    name: &'info str,
    metadata: &'info str,
}

pub struct CreateClassAccounts<'info> {
    authority: &'info AccountInfo,
    class: &'info AccountInfo,
    credential: Option<&'info AccountInfo>
}

pub const CREATE_CLASS_MIN_IX_LENGTH: usize = size_of::<bool>() + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateClass<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateClassAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length of at least 5 for boolean and length bytes
        if ctx.data.len() < CREATE_CLASS_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }
        let name_len = ctx.data[1] as usize;

        // Check IX data matches our name length
        if ctx.data.len() < CREATE_CLASS_MIN_IX_LENGTH + name_len {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Get name slice
        let name = str::from_utf8(&ctx.data[CREATE_CLASS_MIN_IX_LENGTH..CREATE_CLASS_MIN_IX_LENGTH+name_len]).map_err(|_| ProgramError::InvalidInstructionData)?;

        // Get metadata slice (could be empty)
        let metadata= str::from_utf8(&ctx.data[CREATE_CLASS_MIN_IX_LENGTH+name_len..]).map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok( Self { 
            accounts,
            name, 
            metadata
        })
    }
}

impl <'info> CreateClass<'info> {
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

        let class = Class {
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

        class.initialize(self.accounts.class)
    }
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateClassAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, class, _system_program, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let credential = rest.first();

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // If a credential account exists, check that it belongs to our program
        if let Some(credential_account) = credential {
            if unsafe { credential_account.owner().ne(&crate::ID)  } {
                return Err(ProgramError::InvalidAccountOwner);
            }
        }

        Ok(Self {
            authority,
            class,
            credential
        })
    }
}