#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "LiteSVM tests use direct unwrap and focused assertions"
)]

#[path = "litesvm/claim_validation.rs"]
mod claim_validation;
#[path = "litesvm/config_authority.rs"]
mod config_authority;
#[path = "litesvm/epoch_validation.rs"]
mod epoch_validation;
#[path = "litesvm/rewards_flow.rs"]
mod rewards_flow;
