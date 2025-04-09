// #[cfg(test)]
// mod tests {
//     use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
//     use solana_sdk::{
//         account::AccountSharedData,
//         compute_budget,
//         instruction::{AccountMeta, Instruction},
//         message::Message,
//         pubkey::Pubkey,
//         signature::{Keypair, SIGNATURE_BYTES},
//         signer::Signer,
//         transaction::Transaction,
//     };
//     use solana_winternitz::privkey::WinternitzPrivkey;

//     use crate::VaultInstructions;

//     #[test]
//     fn create_class() {
//         let winternitz_pubkey_hash = WinternitzPrivkey::generate().pubkey().merklize();

//         let program_id = Pubkey::new_from_array(crate::ID);

//         // Payer keypair
//         let keypair = Keypair::new();
//         let payer = keypair.pubkey();
//         // Vault
//         let (vault, bump) = Pubkey::find_program_address(&[&winternitz_pubkey_hash], &program_id);
//         //System Program
//         let (system_program, system_program_data) = keyed_account_for_system_program();

//         let open_vault_instruction = Instruction::new_with_bytes(
//             program_id,
//             &[
//                 &[VaultInstructions::OpenVault as u8].as_ref(),
//                 winternitz_pubkey_hash.as_ref(),
//                 &[bump].as_ref(),
//             ]
//             .concat(),
//             vec![
//                 AccountMeta::new(payer, true),
//                 AccountMeta::new(vault, false),
//                 AccountMeta::new_readonly(system_program, false),
//             ],
//         );

//         let mollusk = Mollusk::new(&program_id, "target/deploy/solana_winternitz_vault");

//         let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
//             &open_vault_instruction,
//             &vec![
//                 (
//                     payer,
//                     AccountSharedData::new(100_000_000u64, 0, &Pubkey::default()),
//                 ),
//                 (vault, AccountSharedData::new(0, 0, &Pubkey::default())),
//                 (system_program, system_program_data),
//             ],
//         );

//         assert!(!result.program_result.is_err());
//     }

//     #[test]
//     fn split_vault() {
//         let winternitz_privkey = WinternitzPrivkey::generate();

//         let program_id = Pubkey::new_from_array(crate::ID);

//         let keypair = Keypair::new();

//         let (vault, bump) =
//             Pubkey::find_program_address(&[&winternitz_privkey.pubkey().merklize()], &program_id);
//         let (split, _) = Pubkey::find_program_address(
//             &[&WinternitzPrivkey::generate().pubkey().merklize()],
//             &program_id,
//         );
//         let (refund, _) = Pubkey::find_program_address(
//             &[&WinternitzPrivkey::generate().pubkey().merklize()],
//             &program_id,
//         );

//         // Assemble our Split message
//         let mut message = [0u8; 72];
//         message[0..8].clone_from_slice(&1337u64.to_le_bytes());
//         message[8..40].clone_from_slice(&split.to_bytes());
//         message[40..].clone_from_slice(&refund.to_bytes());

//         let signature = winternitz_privkey.sign(&message.as_ref());

//         let split_vault_instruction = Instruction::new_with_bytes(
//             program_id,
//             &[
//                 &[VaultInstructions::SplitVault as u8].as_ref(),
//                 Into::<[u8; solana_winternitz::HASH_LENGTH * 32]>::into(signature).as_ref(),
//                 1337u64.to_le_bytes().as_ref(),
//                 &[bump].as_ref(),
//             ]
//             .concat(),
//             vec![
//                 AccountMeta::new(vault, false),
//                 AccountMeta::new(split, false),
//                 AccountMeta::new(refund, false),
//             ],
//         );

//         let mollusk = Mollusk::new(&program_id, "target/deploy/solana_winternitz_vault");

//         let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
//             &split_vault_instruction,
//             &vec![
//                 (
//                     vault,
//                     AccountSharedData::new(100_000_000u64, 0, &program_id),
//                 ),
//                 (split, AccountSharedData::new(0, 0, &program_id)),
//                 (refund, AccountSharedData::new(0, 0, &program_id)),
//             ],
//         );

//         assert!(!result.program_result.is_err());

//         // Ensure our transaction isn't oversized
//         let compute_unit_limit_instruction =
//             compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(900000);
//         let compute_unit_price_instruction =
//             compute_budget::ComputeBudgetInstruction::set_compute_unit_price(200000);
//         let message = Message::new(
//             &[
//                 compute_unit_limit_instruction,
//                 compute_unit_price_instruction,
//                 split_vault_instruction,
//             ],
//             Some(&keypair.pubkey()),
//         );
//         let mut tx = Transaction::new_unsigned(message);
//         tx.sign(&[keypair], [0x09u8; 32].into());
//         let len = tx.message.serialize().len() + core::mem::size_of::<u8>() + SIGNATURE_BYTES;
//         assert!(len <= 1232)
//     }

//     #[test]
//     fn close_vault() {
//         let winternitz_privkey = WinternitzPrivkey::generate();

//         let program_id = Pubkey::new_from_array(crate::ID);
//         let keypair = Keypair::new();
//         let to = keypair.pubkey();
//         let (vault, bump) =
//             Pubkey::find_program_address(&[&winternitz_privkey.pubkey().merklize()], &program_id);
//         let signature = winternitz_privkey.sign(&to.as_ref());

//         let close_vault_instruction = Instruction::new_with_bytes(
//             program_id,
//             &[
//                 &[VaultInstructions::CloseVault as u8].as_ref(),
//                 Into::<[u8; solana_winternitz::HASH_LENGTH * 32]>::into(signature).as_ref(),
//                 &[bump].as_ref(),
//             ]
//             .concat(),
//             vec![AccountMeta::new(vault, false), AccountMeta::new(to, true)],
//         );

//         let mollusk = Mollusk::new(&program_id, "target/deploy/solana_winternitz_vault");

//         let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
//             &close_vault_instruction,
//             &vec![
//                 (
//                     vault,
//                     AccountSharedData::new(100_000_000u64, 0, &program_id),
//                 ),
//                 (to, AccountSharedData::new(900_000_000u64, 0, &program_id)),
//             ],
//         );

//         assert!(!result.program_result.is_err());

//         // Ensure our transaction isn't oversized
//         let compute_unit_limit_instruction =
//             compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(900000);
//         let compute_unit_price_instruction =
//             compute_budget::ComputeBudgetInstruction::set_compute_unit_price(200000);
//         let message = Message::new(
//             &[
//                 compute_unit_limit_instruction,
//                 compute_unit_price_instruction,
//                 close_vault_instruction,
//             ],
//             Some(&keypair.pubkey()),
//         );
//         let mut tx = Transaction::new_unsigned(message);
//         tx.sign(&[keypair], [0x09u8; 32].into());
//         let len = tx.message.serialize().len() + core::mem::size_of::<u8>() + SIGNATURE_BYTES;
//         assert!(len <= 1232)
//     }
// }