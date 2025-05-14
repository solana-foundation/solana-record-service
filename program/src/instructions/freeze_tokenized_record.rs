use crate::{
    state::Record,
    utils::{ByteReader, Context},
};
use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// FreezeRecord instruction.
///
/// This function:
/// 1. Loads the current record state
/// 2. Updates the frozen status
/// 3. Saves the updated state
///
/// # Accounts
/// 1. `authority` - The account that has permission to freeze/unfreeze the record (must be a signer)
/// 2. `mint` - The mint account that that is linked to the record
/// 3. `token_account` - The token account that is linked to the record
/// 2. `record` - The record account to be frozen/unfrozen
/// 3. `record_delegate` - [remaining accounts] Required if the authority is not the record owner
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. A delegate with freeze authority
pub struct FreezeRecordAccounts<'info> {
    mint: &'info AccountInfo,
    token_account: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeRecordAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, mint, token_account, record, _system_program, rest @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if owner is the record owner or has a delegate
        Record::check_owner_or_delegate_tokenized(
            record,
            rest.first(),
            owner,
            mint,
            token_account,
        )?;

        Ok(Self { mint, token_account, record })
    }
}

const IS_FROZEN_OFFSET: usize = 0;

pub struct FreezeRecord<'info> {
    accounts: FreezeRecordAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeRecord
pub const FREEZE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = FreezeRecordAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < FREEZE_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize `is_frozen`
        let is_frozen: bool = ByteReader::read_with_offset(ctx.data, IS_FROZEN_OFFSET)?;

        Ok(Self {
            accounts,
            is_frozen,
        })
    }
}

impl<'info> FreezeRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Freeze Tokenized Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        todo!("Cpi into the Lock or Thaw instruction based on the self.is_frozen value. No need to check if it's currently frozen or not since it will just fail")
    }
}
