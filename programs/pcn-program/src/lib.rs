#![allow(ambiguous_glob_reexports)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod rewards;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use rewards::*;
pub use state::*;

declare_id!("FzHRzKNFB7Mck5FHj2MXUaywAgtB2EA2EeEQEkp59Xfo");

#[program]
pub mod pcn_program {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        args: InitializeConfigArgs,
    ) -> Result<()> {
        instructions::initialize_config(ctx, args)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, args: UpdateConfigArgs) -> Result<()> {
        instructions::update_config(ctx, args)
    }

    pub fn open_epoch(ctx: Context<OpenEpoch>, args: OpenEpochArgs) -> Result<()> {
        instructions::open_epoch(ctx, args)
    }

    pub fn finalize_epoch(ctx: Context<FinalizeEpoch>, args: FinalizeEpochArgs) -> Result<()> {
        instructions::finalize_epoch(ctx, args)
    }

    pub fn create_claim(ctx: Context<CreateClaim>, args: CreateClaimArgs) -> Result<()> {
        instructions::create_claim(ctx, args)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, args: ClaimRewardArgs) -> Result<()> {
        instructions::claim_reward(ctx, args)
    }

    pub fn sweep_epoch(ctx: Context<SweepEpoch>, args: SweepEpochArgs) -> Result<()> {
        instructions::sweep_epoch(ctx, args)
    }
}
