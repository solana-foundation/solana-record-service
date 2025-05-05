use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// UpdateClass instruction.
///
/// This function:
/// 1. Loads the current class state
/// 2. Updates the metadata
/// 3. Saves the updated state
///
/// # Accounts
/// * `authority` - The account that has permission to update the class (must be a signer)
/// * `class` - The class account to be updated
/// * `system_program` - Required for account resizing operations
///
/// # Security
/// The authority must be:
/// 1. The class owner
/// 2. A signer
pub struct UpdateClassAccounts<'info> {
    authority: &'info AccountInfo,
    class: &'info AccountInfo,
    // system_program: &'info AccountInfo
}

impl<'info> TryFrom<&'info [AccountInfo]> for UpdateClassAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, class, ..] = &accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account Checks
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Self { authority, class })
    }
}

// UpdateClassMetadata
pub struct UpdateClassMetadata<'info> {
    accounts: UpdateClassAccounts<'info>,
    metadata: &'info str,
}

impl<'info> TryFrom<Context<'info>> for UpdateClassMetadata<'info> {
    type Error = ProgramError;

    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = UpdateClassAccounts::try_from(ctx.accounts)?;

        // Create a byte reader
        let mut data = ByteReader::new(ctx.data);

        // Deserialize metadata
        let metadata = data.read_str(data.remaining_bytes())?;

        // Validate metadata length
        if metadata.len() > MAX_METADATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(UpdateClassMetadata { accounts, metadata })
    }
}

impl<'info> UpdateClassMetadata<'info> {
    pub fn process(ctx: Context<'info>) -> ProgramResult {
        #[cfg(not(feature = "perf"))]
        sol_log("Update Class Metadata");
        Self::try_from(ctx)?.execute()
    }

    pub fn execute(&self) -> ProgramResult {
        Class::update_metadata(self.accounts.class, self.accounts.authority, self.metadata)
    }
}
