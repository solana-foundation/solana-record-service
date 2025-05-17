use crate::{
    state::Record,
    token2022::{FreezeAccount, ThawAccount, Token},
    utils::{ByteReader, Context},
};
use core::mem::size_of;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::try_find_program_address,
    ProgramResult,
};

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
/// 4. `record` - The record account to be frozen/unfrozen
/// 5. `token_2022_program` - Required for freezing/unfreezing the token account
/// 6. `class` - [remaining accounts] Required if the authority is not the record owner
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. A delegate with freeze authority
pub struct FreezeTokenizedRecordAccounts<'info> {
    mint: &'info AccountInfo,
    token_account: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for FreezeTokenizedRecordAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, mint, token_account, record, _token_2022_program, rest @ ..] = accounts else {
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

        Ok(Self {
            mint,
            token_account,
            record,
        })
    }
}

const IS_FROZEN_OFFSET: usize = 0;

pub struct FreezeTokenizedRecord<'info> {
    accounts: FreezeTokenizedRecordAccounts<'info>,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeRecord
pub const FREEZE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeTokenizedRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = FreezeTokenizedRecordAccounts::try_from(ctx.accounts)?;

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

impl<'info> FreezeTokenizedRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Freeze Tokenized Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let is_frozen = unsafe {
            Token::get_is_frozen_unchecked(&self.accounts.token_account.try_borrow_data()?)?
        };

        if is_frozen.eq(&self.is_frozen) {
            return Ok(());
        }

        let bump =
            [
                try_find_program_address(
                    &[b"mint", self.accounts.record.key()],
                    &crate::ID,
                )
                .ok_or(ProgramError::InvalidArgument)?
                .1,
            ];

        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(&bump),
        ];

        let signers = [Signer::from(&seeds)];

        if self.is_frozen {
            FreezeAccount {
                mint: self.accounts.mint,
                account: self.accounts.token_account,
                freeze_authority: self.accounts.mint,
            }
            .invoke_signed(&signers)?;
        } else {
            ThawAccount {
                mint: self.accounts.mint,
                account: self.accounts.token_account,
                freeze_authority: self.accounts.mint,
            }
            .invoke_signed(&signers)?;
        }

        Ok(())
    }
}
