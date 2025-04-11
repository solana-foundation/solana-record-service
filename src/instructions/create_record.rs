use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::try_find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{sdk::Context, state::{Record, Class, Credential}};

/// # CreateRecord
/// 
/// Creates a new record (e.g., a Twitter handle, a D3 domain) 
/// that defines a namespace for records. D3, Integrator and Users
/// can create record.
/// 
/// Accounts:
/// 1. Owner                [signer, mut]
/// 2. class                [mut]
/// 3. record               [mut]
/// 4. system_program       [executable]
/// 5. credential           [optional]
/// 6. credential_authority [optional]
/// 
/// Parameters:
/// 1. expiry               [Option<i64>] 
/// 2. name                 [str]
/// 3. data                 [str]
pub struct CreateRecordAccounts<'info> {
    owner: &'info AccountInfo,
    class: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, class, record, _system_program, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if the class has a credential account, if it does, check that the credential account passed in is the correct one
        let class_borrowed_data = class.try_borrow_data()?;
        let class_data = Class::from_bytes(class_borrowed_data.as_ref())?;
        
        if class_data.credential_account.is_some() {
            let [credential_account, credential_authority, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };

            class_data.validate_credential(credential_account, credential_authority)?;
        }

        Ok(Self {
            owner,
            class,
            record,
        })
    }
}
pub struct CreateRecord<'info> {
    accounts: CreateRecordAccounts<'info>,
    expiry: Option<i64>,
    name: &'info str,
    data: &'info str,
}

pub const CREATE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>() + size_of::<u8>() + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length of at least 5 for boolean and length bytes
        if ctx.data.len() < CREATE_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut offset = 0;

        let expiry = if ctx.data[offset] == 1 {
            offset += 1;

            if ctx.data.len() < offset + size_of::<i64>() {
                return Err(ProgramError::InvalidInstructionData);
            }

            let expiry_bytes = ctx.data[offset..offset + size_of::<i64>()]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            
            offset += size_of::<i64>();

            Some(i64::from_le_bytes(expiry_bytes))
        } else {
            offset += 1;

            None
        };

        let name_len = ctx.data[offset] as usize;

        offset += 1;

        if ctx.data.len() < offset + name_len {
            return Err(ProgramError::InvalidInstructionData);
        }

        let name = std::str::from_utf8(
            &ctx.data[offset..offset + name_len]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        offset += name_len;

        let data_len = ctx.data[offset] as usize;

        offset += 1;

        if ctx.data.len() < offset + data_len {
            return Err(ProgramError::InvalidInstructionData);
        }

        let data = std::str::from_utf8(
            &ctx.data[offset..offset + data_len]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            accounts,
            expiry,
            name,
            data
        })
        
    }
}

impl <'info> CreateRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = Record::MINIMUM_CLASS_SIZE + self.name.len() + self.data.len();
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.record.lamports());

        let name_hash = solana_nostd_sha256::hash(self.name.as_bytes());

        let seeds = [
            b"credential",
            self.accounts.class.key().as_ref(),
            &name_hash,
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"credential"),
            Seed::from(self.accounts.class.key()),
            Seed::from(&name_hash),
            Seed::from(&bump)
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.owner,
            to: self.accounts.record,
            lamports,
            space: space as u64,
            owner: &crate::ID
        }.invoke_signed(
            &signers
        )?;

        let record = Record {
            class: *self.accounts.class.key(),
            owner: *self.accounts.owner.key(),
            is_frozen: false,
            has_authority_extension: self.expiry.is_some(),
            expiry: self.expiry,
            name: self.name,
            data: self.data
        };

        record.initialize(self.accounts.record)
    }
}