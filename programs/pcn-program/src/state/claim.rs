use anchor_lang::prelude::*;

use crate::{error::PcnError, PERFORMANCE_PPM_SCALE};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub struct PerformanceMetrics {
    pub uptime_ppm: u64,
    pub bandwidth_ppm: u64,
    pub fulfilment_rate_ppm: u64,
    pub quest_score_ppm: u64,
}

impl PerformanceMetrics {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.uptime_ppm <= PERFORMANCE_PPM_SCALE
                && self.bandwidth_ppm <= PERFORMANCE_PPM_SCALE
                && self.fulfilment_rate_ppm <= PERFORMANCE_PPM_SCALE
                && self.quest_score_ppm <= PERFORMANCE_PPM_SCALE,
            PcnError::InvalidPerformanceMetrics
        );
        Ok(())
    }
}

#[account]
#[derive(Debug, InitSpace)]
pub struct Claim {
    pub epoch: Pubkey,
    pub epoch_id: u64,
    pub user: Pubkey,
    pub performance: PerformanceMetrics,
    pub reward_weight: u64,
    pub reward_amount: u64,
    pub claimed: bool,
    pub bump: u8,
}

impl Claim {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_allocation_matches_serialized_space() {
        let claim = Claim {
            epoch: Pubkey::new_unique(),
            epoch_id: u64::MAX,
            user: Pubkey::new_unique(),
            performance: PerformanceMetrics {
                uptime_ppm: PERFORMANCE_PPM_SCALE,
                bandwidth_ppm: PERFORMANCE_PPM_SCALE,
                fulfilment_rate_ppm: PERFORMANCE_PPM_SCALE,
                quest_score_ppm: PERFORMANCE_PPM_SCALE,
            },
            reward_weight: u64::MAX,
            reward_amount: u64::MAX,
            claimed: true,
            bump: u8::MAX,
        };

        let mut data = Vec::new();
        claim.serialize(&mut data).unwrap();
        assert_eq!(data.len(), Claim::INIT_SPACE);
        assert_eq!(Claim::INIT_SPACE, 122);
        assert_eq!(Claim::LEN, 8 + Claim::INIT_SPACE);
        assert_eq!(
            serialized_len(&claim.performance),
            PerformanceMetrics::INIT_SPACE
        );
    }

    fn serialized_len(value: &impl AnchorSerialize) -> usize {
        let mut data = Vec::new();
        value.serialize(&mut data).unwrap();
        data.len()
    }
}
