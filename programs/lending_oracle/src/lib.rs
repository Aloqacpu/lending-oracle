pub mod errors;
pub mod risk;   
pub mod instructions;
pub mod math;
pub mod price;
pub mod state;
pub mod events;

use anchor_lang::prelude::*;
use instructions::*;
use state::*;
declare_id!("y3PH8jwx6wUBGwdEN7W6GdYZfDTA68ohQsBvNa9jJ9W");

#[program]
pub mod lending_oracle {
    use super::*;

    pub fn init_config(
        ctx: Context<GlobalInitAccount>,
        ltv_bps: u64,
        liquidation_threshold_bps: u64,
        liquidation_bonus_bps: u64,
        price_update: Pubkey,
        price_feed_id: [u8; 32],
        max_price_age: u64,
        max_confidence_bps: u64,
) -> Result<()> {
    instructions::init_config::init_config(
        ctx,
        ltv_bps,
        liquidation_threshold_bps,
        liquidation_bonus_bps,
        price_update,
        price_feed_id,
        max_price_age,
        max_confidence_bps,
    )
}

    pub fn init_user_account(ctx: Context<InitAccount>) -> Result<()> {
        instructions::init::init_user_account(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit(ctx, amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        instructions::borrow::borrow(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        instructions::repay::repay(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::withdraw(ctx, amount)
    }

    pub fn liquidate(
        ctx: Context<Liquidate>,
        repay_amount: u64,
    ) -> Result<()> {
        instructions::liquidate::liquidate(
            ctx,
            repay_amount,
        )
    }
    pub fn set_pause(
        ctx: Context<SetPause>,
        paused: bool,
    ) -> Result<()> {
        instructions::admin::set_pause(
            ctx,
            paused,
        )
    }
    pub fn set_risk_parameters(
        ctx: Context<SetPause>,
        ltv_bps: u64,
        liquidation_threshold_bps: u64,
        liquidation_bonus_bps: u64,
    ) -> Result<()> {
        instructions::admin::set_risk_parameters(
            ctx,
            ltv_bps,
            liquidation_threshold_bps,
            liquidation_bonus_bps,
        )
    }
}
