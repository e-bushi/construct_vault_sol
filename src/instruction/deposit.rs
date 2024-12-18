use {
    crate::state::Vault, borsh::{BorshDeserialize, BorshSerialize}, solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        msg,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvar::Sysvar,
    }, spl_token::instruction as token_instruction
};

pub fn deposit(
    program_id: &Pubkey, 
    accounts: &[AccountInfo], 
    amount: u64
) -> ProgramResult {
    msg!("Depositing funds into the vault");

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive PDA for vault
    let (pda, _) = Pubkey::find_program_address(
        &[
            Vault::SEED_PREFIX.as_bytes(),
            initializer.key.as_ref(),
        ],
        program_id
    );

    if pda != *vault_account.key {
        return Err(ProgramError::InvalidSeeds);
    }


    if *vault_ata.owner != spl_token::id() {
        msg!("Vault ATA is not owned by the vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut vault_data = vault_account.data.borrow_mut();
    let mut vault = Vault::deserialize(&mut &vault_data[..])?;

    msg!("Depositing {} tokens", amount);

    let transfer_instruction = token_instruction::transfer(
        token_program.key,
        user_token_account.key,
        vault_ata.key,
        initializer.key,
        &[initializer.key],
        amount,
    )?;

    invoke(
        &transfer_instruction,
        &[
            user_token_account.clone(),
            vault_ata.clone(),
            initializer.clone(),
            token_program.clone(),
        ],
    )?;

    vault.amount_locked += amount;
    vault.deposit_timestamp = Clock::get()?.unix_timestamp as u64;
    vault.is_locked = true;
    vault.lock_duration = Vault::LOCK_DURATION;

    // Serialize back into the same borrowed data
    vault.serialize(&mut &mut vault_data[..])?;

    msg!("Successfully deposited {} tokens and updated the vault", amount);
    Ok(())
}