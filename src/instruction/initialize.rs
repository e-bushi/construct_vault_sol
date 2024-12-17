use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
        msg,
    },
    spl_associated_token_account::instruction::create_associated_token_account,
    crate::state::Vault,
    borsh::BorshSerialize,
};

pub fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Get all necessary accounts
    let initializer = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let associated_token_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    // Verify initializer signed the transaction
    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
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
    let vault_size = Vault::SIZE;
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

    // Initialize vault data
    let vault = Vault::new(*initializer.key, 0);
    vault.serialize(&mut *vault_account.data.borrow_mut())?;

    msg!("Vault initialized successfully");
    Ok(())
}
