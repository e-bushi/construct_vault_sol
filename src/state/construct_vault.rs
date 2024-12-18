use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Display;
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Vault {
    pub owner: Pubkey,
    pub lock_duration: u64,
    pub amount_locked: u64,
    pub deposit_timestamp: u64,
    pub is_locked: bool,
    pub bump: u8,
}

impl Vault {
    pub const SIZE: usize = 32 + 8 + 8 + 8 + 1 + 1;

    pub const LOCK_DURATION: u64 = 60 * 60 * 24 * 30;

    pub const SEED_PREFIX: &'static str = "kuza_vault";

    pub fn new(owner: Pubkey, amount_locked: u64) -> Self {
        Self {
            owner,
            lock_duration: 0,
            amount_locked,
            deposit_timestamp: 0,
            is_locked: false,
            bump: 0,
        }
    }
}