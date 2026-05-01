#![cfg_attr(not(test), no_std)]
use instructions::*;
use pinocchio::{
    account_info::AccountInfo, default_allocator, program_entrypoint, program_error::ProgramError,
    pubkey::Pubkey, ProgramResult,
};
use utils::Context;

pub mod constants;
pub mod instructions;
pub mod state;
#[cfg(test)]
pub mod tests;
pub mod token2022;
pub mod utils;

program_entrypoint!(process_instruction);
default_allocator!();

#[cfg(not(test))]
nostd_panic_handler!();

// srsWjm76StJucL7atFyPSdXFaVLNPFqEt1uFEDPrZsn
pub const ID: Pubkey = [
    0x0d, 0x07, 0x6d, 0xfe, 0xdc, 0x66, 0x80, 0x9f,
    0xb0, 0x4b, 0x17, 0xdd, 0x2c, 0xef, 0xe2, 0xe6,
    0xf2, 0x65, 0x86, 0x5d, 0xbd, 0x35, 0x41, 0x2d,
    0xb9, 0x05, 0xe9, 0xcb, 0x33, 0x00, 0x60, 0xc9,
];

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => CreateClass::process(Context { accounts, data }),
        1 => UpdateClassMetadata::process(Context { accounts, data }),
        2 => UpdateClassAuthority::process(Context { accounts, data }),
        3 => FreezeClass::process(Context { accounts, data }),
        4 => CreateRecord::process(Context { accounts, data }),
        5 => UpdateRecordData::process(Context { accounts, data }),
        6 => UpdateRecordExpiry::process(Context { accounts, data }),
        7 => TransferRecord::process(Context { accounts, data }),
        8 => DeleteRecord::process(Context { accounts, data }),
        9 => FreezeRecord::process(Context { accounts, data }),
        10 => MintTokenizedRecord::process(Context { accounts, data }),
        11 => FreezeTokenizedRecord::process(Context { accounts, data }),
        12 => TransferTokenizedRecord::process(Context { accounts, data }),
        13 => BurnTokenizedRecord::process(Context { accounts, data }),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
