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
    spl_token::instruction as token_instruction,
    crate::release
};

const WITHDRAW_FEE: u64 = 1000000000 / 2; // 0.5 SOL (50%)

pub fn withdraw(
    program_id: &Pubkey, 
    accounts: &[AccountInfo]
) -> ProgramResult {
    msg!("Withdrawing funds from the vault early");

    let account_info_iter = &mut accounts.iter();

    let user = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

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

    let mut vault_data = vault_account.data.borrow_mut();
    let mut vault = Vault::deserialize(&mut &vault_data[..])?;
    
    if !vault.is_locked {
        msg!("Vault is not locked");
        
        release(program_id, accounts)?;
        return Ok(());
    }


    Ok(())
}