use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use borsh::{BorshDeserialize, BorshSerialize};

use crate::instruction::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    let instruction = VaultInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        VaultInstruction::Initialize { amount } => initialize(program_id, accounts, amount),
        VaultInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
        VaultInstruction::Withdraw => withdraw(program_id, accounts),
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum VaultInstruction {
    Initialize { amount: u64 },
    Deposit { amount: u64 },
    Withdraw,
}