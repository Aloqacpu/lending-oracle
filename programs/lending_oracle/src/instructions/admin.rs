use crate::{
    errors::ErrorCode,
    events::ProtocolPauseEvent,
    state::GlobalAccount,
};

use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        has_one = admin @ ErrorCode::Unauthorized
    )]
    pub config: Account<'info, GlobalAccount>,

    pub admin: Signer<'info>,
}

pub fn set_pause(
    ctx: Context<SetPause>,
    paused: bool,
) -> Result<()> {
    ctx.accounts.config.paused = paused;

    emit!(ProtocolPauseEvent {
        admin: ctx.accounts.admin.key(),
        paused,
    });

    Ok(())
}

pub fn set_risk_parameters(
    ctx: Context<SetPause>,
    ltv_bps: u64,
    liquidation_threshold_bps: u64,
    liquidation_bonus_bps: u64,
) -> Result<()> {
    require!(
        ltv_bps > 0
            && ltv_bps < liquidation_threshold_bps
            && liquidation_threshold_bps <= 10_000,
        ErrorCode::InvalidRiskParameters
    );

    require!(
        liquidation_bonus_bps <= 2_000,
        ErrorCode::InvalidLiquidationBonus
    );

    let config = &mut ctx.accounts.config;

    config.ltv_bps = ltv_bps;
    config.liquidation_threshold_bps = liquidation_threshold_bps;
    config.liquidation_bonus_bps = liquidation_bonus_bps;

    emit!(crate::events::RiskParametersUpdatedEvent {
        admin: ctx.accounts.admin.key(),
        ltv_bps,
        liquidation_threshold_bps,
        liquidation_bonus_bps,
    });

    Ok(())
}