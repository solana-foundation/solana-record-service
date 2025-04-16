use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::{sdk::Context, state::{Record, RecordAuthorityExtension}};

/// # CreateRecordDelegate
/// 
/// Creates a new record delegate.
/// 
/// Accounts:
/// 1. owner            [signer, mut]
/// 2. record           [mut]
/// 3. delegate         [mut]
/// 4. system_program   [executable]
/// 
/// Parameters:
/// 1. update_authority: Pubkey
/// 2. freeze_authority: Pubkey
/// 3. transfer_authority: Pubkey
/// 4. authority_program: Option<Pubkey>
pub struct CreateRecordDelegateAccounts<'info> {
    owner: &'info AccountInfo,
    record: &'info AccountInfo,
    delegate: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for CreateRecordDelegateAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, record, delegate, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if Record::from_bytes(&record.try_borrow_data()?)?.owner != *owner.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            owner,
            record,
            delegate
        })
    }
}
pub struct CreateRecordDelegate<'info> {
    accounts: CreateRecordDelegateAccounts<'info>,
    update_authority: Pubkey,
    freeze_authority: Pubkey,
    transfer_authority: Pubkey,
    authority_program: Option<Pubkey>,
}

pub const CREATE_RECORD_DELEGATE_MIN_IX_LENGTH: usize = size_of::<Pubkey>() * 3 + size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for CreateRecordDelegate<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = CreateRecordDelegateAccounts::try_from(ctx.accounts)?;

        // Check ix data has minimum length of at least 5 for boolean and length bytes
        if ctx.data.len() < CREATE_RECORD_DELEGATE_MIN_IX_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let update_authority: Pubkey = ctx.data[0..32].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let freeze_authority: Pubkey = ctx.data[32..64].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let transfer_authority: Pubkey = ctx.data[64..96].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let authority_program = if ctx.data[96] != 0 {
            Some(ctx.data[97..129].try_into().map_err(|_| ProgramError::InvalidInstructionData)?)
        } else {
            None
        };
        
        Ok(Self {
            accounts,
            update_authority,
            freeze_authority,
            transfer_authority,
            authority_program
        })
    }
}

impl <'info> CreateRecordDelegate<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        let space = RecordAuthorityExtension::MINIMUM_RECORD_SIZE;
        let rent = Rent::get()?.minimum_balance(space);
        let lamports = rent.saturating_sub(self.accounts.record.lamports());

        let seeds = [
            b"authority",
            self.accounts.record.key().as_ref(),
        ];
            
        let bump: [u8; 1] = [try_find_program_address(&seeds,&crate::ID).ok_or(ProgramError::InvalidArgument)?.1];

        let seeds = [
            Seed::from(b"authority"),
            Seed::from(self.accounts.record.key()),
            Seed::from(&bump)
        ];

        let signers = [Signer::from(&seeds)];

        CreateAccount {
            from: self.accounts.owner,
            to: self.accounts.record,
            lamports,
            space: space as u64,
            owner: &crate::ID
        }.invoke_signed(
            &signers
        )?;

        let record = RecordAuthorityExtension {
            record: *self.accounts.record.key(),
            update_authority: self.update_authority,
            burn_authority: self.update_authority,
            freeze_authority: self.freeze_authority,
            transfer_authority: self.transfer_authority,
            authority_program: self.authority_program,
        };

        record.initialize(self.accounts.record)
    }
}