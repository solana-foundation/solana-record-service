use core::mem::size_of;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

const MINT_OFFSET: usize = 0;
const OWNER_OFFSET: usize = MINT_OFFSET + size_of::<Pubkey>();