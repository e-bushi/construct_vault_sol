
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey
};
use crate::state::Vault;
use spl_token::instruction as token_instruction;
use borsh::{BorshDeserialize, BorshSerialize};

pub fn release(
    program_id: &Pubkey, 
    accounts: &[AccountInfo]
) -> ProgramResult {
    msg!("Releasing tokens from the vault");

    let account_info_iter = &mut accounts.iter();

    let user = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let (pda, _) = Pubkey::find_program_address(
        &[
            Vault::SEED_PREFIX.as_bytes(),
            user.key.as_ref(),
        ],
        program_id
    );

    let mut vault_data = vault_account.data.borrow_mut();
    let mut vault = Vault::deserialize(&mut &vault_data[..])?;

    msg!("Releasing {} tokens", vault.amount_locked);

    let transfer_instruction = token_instruction::transfer(
        token_program.key,
        vault_ata.key,
        user_token_account.key,
        &pda,
        &[&pda],
        vault.amount_locked,
    )?;

    invoke(
        &transfer_instruction,
        &[
            user_token_account.clone(),
            vault_ata.clone(),
            user.clone(),
            token_program.clone(),
            system_program.clone(),
        ],
    )?;

    vault.amount_locked = 0;
    vault.is_locked = false;
    vault.deposit_timestamp = 0;
    vault.lock_duration = 0;

    vault.serialize(&mut &mut vault_data[..])?;
    
    Ok(())
}