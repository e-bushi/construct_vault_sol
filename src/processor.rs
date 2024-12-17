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
        VaultInstruction::Initialize => initialize(program_id, accounts),
        VaultInstruction::Deposit => deposit(program_id, accounts),
        VaultInstruction::Withdraw => withdraw(program_id, accounts),
        VaultInstruction::Release => release(program_id, accounts),
        VaultInstruction::Extend => extend(program_id, accounts),
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum VaultInstruction {
    Initialize,
    Deposit,
    Withdraw,
    Release,
    Extend,
}
