use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{sdk::Context, state::Credential};

/// # CreateCredential
/// 
/// Creates credential account identifying authorities who 
/// can manage a class. D3 and Registry Operators can 
/// create credentials.
/// 
/// Accounts:
/// 1. authority            [signer, mut]
/// 2. credential           [mut]
/// 3. system_program       [executable]
/// 
/// Parameters:
/// 1. name                 [str]
/// 2. authorized_signers   [Vec<Pubkey>]
pub struct CreateCredentialAccounts<'info> {
    authority: &'info AccountInfo,
    credential: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateCredentialAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, credential, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            authority,
            credential
        })
    }
}
pub struct CreateCredential<'info> {
    accounts: CreateCredentialAccounts<'info>,
    name: &'info str,
    authorized_signers: &'info [Pubkey],
}

pub const CREATE_CREDENTIAL_MIN_IX_LENGTH: usize = size_of::<u8>() + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateCredential<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateCredentialAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length of at least 5 for boolean and length bytes
        if ctx.data.len() < CREATE_CREDENTIAL_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        let name_len = ctx.data[0] as usize;

        let name = std::str::from_utf8(
            &ctx.data[CREATE_CREDENTIAL_MIN_IX_LENGTH..CREATE_CREDENTIAL_MIN_IX_LENGTH + name_len]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        let authorized_signers_len = ctx.data[CREATE_CREDENTIAL_MIN_IX_LENGTH + name_len] as usize;

        if authorized_signers_len < 1 || authorized_signers_len > 16 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let authorized_signers = unsafe {
            std::slice::from_raw_parts(
                ctx.data[CREATE_CREDENTIAL_MIN_IX_LENGTH + name_len + 1..CREATE_CREDENTIAL_MIN_IX_LENGTH + name_len + authorized_signers_len * 32].as_ptr() as *const Pubkey,
                authorized_signers_len
            )
        };

        Ok(Self {
            accounts,
            name,
            authorized_signers
        })
    }
}

impl <'info> CreateCredential<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = Credential::MINIMUM_CLASS_SIZE + self.name.len() + self.authorized_signers.len() * 32;
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.credential.lamports());

        let name_hash = solana_nostd_sha256::hash(self.name.as_bytes());

        let seeds = [
            b"credential",
            self.accounts.authority.key().as_ref(),
            &name_hash,
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"credential"),
            Seed::from(self.accounts.authority.key()),
            Seed::from(&name_hash),
            Seed::from(&bump)
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.authority,
            to: self.accounts.credential,
            lamports,
            space: space as u64,
            owner: &crate::ID
        }.invoke_signed(
            &signers
        )?;

        let credential = Credential {
            authority: *self.accounts.authority.key(),
            name: self.name,
            authorized_signers: self.authorized_signers
        };

        credential.initialize(&self.accounts.credential)?;

        Ok(())
    }
}