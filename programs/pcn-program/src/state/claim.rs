use anchor_lang::prelude::*;
use core::mem::size_of;

#[account]
#[derive(Debug)]
pub struct Claim {
    pub epoch: Pubkey,
    pub epoch_id: u64,
    pub user: Pubkey,
    pub bandwidth_units: u64,
    pub quality_factor_ppm: u64,
    pub reward_weight: u64,
    pub reward_amount: u64,
    pub claimed: bool,
    pub bump: u8,
}

impl Claim {
    pub const LEN: usize = 8 + size_of::<Self>();
}
