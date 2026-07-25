use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::{
    compute_reward_pool, error::PcnError, Config, Epoch, EpochStatus, SolReserve, CONFIG_SEED,
    EPOCH_SEED, MINT_AUTHORITY_SEED, SOL_RESERVE_SEED,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalizeEpochArgs {
    pub total_reward_weight: u64,
}

pub fn finalize_epoch(ctx: Context<FinalizeEpoch>, args: FinalizeEpochArgs) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.oracle.key(),
        ctx.accounts.config.oracle,
        PcnError::UnauthorizedOracle
    );
    require!(
        ctx.accounts.epoch.status == EpochStatus::Open,
        PcnError::EpochNotOpen
    );
    let current_slot = Clock::get()?.slot;
    require!(
        current_slot >= ctx.accounts.epoch.end_slot,
        PcnError::InvalidEpochWindow
    );

    let amounts = compute_reward_pool(
        ctx.accounts.config.curve,
        args.total_reward_weight,
        ctx.accounts.config.lifetime_curve_minted_amount,
        ctx.accounts.epoch.support_budget_lamports,
    )?;

    move_lamports(
        &ctx.accounts.epoch.to_account_info(),
        &ctx.accounts.sol_reserve.to_account_info(),
        amounts.consumed_support_lamports,
    )?;
    let refund_amount = ctx
        .accounts
        .epoch
        .support_budget_lamports
        .checked_sub(amounts.consumed_support_lamports)
        .ok_or(PcnError::MathOverflow)?;
    move_lamports(
        &ctx.accounts.epoch.to_account_info(),
        &ctx.accounts.support_funder.to_account_info(),
        refund_amount,
    )?;

    let mint_authority_seeds: &[&[u8]] = &[
        MINT_AUTHORITY_SEED,
        &[ctx.accounts.config.mint_authority_bump],
    ];
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            MintTo {
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx.accounts.epoch_token_vault.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            &[mint_authority_seeds],
        ),
        amounts.reward_pool,
    )?;

    ctx.accounts.config.lifetime_curve_minted_amount = ctx
        .accounts
        .config
        .lifetime_curve_minted_amount
        .checked_add(amounts.reward_pool)
        .ok_or(PcnError::MathOverflow)?;

    let epoch = &mut ctx.accounts.epoch;
    epoch.status = EpochStatus::Finalized;
    epoch.total_reward_weight = args.total_reward_weight;
    epoch.reward_pool_amount = amounts.reward_pool;
    epoch.consumed_support_lamports = amounts.consumed_support_lamports;
    epoch.claim_deadline_slot = current_slot
        .checked_add(ctx.accounts.config.claim_window_slots)
        .ok_or(PcnError::MathOverflow)?;
    Ok(())
}

#[derive(Accounts)]
pub struct FinalizeEpoch<'info> {
    pub oracle: Signer<'info>,
    #[account(mut, seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [EPOCH_SEED, epoch.epoch_id.to_le_bytes().as_ref()],
        bump = epoch.bump,
        has_one = epoch_token_vault
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        address = epoch.support_funder @ PcnError::InvalidRefundTarget
    )]
    pub support_funder: SystemAccount<'info>,
    #[account(
        mut,
        address = epoch.epoch_token_vault,
        constraint = epoch_token_vault.mint == config.reward_mint @ PcnError::InvalidTokenAccount,
        constraint = epoch_token_vault.owner == mint_authority.key() @ PcnError::InvalidTokenAccount
    )]
    pub epoch_token_vault: Account<'info, TokenAccount>,
    #[account(mut, address = config.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    /// CHECK: PDA signer only; no data is read.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [SOL_RESERVE_SEED],
        bump = config.sol_reserve_bump,
        address = config.sol_reserve
    )]
    pub sol_reserve: Account<'info, SolReserve>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

fn move_lamports<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    **from.try_borrow_mut_lamports()? = from
        .lamports()
        .checked_sub(amount)
        .ok_or(PcnError::MathOverflow)?;
    **to.try_borrow_mut_lamports()? = to
        .lamports()
        .checked_add(amount)
        .ok_or(PcnError::MathOverflow)?;
    Ok(())
}
