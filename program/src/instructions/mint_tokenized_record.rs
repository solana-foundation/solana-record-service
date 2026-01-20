use core::mem::size_of;

#[cfg(not(feature = "perf"))]
use pinocchio::log::sol_log;
use pinocchio_associated_token_account::instructions::Create;

use crate::{
    state::{OwnerType, Record, CLASS_OFFSET, IS_FROZEN_OFFSET, OWNER_OFFSET},
    token2022::{
        constants::{
            TOKEN_2022_CLOSE_MINT_AUTHORITY_LEN, TOKEN_2022_GROUP_LEN, TOKEN_2022_GROUP_POINTER_LEN, TOKEN_2022_MEMBER_LEN, TOKEN_2022_MEMBER_POINTER_LEN, TOKEN_2022_METADATA_LEN, TOKEN_2022_METADATA_POINTER_LEN, TOKEN_2022_MINT_BASE_LEN, TOKEN_2022_MINT_LEN, TOKEN_2022_PERMANENT_DELEGATE_LEN, TOKEN_2022_PROGRAM_ID
        }, FreezeAccount, InitializeGroup, InitializeGroupMemberPointer, InitializeGroupPointer, InitializeMember, InitializeMetadata, InitializeMetadataPointer, InitializeMint2, InitializeMintCloseAuthority, InitializePermanentDelegate, Mint, MintToChecked, Token, UpdateMetadata
    },
    utils::Context, ID,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::{Allocate, Assign, CreateAccount, Transfer};

/// MintTokenizedRecord instruction.
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
/// 2. `payer` - The account that will pay for the mint account
/// 3. `authority` - The authority of minting this record, could be the owner or a delegate
/// 4. `record` - The record for which the token will be minted
/// 5. `mint` - The mint account of the record token
/// 6. `class` - The class of the record
/// 7. `group` - The group of the record
/// 8. `token_account` - The associated token account where we mint the record token to
/// 9. `token_2022_program` - The Token2022 program
/// 10. `system_program` - Required for initializing our accounts
///
/// # Security
/// 1. The authority must be:
///    a. The record's owner, or
///    b. if the class is permissioned, the authority can be the permissioned authority
pub struct MintTokenizedRecordAccounts<'info> {
    owner: &'info AccountInfo,
    payer: &'info AccountInfo,
    record: &'info AccountInfo,
    mint: &'info AccountInfo,
    class: &'info AccountInfo,
    group: &'info AccountInfo,
    token_account: &'info AccountInfo,
    token_2022_program: &'info AccountInfo,
    system_program: &'info AccountInfo,
    mint_bump: u8,
    group_bump: u8,
}

impl<'info> TryFrom<&'info [AccountInfo]> for MintTokenizedRecordAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, payer, authority, record, mint, class, group, token_account, _associated_token_program, token_2022_program, system_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if authority is the record owner
        Record::check_owner_or_delegate(record, Some(class), authority)?;

        let record_data = record.try_borrow_data()?;

        // Check if the owner of the record is the same as the owner of the token account
        if record_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()].ne(owner.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check that the class of the record is the same as the class passed in 
        if record_data[CLASS_OFFSET..CLASS_OFFSET + size_of::<Pubkey>()].ne(class.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        let seeds = [owner.key(), TOKEN_2022_PROGRAM_ID.as_ref(), mint.key()];
        let (token_account_address, _) =
            find_program_address(&seeds, &pinocchio_associated_token_account::ID);

        if token_account_address.ne(token_account.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        let (group_key, group_bump) = find_program_address(&[b"group", class.key()], &ID);
        if group_key.ne(group.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        let (mint_key, mint_bump) = find_program_address(&[b"mint", record.key()], &ID);
        if mint_key.ne(mint.key()) {
            return Err(ProgramError::InvalidSeeds);
        }

        Ok(Self {
            owner,
            payer,
            record,
            mint,
            class,
            group,
            token_account,
            token_2022_program,
            system_program,
            mint_bump,
            group_bump,
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
        let mint_bump = [self.accounts.mint_bump];
        let group_bump = [self.accounts.group_bump];

        // Check if the group already exists
        if !Mint::check_discriminator(self.accounts.group)? {
            // Create the group mint account if needed
            self.create_group_mint_account(&group_bump)?;
            // Initialize the group pointer extension
            self.initialize_group_pointer()?;
            // Initialize the group mint account
            self.initialize_group_mint_account()?;
            // Initialize the group
            self.initialize_group(&group_bump)?;
        }

        // Create mint account
        self.create_mint_account(&mint_bump)?;
        // Initialize mint close authority extension
        self.initialize_mint_close_authority()?;
        // Initialize permanent delegate extension
        self.initialize_permanent_delegate()?;
        // Initialize the metadata pointer extension
        self.initialize_metadata_pointer()?;
        // Initialize the group member pointer extension
        self.initialize_group_member_pointer()?;
        // Initialize mint
        self.initialize_mint()?;
        // Initialize metadata
        self.initialize_metadata(&mint_bump)?;
        // Initialize the group member
        self.initialize_group_member(&group_bump, &mint_bump)?;
        // Initialize token account for user
        self.initialize_token_account()?;
        // Mint record token
        self.mint_to_token_account(&mint_bump)?;

        let mut record_data = self.accounts.record.try_borrow_mut_data()?;

        // 1. Check if the current record is frozen, if it is, we need to freeze the token as well
        if record_data[IS_FROZEN_OFFSET] == 1 {
            let seeds = [
                Seed::from(b"mint"),
                Seed::from(self.accounts.record.key()),
                Seed::from(&mint_bump),
            ];

            FreezeAccount {
                account: self.accounts.token_account,
                mint: self.accounts.mint,
                freeze_authority: self.accounts.mint,
            }
            .invoke_signed(&[Signer::from(&seeds)])?;
        }

        // 2. Update the record_owner to be the mint
        record_data[OWNER_OFFSET..OWNER_OFFSET + size_of::<Pubkey>()]
            .clone_from_slice(self.accounts.mint.key());

        // 3. Update the record_type to be tokenized
        unsafe { Record::update_owner_type_unchecked(&mut record_data, OwnerType::Token) }
    }

    fn create_group_mint_account(&self, bump: &[u8; 1]) -> Result<(), ProgramError> {
        // Space of all our static extensions
        let space = TOKEN_2022_MINT_LEN + TOKEN_2022_MINT_BASE_LEN + TOKEN_2022_GROUP_POINTER_LEN;

        let rent = Rent::get()?.minimum_balance(space + TOKEN_2022_GROUP_LEN);

        let seeds = [
            Seed::from(b"group"),
            Seed::from(self.accounts.class.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        let current_lamports = self.accounts.group.lamports();
        if current_lamports > 0 {
            Allocate {
                account: self.accounts.group,
                space: space as u64,
            }
            .invoke_signed(&signers)?;

            Assign {
                account: self.accounts.group,
                owner: &TOKEN_2022_PROGRAM_ID,
            }
            .invoke_signed(&signers)?;

            // Transfer only the delta needed to reach rent-exemption
            if current_lamports < rent {
                Transfer {
                    from: self.accounts.payer,
                    to: self.accounts.group,
                    lamports: rent - current_lamports,
                }
                .invoke()?;
            }
        } else {
            CreateAccount {
                from: self.accounts.payer,
                to: self.accounts.group,
                lamports: rent,
                space: space as u64,
                owner: &TOKEN_2022_PROGRAM_ID,
            }
            .invoke_signed(&signers)?;
        }

        Ok(())
    }

    fn initialize_group_pointer(&self) -> Result<(), ProgramError> {
        InitializeGroupPointer {
            mint: self.accounts.group,
            authority: self.accounts.group.key(),
            group_address: self.accounts.group.key(),
        }
        .invoke()
    }

    fn initialize_group(&self, bump: &[u8; 1]) -> Result<(), ProgramError> {
        let seeds = [
            Seed::from(b"group"),
            Seed::from(self.accounts.class.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        InitializeGroup {
            group: self.accounts.group,
            mint: self.accounts.group,
            mint_authority: self.accounts.group,
            update_authority: self.accounts.group.key(),
            max_size: u64::MAX,
        }
        .invoke_signed(&signers)
    }

    fn initialize_group_mint_account(&self) -> Result<(), ProgramError> {
        InitializeMint2 {
            mint: self.accounts.group,
            decimals: 0,
            mint_authority: self.accounts.group.key(),
            freeze_authority: Some(self.accounts.group.key()),
        }
        .invoke()
    }

    fn create_mint_account(&self, bump: &[u8; 1]) -> Result<(), ProgramError> {
        // Space of all our static extensions
        let space = TOKEN_2022_MINT_LEN
            + TOKEN_2022_MINT_BASE_LEN
            + TOKEN_2022_PERMANENT_DELEGATE_LEN
            + TOKEN_2022_CLOSE_MINT_AUTHORITY_LEN
            + TOKEN_2022_METADATA_POINTER_LEN
            + TOKEN_2022_MEMBER_POINTER_LEN;


        // To avoid resizing the mint, we calculate the correct lamports for our token AOT with:
        // 1. `space` - The sum of the above static extension lengths
        // 2. `metadata_data.len()` - The full length of the metadata data
        let rent = Rent::get()?.minimum_balance(
            space
                + unsafe {
                    Record::get_metadata_len_unchecked(&self.accounts.record.try_borrow_data()?)?
                }
                + TOKEN_2022_MEMBER_LEN
                + TOKEN_2022_METADATA_LEN
        );

        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        let current_lamports = self.accounts.mint.lamports();
        if current_lamports > 0 {
            Allocate {
                account: self.accounts.mint,
                space: space as u64,
            }
            .invoke_signed(&signers)?;

            Assign {
                account: self.accounts.mint,
                owner: &TOKEN_2022_PROGRAM_ID,
            }
            .invoke_signed(&signers)?;

            // Transfer only the delta needed to reach rent-exemption
            if current_lamports < rent {
                Transfer {
                    from: self.accounts.payer,
                    to: self.accounts.mint,
                    lamports: rent - current_lamports,
                }
                .invoke()?;
            }
        } else {
            CreateAccount {
                from: self.accounts.payer,
                to: self.accounts.mint,
                lamports: rent,
                space: space as u64,
                owner: &TOKEN_2022_PROGRAM_ID,
            }
            .invoke_signed(&signers)?;
        }

        Ok(())
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
            freeze_authority: Some(self.accounts.mint.key()),
        }
        .invoke()
    }

    fn initialize_group_member_pointer(&self) -> Result<(), ProgramError> {
        InitializeGroupMemberPointer {
            mint: self.accounts.mint,
            authority: self.accounts.group.key(),
            member_address: self.accounts.mint.key(),
        }
        .invoke()
    }

    fn initialize_metadata(&self, bump: &[u8; 1]) -> Result<(), ProgramError> {
        let record_data = self.accounts.record.try_borrow_data()?;
        let (metadata_data, additional_metadata_data) =
            unsafe { Record::get_metadata_data_unchecked(&record_data)? };

        let seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(bump),
        ];

        let signers = [Signer::from(&seeds)];

        InitializeMetadata {
            metadata: self.accounts.mint,
            mint: self.accounts.mint,
            update_authority: self.accounts.mint,
            mint_authority: self.accounts.mint,
            metadata_data,
        }
        .invoke_signed(&signers)?;

        if let Some(additional_metadata_data) = additional_metadata_data {
            let additional_metadata_num = u32::from_le_bytes(
                additional_metadata_data[0..size_of::<u32>()]
                    .try_into()
                    .unwrap(),
            );

            let mut offset = size_of::<u32>();

            // Process each additional metadata entry
            for _ in 0..additional_metadata_num {
                let starting_value_offset = offset;

                let field_len = u32::from_le_bytes(
                    additional_metadata_data[offset..offset + size_of::<u32>()]
                        .try_into()
                        .unwrap(),
                ) as usize;
                offset += size_of::<u32>() + field_len;
                let value_len = u32::from_le_bytes(
                    additional_metadata_data[offset..offset + size_of::<u32>()]
                        .try_into()
                        .unwrap(),
                ) as usize;
                offset += size_of::<u32>() + value_len;

                let entry_data = additional_metadata_data[starting_value_offset..offset].to_vec();

                // Call UpdateMetadata for this entry
                UpdateMetadata {
                    metadata: self.accounts.mint,
                    update_authority: self.accounts.mint,
                    additional_metadata: &entry_data,
                }
                .invoke_signed(&signers)?;
            }
        }

        Ok(())
    }

    fn initialize_group_member(
        &self,
        group_bump: &[u8; 1],
        mint_bump: &[u8; 1],
    ) -> Result<(), ProgramError> {
        let group_seeds = [
            Seed::from(b"group"),
            Seed::from(self.accounts.class.key()),
            Seed::from(group_bump),
        ];

        let mint_seeds = [
            Seed::from(b"mint"),
            Seed::from(self.accounts.record.key()),
            Seed::from(mint_bump),
        ];

        let signers = [Signer::from(&mint_seeds), Signer::from(&group_seeds)];

        InitializeMember {
            mint: self.accounts.mint,
            member: self.accounts.mint,
            mint_authority: self.accounts.mint,
            group: self.accounts.group,
            group_update_authority: self.accounts.group,
        }
        .invoke_signed(&signers)
    }

    fn initialize_token_account(&self) -> Result<(), ProgramError> {
        // Check if the token account already exists
        if self
            .accounts
            .token_account
            .is_owned_by(&TOKEN_2022_PROGRAM_ID)
        {
            // Check Discriminator
            let data = self.accounts.token_account.try_borrow_data()?;
            unsafe { Token::check_discriminator_unchecked(&data)? };

            // Check Ownership
            if unsafe { Token::get_owner_unchecked(&data)? }.ne(self.accounts.owner.key()) {
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }

        Create {
            funding_account: self.accounts.payer,
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
