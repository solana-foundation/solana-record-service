#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio_associated_token_account::instructions::Create;

use crate::{
    token2022::constants::{
        TOKEN_2022_CLOSE_MINT_AUTHORITY_LEN, TOKEN_2022_METADATA_POINTER_LEN,
        TOKEN_2022_MINT_BASE_LEN, TOKEN_2022_MINT_LEN, TOKEN_2022_PERMANENT_DELEGATE_LEN,
        TOKEN_2022_PROGRAM_ID,
    },
    constants::SRS_TICKER,
    state::{Record, NAME_OFFSET, OWNER_OFFSET},
    token2022::{
        InitializeMetadata, InitializeMetadataPointer, InitializeMint2,
        InitializeMintCloseAuthority, InitializePermanentDelegate, Metadata, MintToChecked, Token,
    },
    utils::Context,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{try_find_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
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
///    b. An authorized delegate with update permissions
pub struct MintRecordTokenAccounts<'info> {
    authority: &'info AccountInfo,
    record: &'info AccountInfo,
    mint: &'info AccountInfo,
    token_account: &'info AccountInfo,
    token_2022_program: &'info AccountInfo,
    system_program: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for MintRecordTokenAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, authority, record, mint, token_account, _associated_token_program, token_2022_program, system_program, rest @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the record owner
        Record::check_owner_or_delegate(
            record,
            rest.first(),
            authority,
        )?;

        // Check if the owner of the record is the same as the owner of the token account
        if owner.key().ne(unsafe { &Token::get_owner_unchecked(&token_account.try_borrow_data()?)? }) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if the owner is the same as the owner of the Record
        let token_data = token_account.try_borrow_data()?;

        if owner.key().ne(&token_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self {
            authority,
            record,
            mint,
            token_account,
            token_2022_program,
            system_program,
        })
    }
}

pub struct MintRecordToken<'info> {
    accounts: MintRecordTokenAccounts<'info>,
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
        sol_log("Mint Record Token");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // Get Mint length
        let bump = self.derive_mint_address_bump()?;
        // Create mint account
        self.create_mint_account(&bump)?;
        // Initialize permanent delegate extension
        self.initialize_permanent_delegate()?;
        // Initialize mint close authority extension
        self.initialize_mint_close_authority()?;
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

        // Update the record to be frozen, the check will be perfomed on the token account
        unsafe { Record::update_is_frozen_unchecked(self.accounts.record.try_borrow_mut_data()?, true) }
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
            wallet: self.accounts.authority,
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

// {
//     "extensionLength": 177,
//     "updateAuthority": "PEPEYvBXiPzhAM66sTe9PjdBBeibLmngPkkovcHteHn",
//     "mint": "PFireKhT5WG7axMSLBmMRpvYH7cgHx9CRWHU8F8HNbr",
//     "name": "Pepe On Fire",
//     "symbol": "PFIRE",
//     "uri": "https://gateway.pinata.cloud/ipfs/QmeUqonNx6KD5bGb1J9TuhDrgQtbFPQm7PaTSLJ5bhbMVw",
//     "additionalMetadata": {},
//     "enumType": "tokenMetadata"
// }

// 64 + 4 + 12 + 4 + 5 + 4 + 80 = 173
// 16 left over

// 00000000
// 05b1e90f3a17ad8be78b5824e6e1dfb98cdd2148410d69f5ff54e0c82f423d19 // PEPEYvBXiPzhAM66sTe9PjdBBeibLmngPkkovcHteHn
// 963bcdfef667935a
// 06
// 01
// 00
// 0000000000000000000000000000000000000000000000000000000000000000

// 0000000000000000000000000000000000000000000000000000000000000000
// 0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000101006c 00 0000000000000000000000000000000000000000000000000000000000000000 05b1e90f3a17ad8be78b5824e6e1dfb98cdd2148410d69f5ff54e0c82f423d19 0bedc968c4f10000 5b02000000000000 0000e8890423c78a f401 5b020000000000000000e8890423c78af40112004000 05b1e90f3a17ad8be78b5824e6e1dfb98cdd2148410d69f5ff54e0c82f423d19 05b36c72ccbfad097cda694fd19e374eb4e0b2d7306dc5eafe9358b46c73c9c9 1300b100 05b1e90f3a17ad8be78b5824e6e1dfb98cdd2148410d69f5ff54e0c82f423d19 05b36c72ccbfad097cda694fd19e374eb4e0b2d7306dc5eafe9358b46c73c9c9
// 0c000000 // name length
// 50657065204f6e2046697265 // name
// 05000000 //
// 5046495245
// 50000000
// 68747470733a2f2f676174657761792e70696e6174612e636c6f75642f697066732f516d6555716f6e4e78364b4435624762314a395475684472675174624650516d37506154534c4a356268624d5677 00000000
