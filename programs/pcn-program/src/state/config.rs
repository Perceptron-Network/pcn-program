use anchor_lang::prelude::*;
use core::mem::size_of;

use crate::error::PcnError;

#[account]
#[derive(Debug)]
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
    pub const LEN: usize = 8 + size_of::<Self>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurveParams {
    pub max_epoch_mint: u64,
    pub saturation_units: u64,
    pub history_minted: u64,
    pub target_support_lamports_per_token: u64,
    pub max_supply: u64,
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
