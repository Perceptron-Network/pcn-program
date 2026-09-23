use anchor_lang::prelude::*;

use crate::{
    error::PcnError, CurveParams, PerformanceMetrics, PerformanceWeights,
    EMISSION_MULTIPLIER_PPM_SCALE, PERFORMANCE_PPM_SCALE, TOKEN_BASE_UNITS,
};

const Q64_ONE: u128 = 1_u128 << 64;
const Q64_HALF: u128 = Q64_ONE >> 1;
const LN_2_Q64: u128 = 12_786_308_645_202_655_660;
const EXP_TAYLOR_TERMS: u32 = 20;
const EXP_SATURATION_HALVINGS: u128 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardPoolAmounts {
    pub scarcity_cap: u64,
    pub support_cap: u64,
    pub remaining_supply: u64,
    pub reward_pool: u64,
    pub consumed_support_lamports: u64,
}

pub fn compute_reward_weight(
    performance: PerformanceMetrics,
    weights: PerformanceWeights,
) -> Result<u64> {
    performance.validate()?;
    weights.validate()?;
    let weighted_uptime = u128::from(performance.uptime_ppm)
        .checked_mul(u128::from(weights.uptime_ppm))
        .ok_or(PcnError::MathOverflow)?;
    let weighted_bandwidth = u128::from(performance.bandwidth_ppm)
        .checked_mul(u128::from(weights.bandwidth_ppm))
        .ok_or(PcnError::MathOverflow)?;
    let weighted_fulfilment = u128::from(performance.fulfilment_rate_ppm)
        .checked_mul(u128::from(weights.fulfilment_rate_ppm))
        .ok_or(PcnError::MathOverflow)?;
    let weighted_quest = u128::from(performance.quest_score_ppm)
        .checked_mul(u128::from(weights.quest_score_ppm))
        .ok_or(PcnError::MathOverflow)?;
    let weight = weighted_uptime
        .checked_add(weighted_bandwidth)
        .and_then(|value| value.checked_add(weighted_fulfilment))
        .and_then(|value| value.checked_add(weighted_quest))
        .ok_or(PcnError::MathOverflow)?
        .checked_div(u128::from(PERFORMANCE_PPM_SCALE))
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
    let exponent_q64 = ratio_to_q64(total_reward_weight, curve.saturation_units)?;
    let emission_factor_q64 = Q64_ONE
        .checked_sub(exp_neg_q64(exponent_q64)?)
        .ok_or(PcnError::MathOverflow)?;

    let history_denominator = u128::from(curve.history_minted)
        .checked_add(u128::from(lifetime_curve_minted_amount))
        .ok_or(PcnError::MathOverflow)?;
    let history_factor_q64 =
        ratio_u128_to_q64(u128::from(curve.history_minted), history_denominator)?;
    let emission_multiplier_q64 =
        ratio_to_q64(curve.emission_multiplier_ppm, EMISSION_MULTIPLIER_PPM_SCALE)?;
    let scarcity_factor_q64 = q64_mul(
        q64_mul(emission_factor_q64, history_factor_q64)?,
        emission_multiplier_q64,
    )?;
    q64_scale_u64_round(curve.max_epoch_mint, scarcity_factor_q64)
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

fn ratio_to_q64(numerator: u64, denominator: u64) -> Result<u128> {
    ratio_u128_to_q64(u128::from(numerator), u128::from(denominator))
}

fn ratio_u128_to_q64(numerator: u128, denominator: u128) -> Result<u128> {
    require!(denominator > 0, PcnError::MathOverflow);
    let scaled = numerator.checked_shl(64).ok_or(PcnError::MathOverflow)?;
    let rounded = scaled
        .checked_add(denominator / 2)
        .ok_or(PcnError::MathOverflow)?
        .checked_div(denominator)
        .ok_or(PcnError::MathOverflow)?;
    Ok(rounded)
}

/// Returns `e^-x` in Q64.64. Inputs at or above `64 * ln(2)` saturate to zero,
/// because their real result is below one Q64.64 unit.
fn exp_neg_q64(exponent_q64: u128) -> Result<u128> {
    let saturation = LN_2_Q64
        .checked_mul(EXP_SATURATION_HALVINGS)
        .ok_or(PcnError::MathOverflow)?;
    if exponent_q64 >= saturation {
        return Ok(0);
    }

    let halvings = exponent_q64
        .checked_div(LN_2_Q64)
        .ok_or(PcnError::MathOverflow)?;
    let remainder = exponent_q64
        .checked_sub(
            halvings
                .checked_mul(LN_2_Q64)
                .ok_or(PcnError::MathOverflow)?,
        )
        .ok_or(PcnError::MathOverflow)?;

    // Approximate e^remainder with a positive-term Taylor series, then take its
    // reciprocal. Positive terms make the integer approximation monotonic;
    // the alternating e^-x series can oscillate by one Q64 unit after rounding.
    let mut exp_positive = Q64_ONE;
    let mut term = Q64_ONE;
    for order in 1..=EXP_TAYLOR_TERMS {
        term = q64_mul_div_u32(term, remainder, order)?;
        exp_positive = exp_positive
            .checked_add(term)
            .ok_or(PcnError::MathOverflow)?;
        if term == 0 {
            break;
        }
    }
    let reduced_exp_neg = q64_reciprocal(exp_positive)?;

    if halvings == 0 {
        return Ok(reduced_exp_neg);
    }
    let shift = u32::try_from(halvings).map_err(|_| PcnError::MathOverflow)?;
    let rounding = 1_u128
        .checked_shl(shift - 1)
        .ok_or(PcnError::MathOverflow)?;
    reduced_exp_neg
        .checked_add(rounding)
        .ok_or(PcnError::MathOverflow)?
        .checked_shr(shift)
        .ok_or(PcnError::MathOverflow.into())
}

fn q64_reciprocal(value_q64: u128) -> Result<u128> {
    require!(value_q64 >= Q64_ONE, PcnError::MathOverflow);
    // Divide 2^128 by the Q64.64 denominator without materializing 2^128.
    let quotient = u128::MAX
        .checked_div(value_q64)
        .ok_or(PcnError::MathOverflow)?;
    let remainder_plus_one = u128::MAX
        .checked_rem(value_q64)
        .ok_or(PcnError::MathOverflow)?
        .checked_add(1)
        .ok_or(PcnError::MathOverflow)?;
    let round_up = remainder_plus_one >= value_q64 - remainder_plus_one;
    quotient
        .checked_add(u128::from(round_up))
        .ok_or(PcnError::MathOverflow.into())
}

fn q64_mul_div_u32(left: u128, right: u128, divisor: u32) -> Result<u128> {
    let denominator = Q64_ONE
        .checked_mul(u128::from(divisor))
        .ok_or(PcnError::MathOverflow)?;
    left.checked_mul(right)
        .ok_or(PcnError::MathOverflow)?
        .checked_add(denominator / 2)
        .ok_or(PcnError::MathOverflow)?
        .checked_div(denominator)
        .ok_or(PcnError::MathOverflow.into())
}

fn q64_mul(left: u128, right: u128) -> Result<u128> {
    require!(left <= Q64_ONE && right <= Q64_ONE, PcnError::MathOverflow);
    if left == Q64_ONE {
        return Ok(right);
    }
    if right == Q64_ONE {
        return Ok(left);
    }
    left.checked_mul(right)
        .ok_or(PcnError::MathOverflow)?
        .checked_add(Q64_HALF)
        .ok_or(PcnError::MathOverflow)?
        .checked_div(Q64_ONE)
        .ok_or(PcnError::MathOverflow.into())
}

fn q64_scale_u64_round(value: u64, factor_q64: u128) -> Result<u64> {
    require!(factor_q64 <= Q64_ONE, PcnError::MathOverflow);
    if factor_q64 == Q64_ONE {
        return Ok(value);
    }
    let scaled = u128::from(value)
        .checked_mul(factor_q64)
        .ok_or(PcnError::MathOverflow)?
        .checked_add(Q64_HALF)
        .ok_or(PcnError::MathOverflow)?
        .checked_div(Q64_ONE)
        .ok_or(PcnError::MathOverflow)?;
    u64::try_from(scaled).map_err(|_| PcnError::MathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn curve() -> CurveParams {
        CurveParams {
            max_epoch_mint: 1_000 * TOKEN_BASE_UNITS,
            emission_multiplier_ppm: EMISSION_MULTIPLIER_PPM_SCALE,
            saturation_units: 10_000,
            history_minted: 10_000 * TOKEN_BASE_UNITS,
            target_support_lamports_per_token: 50_000_000,
            max_supply: 1_000_000 * TOKEN_BASE_UNITS,
        }
    }

    fn equal_weights() -> PerformanceWeights {
        PerformanceWeights {
            uptime_ppm: 250_000,
            bandwidth_ppm: 250_000,
            fulfilment_rate_ppm: 250_000,
            quest_score_ppm: 250_000,
        }
    }

    #[test]
    fn reward_weight_is_governance_weighted_four_metric_score() {
        let performance = PerformanceMetrics {
            uptime_ppm: 1_000_000,
            bandwidth_ppm: 800_000,
            fulfilment_rate_ppm: 600_000,
            quest_score_ppm: 400_000,
        };
        assert_eq!(
            compute_reward_weight(performance, equal_weights()).unwrap(),
            700_000
        );
    }

    #[test]
    fn reward_weight_rejects_out_of_range_metric_and_invalid_weights() {
        let mut performance = PerformanceMetrics {
            uptime_ppm: 1_000_000,
            bandwidth_ppm: 800_000,
            fulfilment_rate_ppm: 600_000,
            quest_score_ppm: 400_000,
        };
        performance.uptime_ppm = PERFORMANCE_PPM_SCALE + 1;
        assert!(compute_reward_weight(performance, equal_weights()).is_err());

        performance.uptime_ppm = PERFORMANCE_PPM_SCALE;
        let mut invalid_weights = equal_weights();
        invalid_weights.quest_score_ppm -= 1;
        assert!(compute_reward_weight(performance, invalid_weights).is_err());
    }

    #[test]
    fn scarcity_matches_zinc_curve_shape_and_rounding() {
        let params = curve();
        let low = calculate_scarcity_cap(params, 1_000, 0).unwrap();
        let high = calculate_scarcity_cap(params, 10_000, 0).unwrap();
        let saturated = calculate_scarcity_cap(params, 1_000_000, 0).unwrap();

        assert_eq!(low, 95_162_582);
        assert_eq!(high, 632_120_559);
        assert!(high > low);
        assert_eq!(saturated, params.max_epoch_mint);
    }

    #[test]
    fn scarcity_decreases_as_lifetime_mint_grows() {
        let params = curve();
        let low_history = calculate_scarcity_cap(params, 10_000, 0).unwrap();
        let high_history = calculate_scarcity_cap(params, 10_000, params.history_minted).unwrap();

        assert_eq!(low_history, 632_120_559);
        assert_eq!(high_history, 316_060_279);
        assert!(high_history < low_history);
    }

    #[test]
    fn emission_multiplier_one_preserves_and_reduced_multiplier_scales_curve() {
        let full = curve();
        let full_cap = calculate_scarcity_cap(full, 10_000, 0).unwrap();
        assert_eq!(full_cap, 632_120_559);

        let reduced = CurveParams {
            emission_multiplier_ppm: 500_000,
            ..full
        };
        assert_eq!(
            calculate_scarcity_cap(reduced, 10_000, 0).unwrap(),
            316_060_279
        );
    }

    #[test]
    fn emission_multiplier_rejects_zero_and_above_one() {
        let mut params = curve();
        params.emission_multiplier_ppm = 0;
        assert!(params.validate().is_err());
        params.emission_multiplier_ppm = EMISSION_MULTIPLIER_PPM_SCALE + 1;
        assert!(params.validate().is_err());
    }

    #[test]
    fn scarcity_handles_boundaries_and_large_exponents() {
        let mut params = curve();
        assert_eq!(calculate_scarcity_cap(params, 0, 0).unwrap(), 0);
        assert_eq!(
            calculate_scarcity_cap(params, u64::MAX, 0).unwrap(),
            params.max_epoch_mint
        );

        params.max_epoch_mint = u64::MAX;
        params.saturation_units = 1;
        params.history_minted = u64::MAX;
        params.max_supply = u64::MAX;
        assert_eq!(
            calculate_scarcity_cap(params, u64::MAX, 0).unwrap(),
            u64::MAX
        );
        assert!(calculate_scarcity_cap(params, u64::MAX, u64::MAX).unwrap() > 0);
    }

    #[test]
    fn fixed_point_scarcity_matches_reference_samples() {
        let params = curve();
        for weight in [1, 10, 100, 1_000, 10_000, 100_000, u32::MAX as u64] {
            for minted in [0, 1, params.history_minted, params.max_supply - 1] {
                let actual = calculate_scarcity_cap(params, weight, minted).unwrap();
                let expected = reference_scarcity(params, weight, minted);
                assert!(
                    actual.abs_diff(expected) <= 1,
                    "weight={weight} minted={minted} actual={actual} expected={expected}"
                );
            }
        }
    }

    #[test]
    fn scarcity_regression_is_monotonic_at_extreme_adjacent_weights() {
        let params = CurveParams {
            max_epoch_mint: 10_726_446_547_357_273_219,
            emission_multiplier_ppm: 336_694,
            saturation_units: 11_966_486_233_823_956_054,
            history_minted: 2_385_622_954_990_824_523,
            target_support_lamports_per_token: 273_555_157_196_043_699,
            max_supply: 16_192_490_781_594_758_800,
        };
        let weight = 6_292_912_321_978_190_319;
        let minted = 12_869_844_569_894_213_994;
        let result = calculate_scarcity_cap(params, weight, minted).unwrap();
        let increased = calculate_scarcity_cap(params, weight + 1, minted).unwrap();
        assert!(increased >= result);
    }

    proptest! {
        #[test]
        fn scarcity_is_monotonic_and_close_to_reference(
            max_epoch_mint in 1_u64..1_000_000_000_000_000,
            saturation_units in 1_u64..1_000_000_000_000,
            history_minted in 1_u64..1_000_000_000_000_000,
            first_weight in 0_u64..1_000_000_000_000,
            extra_weight in 0_u64..1_000_000_000_000,
            first_minted in 0_u64..1_000_000_000_000_000,
            extra_minted in 0_u64..1_000_000_000_000_000,
        ) {
            let params = CurveParams {
                max_epoch_mint,
                emission_multiplier_ppm: EMISSION_MULTIPLIER_PPM_SCALE,
                saturation_units,
                history_minted,
                target_support_lamports_per_token: 1,
                max_supply: u64::MAX,
            };
            let second_weight = first_weight.saturating_add(extra_weight);
            let second_minted = first_minted.saturating_add(extra_minted);
            let base = calculate_scarcity_cap(params, first_weight, first_minted).unwrap();
            let more_work = calculate_scarcity_cap(params, second_weight, first_minted).unwrap();
            let more_history = calculate_scarcity_cap(params, first_weight, second_minted).unwrap();
            prop_assert!(more_work >= base);
            prop_assert!(more_history <= base);

            let reference = reference_scarcity(params, first_weight, first_minted);
            prop_assert!(base.abs_diff(reference) <= 2);
        }
    }

    fn reference_scarcity(curve: CurveParams, weight: u64, minted: u64) -> u64 {
        let work_ratio = weight as f64 / curve.saturation_units as f64;
        let history_ratio = minted as f64 / curve.history_minted as f64;
        (curve.max_epoch_mint as f64
            * (curve.emission_multiplier_ppm as f64 / EMISSION_MULTIPLIER_PPM_SCALE as f64)
            * (1.0 - (-work_ratio).exp())
            / (1.0 + history_ratio))
            .round() as u64
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
        assert_eq!(calculate_consumed_support(params, 1).unwrap(), 50);
    }

    #[test]
    fn reward_pool_caps_by_support_and_remaining_supply() {
        let mut params = curve();
        let abundant_support = 10_000 * params.target_support_lamports_per_token;
        let pool = compute_reward_pool(params, 10_000, 0, abundant_support).unwrap();

        assert_eq!(pool.scarcity_cap, 632_120_559);
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
