use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    error::PcnError, Claim, Config, Epoch, EpochStatus, CLAIM_SEED, CONFIG_SEED, EPOCH_SEED,
    MINT_AUTHORITY_SEED,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimRewardArgs {
    pub epoch_id: u64,
}

pub fn claim_reward(ctx: Context<ClaimReward>, args: ClaimRewardArgs) -> Result<()> {
    require!(
        args.epoch_id == ctx.accounts.epoch.epoch_id,
        PcnError::InvalidClaimAccount
    );
    require!(
        ctx.accounts.epoch.status == EpochStatus::Finalized,
        PcnError::EpochNotFinalized
    );
    require!(!ctx.accounts.claim.claimed, PcnError::ClaimAlreadyRedeemed);
    require_keys_eq!(
        ctx.accounts.claim.epoch,
        ctx.accounts.epoch.key(),
        PcnError::InvalidClaimAccount
    );
    require_keys_eq!(
        ctx.accounts.claim.user,
        ctx.accounts.user.key(),
        PcnError::InvalidClaimAccount
    );

    let next_claimed = ctx
        .accounts
        .epoch
        .claimed_amount
        .checked_add(ctx.accounts.claim.reward_amount)
        .ok_or(PcnError::MathOverflow)?;
    require!(
        next_claimed <= ctx.accounts.epoch.reward_pool_amount,
        PcnError::ClaimOverAllocation
    );
    require!(
        ctx.accounts.epoch_token_vault.amount >= ctx.accounts.claim.reward_amount,
        PcnError::ClaimOverAllocation
    );

    if ctx.accounts.claim.reward_amount > 0 {
        let mint_authority_seeds: &[&[u8]] = &[
            MINT_AUTHORITY_SEED,
            &[ctx.accounts.config.mint_authority_bump],
        ];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.epoch_token_vault.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[mint_authority_seeds],
            ),
            ctx.accounts.claim.reward_amount,
        )?;
    }

    ctx.accounts.claim.claimed = true;
    ctx.accounts.epoch.claimed_amount = next_claimed;
    Ok(())
}

#[derive(Accounts)]
#[instruction(args: ClaimRewardArgs)]
pub struct ClaimReward<'info> {
    pub user: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [EPOCH_SEED, args.epoch_id.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [CLAIM_SEED, args.epoch_id.to_le_bytes().as_ref(), user.key().as_ref()],
        bump = claim.bump
    )]
    pub claim: Account<'info, Claim>,
    #[account(
        mut,
        address = epoch.epoch_token_vault,
        constraint = epoch_token_vault.mint == config.reward_mint @ PcnError::InvalidTokenAccount,
        constraint = epoch_token_vault.owner == mint_authority.key() @ PcnError::InvalidTokenAccount
    )]
    pub epoch_token_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_token_account.mint == config.reward_mint @ PcnError::InvalidTokenAccount,
        constraint = user_token_account.owner == user.key() @ PcnError::InvalidTokenAccount
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    /// CHECK: PDA signer only; no data is read.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}
