use crate::errors::ErrorCode;
use crate::price::{check_price_fresh, read_price_info};
use crate::state::{GlobalAccount, UserAccount};
use anchor_lang::prelude::*;

const LAMPORTS_PER_SOL: i128 = 1_000_000_000;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    pub system_program: Program<'info, System>,
    /// CHECK: Address is checked against config.price_feed and data is validated by read_price_info.
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
    check_price_fresh(&price)?;
    let normalized_price = if price.expo < 0 {
        (price.price as i128)
            .checked_div(10i128.pow(price.expo.unsigned_abs()))
            .ok_or(ErrorCode::Overflow)?
    } else {
        (price.price as i128)
            .checked_mul(10i128.pow(price.expo as u32))
            .ok_or(ErrorCode::Overflow)?
    };

    let collateral_after = ctx
        .accounts
        .user_account
        .deposit
        .checked_sub(amount)
        .ok_or(ErrorCode::WithdrawTooLarge)?;

    // lamports -> SOL, ИНАЧЕ collateral_value завышен в ~1e9 раз
    let collateral_value = (collateral_after as i128)
        .checked_mul(normalized_price)
        .and_then(|v| v.checked_div(LAMPORTS_PER_SOL))
        .ok_or(ErrorCode::Overflow)?;

    let max_borrow = collateral_value
        .checked_mul(ctx.accounts.config.ltv as i128)
        .and_then(|v| v.checked_div(100))
        .ok_or(ErrorCode::Overflow)?;

    require!(
        (ctx.accounts.user_account.credit as i128) <= max_borrow,
        ErrorCode::BorrowTooLarge
    );

    // user_account принадлежит нашей программе -> напрямую списываем лампорты,
    // system_program::transfer с таким source упадёт (owner mismatch)
    **ctx
        .accounts
        .user_account
        .to_account_info()
        .try_borrow_mut_lamports()? -= amount;
    **ctx
        .accounts
        .user
        .to_account_info()
        .try_borrow_mut_lamports()? += amount;

    ctx.accounts.user_account.deposit = collateral_after;
    Ok(())
}
