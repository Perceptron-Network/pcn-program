use anchor_lang::prelude::*;

use crate::error::PcnError;

#[account]
#[derive(Debug, InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub oracle: Pubkey,
    pub reward_mint: Pubkey,
    pub mint_authority_bump: u8,
    pub config_bump: u8,
    pub sol_reserve: Pubkey,
    pub sol_reserve_bump: u8,
    pub token_reserve_vault: Pubkey,
    pub token_reserve_bump: u8,
    pub lifetime_curve_minted_amount: u64,
    pub claim_window_slots: u64,
    pub curve: CurveParams,
}

impl Config {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub struct CurveParams {
    pub max_epoch_mint: u64,
    pub saturation_units: u64,
    pub history_minted: u64,
    pub target_support_lamports_per_token: u64,
    pub max_supply: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_allocation_matches_serialized_space() {
        let config = Config {
            admin: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
            reward_mint: Pubkey::new_unique(),
            mint_authority_bump: u8::MAX,
            config_bump: u8::MAX,
            sol_reserve: Pubkey::new_unique(),
            sol_reserve_bump: u8::MAX,
            token_reserve_vault: Pubkey::new_unique(),
            token_reserve_bump: u8::MAX,
            lifetime_curve_minted_amount: u64::MAX,
            claim_window_slots: u64::MAX,
            curve: CurveParams {
                max_epoch_mint: u64::MAX,
                saturation_units: u64::MAX,
                history_minted: u64::MAX,
                target_support_lamports_per_token: u64::MAX,
                max_supply: u64::MAX,
            },
        };

        assert_eq!(serialized_len(&config), Config::INIT_SPACE);
        assert_eq!(Config::LEN, 8 + Config::INIT_SPACE);
        assert_eq!(serialized_len(&config.curve), CurveParams::INIT_SPACE);
    }

    fn serialized_len(value: &impl AnchorSerialize) -> usize {
        let mut data = Vec::new();
        value.serialize(&mut data).unwrap();
        data.len()
    }
}

impl CurveParams {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.max_epoch_mint > 0
                && self.saturation_units > 0
                && self.history_minted > 0
                && self.target_support_lamports_per_token > 0
                && self.max_supply > 0,
            PcnError::InvalidCurveParams
        );
        Ok(())
    }
}
