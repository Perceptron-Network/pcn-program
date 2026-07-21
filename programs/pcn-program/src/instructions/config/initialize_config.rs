use anchor_lang::{prelude::*, solana_program::program_option::COption};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    error::PcnError, Config, CurveParams, SolReserve, CONFIG_SEED, MINT_AUTHORITY_SEED,
    SOL_RESERVE_SEED, TOKEN_DECIMALS, TOKEN_RESERVE_SEED,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitializeConfigArgs {
    pub admin: Pubkey,
    pub oracle: Pubkey,
    pub claim_window_slots: u64,
    pub curve: CurveParams,
}

pub fn initialize_config(ctx: Context<InitializeConfig>, args: InitializeConfigArgs) -> Result<()> {
    args.curve.validate()?;
    require!(args.admin != Pubkey::default(), PcnError::UnauthorizedAdmin);
    require!(
        args.oracle != Pubkey::default(),
        PcnError::UnauthorizedOracle
    );
    require!(args.claim_window_slots > 0, PcnError::InvalidClaimWindow);
    require!(
        ctx.accounts.reward_mint.decimals == TOKEN_DECIMALS,
        PcnError::InvalidTokenAccount
    );
    require!(
        ctx.accounts.reward_mint.mint_authority == COption::Some(ctx.accounts.mint_authority.key()),
        PcnError::InvalidTokenAccount
    );

    let config = &mut ctx.accounts.config;
    config.admin = args.admin;
    config.oracle = args.oracle;
    config.reward_mint = ctx.accounts.reward_mint.key();
    config.mint_authority_bump = ctx.bumps.mint_authority;
    config.config_bump = ctx.bumps.config;
    config.sol_reserve = ctx.accounts.sol_reserve.key();
    config.sol_reserve_bump = ctx.bumps.sol_reserve;
    config.token_reserve_vault = ctx.accounts.token_reserve_vault.key();
    config.token_reserve_bump = ctx.bumps.token_reserve_vault;
    config.lifetime_curve_minted_amount = 0;
    config.claim_window_slots = args.claim_window_slots;
    config.curve = args.curve;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, seeds = [CONFIG_SEED], bump, space = Config::LEN)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = payer,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = mint_authority
    )]
    pub reward_mint: Account<'info, Mint>,
    /// CHECK: PDA signer only; no data is read.
    #[account(seeds = [MINT_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [SOL_RESERVE_SEED],
        bump,
        space = SolReserve::LEN
    )]
    pub sol_reserve: Account<'info, SolReserve>,
    #[account(
        init,
        payer = payer,
        seeds = [TOKEN_RESERVE_SEED],
        bump,
        token::mint = reward_mint,
        token::authority = mint_authority
    )]
    pub token_reserve_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
