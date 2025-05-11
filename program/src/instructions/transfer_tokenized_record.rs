use crate::{
    constants::TOKEN_ACCOUNT_OWNER_OFFSET, state::Record, token2022::TransferChecked, utils::Context
};
use core::mem::size_of;
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, ProgramResult
};

/// TransferRecord instruction.
///
/// This function:
/// 1. Loads the current record state
/// 2. Updates the owner to the new owner
/// 3. Saves the updated state
///
/// # Accounts
/// 1. `authority` - The account that has permission to update the record (must be a signer)
/// 2. `mint` - The mint account that that is linked to the record
/// 3. `token_account` - The token account that is linked to the record
/// 4. `new_token_account` - The new owner of the token account
/// 5. `record` - The record account to be updated
/// 6. `system_program` - Required for account resizing operations
/// 7. `delegate` - The delegate of the token account
///
/// # Security
/// 1. The authority must be:
///    a. The mint's owner, or
///    b. An authorized delegate with transfer permissions
/// 2. The record must not be frozen
pub struct TransferRecordAccounts<'info> {
    authority: &'info AccountInfo,
    mint: &'info AccountInfo,
    token_account: &'info AccountInfo,
    new_token_account: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for TransferRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, mint, token_account, new_token_account, record, _system_program, rest @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if authority is the record owner or has a delegate
        Record::check_authority_or_delegate_tokenized(
            record,
            authority,
            mint,
            token_account,
            rest.first(),
            Record::TRANSFER_AUTHORITY_DELEGATION_TYPE,
        )?;

        Ok(Self { authority, mint, token_account, new_token_account, record })
    }
}

pub struct TransferRecord<'info> {
    accounts: TransferRecordAccounts<'info>,
}

impl<'info> TryFrom<Context<'info>> for TransferRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = TransferRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self {
            accounts,
        })
    }
}

impl<'info> TransferRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Transfer Tokenized Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let token_account_data = self.accounts.token_account.try_borrow_data()?;
        if token_account_data[TOKEN_ACCOUNT_OWNER_OFFSET..TOKEN_ACCOUNT_OWNER_OFFSET + size_of::<Pubkey>()].eq(self.accounts.authority.key()) {
            TransferChecked {
                source: self.accounts.token_account,
                mint: self.accounts.mint,
                destination: self.accounts.new_token_account,
                authority: self.accounts.authority,
                amount: 1,
                decimals: 0,
            }.invoke()
        } else {
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

            TransferChecked {
                source: self.accounts.token_account,
                mint: self.accounts.mint,
                destination: self.accounts.new_token_account,
                authority: self.accounts.mint,
                amount: 1,
                decimals: 0,
            }.invoke_signed(&signers)
        }
    }
}
