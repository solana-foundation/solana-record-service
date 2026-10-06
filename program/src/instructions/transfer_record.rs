use crate::{
    state::Record,
    utils::{ByteReader, Context},
};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// TransferRecord instruction.
///
/// # Accounts
/// 1. `authority` - The account that has permission to transfer the record (must be a signer)
/// 2. `record` - The record account to be transferred
/// 3. `class` - [optional] The class of the record to be transferred
///
/// # Security
/// 1. The authority must be either:
///    a. The record owner, or
///    b. if the class is permissioned, the authority can be the permissioned authority
/// 2. The record must not be frozen
pub struct TransferRecordAccounts {
    record: AccountView,
}

impl TryFrom<&[AccountView]> for TransferRecordAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, record, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Record::check_owner_or_delegate(record, rest.first(), authority)?;

        Ok(Self { record: *record })
    }
}

const NEW_OWNER_OFFSET: usize = 0;

pub struct TransferRecord {
    accounts: TransferRecordAccounts,
    new_owner: Address,
}

impl<'info> TryFrom<Context<'info>> for TransferRecord {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = TransferRecordAccounts::try_from(ctx.accounts)?;

        let new_owner: Address = ByteReader::read_with_offset(ctx.data, NEW_OWNER_OFFSET)?;

        Ok(Self { accounts, new_owner })
    }
}

impl TransferRecord {
    pub fn process(ctx: Context<'_>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        unsafe { Record::update_owner_unchecked(&mut self.accounts.record.try_borrow_mut()?, &self.new_owner) }
    }
}
