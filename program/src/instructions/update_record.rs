use crate::{
    state::Record,
    utils::{ByteReader, Context},
};
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// UpdateRecord instruction.
///
/// This instruction:
/// 1. Validates the authority and record
/// 2. Updates the record's data content
/// 3. Resizes the account if needed
///
/// # Accounts
/// * `authority` - The account that has permission to update the record (must be a signer)
/// * `record` - The record account to be updated
/// * `system_program` - Required for account resizing operations
/// * `record_delegate` - Optional account that has been delegated update authority
///
/// # Security
/// The authority must be:
/// 1. The record's owner, or
/// 2. An authorized delegate with update permissions
pub struct UpdateRecordAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, _system_program, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if authority is the record owner or has a delegate
        Record::check_authority_or_delegate(record, authority.key(), rest.first())?;

        Ok(Self { authority, record })
    }
}

pub struct UpdateRecord<'info> {
    accounts: UpdateRecordAccounts<'info>,
    data: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = UpdateRecordAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length and create a byte reader
        let mut instruction_data = ByteReader::new(ctx.data);

        // Deserialize `data`
        let data: &str = instruction_data.read_str(instruction_data.remaining_bytes())?;

        Ok(Self { accounts, data })
    }
}

impl<'info> UpdateRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        Record::update_data(self.accounts.record, self.accounts.authority, self.data)
    }
}
