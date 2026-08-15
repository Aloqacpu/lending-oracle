pub mod errors;
pub mod price;
pub mod state;
pub mod instructions;
use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
use crate::instructions::{
    borrow::Borrow,
    deposit::Deposit,
    init::InitAccount,
    init_config::GlobalInitAccount,
    liquidate::Liquidate,
    repay::Repay,
    withdraw::Withdraw,
};

declare_id!("2fLc1vQc4LpYBByhVgBjRThwBxnLU4jwop89YUVZnRTT");

#[cfg(feature = "anchor")]
#[program]
pub mod lending_oracle {
    use super::*;
    use crate::instructions as ins;

    pub fn init_config(
        ctx: Context<GlobalInitAccount>,
        ltv: u64,
        liquidation: u64,
        price_feed: Pubkey,
    ) -> Result<()> {
        ins::init_config::init_config(ctx, ltv, liquidation, price_feed)
    }

    pub fn init_user_account(ctx: Context<InitAccount>) -> Result<()> {
        ins::init::init_user_account(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ins::deposit::deposit(ctx, amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        ins::borrow::borrow(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        ins::repay::repay(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ins::withdraw::withdraw(ctx, amount)
    }

    pub fn liquidate(ctx: Context<Liquidate>, amount: u64) -> Result<()> {
        ins::liquidate::liquidate(ctx, amount)
    }
}

// MAIN IDEA:
// this file just exposes all instructions and connects them to instruction modules
// borrow/withdraw/liquidate all use the same logic:
// 1) validate oracle
// 2) compute max allowed debt from collateral and price
// 3) require the condition to pass
// 4) mutate state / transfer tokens
