use core::mem::size_of;
use crate::{
    state::{Class, Record, CLASS_OFFSET, IS_PERMISSIONED_OFFSET},
    utils::{ByteReader, Context},
};
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

/// UpdateRecord instruction.
///
/// This instruction:
/// 1. Validates the authority and record
/// 2. Updates the record's data content
/// 3. Resizes the account if needed
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
pub struct UpdateRecordAccounts<'info> {
    payer: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, payer, record, class, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check class + record are valid
        Class::check_program_id(class)?;
        Record::check_program_id_and_discriminator(record)?;

        // Check if the class is the correct class
        if class
            .key()
            .ne(&record.try_borrow_data()?[CLASS_OFFSET..CLASS_OFFSET + size_of::<Pubkey>()])
        {
            return Err(ProgramError::InvalidAccountData);
        }

        // Permissioned classes: only class authority can update
        let class_data = class.try_borrow_data()?;
        unsafe { Class::check_discriminator_unchecked(&class_data)? };

        if class_data[IS_PERMISSIONED_OFFSET].eq(&1u8) {
            unsafe { Class::check_authority_unchecked(&class_data, authority)? };
        } else {
            // Permissionless: allow class authority or record owner
            if unsafe { Class::check_authority_unchecked(&class_data, authority) }.is_err() {
                Record::check_owner_or_delegate(record, Some(class), authority)?;
            }
        }

        Ok(Self { payer, record })
    }
}

pub struct UpdateRecordData<'info> {
    accounts: UpdateRecordAccounts<'info>,
    data: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateRecordData<'info> {
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

impl<'info> UpdateRecordData<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Record Data");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Update the record data [this is safe, check safety docs]
        unsafe {
            Record::update_data_unchecked(self.accounts.record, self.accounts.payer, self.data)
        }
    }
}

pub struct UpdateRecordExpiry<'info> {
    accounts: UpdateRecordAccounts<'info>,
    expiry: i64,
}

impl<'info> TryFrom<Context<'info>> for UpdateRecordExpiry<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = UpdateRecordAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < size_of::<i64>() {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize `data`
        let expiry = i64::from_le_bytes( ctx.data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { accounts, expiry })
    }
}

impl<'info> UpdateRecordExpiry<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Record Expiry");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Update the record data [this is safe, check safety docs]
        unsafe {
            Record::update_expiry_unchecked(&mut self.accounts.record.try_borrow_mut_data()?, self.expiry)
        }
    }
}
