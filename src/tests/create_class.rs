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
    fn create_class() {
        let program_id = Pubkey::new_from_array(crate::ID);

        let name = "test";
        let name_hash = solana_nostd_sha256::hash(name.as_bytes());

        // Accounts
        let authority = Pubkey::new_from_array([2; 32]);
        let (class, _) = Pubkey::find_program_address(&[b"class", &authority.to_bytes(), &name_hash], &program_id);
        let (system_program, system_program_data) = keyed_account_for_system_program();

        let create_class_instruction = Instruction::new_with_bytes(
            program_id,
            &[
                &[0u8], // discriminator
                &[0u8], // is_permissioned
                name.as_ref(),
            ]
            .concat(),
            vec![
                AccountMeta::new(authority, true),
                AccountMeta::new(class, false),
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
    fn create_class_with_metadata() {
        let program_id = Pubkey::new_from_array(crate::ID);

        let name = "test";
        let name_hash = solana_nostd_sha256::hash(name.as_bytes());
        let metadata = "metadata";
        // Accounts
        let authority = Pubkey::new_from_array([2; 32]);
        let (class, _) = Pubkey::find_program_address(&[b"class", &authority.to_bytes(), &name_hash], &program_id);
        let (system_program, system_program_data) = keyed_account_for_system_program();

        let create_class_instruction = Instruction::new_with_bytes(
            program_id,
            &[
                &[0u8], // discriminator
                &[0u8], // is_permissioned
                name.as_ref(),
                metadata.as_ref(),
            ]
            .concat(),
            vec![
                AccountMeta::new(authority, true),
                AccountMeta::new(class, false),
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
    fn create_permissioned_class() {
        let program_id = Pubkey::new_from_array(crate::ID);

        let credential_name = "credential_test";
        let credential_name_hash = solana_nostd_sha256::hash(credential_name.as_bytes());
        
        let class_name = "class_test";
        let class_name_hash = solana_nostd_sha256::hash(class_name.as_bytes());

        // Accounts
        let authority = Pubkey::new_from_array([2; 32]);
        let (credential, _) = Pubkey::find_program_address(&[b"credential", &authority.to_bytes(), &credential_name_hash], &program_id);
        let (class, _) = Pubkey::find_program_address(&[b"class", &authority.to_bytes(), &class_name_hash], &program_id);
        let (system_program, system_program_data) = keyed_account_for_system_program();

        let credential_account = create_credential_account(&mut mollusk, authority, credential_name, "metadata");

        let create_class_instruction = Instruction::new_with_bytes(
            program_id,
            &[
                &[0u8], // discriminator
                &[1u8], // is_permissioned
                class_name.as_ref(),
            ]
            .concat(),
            vec![
                AccountMeta::new(authority, true),
                AccountMeta::new(class, false),
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
                (credential, credential_account),
                (system_program, system_program_data),
            ],
        );

        assert!(!result.program_result.is_err());
    }
}

fn create_credential_account(mollusk: &mut Mollusk, authority: Pubkey, name: &str, metadata: &str) -> AccountSharedData {
    let data = [
        &0u64.to_le_bytes()[..],
        &authority.to_bytes()[..],
        &name.as_bytes()[..],
        &metadata.as_bytes()[..],
    ].concat();

    let mut credential_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance( data.len() as usize),
        data.len() as usize,
        &crate::ID
    );
    credential_account.set_data_from_slice(&data);

    credential_account
}