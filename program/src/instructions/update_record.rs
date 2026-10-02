use crate::{
    state::{Class, Record, CLASS_OFFSET},
    utils::{ByteReader, Context},
};
use core::mem::size_of;
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// UpdateRecord instruction.
///
/// # Accounts
/// 1. `authority` - The account that has permission to update the record (must be a signer)
/// 2. `payer` - The account that will pay for the record account
/// 3. `record` - The record account to be updated
/// 4. `class` - The class account of the record
/// 5. `system_program` - Required for account resizing operations
///
/// # Security
/// 1. The authority must be the class authority
pub struct UpdateRecordAccounts {
    payer: AccountView,
    record: AccountView,
}

impl TryFrom<&[AccountView]> for UpdateRecordAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [authority, payer, record, class, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Class::check_authority(class, authority)?;

        Record::check_program_id_and_discriminator(record)?;

        if record.try_borrow()?[CLASS_OFFSET..CLASS_OFFSET + size_of::<Address>()].ne(class.address().as_array()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self { payer: *payer, record: *record })
    }
}

pub struct UpdateRecordData<'info> {
    accounts: UpdateRecordAccounts,
    data: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateRecordData<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateRecordAccounts::try_from(ctx.accounts)?;

        let mut instruction_data = ByteReader::new(ctx.data);

        let data: &str = instruction_data.read_str(instruction_data.remaining_bytes())?;

        Ok(Self { accounts, data })
    }
}

impl<'info> UpdateRecordData<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        unsafe { Record::update_data_unchecked(&mut self.accounts.record, &mut self.accounts.payer, self.data) }
    }
}

pub struct UpdateRecordExpiry {
    accounts: UpdateRecordAccounts,
    expiry: i64,
}

impl<'info> TryFrom<Context<'info>> for UpdateRecordExpiry {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateRecordAccounts::try_from(ctx.accounts)?;

        let expiry: i64 = ByteReader::read_with_offset(ctx.data, 0)?;

        Ok(Self { accounts, expiry })
    }
}

impl UpdateRecordExpiry {
    pub fn process(ctx: Context<'_>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        unsafe { Record::update_expiry_unchecked(&mut self.accounts.record.try_borrow_mut()?, self.expiry) }
    }
}
