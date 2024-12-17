use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey,
};

pub fn initialize(program_id: Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
