use pinocchio::pubkey::Pubkey;

// Token2022 Constants
pub const TOKEN_2022_MINT_LEN: usize = 0x52;
pub const TOKEN_2022_MINT_BASE_LEN: usize = 0x54;
pub const TOKEN_2022_PERMANENT_DELEGATE_LEN: usize = 0x24;
pub const TOKEN_2022_CLOSE_MINT_AUTHORITY_LEN: usize = 0x24;
pub const TOKEN_2022_METADATA_POINTER_LEN: usize = 0x44;

// CloseAccount - 9
pub const TOKEN_2022_CLOSE_ACCOUNT_IX: u8 = 0x09;

// TransferChecked - 12
pub const TOKEN_2022_TRANSFER_CHECKED_IX: u8 = 0x0c;

// BurnChecked - 15
pub const TOKEN_2022_BURN_CHECKED_IX: u8 = 0x0f;

// InitializeMintCloseAuthority - 25
pub const TOKEN_2022_INITIALIZE_MINT_CLOSE_AUTHORITY_IX: u8 = 0x19;

// InitializePermanentDelegate - 35
pub const TOKEN_2022_INITIALIZE_PERMANENT_DELEGATE_IX: u8 = 0x23;

// MetadataPointerExtension - 39
pub const TOKEN_2022_METADATA_POINTER_EXTENSION_IX: u8 = 0x27;
pub const TOKEN_2022_METADATA_POINTER_INITIALIZE_IX: u8 = 0x00;
pub const TOKEN_2022_METADATA_POINTER_UPDATE_IX: u8 = 0x01;

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_2022_PROGRAM_ID: Pubkey = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];
