use construct_vault_sol::processor;
mod tests {
    use super::*;
    use borsh::BorshDeserialize;
    use solana_program_test::*;
    use solana_sdk::{
        hash::Hash, instruction::{AccountMeta, Instruction}, msg, pubkey::Pubkey, signature::{Keypair, Signer}, system_program, sysvar, transaction::Transaction
    };
    use spl_token::state::Mint;
    use spl_associated_token_account::get_associated_token_address;
    use construct_vault_sol::state::construct_vault::Vault;
    use solana_program::program_pack::Pack;

    async fn create_mint(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
    ) -> Keypair {
        let mint_keypair = Keypair::new();
        let mint_rent = banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Mint::LEN);

        msg!("Creating mint account");

        let create_mint_tx = Transaction::new_signed_with_payer(
            &[solana_sdk::system_instruction::create_account(
                &payer.pubkey(),
                &mint_keypair.pubkey(),
                mint_rent,
                Mint::LEN.try_into().unwrap(),
                &spl_token::id(),
            )],
            Some(&payer.pubkey()),
            &[payer, &mint_keypair],
            recent_blockhash,
        );

        banks_client.process_transaction(create_mint_tx).await.unwrap();

        msg!("Initializing mint account");
        // Initialize the mint
        let init_mint_tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_keypair.pubkey(),
            &payer.pubkey(), // mint authority
            None,            // freeze authority
            0,              // decimals
        ).unwrap()],
        Some(&payer.pubkey()),
        &[payer],
            recent_blockhash,
        );

        
        banks_client.process_transaction(init_mint_tx).await.unwrap();
        mint_keypair
    }

    #[tokio::test]
    async fn test_initialize_vault() {
        // Create program test environment
        let program_id = Pubkey::new_unique();

        let program_test = ProgramTest::new(
            "construct_vault_sol",
            program_id,
            processor!(processor::process_instruction),
        );

        // Start the test environment
        let (mut banks_client, 
            payer, 
            recent_blockhash) = program_test.start().await;

        let mint_keypair = create_mint(
            &mut banks_client,
            &payer,
            recent_blockhash
        ).await;

        msg!("Mint created");
        // Derive vault PDA
        let (vault_pda, _bump) = Pubkey::find_program_address(
            &[
                Vault::SEED_PREFIX.as_bytes(),
                payer.pubkey().as_ref(),
            ],
            &program_id,
        );

        // Get vault's associated token account
        let vault_ata = get_associated_token_address(
            &vault_pda,
            &mint_keypair.pubkey(),
        );

        // Create initialization instruction
        let init_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new(vault_ata, false),
                AccountMeta::new_readonly(mint_keypair.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
            ],
            data: vec![0], // Assuming 0 is the instruction enum variant for Initialize
        };

        // Create and send transaction
        let transaction = Transaction::new_signed_with_payer(
            &[init_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        // Process transaction
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify vault account was created
        let vault_account = match banks_client.get_account(vault_pda).await {
            Ok(Some(account)) => {
                msg!("Found vault account. Data length: {}", account.data.len());
                msg!("Account owner: {}", account.owner);
                account
            },
            Ok(None) => panic!("Vault account not found"),
            Err(e) => panic!("Failed to get vault account: {}", e),
        };
        
        // Add more detailed debugging before deserialization attempt
        msg!("Attempting to deserialize vault data...");
        msg!("Account data length: {}", vault_account.data.len());
        msg!("First few bytes: {:?}", &vault_account.data[..std::cmp::min(8, vault_account.data.len())]);
        
        let vault_data = match Vault::try_from_slice(&vault_account.data) {
            Ok(data) => {
                msg!("Successfully deserialized vault data");
                data
            },
            Err(e) => {
                msg!("Failed to deserialize vault data. Data length: {}", vault_account.data.len());
                msg!("Raw data (hex): {:02x?}", vault_account.data);
                msg!("Raw data (utf8): {:?}", String::from_utf8_lossy(&vault_account.data));
                panic!("Deserialization error: {}", e);
            }
        };

        msg!("Vault Data: {:?}", vault_data);
        assert_eq!(vault_data.owner, payer.pubkey());
        assert_eq!(vault_data.amount_locked, 0);

        // Verify vault ATA was created
        let vault_ata_account = match banks_client.get_account(vault_ata).await {
            Ok(Some(account)) => account,
            Ok(None) => panic!("Vault ATA account not found"),
            Err(e) => panic!("Failed to get vault ATA account: {}", e),
        };
        assert_eq!(vault_ata_account.owner, spl_token::id());
    }
}