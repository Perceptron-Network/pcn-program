use anchor_lang::prelude::*;

#[error_code]
pub enum PcnError {
    #[msg("Only the configured admin may perform this action")]
    UnauthorizedAdmin,
    #[msg("Only the configured oracle may perform this action")]
    UnauthorizedOracle,
    #[msg("Invalid curve parameters")]
    InvalidCurveParams,
    #[msg("Invalid claim window")]
    InvalidClaimWindow,
    #[msg("Invalid epoch slot window")]
    InvalidEpochWindow,
    #[msg("Invalid support budget")]
    InvalidSupportBudget,
    #[msg("Epoch is not open")]
    EpochNotOpen,
    #[msg("Epoch is not finalized")]
    EpochNotFinalized,
    #[msg("Epoch claim deadline has passed")]
    ClaimDeadlinePassed,
    #[msg("Epoch claim deadline has not passed")]
    ClaimWindowStillOpen,
    #[msg("Total reward weight must be greater than zero")]
    ZeroTotalRewardWeight,
    #[msg("Quality factor exceeds 1.0x")]
    InvalidQualityFactor,
    #[msg("Reward pool is zero")]
    ZeroRewardPool,
    #[msg("Maximum curve supply is exhausted")]
    MaxSupplyExhausted,
    #[msg("Claim allocation exceeds epoch reward pool")]
    ClaimOverAllocation,
    #[msg("Claim already redeemed")]
    ClaimAlreadyRedeemed,
    #[msg("Claim account does not match epoch or user")]
    InvalidClaimAccount,
    #[msg("Reward mint or token account does not match config")]
    InvalidTokenAccount,
    #[msg("Arithmetic overflow or underflow")]
    MathOverflow,
}
