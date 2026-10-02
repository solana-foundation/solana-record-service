use instructions::*;
use pinocchio::{address::declare_id, entrypoint, error::ProgramError, AccountView, Address, ProgramResult};
use utils::Context;

pub mod constants;
pub mod instructions;
pub mod state;
pub mod token2022;
pub mod utils;

entrypoint!(process_instruction);

declare_id!("srsWjm76StJucL7atFyPSdXFaVLNPFqEt1uFEDPrZsn");

fn process_instruction(_program_id: &Address, accounts: &mut [AccountView], instruction_data: &[u8]) -> ProgramResult {
    let (discriminator, data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    let accounts = &*accounts;

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
