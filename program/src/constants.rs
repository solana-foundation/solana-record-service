use codama::CodamaAccount;

/// Variable data length constraints
pub const MAX_METADATA_LEN: usize = 0xff;

/// Token-2022 mint of a tokenized record. Declared for PDA derivation in the IDL only.
#[derive(CodamaAccount)]
#[codama(seed(type = string(utf8), value = "mint"))]
#[codama(seed(name = "record", type = public_key))]
pub struct Mint;

/// Token-2022 group mint shared by the tokenized records of a class. Declared for PDA derivation in the IDL only.
#[derive(CodamaAccount)]
#[codama(seed(type = string(utf8), value = "group"))]
#[codama(seed(name = "class", type = public_key))]
pub struct Group;
