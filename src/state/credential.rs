use core::str;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
pub struct Credential<'info> {
    pub authority: Pubkey,                    // The authority that controls this credential
    pub name: &'info str,                     // Human-readable name for the credential
    pub authorized_signers: &'info [Pubkey],  // Slice of authorized signers
}

impl<'info> Credential<'info> {
    pub const DISCRIMINATOR: u8 = 0;
    pub const MINIMUM_CLASS_SIZE: usize = 1  // discriminator
        + 32                                 // authority
        + 1;                                 // signers_len
}