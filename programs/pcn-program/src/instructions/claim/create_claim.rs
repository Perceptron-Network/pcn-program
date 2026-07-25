use anchor_lang::prelude::*;

use crate::{
    compute_claim_amount, compute_reward_weight, error::PcnError, Claim, Config, Epoch,
    EpochStatus, PerformanceMetrics, CLAIM_SEED, CONFIG_SEED, EPOCH_SEED,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateClaimArgs {
    pub epoch_id: u64,
    pub user: Pubkey,
    pub performance: PerformanceMetrics,
}

pub fn create_claim(ctx: Context<CreateClaim>, args: CreateClaimArgs) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.oracle.key(),
        ctx.accounts.config.oracle,
        PcnError::UnauthorizedOracle
    );
    require!(
        ctx.accounts.epoch.status == EpochStatus::Finalized,
        PcnError::EpochNotFinalized
    );
    require!(
        Clock::get()?.slot <= ctx.accounts.epoch.claim_deadline_slot,
        PcnError::ClaimDeadlinePassed
    );

    let reward_weight =
        compute_reward_weight(args.performance, ctx.accounts.epoch.performance_weights)?;
    let reward_amount = compute_claim_amount(
        ctx.accounts.epoch.reward_pool_amount,
        reward_weight,
        ctx.accounts.epoch.total_reward_weight,
    )?;
    let next_allocated = ctx
        .accounts
        .epoch
        .allocated_amount
        .checked_add(reward_amount)
        .ok_or(PcnError::MathOverflow)?;
    require!(
        next_allocated <= ctx.accounts.epoch.reward_pool_amount,
        PcnError::ClaimOverAllocation
    );

    let claim = &mut ctx.accounts.claim;
    claim.epoch = ctx.accounts.epoch.key();
    claim.epoch_id = args.epoch_id;
    claim.user = args.user;
    claim.performance = args.performance;
    claim.reward_weight = reward_weight;
    claim.reward_amount = reward_amount;
    claim.claimed = false;
    claim.bump = ctx.bumps.claim;
    ctx.accounts.epoch.allocated_amount = next_allocated;
    Ok(())
}

#[derive(Accounts)]
#[instruction(args: CreateClaimArgs)]
pub struct CreateClaim<'info> {
    pub oracle: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [EPOCH_SEED, args.epoch_id.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        init,
        payer = payer,
        seeds = [CLAIM_SEED, args.epoch_id.to_le_bytes().as_ref(), args.user.as_ref()],
        bump,
        space = Claim::LEN
    )]
    pub claim: Account<'info, Claim>,
    pub system_program: Program<'info, System>,
}
