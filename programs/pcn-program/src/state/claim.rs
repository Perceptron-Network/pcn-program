use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
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
            bandwidth_units: u64::MAX,
            quality_factor_ppm: u64::MAX,
            reward_weight: u64::MAX,
            reward_amount: u64::MAX,
            claimed: true,
            bump: u8::MAX,
        };

        let mut data = Vec::new();
        claim.serialize(&mut data).unwrap();
        assert_eq!(data.len(), Claim::INIT_SPACE);
        assert_eq!(Claim::LEN, 8 + Claim::INIT_SPACE);
    }
}
