use anchor_lang::prelude::*;
use crate::errors::ErrorCode;
use crate::price::read_price_info;
use crate::state::{GlobalAccount, UserAccount};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,

    pub system_program: Program<'info, System>,
    pub price_feed: UncheckedAccount<'info>,
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require_keys_eq!(
        ctx.accounts.price_feed.key(),
        ctx.accounts.config.price_feed,
        ErrorCode::InvalidPriceFeed
    );

    require!(
        ctx.accounts.user_account.deposit >= amount,
        ErrorCode::WithdrawTooLarge
    );

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

    let collateral_after = ctx
        .accounts
        .user_account
        .deposit
        .checked_sub(amount)
        .ok_or(ErrorCode::WithdrawTooLarge)?;

    let collateral_value = (collateral_after as i128)
        .checked_mul(normalized_price)
        .ok_or(ErrorCode::Overflow)?;

    let max_borrow = collateral_value
        .checked_mul(ctx.accounts.config.ltv as i128)
        .and_then(|v| v.checked_div(100))
        .ok_or(ErrorCode::Overflow)?;

    require!(
        ctx.accounts.user_account.credit as i128 <= max_borrow,
        ErrorCode::BorrowTooLarge
    );

    let bump = ctx.bumps.user_account;
    let user_key = ctx.accounts.user.key();
    let seeds = &[b"user", user_key.as_ref(), &[bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.user_account.to_account_info(),
        to: ctx.accounts.user.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        cpi_accounts,
        signer_seeds,
    );

    anchor_lang::system_program::transfer(cpi_ctx, amount)?;

    let user_acc = &mut ctx.accounts.user_account;
    user_acc.deposit = collateral_after;

    Ok(())
}

// QUICK LOGIC:
// 1) validate amount
// 2) validate price feed
// 3) ensure user has enough deposit
// 4) compute collateral_after
// 5) compute max borrow after withdrawal
// 6) require credit <= max_borrow
// 7) transfer funds from PDA to user
// 8) update deposit
