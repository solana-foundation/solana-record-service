use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
pub struct RecordAuthorityExtension {
    pub record: Pubkey,                   // The record this extension belongs to
    pub update_authority: Pubkey,
    pub freeze_authority: Pubkey,
    pub transfer_authority: Pubkey,
    pub burn_authority: Pubkey,
    pub authority_program: Option<Pubkey>,
}

impl RecordAuthorityExtension {
    pub const DISCRIMINATOR: u8 = 3;
    pub const MINIMUM_CLASS_SIZE: usize = 1 // discriminator
        + 32                                // record
        + 32                                // update_authority
        + 32                                // freeze_authority
        + 32                                // transfer_authority
        + 32                                // burn_authority
        + 33;                               // authority_program (option)

    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::MINIMUM_CLASS_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let record: Pubkey = data[..32].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let update_authority: Pubkey = data[32..64].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let freeze_authority: Pubkey = data[64..96].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let transfer_authority: Pubkey = data[96..128].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let burn_authority: Pubkey = data[128..160].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

        let authority_program: Option<Pubkey> = if data[160] == 0 {
            None
        } else {
            Some(data[161..193].try_into().map_err(|_| ProgramError::InvalidAccountData)?)
        };

        Ok(Self {
            record,
            update_authority,
            freeze_authority,
            transfer_authority,
            burn_authority,
            authority_program,
        })
    }

    pub fn initialize(&self, account_info: &AccountInfo) -> Result<(), ProgramError> {
        // Borrow our account data
        let mut data = account_info.try_borrow_mut_data()?;

        // Write our discriminator
        data[0] = Self::DISCRIMINATOR;
        
        // Write our authority
        data[1..33].clone_from_slice(&self.record);

        // Write our update authority
        data[33..65].clone_from_slice(&self.update_authority);

        // Write our freeze authority
        data[65..97].clone_from_slice(&self.freeze_authority);

        // Write our transfer authority
        data[97..129].clone_from_slice(&self.transfer_authority);

        // Write our burn authority
        data[129..161].clone_from_slice(&self.burn_authority);

        // Write our authority program
        if let Some(authority_program) = self.authority_program {
            data[161..193].clone_from_slice(&authority_program);
        } else {
            data[161..193].clone_from_slice(&[0u8; 32]);
        }

        Ok(())
    }
}