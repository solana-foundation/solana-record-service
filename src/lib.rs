use instructions::{CreateClass, CreateCredential, UpdateClassMetadata, UpdateClassPermission, UpdateCredential, CreateRecord};
use pinocchio::{account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use sdk::Context;

pub mod instructions;
pub mod state;
pub mod sdk;
pub mod utils;

entrypoint!(process_instruction);

// 22222222222222222222222222222222222222222222
pub const ID: Pubkey = [
    0x0f, 0x1e, 0x6b, 0x14, 0x21, 0xc0, 0x4a, 0x07, 
    0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee, 
    0x19, 0x92, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07, 
    0x8e, 0xf8, 0xaf, 0x70, 0x47, 0xdc, 0x11, 0xf7, 
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
        2 => UpdateClassPermission::process(Context { accounts, data }),
        3 => CreateCredential::process(Context { accounts, data }),
        4 => UpdateCredential::process(Context { accounts, data }),
        5 => CreateRecord::process(Context { accounts, data }),
        _ => Err(ProgramError::InvalidInstructionData)
    }
}