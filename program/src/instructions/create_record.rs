#[cfg(not(feature = "perf"))]
use crate::constants::MAX_NAME_LEN;
#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;

use core::mem::size_of;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::try_find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    state::{Class, Record},
    utils::{ByteReader, Context},
};

/// CreateRecord instruction.
///
/// A record represents an entity within a class (e.g., a Twitter handle, a D3 domain).
///
/// This function:
/// 1. Calculates required account space and rent
/// 2. Derives the PDA for the record account
/// 3. Creates the new account
/// 4. Initializes the record data
///
/// # Accounts
/// * `owner` - The account that will own the record (must be a signer)
/// * `class` - The class account that this record belongs to
/// * `record` - The new record account to be created
///
/// # Security
///
/// The owner account must be a signer.
pub struct CreateRecordAccounts<'info> {
    owner: &'info AccountInfo,
    class: &'info AccountInfo,
    record: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, class, record, _system_program, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check owner is a signer
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check class permission
        Class::check_permission(&class, rest.first())?;

        Ok(Self {
            owner,
            class,
            record,
        })
    }
}

const EXPIRY_OFFSET: usize = 0;
const NAME_LEN_OFFSET_WITH_EXPIRY: usize = EXPIRY_OFFSET + size_of::<u8>() +size_of::<i64>();
const NAME_LEN_OFFSET_WITHOUT_EXPIRY: usize = EXPIRY_OFFSET + size_of::<u8>();

pub struct CreateRecord<'info> {
    accounts: CreateRecordAccounts<'info>,
    expiry: i64,
    name: &'info str,
    data: &'info str,
}

/// Minimum length of instruction data required for CreateRecord
pub const CREATE_RECORD_MIN_IX_LENGTH: usize = size_of::<u8>() * 2;

impl<'info> TryFrom<Context<'info>> for CreateRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateRecordAccounts::try_from(ctx.accounts)?;

        // Check minimum instruction data length
        #[cfg(not(feature = "perf"))]
        if ctx.data.len() < CREATE_RECORD_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize `expiry`
        let expiry: Option<i64> = ByteReader::read_optional_with_offset(ctx.data, EXPIRY_OFFSET)?;

        // Deserialize variable length data
        let mut variable_data: ByteReader<'info> = ByteReader::new_with_offset(ctx.data, if expiry.is_some() { NAME_LEN_OFFSET_WITH_EXPIRY } else { NAME_LEN_OFFSET_WITHOUT_EXPIRY });
        
        // Deserialize `name`
        let name: &str = variable_data.read_str_with_length()?;

        #[cfg(not(feature = "perf"))]
        if name.len() > MAX_NAME_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        // Deserialize `data`
        let data: &str = variable_data.read_str(variable_data.remaining_bytes())?;

        Ok(Self {
            accounts,
            expiry,
            name,
            data,
        })
    }
}

impl<'info> CreateRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Create Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = Record::MINIMUM_CLASS_SIZE + self.name.len() + self.data.len();
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.record.lamports());

        let seeds = [
            b"record",
            self.accounts.class.key().as_ref(),
            self.name.as_bytes(),
        ];

        let bump: [u8; 1] = [try_find_program_address(&seeds, &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?
            .1];

        let seeds = [
            Seed::from(b"record"),
            Seed::from(self.accounts.class.key()),
            Seed::from(self.name.as_bytes()),
            Seed::from(&bump),
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.owner,
            to: self.accounts.record,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signers)?;

        let record = Record {
            class: *self.accounts.class.key(),
            owner: *self.accounts.owner.key(),
            is_frozen: false,
            has_authority_extension: false,
            expiry: self.expiry.unwrap_or(0),
            name: self.name,
            data: self.data,
        };

        record.initialize(self.accounts.record)
    }
}
