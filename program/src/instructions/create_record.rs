use core::mem::size_of;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::{Allocate, Assign, CreateAccount, Transfer};

use crate::{
    state::{Class, OwnerType, Record},
    utils::{ByteReader, Context},
};

/// CreateRecord instruction.
///
/// This function:
/// 1. Calculates required account space and rent
/// 2. Derives the PDA for the record account
/// 3. Creates the new account
/// 4. Initializes the record data
///
/// # Accounts
/// 1. `owner` - The account that will own the record
/// 2. `payer` - The account that will pay for the record account
/// 3. `class` - The class account that this record belongs to
/// 4. `record` - The new record account to be created
/// 5. `authority` - [as remaining accounts] The authority account of the class
///
/// # Security
/// 1. Check if the class is permissioned, if so, the instruction must pass
///    the class authority as signer in the remaining accounts
/// 2. The class must not be frozen
pub struct CreateRecordAccounts {
    owner: AccountView,
    payer: AccountView,
    class: AccountView,
    record: AccountView,
}

impl TryFrom<&[AccountView]> for CreateRecordAccounts {
    type Error = ProgramError;

    fn try_from(accounts: &[AccountView]) -> Result<Self, Self::Error> {
        let [owner, payer, class, record, _system_program, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check class permission
        Class::check_permission(class, rest.first())?;

        Ok(Self { owner: *owner, payer: *payer, class: *class, record: *record })
    }
}

const EXPIRY_OFFSET: usize = 0;
const SEED_LEN_OFFSET: usize = EXPIRY_OFFSET + size_of::<i64>();

pub struct CreateRecord<'info> {
    accounts: CreateRecordAccounts,
    expiry: i64,
    seed: &'info [u8],
    data: &'info str,
}

impl<'info> TryFrom<Context<'info>> for CreateRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateRecordAccounts::try_from(ctx.accounts)?;

        // Deserialize `expiry`
        let expiry: i64 = ByteReader::read_with_offset(ctx.data, EXPIRY_OFFSET)?;

        // Deserialize variable length data
        let mut variable_data: ByteReader<'info> = ByteReader::new_with_offset(ctx.data, SEED_LEN_OFFSET);

        // Deserialize `seed`
        let seed: &[u8] = variable_data.read_bytes_with_length()?;

        // Deserialize `data`
        let data: &str = variable_data.read_str(variable_data.remaining_bytes())?;

        Ok(Self { accounts, expiry, seed, data })
    }
}

impl<'info> CreateRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&mut self) -> ProgramResult {
        let space = Record::MINIMUM_RECORD_SIZE + self.seed.len() + self.data.len();
        let rent = Rent::get()?.try_minimum_balance(space)?;
        let lamports = rent.saturating_sub(self.accounts.record.lamports());

        let seeds = [b"record", self.accounts.class.address().as_ref(), self.seed];

        let bump: [u8; 1] =
            [Address::try_find_program_address(&seeds, &crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"record"),
            Seed::from(self.accounts.class.address().as_ref()),
            Seed::from(self.seed),
            Seed::from(&bump),
        ];

        let signers = [Signer::from(&seeds)];

        // Create the account with our program as owner
        if self.accounts.record.lamports() > 0 {
            Allocate { account: &self.accounts.record, space: space as u64 }.invoke_signed(&signers)?;

            Assign { account: &self.accounts.record, owner: &crate::ID }.invoke_signed(&signers)?;

            if self.accounts.record.lamports() < lamports {
                Transfer {
                    from: &self.accounts.payer,
                    to: &self.accounts.record,
                    lamports: lamports - self.accounts.record.lamports(),
                }
                .invoke()?;
            }
        } else {
            CreateAccount {
                from: &self.accounts.payer,
                to: &self.accounts.record,
                lamports,
                space: space as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&signers)?;
        }

        let record = Record {
            class: *self.accounts.class.address(),
            owner_type: OwnerType::Pubkey,
            owner: *self.accounts.owner.address(),
            is_frozen: false,
            expiry: self.expiry,
            seed: self.seed,
            data: self.data,
        };

        unsafe { record.initialize_unchecked(&mut self.accounts.record) }
    }
}
