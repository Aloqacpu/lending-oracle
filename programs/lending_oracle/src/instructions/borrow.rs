use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::price::read_price_info;
use crate::state::GlobalAccount;
use crate::state::UserAccount;

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    pub system_program: Program<'info, System>,
    #[account(mut, token::mint = mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = mint, token::authority = config)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub price_feed: UncheckedAccount<'info>,
}

pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require_keys_eq!(
        ctx.accounts.price_feed.key(),
        ctx.accounts.config.price_feed,
        ErrorCode::InvalidPriceFeed
    );

    let price = read_price_info(&ctx.accounts.price_feed)?;

    let sol_price = price.price as i128;
    let expo = price.expo;

    let user_acc = &mut ctx.accounts.user_account;
    let config = &ctx.accounts.config;

    let collateral = user_acc.deposit as i128;
    let ltv = config.ltv as i128;
    let current_credit = user_acc.credit as i128;

    let new_debt = current_credit
        .checked_add(amount as i128)
        .ok_or(ErrorCode::Overflow)?;

    let normalized_price = if expo < 0 {
        sol_price
            .checked_div(10i128.pow(expo.abs() as u32))
            .ok_or(ErrorCode::Overflow)?
    } else {
        sol_price
            .checked_mul(10i128.pow(expo as u32))
            .ok_or(ErrorCode::Overflow)?
    };

    let collateral_value = collateral
        .checked_mul(normalized_price)
        .ok_or(ErrorCode::Overflow)?;

    let max_borrow = collateral_value
        .checked_mul(ltv)
        .and_then(|value| value.checked_div(100i128))
        .ok_or(ErrorCode::Overflow)?;

    require!(new_debt <= max_borrow, ErrorCode::BorrowTooLarge);

    let bump = ctx.bumps.config;
    let config_seed = b"config";
    let bump_bytes = [bump];
    let signer_seeds = [&config_seed[..], &bump_bytes[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.key();
    let binding = [&signer_seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &binding);
    token::transfer(cpi_ctx, amount)?;

    user_acc.credit = new_debt as u64;

    Ok(())
}