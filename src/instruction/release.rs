use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey
};
use crate::state::Vault;
use spl_token::instruction as token_instruction;
use borsh::BorshSerialize;

pub fn release(
    program_id: &Pubkey, 
    accounts: &[AccountInfo],
    vault: &mut Vault
) -> ProgramResult {
    msg!("Releasing tokens from the vault");

    let account_info_iter = &mut accounts.iter();

    let user = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let (pda, bump) = Pubkey::find_program_address(
        &[
            Vault::SEED_PREFIX.as_bytes(),
            user.key.as_ref(),
        ],
        program_id
    );

    msg!("Releasing {} tokens", vault.amount_locked);

    let transfer_instruction = token_instruction::transfer(
        token_program.key,
        &vault_ata.key,
        &user_token_account.key,
        &pda,
        &[&pda],
        vault.amount_locked,
    )?;

    msg!("Signing the transfer");

    invoke_signed(
        &transfer_instruction,
        &[
            vault_ata.clone(),
            user_token_account.clone(),
            vault_account.clone(),
            token_program.clone(),
        ],
        &[&[
            Vault::SEED_PREFIX.as_bytes(),
            user.key.as_ref(),
            &[bump]
        ]],
    )?;

    vault.amount_locked = 0;
    vault.is_locked = false;
    vault.deposit_timestamp = 0;
    vault.lock_duration = 0;

    let mut vault_data = vault_account.data.borrow_mut();
    vault.serialize(&mut &mut vault_data[..])?;
    
    Ok(())
}