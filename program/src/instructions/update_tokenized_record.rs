use crate::{
    state::Record,
    token2022::UpdateMetadata,
    utils::{ByteReader, Context},
};
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::try_find_program_address,
    ProgramResult,
};

/// UpdateRecord instruction.
///
/// This instruction:
/// 1. Validates the authority and record
/// 2. Updates the record's data content
/// 3. Resizes the account if needed
///
/// # Accounts
/// 1. `authority` - The account that has permission to update the record (must be a signer)
/// 2. `mint` - The mint account that that is linked to the record
/// 3. `token_account` - The token account that is linked to the mint
/// 4. `metadata` - The metadata account that is linked to the mint
/// 5. `record` - The record account to be updated
/// 6. `system_program` - Required for account resizing operations
/// 7. `class` - [optional] The class of the token account
///
/// # Security
/// 1. The authority must be:
///    a. The mint's owner, or
///    b. if the class is permissioned, the authority must be the permissioned authority
pub struct UpdateTokenizedRecordAccounts<'info> {
    authority: &'info AccountInfo,
    mint: &'info AccountInfo,
    metadata: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateTokenizedRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, mint, token_account, metadata, record, _system_program, rest @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if authority is the record owner or has a delegate
        Record::check_owner_or_delegate_tokenized(
            record,
            rest.first(),
            authority,
            mint,
            token_account,
        )?;

        Ok(Self {
            authority,
            mint,
            metadata,
            record,
        })
    }
}

pub struct UpdateTokenizedRecord<'info> {
    accounts: UpdateTokenizedRecordAccounts<'info>,
    new_data: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateTokenizedRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = UpdateTokenizedRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length and create a byte reader
        let mut instruction_data = ByteReader::new(ctx.data);

        // Deserialize `data`
        let new_data: &str = instruction_data.read_str(instruction_data.remaining_bytes())?;

        Ok(Self { accounts, new_data })
    }
}

impl<'info> UpdateTokenizedRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Update the record data [this is safe, check safety docs]
        unsafe {
            Record::update_data_unchecked(
                self.accounts.record,
                self.accounts.authority,
                self.new_data,
            )?
        }

        let record_data = self.accounts.record.try_borrow_data()?;
        let (_, data) = unsafe { Record::get_name_and_data_unchecked(&record_data)? };

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

        UpdateMetadata {
            metadata: self.accounts.metadata,
            update_authority: self.accounts.mint,
            new_uri: data,
        }
        .invoke_signed(&signers)
    }
}
