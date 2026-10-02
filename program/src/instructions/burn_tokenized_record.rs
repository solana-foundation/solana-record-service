use crate::{
    state::{OwnerType, Record},
    token2022::Token,
    utils::Context,
};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_token_2022::instructions::{BurnChecked, CloseAccount, ThawAccount};

/// BurnTokenizedRecord instruction.
///
/// This function:
/// 1. Burns the mint
/// 2. Closes the mint account
/// 3. Sets the record owner to the owner of the token account and the owner type to pubkey
///
/// # Accounts
/// 1. `authority` - The account that has permission to burn the record token (must be a signer)
/// 2. `destination` - The account that will get refunded for the record account
/// 2. `mint` - The mint account of the record token
/// 3. `token_account` - The token account of the record token
/// 4. `record` - The record account to be deleted
/// 5. `token_2022_program` - Required for burning the token account
/// 6. `class` - [remaining accounts] Required if the authority is not the record owner but the permissioned authority
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. if the class is permissioned, the authority must be the permissioned authority
pub struct BurnTokenizedRecordAccounts {
    destination: AccountView,
    record: AccountView,
    mint: AccountView,
    token_account: AccountView,
}

impl TryFrom<&[AccountView]> for BurnTokenizedRecordAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, destination, mint, token_account, record, _token_2022_program, rest @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the record owner or has a delegate
        Record::check_owner_or_delegate_tokenized(
            record,
            rest.first(),
            authority,
            mint,
            token_account,
        )?;

        Ok(Self {
            destination: *destination,
            record: *record,
            mint: *mint,
            token_account: *token_account,
        })
    }
}

pub struct BurnTokenizedRecord {
    accounts: BurnTokenizedRecordAccounts,
}

impl<'info> TryFrom<Context<'info>> for BurnTokenizedRecord {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = BurnTokenizedRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self { accounts })
    }
}

impl BurnTokenizedRecord {
    pub fn process(ctx: Context<'_>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        let bump = [Address::try_find_program_address(&[b"mint", self.accounts.record.address().as_ref()], &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?
            .1];

        let seeds = [Seed::from(b"mint"), Seed::from(self.accounts.record.address().as_ref()), Seed::from(&bump)];

        let signers = [Signer::from(&seeds)];

        let is_frozen = unsafe { Token::get_is_frozen_unchecked(&self.accounts.token_account.try_borrow()?)? };

        if is_frozen {
            ThawAccount::new(&self.accounts.token_account, &self.accounts.mint, &self.accounts.mint)
                .invoke_signed(&signers)?;
        }

        // Burn the mint
        BurnChecked::new(&self.accounts.token_account, &self.accounts.mint, &self.accounts.mint, 1, 0)
            .invoke_signed(&signers)?;

        // Close the mint account
        CloseAccount::new(&self.accounts.mint, &self.accounts.destination, &self.accounts.mint)
            .invoke_signed(&signers)?;

        // Set the record owner, to the owner of the token account and the owner type to pubkey
        let record_owner = unsafe { Token::get_owner_unchecked(&self.accounts.token_account.try_borrow()?)? };

        unsafe {
            Record::update_is_frozen_unchecked(&mut self.accounts.record.try_borrow_mut()?, is_frozen)?;
            Record::update_owner_unchecked(&mut self.accounts.record.try_borrow_mut()?, &record_owner)?;
            Record::update_owner_type_unchecked(&mut self.accounts.record.try_borrow_mut()?, OwnerType::Pubkey)?;
        };

        Ok(())
    }
}
