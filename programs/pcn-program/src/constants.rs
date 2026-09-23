use anchor_lang::prelude::*;

#[constant]
pub const TOKEN_DECIMALS: u8 = 6;
pub const TOKEN_BASE_UNITS: u64 = 1_000_000;
pub const PERFORMANCE_PPM_SCALE: u64 = 1_000_000;
pub const EMISSION_MULTIPLIER_PPM_SCALE: u64 = 1_000_000;

pub const CONFIG_SEED: &[u8] = b"config";
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";
pub const SOL_RESERVE_SEED: &[u8] = b"sol_reserve";
pub const TOKEN_RESERVE_SEED: &[u8] = b"token_reserve";
pub const EPOCH_SEED: &[u8] = b"epoch";
pub const EPOCH_VAULT_SEED: &[u8] = b"epoch_vault";
pub const CLAIM_SEED: &[u8] = b"claim";
