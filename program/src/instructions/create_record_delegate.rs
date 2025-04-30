use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{ctx::Context, state::{Record, RecordAuthorityExtension}};

/// Represents the accounts required for creating a new record delegate.
/// 
/// A record delegate is an account that holds authority information for a record,
/// allowing different entities to have specific permissions over the record.
/// 
/// # Accounts
/// 
/// * `owner` - The current owner of the record (must be a signer)
/// * `record` - The record account that will be associated with the delegate
/// * `delegate` - The new delegate account to be created
/// 
/// # Security
/// 
/// The owner account must be a signer and must match the current owner of the record.
/// The delegate account will be owned by the program and can only be modified
/// through program instructions.
pub struct CreateRecordDelegateAccounts<'info> {
    owner: &'info AccountInfo,
    record: &'info AccountInfo,
    delegate: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateRecordDelegateAccounts<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateRecordDelegateAccounts from a slice of AccountInfo.
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
    /// * `ProgramError::InvalidAccountData` - If owner doesn't match record owner
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, record, delegate, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if Record::from_bytes(&record.try_borrow_data()?)?.owner != *owner.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            owner,
            record,
            delegate
        })
    }
}

/// Represents the CreateRecordDelegate instruction with all its parameters.
/// 
/// This struct contains all the data needed to create a new record delegate,
/// including the accounts and various authority settings.
/// 
/// # Fields
/// 
/// * `accounts` - The required accounts for the instruction
/// * `update_authority` - Public key of the entity that can update the record
/// * `freeze_authority` - Public key of the entity that can freeze the record
/// * `transfer_authority` - Public key of the entity that can transfer the record
/// * `burn_authority` - Public key of the entity that can burn the record
/// * `authority_program` - Optional program that can manage authorities
pub struct CreateRecordDelegate<'info> {
    accounts: CreateRecordDelegateAccounts<'info>,
    update_authority: Pubkey,
    freeze_authority: Pubkey,
    transfer_authority: Pubkey,
    burn_authority: Pubkey,
    authority_program: Option<Pubkey>,
}

/// Minimum length of instruction data required for CreateRecordDelegate
/// 
/// This constant represents the minimum number of bytes needed in the instruction
/// data, which includes:
/// * 32 bytes for each authority (update, freeze, transfer, burn)
/// * 1 byte for the authority program flag
pub const CREATE_RECORD_DELEGATE_MIN_IX_LENGTH: usize = size_of::<Pubkey>() * 4 + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateRecordDelegate<'info> {
    type Error = ProgramError;

    /// Attempts to create a CreateRecordDelegate instruction from a Context.
    /// 
    /// This function deserializes and validates the instruction data,
    /// including parsing all authority public keys and the optional authority program.
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
    /// * `ProgramError::InvalidArgument` - If authority program data is invalid
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateRecordDelegateAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length of at least 5 for boolean and length bytes
        if ctx.data.len() < CREATE_RECORD_DELEGATE_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let update_authority: Pubkey = ctx.data[0..32].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let freeze_authority: Pubkey = ctx.data[32..64].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let transfer_authority: Pubkey = ctx.data[64..96].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let burn_authority: Pubkey = ctx.data[96..128].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let authority_program = if ctx.data[128] != 0 {
            Some(ctx.data[97..129].try_into().map_err(|_| ProgramError::InvalidInstructionData)?)
        } else {
            None
        };
        
        Ok(Self {
            accounts,
            update_authority,
            freeze_authority,
            transfer_authority,
            burn_authority,
            authority_program
        })
    }
}

impl <'info> CreateRecordDelegate<'info> {
    /// Processes the CreateRecordDelegate instruction.
    /// 
    /// This is the main entry point for the CreateRecordDelegate instruction.
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

    /// Executes the CreateRecordDelegate instruction.
    /// 
    /// This function:
    /// 1. Calculates required account space and rent
    /// 2. Derives the PDA for the delegate account
    /// 3. Creates the new account
    /// 4. Initializes the delegate data with authority settings
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
        let space = RecordAuthorityExtension::MINIMUM_RECORD_SIZE;
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.delegate.lamports());

        let seeds = [
            b"authority",
            self.accounts.record.key().as_ref(),
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"authority"),
            Seed::from(self.accounts.record.key()),
            Seed::from(&bump)
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.owner,
            to: self.accounts.delegate,
            lamports,
            space: space as u64,
            owner: &crate::ID
        }.invoke_signed(
            &signers
        )?;

        let record = RecordAuthorityExtension {
            record: *self.accounts.delegate.key(),
            update_authority: self.update_authority,
            burn_authority: self.burn_authority,
            freeze_authority: self.freeze_authority,
            transfer_authority: self.transfer_authority,
            authority_program: self.authority_program,
        };

        record.initialize(self.accounts.record)
    }
}