use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::try_find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{sdk::Context, state::{Class, Credential}};

/// # CreateClass
/// 
/// Creates a new class (e.g., TLD class, Twitter handles class) 
/// that defines a namespace for records. D3 and Ecosystem Partners 
/// can create classes.
/// 
/// Accounts:
/// 1. authority            [signer, mut]
/// 2. class                [mut]
/// 3. system_program       [executable]
/// 4. credential           [optional]
/// 
/// Parameters:
/// 1. is_permissioned      [bool] 
/// 2. name                 [str]
/// 3. metadata             [Option<str>]
pub struct CreateClassAccounts<'info> {
    authority: &'info AccountInfo,
    class: &'info AccountInfo,
    credential: Option<&'info AccountInfo>
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

        // If a credential account exists, check that it belongs to our program and that it has the correct discriminator
        if let Some(credential_account) = credential {
            if unsafe { credential_account.owner().ne(&crate::ID)  } {
                return Err(ProgramError::InvalidAccountOwner);
            }

            if unsafe { credential_account.borrow_data_unchecked() }[0].ne(&Credential::DISCRIMINATOR) {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        Ok(Self {
            authority,
            class,
            credential
        })
    }
}
pub struct CreateClass<'info> {
    accounts: CreateClassAccounts<'info>,
    is_permissioned: bool,
    name: &'info str,
    metadata: Option<&'info str>,
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

        let is_permissioned = ctx.data[0] == 1;
        let name_len = ctx.data[1] as usize;

        // Check IX data matches our name length
        if ctx.data.len() == CREATE_CLASS_MIN_IX_LENGTH + name_len {
            let name = std::str::from_utf8(
                &ctx.data[CREATE_CLASS_MIN_IX_LENGTH..CREATE_CLASS_MIN_IX_LENGTH + name_len]
            ).map_err(|_| ProgramError::InvalidInstructionData)?;

            Ok(Self {
                accounts,
                is_permissioned,
                name,
                metadata: None
            })
        } else if ctx.data.len() > CREATE_CLASS_MIN_IX_LENGTH + name_len {
            let name = std::str::from_utf8(
                &ctx.data[CREATE_CLASS_MIN_IX_LENGTH..CREATE_CLASS_MIN_IX_LENGTH + name_len]
            ).map_err(|_| ProgramError::InvalidInstructionData)?;
            
            let metadata_len = ctx.data[CREATE_CLASS_MIN_IX_LENGTH + name_len] as usize;
            if ctx.data.len() != CREATE_CLASS_MIN_IX_LENGTH + name_len + metadata_len {
                return Err(ProgramError::InvalidInstructionData);
            }

            let metadata = std::str::from_utf8(
                &ctx.data[CREATE_CLASS_MIN_IX_LENGTH + name_len..CREATE_CLASS_MIN_IX_LENGTH + name_len + metadata_len]
            ).map_err(|_| ProgramError::InvalidInstructionData)?;

            Ok(Self {
                accounts,
                is_permissioned,
                name,
                metadata: Some(metadata)
            })
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

impl <'info> CreateClass<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = Class::MINIMUM_CLASS_SIZE + self.name.len() + self.metadata.map_or(0, |m| m.len());
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.class.lamports());

        let name_hash = solana_nostd_sha256::hash(self.name.as_bytes());

        let seeds = [
            b"class",
            self.accounts.authority.key().as_ref(),
            &name_hash,
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

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
               Some(*self.accounts.credential.unwrap().key())
            } else {
                None
            },
            name: self.name,
            metadata: self.metadata.unwrap_or("")
        };

        class.initialize(self.accounts.class)
    }
}