use anchor_lang::prelude::*;

use crate::{error::PcnError, Config, CurveParams, CONFIG_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateConfigArgs {
    pub oracle: Option<Pubkey>,
    pub claim_window_slots: Option<u64>,
    pub curve: Option<CurveParams>,
}

pub fn update_config(ctx: Context<UpdateConfig>, args: UpdateConfigArgs) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.admin.key(),
        ctx.accounts.config.admin,
        PcnError::UnauthorizedAdmin
    );

    if let Some(oracle) = args.oracle {
        require!(oracle != Pubkey::default(), PcnError::UnauthorizedOracle);
        ctx.accounts.config.oracle = oracle;
    }
    if let Some(claim_window_slots) = args.claim_window_slots {
        require!(claim_window_slots > 0, PcnError::InvalidClaimWindow);
        ctx.accounts.config.claim_window_slots = claim_window_slots;
    }
    if let Some(curve) = args.curve {
        curve.validate()?;
        require!(
            curve.max_supply >= ctx.accounts.config.lifetime_curve_minted_amount,
            PcnError::MaxSupplyExhausted
        );
        ctx.accounts.config.curve = curve;
    }
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, Config>,
}
