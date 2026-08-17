pub mod errors;
pub mod instructions;
pub mod price;
pub mod state;

use anchor_lang::prelude::*;
pub use instructions::*;

declare_id!("2fLc1vQc4LpYBByhVgBjRThwBxnLU4jwop89YUVZnRTT");

#[program]
pub mod lending_oracle {
    use super::*;

    pub fn init_config(
        ctx: Context<GlobalInitAccount>,
        ltv: u64,
        liquidation: u64,
        price_feed: Pubkey,
    ) -> Result<()> {
        instructions::init_config::init_config(ctx, ltv, liquidation, price_feed)
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

    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        instructions::liquidate::liquidate(ctx)
    }
}
