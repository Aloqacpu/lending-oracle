use anchor_lang::prelude::*;
use crate::errors::ErrorCode;
use crate::price::read_price_info;
use crate::state::{GlobalAccount, UserAccount};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(mut)]
    pub owner: SystemAccount<'info>,

    #[account(mut, seeds = [b"user", owner.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,

    pub system_program: Program<'info, System>,
    pub price_feed: UncheckedAccount<'info>,
}

pub fn liquidate(ctx: Context<Liquidate>, _amount: u64) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.price_feed.key(),
        ctx.accounts.config.price_feed,
        ErrorCode::InvalidPriceFeed
    );

    let user_acc = &mut ctx.accounts.user_account;

    let price = read_price_info(&ctx.accounts.price_feed)?;

    let sol_price = price.price as i128;
    let expo = price.expo;

    let normalized_price = if expo < 0 {
        sol_price
            .checked_div(10i128.pow(expo.abs() as u32))
            .ok_or(ErrorCode::Overflow)?
    } else {
        sol_price
            .checked_mul(10i128.pow(expo as u32))
            .ok_or(ErrorCode::Overflow)?
    };

    let collateral_value = (user_acc.deposit as i128)
        .checked_mul(normalized_price)
        .ok_or(ErrorCode::Overflow)?;

    let max_borrow = collateral_value
        .checked_mul(ctx.accounts.config.ltv as i128)
        .and_then(|v| v.checked_div(100))
        .ok_or(ErrorCode::Overflow)?;

    if user_acc.credit as i128 <= max_borrow {
        return Err(error!(ErrorCode::PositionHealthy));
    }

    user_acc.deposit = 0;
    user_acc.credit = 0;

    Ok(())
}

// QUICK LOGIC:
// 1) read price
// 2) compute collateral_value = deposit * price
// 3) compute max_borrow = collateral_value * ltv / 100
// 4) if credit > max_borrow, position is unhealthy
// 5) liquidate by clearing deposit and credit
