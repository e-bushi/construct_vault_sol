use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey,
};

pub fn deposit(_program_id: &Pubkey, _caccounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
