use crate::utils::{write_bytes, UNINIT_BYTE};
use core::slice::from_raw_parts;
use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};
use pinocchio_token_2022::ID as TOKEN_2022_PROGRAM_ID;

const MAX_INSTRUCTION_DATA_LEN: usize = 2_000;

/// Initializes the Token-2022 TokenMetadata extension on a mint.
///
/// `metadata_data` is the Borsh-encoded `name`, `symbol` and `uri`, passed through as stored on the record.
pub struct InitializeMetadata<'a> {
    pub metadata: &'a AccountView,
    pub update_authority: &'a AccountView,
    pub mint: &'a AccountView,
    pub mint_authority: &'a AccountView,
    pub metadata_data: &'a [u8],
}

impl InitializeMetadata<'_> {
    const DISCRIMINATOR: [u8; 8] = [0xd2, 0xe1, 0x1e, 0xa2, 0x58, 0xb8, 0x4d, 0x8d];

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let data_len = Self::DISCRIMINATOR.len() + self.metadata_data.len();
        if data_len > MAX_INSTRUCTION_DATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut data = [UNINIT_BYTE; MAX_INSTRUCTION_DATA_LEN];
        write_bytes(&mut data, &Self::DISCRIMINATOR);
        write_bytes(&mut data[Self::DISCRIMINATOR.len()..], self.metadata_data);

        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly(self.update_authority.address()),
                    InstructionAccount::readonly(self.mint.address()),
                    InstructionAccount::readonly_signer(self.mint_authority.address()),
                ],
                // SAFETY: the first `data_len` bytes were written above.
                data: unsafe { from_raw_parts(data.as_ptr() as *const u8, data_len) },
            },
            &[self.metadata, self.update_authority, self.mint, self.mint_authority],
            signers,
        )
    }
}

/// Sets a custom `Field::Key` entry in the TokenMetadata extension.
///
/// `additional_metadata` is one Borsh-encoded key/value string pair.
pub struct UpdateMetadata<'a> {
    pub metadata: &'a AccountView,
    pub update_authority: &'a AccountView,
    pub additional_metadata: &'a [u8],
}

impl UpdateMetadata<'_> {
    const DISCRIMINATOR: [u8; 8] = [0xdd, 0xe9, 0x31, 0x2d, 0xb5, 0xca, 0xdc, 0xc8];
    const FIELD_KEY_VARIANT: u8 = 3;

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let data_len = Self::DISCRIMINATOR.len() + 1 + self.additional_metadata.len();
        if data_len > MAX_INSTRUCTION_DATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut data = [UNINIT_BYTE; MAX_INSTRUCTION_DATA_LEN];
        write_bytes(&mut data, &Self::DISCRIMINATOR);
        write_bytes(&mut data[Self::DISCRIMINATOR.len()..], &[Self::FIELD_KEY_VARIANT]);
        write_bytes(&mut data[Self::DISCRIMINATOR.len() + 1..], self.additional_metadata);

        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly_signer(self.update_authority.address()),
                ],
                // SAFETY: the first `data_len` bytes were written above.
                data: unsafe { from_raw_parts(data.as_ptr() as *const u8, data_len) },
            },
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}

/// Initializes the Token-2022 TokenGroup extension on a mint.
pub struct InitializeGroup<'a> {
    pub group: &'a AccountView,
    pub mint: &'a AccountView,
    pub mint_authority: &'a AccountView,
    pub update_authority: &'a Address,
    pub max_size: u64,
}

impl InitializeGroup<'_> {
    const DISCRIMINATOR: [u8; 8] = [0x79, 0x71, 0x6c, 0x27, 0x36, 0x33, 0x00, 0x04];

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let mut data = [0u8; 48];
        data[..8].copy_from_slice(&Self::DISCRIMINATOR);
        data[8..40].copy_from_slice(self.update_authority.as_ref());
        data[40..].copy_from_slice(&self.max_size.to_le_bytes());

        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.group.address()),
                    InstructionAccount::readonly(self.mint.address()),
                    InstructionAccount::readonly_signer(self.mint_authority.address()),
                ],
                data: &data,
            },
            &[self.group, self.mint, self.mint_authority],
            signers,
        )
    }
}

/// Initializes the Token-2022 TokenGroupMember extension on a mint and adds it to `group`.
pub struct InitializeMember<'a> {
    pub mint: &'a AccountView,
    pub member: &'a AccountView,
    pub mint_authority: &'a AccountView,
    pub group: &'a AccountView,
    pub group_update_authority: &'a AccountView,
}

impl InitializeMember<'_> {
    const DISCRIMINATOR: [u8; 8] = [0x98, 0x20, 0xde, 0xb0, 0xdf, 0xed, 0x74, 0x86];

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        invoke_signed(
            &InstructionView {
                program_id: &TOKEN_2022_PROGRAM_ID,
                accounts: &[
                    InstructionAccount::writable(self.mint.address()),
                    InstructionAccount::readonly(self.member.address()),
                    InstructionAccount::readonly_signer(self.mint_authority.address()),
                    InstructionAccount::writable(self.group.address()),
                    InstructionAccount::readonly_signer(self.group_update_authority.address()),
                ],
                data: &Self::DISCRIMINATOR,
            },
            &[self.mint, self.member, self.mint_authority, self.group, self.group_update_authority],
            signers,
        )
    }
}
