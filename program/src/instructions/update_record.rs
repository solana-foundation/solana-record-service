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
/// 1. `authority` - The account that has permission to update the record (must be a signer)
/// 2. `record` - The record account to be updated
/// 3. `system_program` - Required for account resizing operations
/// 4. `record_delegate` - [remaining accounts] Required if the authority is not the record owner
/// # Security
/// 1. The authority must be:
///     a. The record's owner, or
///     b. An authorized delegate with update permissions
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
        Record::check_authority_or_delegate(record, authority, rest.first(), Record::UPDATE_AUTHORITY_DELEGATION_TYPE)?;

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
        // Update the record data [this is safe, check safety docs]
        unsafe { Record::update_data_unchecked(self.accounts.record, self.accounts.authority, self.data) }
    }
}
