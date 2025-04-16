#[cfg(test)]
mod tests {
    use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
    use solana_sdk::{
        account::AccountSharedData,
        compute_budget,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, SIGNATURE_BYTES},
        signer::Signer,
        transaction::Transaction,
    };

    use crate::instructions::create_class::CreateClass;

    #[test]
    fn create_credential() {
        let program_id = Pubkey::new_from_array(crate::ID);

        let credential_name = "credential_test";
        let credential_name_hash = solana_nostd_sha256::hash(credential_name.as_bytes());

        // Accounts
        let authority = Pubkey::new_from_array([2; 32]);
        let (credential, _) = Pubkey::find_program_address(&[b"credential", &authority.to_bytes(), &credential_name_hash], &program_id);
        let (system_program, system_program_data) = keyed_account_for_system_program();

        let create_credential_instruction = Instruction::new_with_bytes(
            program_id,
            &[
                &[1u8], // discriminator
                credential_name.as_ref(),
            ]
            .concat(),
            vec![
                AccountMeta::new(authority, true),
                AccountMeta::new(credential, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        );

        let mollusk = Mollusk::new(&program_id, "target/deploy/solana_winternitz_vault");

        let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
            &create_class_instruction,
            &vec![
                (
                    authority,
                    AccountSharedData::new(100_000_000u64, 0, &Pubkey::default()),
                ),
                (class, AccountSharedData::new(0, 0, &Pubkey::default())),
                (system_program, system_program_data),
            ],
        );

        assert!(!result.program_result.is_err());
    }

    #[test]
    fn create_credential_with_authorized_signers() {
        let program_id = Pubkey::new_from_array(crate::ID);

        let credential_name = "credential_test";
        let credential_name_hash = solana_nostd_sha256::hash(credential_name.as_bytes());
        let authorized_signers = vec![Pubkey::new_from_array([3; 32])];
        // Accounts
        let authority = Pubkey::new_from_array([2; 32]);
        let (credential, _) = Pubkey::find_program_address(&[b"credential", &authority.to_bytes(), &credential_name_hash], &program_id);
        let (system_program, system_program_data) = keyed_account_for_system_program();

        let create_class_instruction = Instruction::new_with_bytes(
            program_id,
            &[
                &[0u8], // discriminator
                &[0u8], // is_permissioned
                credential_name.as_ref(),
                authorized_signers.as_ref(),
            ]
            .concat(),
            vec![
                AccountMeta::new(authority, true),
                AccountMeta::new(credential, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        );

        let mollusk = Mollusk::new(&program_id, "target/deploy/solana_winternitz_vault");

        let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
            &create_class_instruction,
            &vec![
                (
                    authority,
                    AccountSharedData::new(100_000_000u64, 0, &Pubkey::default()),
                ),
                (class, AccountSharedData::new(0, 0, &Pubkey::default())),
                (system_program, system_program_data),
            ],
        );

        assert!(!result.program_result.is_err());
    }
}