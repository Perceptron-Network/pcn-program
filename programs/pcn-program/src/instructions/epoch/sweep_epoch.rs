use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    error::PcnError, Config, Epoch, EpochStatus, CONFIG_SEED, EPOCH_SEED, MINT_AUTHORITY_SEED,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SweepEpochArgs {
    pub epoch_id: u64,
}

pub fn sweep_epoch(ctx: Context<SweepEpoch>, args: SweepEpochArgs) -> Result<()> {
    require!(
        args.epoch_id == ctx.accounts.epoch.epoch_id,
        PcnError::InvalidClaimAccount
    );
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
        Clock::get()?.slot > ctx.accounts.epoch.claim_deadline_slot,
        PcnError::ClaimWindowStillOpen
    );

    let amount = ctx.accounts.epoch_token_vault.amount;
    if amount > 0 {
        let mint_authority_seeds: &[&[u8]] = &[
            MINT_AUTHORITY_SEED,
            &[ctx.accounts.config.mint_authority_bump],
        ];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.epoch_token_vault.to_account_info(),
                    to: ctx.accounts.token_reserve_vault.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[mint_authority_seeds],
            ),
            amount,
        )?;
    }
    ctx.accounts.epoch.status = EpochStatus::Swept;
    Ok(())
}

#[derive(Accounts)]
#[instruction(args: SweepEpochArgs)]
pub struct SweepEpoch<'info> {
    pub oracle: Signer<'info>,
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
        address = epoch.epoch_token_vault,
        constraint = epoch_token_vault.mint == config.reward_mint @ PcnError::InvalidTokenAccount,
        constraint = epoch_token_vault.owner == mint_authority.key() @ PcnError::InvalidTokenAccount
    )]
    pub epoch_token_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = config.token_reserve_vault,
        constraint = token_reserve_vault.mint == config.reward_mint @ PcnError::InvalidTokenAccount,
        constraint = token_reserve_vault.owner == mint_authority.key() @ PcnError::InvalidTokenAccount
    )]
    pub token_reserve_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA signer only; no data is read.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}
