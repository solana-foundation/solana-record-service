use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

use crate::{state::Credential, sdk::Context, utils::resize_account};

/// # UpdateCredential
/// 
/// Adds or removes authorized signers for a credential
/// 
/// Callers: Credential Authority
/// 
/// Accounts:
/// 1. authority            [signer, mut]
/// 2. credential           [mut]
/// 3. system_program       [executable]
/// 
/// Parameters:
/// 1. authorized_signers   [Vec<Pubkey>]
pub struct UpdateCredentialAccounts<'info> {
    authority: &'info AccountInfo,
    credential: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateCredentialAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, credential, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if unsafe { credential.owner().ne(&crate::ID) } {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if unsafe { credential.borrow_data_unchecked() }[0].ne(&Credential::DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            authority,
            credential
        })
    }
}

pub struct UpdateCredentialSigners<'info> {
    accounts: UpdateCredentialAccounts<'info>,
    signers: &'info [Pubkey],
}

pub const UPDATE_CREDENTIAL_MIN_LENGTH: usize = size_of::<u8>();

impl<'info> TryFrom<Context<'info>> for UpdateCredentialSigners<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateCredentialAccounts::try_from(ctx.accounts)?;

        if ctx.data.len() < UPDATE_CREDENTIAL_MIN_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        let num_signers = ctx.data[0] as usize;

        // Create a slice of Pubkeys from the instruction data
        let signers = unsafe {
            std::slice::from_raw_parts(
                ctx.data[UPDATE_CREDENTIAL_MIN_LENGTH..].as_ptr() as *const Pubkey,
                num_signers * size_of::<Pubkey>()
            )
        };

        Ok(Self { accounts, signers })
    }
}

impl<'info> UpdateCredentialSigners<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        // First, get the current credential data
        let credential_data = self.accounts.credential.try_borrow_data()?;
        let mut credential = Credential::from_bytes(&credential_data)?;

        // Verify authority
        if credential.authority.ne(self.accounts.authority.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Save the current number of signers
        let initial_signers_len = credential.authorized_signers.len();

        // Add if the signer is not already in the list, remove if it is
        for signer in self.signers {
            credential.modify_signer(*signer)?;
        }

        // Get the new number of signers after all modifications
        let new_signers_len = credential.authorized_signers.len();

        // Calculate the actual size difference based on the real changes
        let size_diff = new_signers_len.saturating_sub(initial_signers_len) * size_of::<Pubkey>();
        let new_account_size = credential_data.len().saturating_add(size_diff);

        // Resize the account if needed, based on the actual changes
        resize_account(
            self.accounts.credential,
            self.accounts.authority,
            new_account_size,
            new_signers_len < initial_signers_len,
        )?;

        Ok(())
    }
}