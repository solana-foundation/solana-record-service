use pinocchio::pubkey::Pubkey;

/// Variable data length constraints
pub const MAX_NAME_LEN: usize = 0x20;
pub const MAX_METADATA_LEN: usize = 0xff;
pub const SRS_TICKER: &str = "SRS";
pub const CLOSE_ACCOUNT_DISCRIMINATOR: u8 = 0xff;

// Token2022 Constants
pub const TOKEN_2022_PERMANENT_DELEGATE_LEN: usize = 0x20;
pub const TOKEN_2022_METADATA_POINTER_LEN: usize = 0x20;

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
pub const TOKEN_2022_PROGRAM_ID: Pubkey = [0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda, 0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc];

