use anchor_lang::prelude::*;

use crate::{error::PcnError, CurveParams, QUALITY_PPM_SCALE, TOKEN_BASE_UNITS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardPoolAmounts {
    pub scarcity_cap: u64,
    pub support_cap: u64,
    pub remaining_supply: u64,
    pub reward_pool: u64,
    pub consumed_support_lamports: u64,
}

pub fn compute_reward_weight(bandwidth_units: u64, quality_factor_ppm: u64) -> Result<u64> {
    require!(
        quality_factor_ppm <= QUALITY_PPM_SCALE,
        PcnError::InvalidQualityFactor
    );
    let weight = u128::from(bandwidth_units)
        .checked_mul(u128::from(quality_factor_ppm))
        .ok_or(PcnError::MathOverflow)?
        .checked_div(u128::from(QUALITY_PPM_SCALE))
        .ok_or(PcnError::MathOverflow)?;
    u64::try_from(weight).map_err(|_| PcnError::MathOverflow.into())
}

pub fn compute_claim_amount(
    reward_pool_amount: u64,
    reward_weight: u64,
    total_reward_weight: u64,
) -> Result<u64> {
    require!(total_reward_weight > 0, PcnError::ZeroTotalRewardWeight);
    let amount = u128::from(reward_pool_amount)
        .checked_mul(u128::from(reward_weight))
        .ok_or(PcnError::MathOverflow)?
        .checked_div(u128::from(total_reward_weight))
        .ok_or(PcnError::MathOverflow)?;
    u64::try_from(amount).map_err(|_| PcnError::MathOverflow.into())
}

pub fn compute_reward_pool(
    curve: CurveParams,
    total_reward_weight: u64,
    lifetime_curve_minted_amount: u64,
    support_budget_lamports: u64,
) -> Result<RewardPoolAmounts> {
    curve.validate()?;
    require!(total_reward_weight > 0, PcnError::ZeroTotalRewardWeight);
    require!(support_budget_lamports > 0, PcnError::InvalidSupportBudget);
    require!(
        lifetime_curve_minted_amount < curve.max_supply,
        PcnError::MaxSupplyExhausted
    );

    let remaining_supply = curve
        .max_supply
        .checked_sub(lifetime_curve_minted_amount)
        .ok_or(PcnError::MathOverflow)?;
    require!(remaining_supply > 0, PcnError::MaxSupplyExhausted);

    let scarcity_cap =
        calculate_scarcity_cap(curve, total_reward_weight, lifetime_curve_minted_amount)?;
    let support_cap = calculate_support_capacity(curve, support_budget_lamports)?;
    let reward_pool = scarcity_cap.min(support_cap).min(remaining_supply);
    require!(reward_pool > 0, PcnError::ZeroRewardPool);

    let consumed_support_lamports = calculate_consumed_support(curve, reward_pool)?;
    require!(
        consumed_support_lamports <= support_budget_lamports,
        PcnError::InvalidSupportBudget
    );

    Ok(RewardPoolAmounts {
        scarcity_cap,
        support_cap,
        remaining_supply,
        reward_pool,
        consumed_support_lamports,
    })
}

pub fn calculate_scarcity_cap(
    curve: CurveParams,
    total_reward_weight: u64,
    lifetime_curve_minted_amount: u64,
) -> Result<u64> {
    curve.validate()?;
    let reward_weight_ratio = total_reward_weight as f64 / curve.saturation_units as f64;
    let minted_history_ratio = lifetime_curve_minted_amount as f64 / curve.history_minted as f64;
    let scarcity_base_units = curve.max_epoch_mint as f64 * (1.0 - libm::exp(-reward_weight_ratio))
        / (1.0 + minted_history_ratio);
    round_curve_base_units(scarcity_base_units)
}

pub fn calculate_support_capacity(curve: CurveParams, support_budget_lamports: u64) -> Result<u64> {
    curve.validate()?;
    let support_base_units = u128::from(support_budget_lamports)
        .checked_mul(u128::from(TOKEN_BASE_UNITS))
        .ok_or(PcnError::MathOverflow)?
        .checked_div(u128::from(curve.target_support_lamports_per_token))
        .ok_or(PcnError::MathOverflow)?;
    u64::try_from(support_base_units).map_err(|_| PcnError::MathOverflow.into())
}

pub fn calculate_consumed_support(curve: CurveParams, reward_pool: u64) -> Result<u64> {
    curve.validate()?;
    let numerator = u128::from(reward_pool)
        .checked_mul(u128::from(curve.target_support_lamports_per_token))
        .ok_or(PcnError::MathOverflow)?;
    let consumed = ceil_div_u128(numerator, u128::from(TOKEN_BASE_UNITS))?;
    u64::try_from(consumed).map_err(|_| PcnError::MathOverflow.into())
}

fn ceil_div_u128(numerator: u128, denominator: u128) -> Result<u128> {
    require!(denominator > 0, PcnError::MathOverflow);
    if numerator == 0 {
        return Ok(0);
    }
    numerator
        .checked_sub(1)
        .and_then(|n| n.checked_div(denominator))
        .and_then(|q| q.checked_add(1))
        .ok_or(PcnError::MathOverflow.into())
}

fn round_curve_base_units(amount: f64) -> Result<u64> {
    let rounded_amount = amount.round();
    if !rounded_amount.is_finite() || rounded_amount < 0.0 || rounded_amount > u64::MAX as f64 {
        return Err(PcnError::MathOverflow.into());
    }
    Ok(rounded_amount as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve() -> CurveParams {
        CurveParams {
            max_epoch_mint: 1_000 * TOKEN_BASE_UNITS,
            saturation_units: 10_000,
            history_minted: 10_000 * TOKEN_BASE_UNITS,
            target_support_lamports_per_token: 50_000_000,
            max_supply: 1_000_000 * TOKEN_BASE_UNITS,
        }
    }

    #[test]
    fn reward_weight_scales_bandwidth_by_quality() {
        assert_eq!(
            compute_reward_weight(1_000, QUALITY_PPM_SCALE).unwrap(),
            1_000
        );
        assert_eq!(compute_reward_weight(1_000, 500_000).unwrap(), 500);
        assert_eq!(compute_reward_weight(999, 333_333).unwrap(), 332);
        assert!(compute_reward_weight(1, QUALITY_PPM_SCALE + 1).is_err());
    }

    #[test]
    fn scarcity_matches_zinc_curve_shape_and_rounding() {
        let params = curve();
        let low = calculate_scarcity_cap(params, 1_000, 0).unwrap();
        let high = calculate_scarcity_cap(params, 10_000, 0).unwrap();
        let saturated = calculate_scarcity_cap(params, 1_000_000, 0).unwrap();

        assert_eq!(low, 95_162_581_964);
        assert_eq!(high, 632_120_558_829);
        assert!(high > low);
        assert_eq!(saturated, params.max_epoch_mint);
    }

    #[test]
    fn scarcity_decreases_as_lifetime_mint_grows() {
        let params = curve();
        let low_history = calculate_scarcity_cap(params, 10_000, 0).unwrap();
        let high_history = calculate_scarcity_cap(params, 10_000, params.history_minted).unwrap();

        assert_eq!(low_history, 632_120_558_829);
        assert_eq!(high_history, 316_060_279_414);
        assert!(high_history < low_history);
    }

    #[test]
    fn support_capacity_and_consumed_support_use_base_units() {
        let params = curve();

        assert_eq!(
            calculate_support_capacity(params, params.target_support_lamports_per_token).unwrap(),
            TOKEN_BASE_UNITS
        );
        assert_eq!(
            calculate_support_capacity(params, params.target_support_lamports_per_token / 2)
                .unwrap(),
            TOKEN_BASE_UNITS / 2
        );
        assert_eq!(
            calculate_consumed_support(params, TOKEN_BASE_UNITS).unwrap(),
            50_000_000
        );
        assert_eq!(calculate_consumed_support(params, 1).unwrap(), 1);
    }

    #[test]
    fn reward_pool_caps_by_support_and_remaining_supply() {
        let mut params = curve();
        let abundant_support = 10_000 * params.target_support_lamports_per_token;
        let pool = compute_reward_pool(params, 10_000, 0, abundant_support).unwrap();

        assert_eq!(pool.scarcity_cap, 632_120_558_829);
        assert_eq!(pool.reward_pool, pool.scarcity_cap);

        let support_limited = compute_reward_pool(
            params,
            10_000_000,
            0,
            params.target_support_lamports_per_token,
        )
        .unwrap();
        assert_eq!(support_limited.reward_pool, TOKEN_BASE_UNITS);
        assert_eq!(support_limited.consumed_support_lamports, 50_000_000);

        params.max_supply = 100;
        let supply_limited = compute_reward_pool(params, 10_000_000, 0, abundant_support).unwrap();
        assert_eq!(supply_limited.reward_pool, 100);
    }

    #[test]
    fn reward_pool_rejects_zero_inputs_and_exhaustion() {
        let params = curve();

        assert!(compute_reward_pool(params, 0, 0, 1).is_err());
        assert!(compute_reward_pool(params, 1, 0, 0).is_err());
        assert!(compute_reward_pool(params, 1, params.max_supply, 1).is_err());
    }

    #[test]
    fn claim_amounts_floor_and_leave_dust() {
        let pool = 100;
        let total_weight = 3;

        assert_eq!(compute_claim_amount(pool, 1, total_weight).unwrap(), 33);
        assert_eq!(compute_claim_amount(pool, 2, total_weight).unwrap(), 66);
        assert!(compute_claim_amount(pool, 1, 0).is_err());
    }
}
