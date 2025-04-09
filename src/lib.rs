use instructions::CreateClass;
use pinocchio::{account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use sdk::Context;

pub mod instructions;
pub mod accounts;
pub mod sdk;
#[cfg(test)]
pub mod tests;

entrypoint!(process_instruction);

pub const ID: Pubkey = [1u8;32];

fn process_instruction(
    _program_id: &Pubkey,      // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // All accounts required to process the instruction
    instruction_data: &[u8],  // Serialized instruction-specific data
) -> ProgramResult {
    let (discriminator, data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match discriminator {
        0 => CreateClass::process(Context { accounts, data }),
        _ => Err(ProgramError::InvalidInstructionData)
    }


// update_class
// Modify class metadata or settings
// Class Authority
// metadata: Option<String> is_frozen: bool
// - Authority (signer) - Class PDA
// Updates metadata or freeze status of an existing class
// create_credential
// Create authority for permissioned classes
// D3, Registry Operators
// name: String authorized_signers: Vec<Pubkey>
// - Authority (signer) - System Program - Credential PDA
// Creates credential account identifying authorities who can manage a class
// update_credential
// Modify authorized signers
// Credential Authority
// add_signers: Vec<Pubkey> remove_signers: Vec<Pubkey>
// - Authority (signer) - Credential PDA
// Adds or removes authorized signers for a credential
// create_record
// Create a new record in a class
// Users, D3, Integrators
// name: String data: String expiry: Option<i64>
// - Owner (signer) - System Program - Class PDA - Record PDA - Credential (if permissioned)
// Creates a record with data (domain, handle, etc.) within a namespace class
// update_record
// Modify a record's data content
// Record Owner, Update Authority
// data: String
// - Signer - Record PDA - Authority Ext. (optional)
// Updates the data content of an existing record
// transfer_record
// Transfer record ownership
// Record Owner, Transfer Authority
// new_owner: Pubkey
// - Owner (signer) - New Owner - Record PDA - Authority Ext. (optional)
// Transfers record ownership to another wallet address
// delete_record
// Delete a record account
// Record Owner
// None
// - Owner (signer) - Record PDA
// Marks a record as deleted or closes the account
// create_authority_extension
// Create flexible authority model
// Record Owner
// update_authority: Pubkey freeze_authority: Pubkey transfer_authority: Pubkey authority_program: Option<Pubkey>
// - Owner (signer) - Record PDA - Authority Ext. PDA - System Program
// Creates an authority extension for flexible permission management
// update_authority_extension
// Update authorities for a record
// Authority Owner
// update_authority: Pubkey freeze_authority: Pubkey transfer_authority: Pubkey
// - Owner (signer) - Authority Ext. PDA
// Updates the authorities of an existing extension
// freeze_record
// Prevent record updates/transfers
// Freeze Authority
// freeze: bool
// - Freeze Authority (signer) - Record PDA - Authority Ext. (optional)
// Freezes or unfreezes a record to prevent modifications
// freeze_class
// Prevent new record creation
// Class Authority
// freeze: bool
// - Class Authority (signer) - Class PDA
// Freezes or unfreezes a class to prevent creation of new records
// tokenize_record
// Create NFT for a record
// Record Owner
// name: String symbol: String uri: String
// - Owner (signer) - Record PDA - Mint - Metadata - Token Programs
// Creates an NFT representing ownership of the record
// renew_record
// Extend record expiration
// Record Owner, Update Authority
// new_expiry: i64
// - Signer - Record PDA - Payment accounts
// Extends the expiration period of a record
// Data Flows
}