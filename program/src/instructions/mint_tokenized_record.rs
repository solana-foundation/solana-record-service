use core::mem::size_of;

#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio_associated_token_account::instructions::Create;

use crate::{
    constants::SRS_TICKER,
    state::{OwnerType, Record, NAME_OFFSET, OWNER_OFFSET},
    token2022::{
        constants::{
            TOKEN_2022_CLOSE_MINT_AUTHORITY_LEN, TOKEN_2022_METADATA_POINTER_LEN,
            TOKEN_2022_MINT_BASE_LEN, TOKEN_2022_MINT_LEN, TOKEN_2022_PERMANENT_DELEGATE_LEN,
            TOKEN_2022_PROGRAM_ID,
        },
        InitializeMetadata, InitializeMetadataPointer, InitializeMint2,
        InitializeMintCloseAuthority, InitializePermanentDelegate, Metadata, MintToChecked,
    },
    utils::Context,
};
use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{find_program_address, try_find_program_address, Pubkey}, sysvars::{rent::Rent, Sysvar}, ProgramResult
};
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
/// 1. `owner` - The owner of the record
/// 2. `authority` - The authority of minting this record, could be the owner or a delegate
/// 3. `record` - The record for which the token will be minted
/// 4. `mint` - The mint account of the record token
/// 5. `tokenAccount` - The token account where we mint the record token to
/// 6. `token2022` - The Token2022 program
/// 7. `system_program` - Required for initializing our accounts
/// 8. `class` - [optional] The class of the record
///
/// # Security
/// 1. The authority must be:
///    a. The record's owner, or
///    b. if the class is permissioned, the authority can be the permissioned authority
pub struct MintTokenizedRecordAccounts<'info> {
    owner: &'info AccountInfo,
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
    mint: &'info AccountInfo,
    token_account: &'info AccountInfo,
    token_2022_program: &'info AccountInfo,
    system_program: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for MintTokenizedRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner,authority, record, mint, token_account, _associated_token_program, token_2022_program, system_program, rest @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the record owner
        Record::check_owner_or_delegate(record, rest.first(), authority)?;

        let record_data = record.try_borrow_data()?;

        // Check if the owner of the record is the same as the owner of the token account
        if record_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()].ne(owner.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        let seeds = [
            owner.key(),
            TOKEN_2022_PROGRAM_ID.as_ref(),
            mint.key(),
        ];
        let (token_account_address, _) = find_program_address(&seeds, &pinocchio_associated_token_account::ID);

        if token_account_address.ne(token_account.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            owner,
            authority,
            record,
            mint,
            token_account,
            token_2022_program,
            system_program,
        })
    }
}

pub struct MintTokenizedRecord<'info> {
    accounts: MintTokenizedRecordAccounts<'info>,
}

impl<'info> TryFrom<Context<'info>> for MintTokenizedRecord<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {        
        // Deserialize our accounts array
        let accounts = MintTokenizedRecordAccounts::try_from(ctx.accounts)?;

        Ok(Self { accounts })
    }
}

impl<'info> MintTokenizedRecord<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Mint Tokenized Record");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Get Mint length
        let bump = self.derive_mint_address_bump()?;

        // Create mint account
        self.create_mint_account(&bump)?;
        // Initialize mint close authority extension
        self.initialize_mint_close_authority()?;
        // Initialize permanent delegate extension
        self.initialize_permanent_delegate()?;
        // Initialize the metadata pointer extension
        self.initialize_metadata_pointer()?;
        // Initialize mint
        self.initialize_mint()?;
        // Initialize metadata
        self.initialize_metadata(&bump)?;
        // Initialize token account for user
        self.initialize_token_account()?;
        // Mint record token
        self.mint_to_token_account(&bump)?;

        let mut record_data = self.accounts.record.try_borrow_mut_data()?;

        // 1. Update the record to be frozen since the check will be perfomed on the token account
        unsafe { Record::update_is_frozen_unchecked(&mut record_data, true)? }

        // 2. Update the record_owner to be the mint
        record_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()].clone_from_slice(self.accounts.mint.key());
        
        // 3. Update the record_type to be tokenized
        unsafe { Record::update_owner_type_unchecked(&mut record_data, OwnerType::Token) }
    }

    fn derive_mint_address_bump(&self) -> Result<[u8; 1], ProgramError> {
        let seeds = [b"mint", self.accounts.record.key().as_ref()];

        Ok([try_find_program_address(&seeds, &crate::ID)
            .ok_or(ProgramError::InvalidArgument)?
            .1])
    }

    fn create_mint_account(&self, bump: &[u8; 1]) -> Result<(), ProgramError> {
        // Space of all our static extensions
        let space = TOKEN_2022_MINT_LEN
            + TOKEN_2022_MINT_BASE_LEN
            + TOKEN_2022_PERMANENT_DELEGATE_LEN
            + TOKEN_2022_CLOSE_MINT_AUTHORITY_LEN
            + TOKEN_2022_METADATA_POINTER_LEN;

        // To avoid resizing the ming, we calculate the correct lamports for our token AOT with:
        // 1. `space` - The sum of the above static extension lengths
        // 2. `record.data_len()` - The full length of the record account
        // 3. `-NAME_OFFSET` - Remove fixed data in record account that isn't used for metadata
        let lamports = Rent::get()?.minimum_balance(
            space + self.accounts.record.data_len() + Metadata::FIXED_HEADER_LEN + NAME_OFFSET, // Deduct static data at start of record account that isn't used in metadata
        );

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
            owner: &TOKEN_2022_PROGRAM_ID,
        }
        .invoke_signed(&signers)
    }

    fn initialize_permanent_delegate(&self) -> Result<(), ProgramError> {
        InitializePermanentDelegate {
            mint: self.accounts.mint,
            delegate: self.accounts.mint.key(),
        }
        .invoke()
    }

    fn initialize_mint_close_authority(&self) -> Result<(), ProgramError> {
        InitializeMintCloseAuthority {
            mint: self.accounts.mint,
            close_authority: self.accounts.mint.key(),
        }
        .invoke()
    }

    fn initialize_metadata_pointer(&self) -> Result<(), ProgramError> {
        InitializeMetadataPointer {
            mint: self.accounts.mint,
            authority: self.accounts.mint.key(),
            metadata_address: self.accounts.mint.key(),
        }
        .invoke()
    }

    fn initialize_mint(&self) -> Result<(), ProgramError> {
        InitializeMint2 {
            mint: self.accounts.mint,
            decimals: 0,
            mint_authority: self.accounts.mint.key(),
            freeze_authority: None,
        }
        .invoke()
    }

    fn initialize_metadata(&self, bump: &[u8; 1]) -> Result<(), ProgramError> {
        let data = self.accounts.record.try_borrow_data()?;

        let (name, uri) = unsafe { Record::get_name_and_data_unchecked(&data)? };

        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        InitializeMetadata {
            metadata: self.accounts.mint,
            update_authority: self.accounts.mint,
            mint: self.accounts.mint,
            mint_authority: self.accounts.mint,
            name,
            symbol: SRS_TICKER,
            uri,
        }
        .invoke_signed(&signers)
    }

    fn initialize_token_account(&self) -> Result<(), ProgramError> {
        Create {
            funding_account: self.accounts.authority,
            account: self.accounts.token_account,
            wallet: self.accounts.owner,
            mint: self.accounts.mint,
            system_program: self.accounts.system_program,
            token_program: self.accounts.token_2022_program,
        }
        .invoke()
    }

    fn mint_to_token_account(&self, bump: &[u8; 1]) -> Result<(), ProgramError> {
        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        MintToChecked {
            mint: self.accounts.mint,
            account: self.accounts.token_account,
            mint_authority: self.accounts.mint,
            amount: 1,
            decimals: 0,
        }
        .invoke_signed(&signers)
    }
}