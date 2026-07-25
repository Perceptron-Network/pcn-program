use anchor_lang::prelude::*;

use crate::PerformanceWeights;

#[account]
#[derive(Debug, InitSpace)]
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
    pub support_funder: Pubkey,
    pub performance_weights: PerformanceWeights,
}

impl Epoch {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum EpochStatus {
    Open,
    Finalized,
    Swept,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_allocation_matches_serialized_space() {
        let epoch = Epoch {
            epoch_id: u64::MAX,
            status: EpochStatus::Swept,
            start_slot: u64::MAX,
            end_slot: u64::MAX,
            support_budget_lamports: u64::MAX,
            consumed_support_lamports: u64::MAX,
            total_reward_weight: u64::MAX,
            reward_pool_amount: u64::MAX,
            allocated_amount: u64::MAX,
            claimed_amount: u64::MAX,
            claim_deadline_slot: u64::MAX,
            epoch_token_vault: Pubkey::new_unique(),
            epoch_vault_bump: u8::MAX,
            bump: u8::MAX,
            support_funder: Pubkey::new_unique(),
            performance_weights: PerformanceWeights {
                uptime_ppm: 250_000,
                bandwidth_ppm: 250_000,
                fulfilment_rate_ppm: 250_000,
                quest_score_ppm: 250_000,
            },
        };

        assert_eq!(serialized_len(&epoch), Epoch::INIT_SPACE);
        assert_eq!(Epoch::INIT_SPACE, 179);
        assert_eq!(Epoch::LEN, 8 + Epoch::INIT_SPACE);
        assert_eq!(serialized_len(&EpochStatus::Swept), EpochStatus::INIT_SPACE);
        assert_eq!(
            serialized_len(&epoch.performance_weights),
            PerformanceWeights::INIT_SPACE
        );
    }

    fn serialized_len(value: &impl AnchorSerialize) -> usize {
        let mut data = Vec::new();
        value.serialize(&mut data).unwrap();
        data.len()
    }
}
