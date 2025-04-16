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
    fn update_class_metadata() {
        let program_id = Pubkey::new_from_array(crate::ID);

        let class_name = "class_test";
        let class_name_hash = solana_nostd_sha256::hash(class_name.as_bytes());
        
        let new_class_metadata = "new_metadata";

        // Accounts
        let authority = Pubkey::new_from_array([2; 32]);
        let (class, _) = Pubkey::find_program_address(&[b"class", &authority.to_bytes(), &class_name_hash], &program_id);
        let (system_program, system_program_data) = keyed_account_for_system_program();

        let class_account = create_class_account(mollusk, authority, false, None, class_name, class_metadata);

        let update_class_metadata_instruction = Instruction::new_with_bytes(
            program_id,
            &[
                &[3u8], // discriminator
                new_class_metadata.as_ref(),
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
                (class, class_account),
                (system_program, system_program_data),
            ],
        );

        assert!(!result.program_result.is_err());
    }

    #[test]
    fn update_class_permission() {
        let program_id = Pubkey::new_from_array(crate::ID);

        let class_name = "class_test";
        let class_name_hash = solana_nostd_sha256::hash(class_name.as_bytes());
        let class_metadata = "metadata";

        // Accounts
        let authority = Pubkey::new_from_array([2; 32]);
        let (class, _) = Pubkey::find_program_address(&[b"class", &authority.to_bytes(), &class_name_hash], &program_id);
        let (system_program, system_program_data) = keyed_account_for_system_program();

        let class_account = create_class_account(mollusk, authority, false, None, class_name, class_metadata);
        
        let update_class_permission_instruction = Instruction::new_with_bytes(
            program_id,
            &[
                &[3u8], // discriminator
                &[1u8], // is_frozen
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
            &update_class_permission_instruction,
            &vec![
                (
                    authority,
                    AccountSharedData::new(100_000_000u64, 0, &Pubkey::default()),
                ),
                (class, class_account),
                (system_program, system_program_data),
            ],
        );

        assert!(!result.program_result.is_err());
    }
}

fn create_class_account(mollusk: &mut Mollusk, authority: Pubkey, is_frozen: bool, credential_account: Option<Pubkey>, name: &str, metadata: &str) -> AccountSharedData {
    let data = [
        &1u64.to_le_bytes()[..],
        &authority.to_bytes()[..],
        &is_frozen.to_le_bytes()[..],
        &[credential_account.is_some() as u8][..],
        &credential_account.map(|c| c.to_bytes()).unwrap_or_default()[..],
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