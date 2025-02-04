use {
    crate::state::Vault, 
    borsh::BorshDeserialize, 
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

const MAINNET_FEE_RECEIVER: Pubkey = Pubkey::from_str_const("5zaUUZWoXaWt2Ht5NNQZuQXyfaQKDLyQoESn6BXvVzBd");
const DEVNET_FEE_RECEIVER: Pubkey = Pubkey::from_str_const("8jHMkdtKK4CCn4ep6Hponmk1ik7ofUNS9bX9qSuiRcN5");

fn get_fee_receiver(fee_receiver: &AccountInfo) -> Result<Pubkey, ProgramError> {
    if *fee_receiver.key == MAINNET_FEE_RECEIVER {
        Ok(MAINNET_FEE_RECEIVER)
    } else if *fee_receiver.key == DEVNET_FEE_RECEIVER {
        Ok(DEVNET_FEE_RECEIVER)
    } else {
        Err(ProgramError::InvalidAccountData)
    }
}

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

    let fee_receiver_key = get_fee_receiver(&fee_receiver).map_err(|_| ProgramError::InvalidAccountData)?;

    if *fee_receiver.key != fee_receiver_key {
        return Err(ProgramError::InvalidAccountData);
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
    // Drop the borrow here
    drop(vault_data);

    let time_elasped_in_days: u64 = (Clock::get()?.unix_timestamp as u64  - vault.deposit_timestamp) / 86400;
    msg!("Time elasped in days: {}", time_elasped_in_days);

    let duration_in_days: u64 = vault.lock_duration / 86400;
    msg!("Lock Period Duration In Days: {}", duration_in_days);

    if time_elasped_in_days < duration_in_days {
        msg!("Vault is still within lock period");

        let percentage_of_lock_period: f64 = (time_elasped_in_days as f64 / duration_in_days as f64) * 100.0;
        msg!("Percentage of lock period completed: {}%", percentage_of_lock_period);

        let fee_percentage: f64 = 0.75 * (1.0 - percentage_of_lock_period / 100.0);
        msg!("Early withdrawal fee percentage: {}%", fee_percentage * 100.0);

        let total_amount_in_lamports: u64 = (5_000_000_000f64 * fee_percentage) as u64;
        msg!("Total fee in Lamports: {}", total_amount_in_lamports);

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
        release(program_id, accounts, &mut vault)?;
        return Ok(());
    } else {
        msg!("Vault is not locked, so it's free to release");
        release(program_id, accounts, &mut vault)?;
        return Ok(());
    }
}