use core::mem::MaybeUninit;

#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;

use crate::{
    constants::{TOKEN_2022_METADATA_POINTER_EXTENSION_INITIALIZE_IX, TOKEN_2022_METADATA_POINTER_EXTENSION_IX, TOKEN_2022_METADATA_POINTER_LEN, TOKEN_2022_PERMANENT_DELEGATE_LEN, TOKEN_2022_PROGRAM_ID}, state::Record, utils::Context
};
use pinocchio::{account_info::AccountInfo, cpi::invoke_signed, instruction::{AccountMeta, Instruction, Seed, Signer}, program_error::ProgramError, pubkey::{try_find_program_address, Pubkey}, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

/// MintRecordToken instruction.
///
/// This instruction:
/// 1. Validates the authority and record
/// 2. Updates the record's status to Tokenized
/// 3. Creates a Token2022 token mint
/// 4. Creates a Token2022 token account
/// 5. Mints a token to the token account
///
/// # Accounts
/// 1. `authority` - The owner of the record (must be a signer)
/// 2. `record` - The record for which the token will be minted
/// 3. `mint` - The mint account of the record token
/// 4. `tokenAccount` - The token account where we mint the record token to
/// 4. `token2022` - The Token2022 program
/// 3. `system_program` - Required for initializing our accounts
/// # Security
/// 1. The authority must be:
///    a. The record's owner, or
///    b. An authorized delegate with update permissions
pub struct MintRecordTokenAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
    mint: &'info AccountInfo,
    token_account: &'info AccountInfo,
    associated_token_program: &'info AccountInfo,
    token_2022_program: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for MintRecordTokenAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, record, mint, token_account, associated_token_program, token_2022_program, _system_program, rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check if authority is the record owner
        Record::check_authority(record, authority.key())?;

        Ok(Self { authority, record, mint, token_account, associated_token_program, token_2022_program })
    }
}

pub struct MintRecordToken<'info> {
    accounts: MintRecordTokenAccounts<'info>
}

impl<'info> TryFrom<Context<'info>> for MintRecordToken<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        // Deserialize our accounts array
        let accounts = MintRecordTokenAccounts::try_from(ctx.accounts)?;

        Ok(Self { accounts })
    }
}

impl<'info> MintRecordToken<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Get Mint length
        let bump = self.get_bump()?;
        // Create the mint account in System Program
        self.create_mint_account(&bump)?;
        // Initialize the permanent delegate extension
        self.initialize_permanent_delegate(&bump)?;
        // Initialize the metadata pointer
        self.initialize_metadata_pointer(&bump)?;
        // Initialize metadata
        self.initialize_metadata(&bump)?;
        // Initialize metadata
        self.initialize_mint2(&bump)?;
        // Initialize token account for user
        self.initialize_token_account_idempotent()?;
        // Step 1) Create our permanent delegate extension account
        self.mint_to_token_account()?;
        
        // createInitializePermanentDelegateInstruction(
        //     mint,
        //     permanentDelegate.publicKey,
        //     TOKEN_2022_PROGRAM_ID,
        // )
        // Update the record data [this is safe, check safety docs]
        unsafe {
            Record::update_is_frozen_unchecked(self.accounts.record, true)
        }
    }

    fn get_bump(&self) -> Result<[u8;1], ProgramError> {
        let seeds = [
            b"mint",
            self.accounts.record.key().as_ref()
        ];

        Ok([try_find_program_address(&seeds, &crate::ID)
                    .ok_or(ProgramError::InvalidArgument)?
                    .1])
    }

    fn create_mint_account(&self, bump: &[u8;1]) -> Result<(), ProgramError> {
        let space = TOKEN_2022_PERMANENT_DELEGATE_LEN + 
        TOKEN_2022_METADATA_POINTER_LEN +
        unsafe { Record::metadata_length_unchecked(self.accounts.record)? };

        let lamports = Rent::get()?.minimum_balance(space);

        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        // Create the account with our program as owner
        CreateAccount {
            from: self.accounts.authority,
            to: self.accounts.mint,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signers)
    }

    fn initialize_permanent_delegate(&self, bump: &[u8;1]) -> Result<(), ProgramError> {
        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        let data = unsafe {
            let mut data: MaybeUninit<[u8; 33]> = core::mem::MaybeUninit::uninit();
            // Get data pointer
            let data_ptr = data.as_mut_ptr();
            // Set first byte to 35
            *(data_ptr as *mut u8) = 35u8;
            // Copy remaining 32 bytes from Mint key
            core::ptr::copy_nonoverlapping(
                self.accounts.mint.key().as_ref().as_ptr() as *const Pubkey,
                data_ptr.add(size_of::<u8>()) as *mut Pubkey,
                size_of::<Pubkey>(),
            );
            data.assume_init()
        };

        invoke_signed(
            &Instruction {
                program_id: &TOKEN_2022_PROGRAM_ID,
                data: data.as_ref(),
                accounts: &[
                    AccountMeta::new(self.accounts.mint.key(), true, true)
                ]
            },
            &[
                self.accounts.mint
            ],
            &signers
        )
    }

    fn initialize_metadata_pointer(&self, bump: &[u8;1]) -> Result<(), ProgramError> {
        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        let data = unsafe {
            let mut data: MaybeUninit<[u8; 65]> = core::mem::MaybeUninit::uninit();
            // Get data pointer
            let data_ptr = data.as_mut_ptr();
            // Set first byte to 35
            *(data_ptr as *mut u8) = TOKEN_2022_METADATA_POINTER_EXTENSION_IX;
            *(data_ptr.add(1) as *mut u8) = TOKEN_2022_METADATA_POINTER_EXTENSION_INITIALIZE_IX;
            // Copy remaining 32 bytes from Mint key
            core::ptr::copy_nonoverlapping(
                self.accounts.mint.key().as_ref().as_ptr() as *const Pubkey,
                data_ptr.add(size_of::<u8>()) as *mut Pubkey,
                size_of::<Pubkey>(),
            );

            core::ptr::copy_nonoverlapping(
                self.accounts.mint.key().as_ref().as_ptr() as *const Pubkey,
                data_ptr.add(size_of::<u8>() + size_of::<Pubkey>()) as *mut Pubkey,
                size_of::<Pubkey>(),
            );
            data.assume_init()
        };

        invoke_signed(
            &Instruction {
                program_id: &self.accounts.token_2022_program.key(),
                data: data.as_ref(),
                accounts: &[
                    AccountMeta::new(self.accounts.mint.key(), true, true),
                    AccountMeta::new(self.accounts.mint.key(), true, true)
                ]
            },
            &[
                self.accounts.mint
            ],
            &signers
        )
    }
}
