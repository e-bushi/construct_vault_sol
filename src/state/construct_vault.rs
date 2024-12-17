use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Vault {
    pub owner: Pubkey,
    pub lock_duration: i64,    // Changed to i64 to match timestamp
    pub amount_locked: u64,
    pub deposit_timestamp: i64,
    pub is_locked: bool,
    pub bump: u8,
}

impl Vault {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1;

    pub const LOCK_DURATION: i64 = 60 * 60 * 24 * 30;

    pub const SEED_PREFIX: &str = "kuza_vault";

    pub fn new(owner: Pubkey, amount_locked: u64) -> Self {
        Self {
            owner,
            lock_duration: Self::LOCK_DURATION,
            amount_locked,
            deposit_timestamp: 0,
            is_locked: false,
            bump: 0,
        }
    }
}
