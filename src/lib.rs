use instructions::{CreateClass, CreateCredential, UpdateClassMetadata, UpdateClassPermission};
use pinocchio::{account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use sdk::Context;

#[cfg(test)]
pub mod tests;

pub mod instructions;
pub mod state;
pub mod sdk;
pub mod utils;

entrypoint!(process_instruction);

pub const ID: Pubkey = [1u8;32];

fn process_instruction(
    _program_id: &Pubkey,      // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // All accounts required to process the instruction
    instruction_data: &[u8],  // Serialized instruction-specific data
) -> ProgramResult {
    let (discriminator, data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match discriminator {
        0 => CreateClass::process(Context { accounts, data }),
        1 => CreateCredential::process(Context { accounts, data }),
        2 => UpdateClassMetadata::process(Context { accounts, data}),
        3 => UpdateClassPermission::process(Context { accounts, data}),
        _ => Err(ProgramError::InvalidInstructionData)
    }
}