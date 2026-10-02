use crate::{
    state::{Class, Record, CLASS_OFFSET, OWNER_OFFSET},
    token2022::Token,
    utils::{ByteReader, Context},
};
use core::mem::size_of;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_token_2022::instructions::{FreezeAccount, ThawAccount};

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
/// 5. `class` - The class of the record to be frozen/unfrozen
/// 6. `token_2022_program` - Required for freezing/unfreezing the token account
///
/// # Security
/// The authority must be: the class authority
pub struct FreezeTokenizedRecordAccounts {
    mint: AccountView,
    token_account: AccountView,
    record: AccountView,
}

impl TryFrom<&[AccountView]> for FreezeTokenizedRecordAccounts {
    type Error = ProgramError;
    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, mint, token_account, record, class, _token_2022_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the class authority
        Class::check_authority(class, authority)?;

        // Check if the Record is correct
        Record::check_program_id_and_discriminator(record)?;

        let record_data = record.try_borrow()?;
        // Check if the class is the correct class
        if record_data[CLASS_OFFSET..CLASS_OFFSET + size_of::<Address>()].ne(class.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the token is linked to the record
        if record_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Address>()].ne(mint.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            mint: *mint,
            token_account: *token_account,
            record: *record,
        })
    }
}

const IS_FROZEN_OFFSET: usize = 0;

pub struct FreezeTokenizedRecord {
    accounts: FreezeTokenizedRecordAccounts,
    is_frozen: bool,
}

/// Minimum length of instruction data required for FreezeRecord
pub const FREEZE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for FreezeTokenizedRecord {
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

impl FreezeTokenizedRecord {
    pub fn process(ctx: Context<'_>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let is_frozen = unsafe {
            Token::get_is_frozen_unchecked(&self.accounts.token_account.try_borrow()?)?
        };

        if is_frozen.eq(&self.is_frozen) {
            return Ok(());
        }

        let bump = [Address::try_find_program_address(&[b"mint", self.accounts.record.address().as_ref()], &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?
            .1];

        let seeds = [Seed::from(b"mint"), Seed::from(self.accounts.record.address().as_ref()), Seed::from(&bump)];

        let signers = [Signer::from(&seeds)];

        if self.is_frozen {
            FreezeAccount::new(&self.accounts.token_account, &self.accounts.mint, &self.accounts.mint)
                .invoke_signed(&signers)?;
        } else {
            ThawAccount::new(&self.accounts.token_account, &self.accounts.mint, &self.accounts.mint)
                .invoke_signed(&signers)?;
        }

        Ok(())
    }
}
