use {
    crate::state::Vault, 
    borsh::{BorshDeserialize, BorshSerialize}, 
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        msg,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvar::Sysvar,
        system_instruction
    },
    crate::release
};

pub fn withdraw(
    program_id: &Pubkey, 
    accounts: &[AccountInfo]
) -> ProgramResult {
    msg!("Withdrawing funds from the vault early");

    let account_info_iter = &mut accounts.iter();

    let user = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;
    let fee_receiver = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, _) = Pubkey::find_program_address(
        &[
            Vault::SEED_PREFIX.as_bytes(),
            user.key.as_ref(),
        ],
        program_id
    );

    if *vault_account.key != pda {
        return Err(ProgramError::InvalidAccountData);
    }

    if *vault_ata.owner != spl_token::id() {
        msg!("Vault ATA is not owned by the vault");
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_data = vault_account.data.borrow_mut();
    let mut vault = Vault::deserialize(&mut &vault_data[..])?;

    let time_elasped_in_days: u64 = (Clock::get()?.unix_timestamp as u64  - vault.deposit_timestamp) / 86400;
    let duration_in_days: u64 = vault.lock_duration / 86400;

    if time_elasped_in_days < duration_in_days {
        msg!("Vault is still within lock period");
        let percentage_of_lock_period = (time_elasped_in_days / duration_in_days) * 100;
        let early_withdraw_fee = 1 / percentage_of_lock_period;
        let total_amount_in_lamports = 1_000_000_000 * early_withdraw_fee;

         // Transfer SOL fee
         msg!("Transferring SOL fee to the fee receiver");
        let sol_transfer_instruction = system_instruction::transfer(
            user.key,
            fee_receiver.key,
            total_amount_in_lamports
        );

        msg!("Invoking SOL transfer instruction");
        invoke(&sol_transfer_instruction, accounts)?;

        msg!("Attempting to release tokens from the vault");
        release(program_id, accounts)?;
        return Ok(());
    } else {
        msg!("Vault is not locked");
        release(program_id, accounts)?;
        return Ok(());
    }
}