use solana_program::entrypoint;

use processor::process_instruction;

pub mod processor;

entrypoint!(process_instruction);