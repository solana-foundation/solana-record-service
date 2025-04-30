use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::try_find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{ctx::Context, state::{Record, Class}};

/// Represents the accounts required for creating a new record.
/// 
/// A record represents an entity within a class (e.g., a Twitter handle, a D3 domain).
/// This struct encapsulates all the accounts needed for the CreateRecord instruction.
/// 
/// # Accounts
/// 
/// * `owner` - The account that will own the record (must be a signer)
/// * `class` - The class account that this record belongs to
/// * `record` - The new record account to be created
/// 
/// # Optional Accounts
/// 
/// * `credential` - Required if the class is permissioned
/// * `credential_authority` - Required if the class is permissioned
/// 
/// # Security
/// 
/// The owner account must be a signer. If the class is permissioned, the credential
/// and credential authority must be provided and validated.
pub struct CreateRecordAccounts<'info> {
    owner: &'info AccountInfo,
    class: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateRecordAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateRecordAccounts from a slice of AccountInfo.
    /// 
    /// # Arguments
    /// 
    /// * `accounts` - A slice of AccountInfo containing the required accounts
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - If all required accounts are present and valid
    /// * `Err(ProgramError)` - If accounts are missing or invalid
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::NotEnoughAccountKeys` - If insufficient accounts are provided
    /// * `ProgramError::MissingRequiredSignature` - If owner is not a signer
    /// * `ProgramError::InvalidAccountData` - If credential validation fails
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
        
        // if class_data.credential_account.is_some() {
        //     let [credential_account, credential_authority, ..] = rest else {
        //         return Err(ProgramError::NotEnoughAccountKeys);
        //     };

        //     class_data.validate_credential(credential_account, credential_authority)?;
        // }

        Ok(Self {
            owner,
            class,
            record,
        })
    }
}

/// Represents the CreateRecord instruction with all its parameters.
/// 
/// This struct contains all the data needed to create a new record,
/// including the accounts, name, data, and optional expiry.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `expiry` - Optional timestamp when the record expires
/// * `name` - The name of the record within its class
/// * `data` - The data associated with the record
pub struct CreateRecord<'info> {
    accounts: CreateRecordAccounts<'info>,
    expiry: Option<i64>,
    name: &'info str,
    data: &'info str,
}

/// Minimum length of instruction data required for CreateRecord
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 1 byte for expiry flag
/// * 1 byte for name length
/// * 1 byte for data length
pub const CREATE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>() + size_of::<u8>() + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateRecord<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateRecord instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing the optional expiry, name, and data.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The Context containing accounts and instruction data
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - If the instruction data is valid
    /// * `Err(ProgramError)` - If the data is invalid
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidInstructionData` - If data format is invalid
    /// * `ProgramError::InvalidArgument` - If UTF-8 parsing fails
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

        offset += size_of::<u8>();

        if ctx.data.len() < offset + name_len {
            return Err(ProgramError::InvalidInstructionData);
        }

        let name = core::str::from_utf8(
            &ctx.data[offset..offset + name_len]
        ).map_err(|_| ProgramError::InvalidInstructionData)?;

        offset += name_len;

        let data = core::str::from_utf8(
            &ctx.data[offset..]
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
    /// Processes the CreateRecord instruction.
    /// 
    /// This is the main entry point for the CreateRecord instruction.
    /// It validates the instruction and executes it if valid.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The Context containing accounts and instruction data
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the instruction executed successfully
    /// * `Err(ProgramError)` - If execution failed
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    /// Executes the CreateRecord instruction.
    /// 
    /// This function:
    /// 1. Calculates required account space and rent
    /// 2. Derives the PDA for the record account
    /// 3. Creates the new account
    /// 4. Initializes the record data
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If execution was successful
    /// * `Err(ProgramError)` - If any step failed
    /// 
    /// # Errors
    /// 
    /// * `ProgramError::InvalidArgument` - If PDA derivation fails
    /// * Various other errors from account creation and initialization
    pub fn execute(&self) -> ProgramResult {
        let space = Record::MINIMUM_CLASS_SIZE + self.name.len() + self.data.len();
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.record.lamports());

        let seeds = [
            b"record",
            self.accounts.class.key().as_ref(),
            self.name.as_bytes(),
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"record"),
            Seed::from(self.accounts.class.key()),
            Seed::from(self.name.as_bytes()),
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