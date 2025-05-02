// #![cfg_attr(not(test), no_std)]
use instructions::{CreateClass, CreateRecord, DeleteRecord, FreezeClass, FreezeRecord, TransferRecord, UpdateClassMetadata, UpdateClassFrozen, UpdateRecord};
use pinocchio::{account_info::AccountInfo, default_allocator, nostd_panic_handler, program_entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use utils::Context;

pub mod instructions;
pub mod state;
pub mod utils;
pub mod constants;
#[cfg(test)]
pub mod tests;

// entrypoint!(process_instruction);

program_entrypoint!(process_instruction);
default_allocator!();
// #[cfg(not(test))]
// nostd_panic_handler!();

// srsUi2TVUUCyGcZdopxJauk8ZBzgAaHHZCVUhm5ifPa
pub const ID: Pubkey = [
    0x0d, 0x07, 0x6d, 0xd2, 0x25, 0x68, 0x1a, 0x37, 
    0x2b, 0x70, 0x18, 0x49, 0xae, 0xc6, 0x09, 0x13, 
    0x88, 0xf0, 0x8d, 0x04, 0x7c, 0x42, 0x8c, 0xcd, 
    0x0d, 0xda, 0x8a, 0x49, 0x4a, 0xcb, 0x24, 0x1d, 
];

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match discriminator {
        0 => CreateClass::process(Context { accounts, data }),
        1 => UpdateClassMetadata::process(Context { accounts, data }),
        2 => UpdateClassFrozen::process(Context { accounts, data }),
        3 => FreezeClass::process(Context { accounts, data }),
        4 => CreateRecord::process(Context { accounts, data }),
        5 => UpdateRecord::process(Context { accounts, data }),
        6 => TransferRecord::process(Context { accounts, data }),
        7 => DeleteRecord::process(Context { accounts, data }),
        8 => FreezeRecord::process(Context { accounts, data }),
        _ => Err(ProgramError::InvalidInstructionData)
    }
}