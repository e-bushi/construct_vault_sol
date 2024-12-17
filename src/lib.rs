use solana_program::entrypoint;

use processor::process_instruction;

pub mod state;
pub mod instruction;
pub mod processor;

pub use instruction::*;

entrypoint!(process_instruction);