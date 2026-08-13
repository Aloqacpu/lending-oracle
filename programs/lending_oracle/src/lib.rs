pub mod errors;
pub mod state;
pub mod instructions;
use anchor_lang::prelude::*;

declare_id!("3AoFCSHtMUhupmF4wBc9F8pY1J9doiFs2SMkWt5Mrj6y");

#[program]
pub mod lending_oracle {
    use super::*;

    pub fn init_config(
        ctx: Context<crate::instructions::init_config::InitAccount>,
        ltv: u64,
        liquidation: u64,
        price_feed: Pubkey,
    ) -> Result<()> {
        crate::instructions::init_config::init_config(ctx, ltv, liquidation, price_feed)
    }

    pub fn init_user_account(ctx: Context<crate::instructions::init::InitAccount>) -> Result<()> {
        crate::instructions::init::init_user_account(ctx)
    }

    pub fn deposit(ctx: Context<crate::instructions::deposit::Deposit>, amount: u64) -> Result<()> {
        crate::instructions::deposit::deposit(ctx, amount)
    }

    pub fn borrow(ctx: Context<crate::instructions::borrow::Borrow>, amount: u64) -> Result<()> {
        crate::instructions::borrow::borrow(ctx, amount)
    }

    pub fn repay(ctx: Context<crate::instructions::repay::Repay>, amount: u64) -> Result<()> {
        crate::instructions::repay::repay(ctx, amount)
    }

    pub fn liquidate(ctx: Context<crate::instructions::liquidate::Liquidate>, amount: u64) -> Result<()> {
        crate::instructions::liquidate::liquidate(ctx, amount)
    }
}