use core::mem::size_of;
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{try_find_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    state::{Record, RecordAuthorityDelegate},
    utils::{ByteReader, Context},
};

/// CreateRecordAuthorityDelegate instruction.
///
/// A record authority delegate is an account that holds authority information for a record,
/// allowing different entities to have specific permissions over the record.
///
/// This function:
/// 1. Calculates required account space and rent
/// 2. Derives the PDA for the delegate account
/// 3. Creates the new account
/// 4. Transfers the minimum rent needed to make the account rent-exempt
/// 5. Initializes the delegate data with authority settings
///
/// # Accounts
/// * `owner` - The current owner of the record (must be a signer)
/// * `record` - The record account that will be associated with the delegate
/// * `delegate` - The new delegate account to be created
///
/// # Security
/// The owner account must be a signer and must match the current owner of the record.
/// The delegate account will be owned by the program and can only be modified
/// through program instructions.
pub struct CreateRecordAuthorityDelegateAccounts<'info> {
    owner: &'info AccountInfo,
    record: &'info AccountInfo,
    delegate: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateRecordAuthorityDelegateAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, record, delegate, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check owner
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check record authority
        Record::check_authority(&record.try_borrow_data()?, owner.key())?;

        Ok(Self {
            owner,
            record,
            delegate,
        })
    }
}
pub struct CreateRecordAuthorityDelegate<'info> {
    accounts: CreateRecordAuthorityDelegateAccounts<'info>,
    update_authority: Pubkey,
    freeze_authority: Pubkey,
    transfer_authority: Pubkey,
    burn_authority: Pubkey,
    authority_program: Option<Pubkey>,
}

/// Minimum length of instruction data required for CreateRecordAuthorityDelegate
pub const CREATE_RECORD_AUTHORITY_DELEGATE_MIN_IX_LENGTH: usize =
    size_of::<Pubkey>() * 4 + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateRecordAuthorityDelegate<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateRecordAuthorityDelegateAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(
            ctx.data,
            CREATE_RECORD_AUTHORITY_DELEGATE_MIN_IX_LENGTH,
        )?;

        // Deserialize `update_authority`
        let update_authority: Pubkey = data.read()?;

        // Deserialize `freeze_authority`
        let freeze_authority: Pubkey = data.read()?;

        // Deserialize `transfer_authority`
        let transfer_authority: Pubkey = data.read()?;

        // Deserialize `burn_authority`
        let burn_authority: Pubkey = data.read()?;

        // Deserialize `authority_program`
        let authority_program: Option<Pubkey> = data.read_optional()?;

        Ok(Self {
            accounts,
            update_authority,
            freeze_authority,
            transfer_authority,
            burn_authority,
            authority_program,
        })
    }
}

impl<'info> CreateRecordAuthorityDelegate<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Create Record Authority Delegate");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = RecordAuthorityDelegate::MINIMUM_RECORD_SIZE;
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.delegate.lamports());

        let seeds = [b"authority", self.accounts.record.key().as_ref()];

        let bump: [u8; 1] = [try_find_program_address(&seeds, &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?
            .1];

        let seeds = [
            Seed::from(b"authority"),
            Seed::from(self.accounts.record.key()),
            Seed::from(&bump),
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.owner,
            to: self.accounts.delegate,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signers)?;

        let record = RecordAuthorityDelegate {
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
