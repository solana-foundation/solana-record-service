extern crate alloc;

use alloc::{string::String, vec::Vec};
use codama::CodamaType;

/// Token-2022 metadata extension compatible metadata, stored as the data of a tokenizable record.
/// Declared for the IDL only; the program reads these bytes in place when minting.
#[derive(CodamaType)]
pub struct Metadata {
    pub name: String,
    #[codama(default_value = "SRS")]
    pub symbol: String,
    pub uri: String,
    pub additional_metadata: Vec<AdditionalMetadata>,
}

/// One key/value entry of the Token-2022 metadata extension.
#[derive(CodamaType)]
pub struct AdditionalMetadata {
    pub label: String,
    pub value: String,
}
