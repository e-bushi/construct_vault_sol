use {
    crate::state::Vault, 
    borsh::BorshSerialize, 
    solana_program::{
        account_info::{next_account_info, AccountInfo}, 
        entrypoint::ProgramResult, 
        msg,
        clock::Clock, 
        program::{invoke, invoke_signed}, 
        program_error::ProgramError, 
        pubkey::Pubkey, 
        system_instruction, 
        sysvar::{rent::Rent, Sysvar},
        system_program,
    }, 
    spl_associated_token_account::instruction::create_associated_token_account, 
    spl_token::instruction as token_instruction
};

const MAINNET_MINT: Pubkey = Pubkey::from_str_const("3PKZCeF6RVw6sAGqCV5BGCATE1gu3bPceWXhfasapXVS");
const DEVNET_MINT: Pubkey = Pubkey::from_str_const("AQYzQ3ZS9tXjhYMuVQ8tGoZMVV5DSuucaJB16mzXic9d");
const MAINNET_FEE_RECEIVER: Pubkey = Pubkey::from_str_const("5zaUUZWoXaWt2Ht5NNQZuQXyfaQKDLyQoESn6BXvVzBd");
const DEVNET_FEE_RECEIVER: Pubkey = Pubkey::from_str_const("8jHMkdtKK4CCn4ep6Hponmk1ik7ofUNS9bX9qSuiRcN5");

fn get_token_mint(token_mint: &AccountInfo) -> Result<Pubkey, ProgramError> {
    if *token_mint.key == MAINNET_MINT {
        Ok(MAINNET_MINT)
    } else if *token_mint.key == DEVNET_MINT {
        Ok(DEVNET_MINT)
    } else {
        Err(ProgramError::InvalidAccountData)
    }
}

fn get_fee_receiver(fee_receiver: &AccountInfo) -> Result<Pubkey, ProgramError> {
    if *fee_receiver.key == MAINNET_FEE_RECEIVER {
        Ok(MAINNET_FEE_RECEIVER)
    } else if *fee_receiver.key == DEVNET_FEE_RECEIVER {
        Ok(DEVNET_FEE_RECEIVER)
    } else {
        Err(ProgramError::InvalidAccountData)
    }
}
// From the client side we must calculate the amount of lamports needed to transfer to the vault.
// The client side we actively monitor how much is needed to satisfy the threshold needed for a user to access the features.
pub fn initialize(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Get all necessary accounts (aligned with deposit's requirements)
    let initializer = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let fee_receiver = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let associated_token_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    msg!("User wants to initialize the vault with {} tokens", amount);

    // Verify initializer signed the transaction
    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if *token_program.key != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    if *system_program.key != system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    if *associated_token_program.key != spl_associated_token_account::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let fee_receiver_key = get_fee_receiver(&fee_receiver).map_err(|_| ProgramError::InvalidAccountData)?;

    let token_mint_key = get_token_mint(&token_mint).map_err(|_| ProgramError::InvalidAccountData)?;

    if *token_mint.key != token_mint_key {
        return Err(ProgramError::InvalidAccountData);
    }

    if *fee_receiver.key != fee_receiver_key {
        return Err(ProgramError::InvalidAccountData);
    }

    // Derive PDA for vault
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[
            Vault::SEED_PREFIX.as_bytes(),
            initializer.key.as_ref(),
        ],
        program_id
    );

    // Verify derived PDA matches the vault account passed in
    if vault_pda != *vault_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // Calculate space and rent
    let vault_size = Vault::LEN;
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(vault_size);

    // Create vault account
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            vault_account.key,
            rent_lamports,
            vault_size as u64,
            program_id,
        ),
        &[
            initializer.clone(),
            vault_account.clone(),
            system_program.clone(),
        ],
        &[&[
            Vault::SEED_PREFIX.as_bytes(),
            initializer.key.as_ref(),
            &[bump],
        ]],
    )?;

    // Create ATA for vault
    invoke(
        &create_associated_token_account(
            initializer.key,
            vault_account.key,
            token_mint.key,
            token_program.key,
        ),
        &[
            initializer.clone(),
            vault_ata.clone(),
            vault_account.clone(),
            token_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            rent_sysvar.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // Transfer tokens directly here instead of calling deposit
    let transfer_instruction = token_instruction::transfer(
        token_program.key,
        user_token_account.key,
        vault_ata.key,
        initializer.key,
        &[initializer.key],
        amount * 1000000000,
    )?;

    invoke(
        &transfer_instruction,
        &[
            user_token_account.clone(),
            vault_ata.clone(),
            initializer.clone(),
            token_program.clone(),
            system_program.clone(),
        ],
    )?;

    // Transfer SOL fee
    let sol_transfer_instruction = system_instruction::transfer(
        initializer.key,
        fee_receiver.key,
        1000000000 / 10 // (10%)
    );

    invoke(
        &sol_transfer_instruction,
        &[
            initializer.clone(),
            fee_receiver.clone(),
            system_program.clone(),
        ],
    )?;

    let mut vault = Vault::new(*initializer.key, amount);
    let mut vault_data = vault_account.data.borrow_mut();
    vault.deposit_timestamp = Clock::get()?.unix_timestamp as u64;
    vault.is_locked = true;
    vault.lock_duration = Vault::LOCK_DURATION;
    vault.serialize(&mut &mut vault_data[..])?;


    msg!("Vault initialized successfully with {} tokens", amount);
    Ok(())
}
