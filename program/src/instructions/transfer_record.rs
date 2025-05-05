use crate::{
    state::Record,
    utils::{ByteReader, Context},
};
use core::mem::size_of;
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

/// TransferRecord instruction.
///
/// This function:
/// 1. Loads the current record state
/// 2. Updates the owner to the new owner
/// 3. Saves the updated state
///
/// # Accounts
/// * `authority` - The account that has permission to transfer the record (must be a signer)
/// * `record` - The record account to be transferred
///
/// # Optional Accounts
/// * `record_delegate` - Required if the authority is not the record owner
///
/// # Security
///
/// The authority must be either:
/// 1. The record owner, or
/// 2. A delegate with transfer authority (requires record_delegate account)
///
/// The instruction will fail if:
/// 1. The record is frozen (frozen records cannot be transferred)
/// 2. The authority is not the record owner or a valid delegate
/// 3. The new owner is the same as the current owner
pub struct TransferRecordAccounts<'info> {
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for TransferRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Record::check_authority_or_delegate(&record, authority.key(), rest.first())?;

        Ok(Self { record })
    }
}

pub struct TransferRecord<'info> {
    accounts: TransferRecordAccounts<'info>,
    new_owner: Pubkey,
}

/// Minimum length of instruction data required for TransferRecord
pub const TRANSFER_RECORD_MIN_IX_LENGTH: usize = size_of::<Pubkey>();

impl<'info> TryFrom<Context<'info>> for TransferRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = TransferRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length and create a byte reader
        let mut data = ByteReader::new_with_minimum_size(ctx.data, TRANSFER_RECORD_MIN_IX_LENGTH)?;

        // Deserialize new owner
        let new_owner: Pubkey = data.read()?;

        Ok(Self {
            accounts,
            new_owner,
        })
    }
}

impl<'info> TransferRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Transfer Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        Record::update_owner(self.accounts.record, self.new_owner)
    }
}
