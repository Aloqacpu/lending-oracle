use crate::errors::ErrorCode;
use crate::price::{check_price_fresh, read_price_info};
use crate::state::{GlobalAccount, UserAccount};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

const LAMPORTS_PER_SOL: i128 = 1_000_000_000;

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
    #[account(mut)]
    pub owner: SystemAccount<'info>,
    #[account(mut, seeds = [b"user", owner.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    #[account(mut, token::mint = mint, token::authority = liquidator)]
    pub liquidator_token_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = mint, token::authority = config)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Address is checked against config.price_feed and data is validated by read_price_info.
    pub price_feed: UncheckedAccount<'info>,
}

pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.price_feed.key(),
        ctx.accounts.config.price_feed,
        ErrorCode::InvalidPriceFeed
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

    let deposit = ctx.accounts.user_account.deposit;
    let debt = ctx.accounts.user_account.credit;

    let collateral_value = (deposit as i128)
        .checked_mul(normalized_price)
        .and_then(|v| v.checked_div(LAMPORTS_PER_SOL))
        .ok_or(ErrorCode::Overflow)?;

    // ВАЖНО: порог ликвидации, а не ltv для займа — иначе буфера между
    // "можно занять" и "пора ликвидировать" не существует
    let liq_limit = collateral_value
        .checked_mul(ctx.accounts.config.liquidation as i128)
        .and_then(|v| v.checked_div(100))
        .ok_or(ErrorCode::Overflow)?;

    require!(debt as i128 > liq_limit, ErrorCode::PositionHealthy);

    // ликвидатор гасит весь долг протоколу
    let cpi_accounts = Transfer {
        from: ctx.accounts.liquidator_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.liquidator.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, debt)?;

    // и забирает весь SOL-залог как вознаграждение
    **ctx
        .accounts
        .user_account
        .to_account_info()
        .try_borrow_mut_lamports()? -= deposit;
    **ctx
        .accounts
        .liquidator
        .to_account_info()
        .try_borrow_mut_lamports()? += deposit;

    let user_acc = &mut ctx.accounts.user_account;
    user_acc.deposit = 0;
    user_acc.credit = 0;
    Ok(())
}
