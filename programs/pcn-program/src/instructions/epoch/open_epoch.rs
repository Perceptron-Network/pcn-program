use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    error::PcnError, Config, Epoch, EpochStatus, CONFIG_SEED, EPOCH_SEED, EPOCH_VAULT_SEED,
    MINT_AUTHORITY_SEED,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenEpochArgs {
    pub epoch_id: u64,
    pub start_slot: u64,
    pub end_slot: u64,
    pub support_budget_lamports: u64,
}

pub fn open_epoch(ctx: Context<OpenEpoch>, args: OpenEpochArgs) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.oracle.key(),
        ctx.accounts.config.oracle,
        PcnError::UnauthorizedOracle
    );
    require!(
        args.start_slot < args.end_slot,
        PcnError::InvalidEpochWindow
    );
    require!(
        args.support_budget_lamports > 0,
        PcnError::InvalidSupportBudget
    );

    invoke(
        &system_instruction::transfer(
            &ctx.accounts.funder.key(),
            &ctx.accounts.epoch.key(),
            args.support_budget_lamports,
        ),
        &[
            ctx.accounts.funder.to_account_info(),
            ctx.accounts.epoch.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    let epoch = &mut ctx.accounts.epoch;
    epoch.epoch_id = args.epoch_id;
    epoch.status = EpochStatus::Open;
    epoch.start_slot = args.start_slot;
    epoch.end_slot = args.end_slot;
    epoch.support_budget_lamports = args.support_budget_lamports;
    epoch.consumed_support_lamports = 0;
    epoch.total_reward_weight = 0;
    epoch.reward_pool_amount = 0;
    epoch.allocated_amount = 0;
    epoch.claimed_amount = 0;
    epoch.claim_deadline_slot = 0;
    epoch.epoch_token_vault = ctx.accounts.epoch_token_vault.key();
    epoch.epoch_vault_bump = ctx.bumps.epoch_token_vault;
    epoch.bump = ctx.bumps.epoch;
    Ok(())
}

#[derive(Accounts)]
#[instruction(args: OpenEpochArgs)]
pub struct OpenEpoch<'info> {
    pub oracle: Signer<'info>,
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = funder,
        seeds = [EPOCH_SEED, args.epoch_id.to_le_bytes().as_ref()],
        bump,
        space = Epoch::LEN
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        init,
        payer = funder,
        seeds = [EPOCH_VAULT_SEED, args.epoch_id.to_le_bytes().as_ref()],
        bump,
        token::mint = reward_mint,
        token::authority = mint_authority
    )]
    pub epoch_token_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA signer only; no data is read.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(address = config.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
