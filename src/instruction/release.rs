
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey,
};

pub fn release(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}