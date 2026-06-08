use anchor_lang::prelude::*;
use core::mem::size_of;

#[account]
#[derive(Debug)]
pub struct Epoch {
    pub epoch_id: u64,
    pub status: EpochStatus,
    pub start_slot: u64,
    pub end_slot: u64,
    pub support_budget_lamports: u64,
    pub consumed_support_lamports: u64,
    pub total_reward_weight: u64,
    pub reward_pool_amount: u64,
    pub allocated_amount: u64,
    pub claimed_amount: u64,
    pub claim_deadline_slot: u64,
    pub epoch_token_vault: Pubkey,
    pub epoch_vault_bump: u8,
    pub bump: u8,
}

impl Epoch {
    pub const LEN: usize = 8 + size_of::<Self>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpochStatus {
    Open,
    Finalized,
    Swept,
}
