use crate::{state::{OwnerType, Record}, token2022::{BurnChecked, CloseAccount, Token}, utils::Context};
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::try_find_program_address, ProgramResult};

/// BurnTokenizedRecord instruction.
///
/// This function:
/// 1. Burns the mint
/// 2. Closes the mint account
/// 3. Sets the record owner to the owner of the token account and the owner type to pubkey
///
/// # Accounts
/// 1. `authority` - The account that has permission to burn the record token (must be a signer)
/// 2. `mint` - The mint account of the record token
/// 3. `token_account` - The token account of the record token
/// 4. `record` - The record account to be deleted
/// 5. `class` - [remaining accounts] Required if the authority is not the record owner but the permissioned authority
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. if the class is permissioned, the authority must be the permissioned authority
pub struct BurnTokenizedRecordAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
    mint: &'info AccountInfo,
    token_account: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for BurnTokenizedRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, mint, token_account, record, rest @ ..] = accounts else {
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

        Ok(Self { authority, record, mint, token_account })
    }
}

pub struct BurnTokenizedRecord<'info> {
    accounts: BurnTokenizedRecordAccounts<'info>,
}

impl<'info> TryFrom<Context<'info>> for BurnTokenizedRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = BurnTokenizedRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self { accounts })
    }
}

impl<'info> BurnTokenizedRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Burn Tokenized Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let bump =
        [
            try_find_program_address(
                &[b"mint", self.accounts.record.key().as_ref()],
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

        // Burn the mint
        BurnChecked {
            mint: self.accounts.mint,
            account: self.accounts.token_account,
            authority: self.accounts.mint,
            amount: 1,
            decimals: 0,
        }.invoke_signed(&signers)?;

        // Close the mint account
        CloseAccount {
            account: self.accounts.mint,
            destination: self.accounts.authority,
            authority: self.accounts.mint,
        }.invoke_signed(&signers)?;

        // Set the record owner, to the owner of the token account and the owner type to pubkey
        let record_owner = unsafe { Token::get_owner_unchecked(&self.accounts.token_account.try_borrow_data()?)? };

        unsafe { 
            Record::update_owner_unchecked(&mut self.accounts.record.try_borrow_mut_data()?, &record_owner);
            Record::update_owner_type_unchecked(&mut self.accounts.record.try_borrow_mut_data()?, OwnerType::Pubkey);
        };

        Ok(())
    }
}
