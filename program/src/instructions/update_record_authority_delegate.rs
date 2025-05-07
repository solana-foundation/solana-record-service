use core::mem::size_of;
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    state::{Record, RecordAuthorityDelegate},
    utils::{ByteReader, Context},
};

/// UpdateRecordAuthorityDelegate instruction.
///
/// This function:
/// 1. Validates the Record Authority and Record Delegate
/// 2. Updates the Record Delegate's data content
///
/// # Accounts
/// 1. `owner` - The current owner of the record (must be a signer)
/// 2. `record` - The record account that will be associated with the delegate
/// 3. `delegate` - The delegate account to be modified
///
/// # Security
/// 1. The owner account must be a signer and must match the current owner of the record.
pub struct UpdateRecordAuthorityDelegateAccounts<'info> {
    delegate: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateRecordAuthorityDelegateAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, record, delegate] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check owner
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check record authority
        Record::check_authority_or_delegate(
            record,
            owner,
            Some(delegate),
            Record::UPDATE_AUTHORITY_DELEGATION_TYPE,
        )?;

        Ok(Self { delegate })
    }
}

const UPDATE_AUTHORITY_OFFSET: usize = 0;
const FREEZE_AUTHORITY_OFFSET: usize = UPDATE_AUTHORITY_OFFSET + size_of::<Pubkey>();
const TRANSFER_AUTHORITY_OFFSET: usize = FREEZE_AUTHORITY_OFFSET + size_of::<Pubkey>();
const BURN_AUTHORITY_OFFSET: usize = TRANSFER_AUTHORITY_OFFSET + size_of::<Pubkey>();
const AUTHORITY_PROGRAM_OFFSET: usize = BURN_AUTHORITY_OFFSET + size_of::<Pubkey>();

pub struct UpdateRecordAuthorityDelegate<'info> {
    accounts: UpdateRecordAuthorityDelegateAccounts<'info>,
    update_authority: Pubkey,
    freeze_authority: Pubkey,
    transfer_authority: Pubkey,
    burn_authority: Pubkey,
    authority_program: Pubkey,
}

/// Minimum length of instruction data required for CreateRecordAuthorityDelegate
pub const UPDATE_RECORD_AUTHORITY_DELEGATE_MIN_IX_LENGTH: usize = size_of::<Pubkey>() * 5;

impl<'info> TryFrom<Context<'info>> for UpdateRecordAuthorityDelegate<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = UpdateRecordAuthorityDelegateAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < UPDATE_RECORD_AUTHORITY_DELEGATE_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize `update_authority`
        let update_authority: Pubkey =
            ByteReader::read_with_offset(ctx.data, UPDATE_AUTHORITY_OFFSET)?;

        // Deserialize `freeze_authority`
        let freeze_authority: Pubkey =
            ByteReader::read_with_offset(ctx.data, FREEZE_AUTHORITY_OFFSET)?;

        // Deserialize `transfer_authority`
        let transfer_authority: Pubkey =
            ByteReader::read_with_offset(ctx.data, TRANSFER_AUTHORITY_OFFSET)?;

        // Deserialize `burn_authority`
        let burn_authority: Pubkey = ByteReader::read_with_offset(ctx.data, BURN_AUTHORITY_OFFSET)?;

        // Deserialize `burn_authority`
        let authority_program: Pubkey =
            ByteReader::read_with_offset(ctx.data, AUTHORITY_PROGRAM_OFFSET)?;

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

impl<'info> UpdateRecordAuthorityDelegate<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Record Authority Delegate");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        RecordAuthorityDelegate::update(
            self.accounts.delegate,
            self.update_authority,
            self.freeze_authority,
            self.transfer_authority,
            self.burn_authority,
            self.authority_program,
        )
    }
}
