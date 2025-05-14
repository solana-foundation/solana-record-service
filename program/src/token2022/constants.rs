use pinocchio::pubkey::Pubkey;

// Token2022 Constants
pub const TOKEN_2022_MINT_LEN: usize = 0x52;
pub const TOKEN_2022_MINT_BASE_LEN: usize = 0x54;
pub const TOKEN_2022_PERMANENT_DELEGATE_LEN: usize = 0x24;
pub const TOKEN_2022_CLOSE_MINT_AUTHORITY_LEN: usize = 0x24;
pub const TOKEN_2022_METADATA_POINTER_LEN: usize = 0x44;
pub const TOKEN_IS_FROZEN_FLAG: u8 = 2;

// CloseAccount - 9
pub const TOKEN_2022_CLOSE_ACCOUNT_IX: u8 = 0x09;

// Close Mint Authority - xx
pub const TOKEN_2022_CLOSE_MINT_AUTHORITY_IX: u8 = 0xff;

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_2022_PROGRAM_ID: Pubkey = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];
