use crate::errors::ErrorCode;
use crate::price::{check_price_fresh, read_price_info};
use crate::state::GlobalAccount;
use crate::state::UserAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

const LAMPORTS_PER_SOL: i128 = 1_000_000_000;

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    #[account(mut, token::mint = mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = mint, token::authority = config)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Address is checked against config.price_feed and data is validated by read_price_info.
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

    let collateral = ctx.accounts.user_account.deposit as i128;
    let current_credit = ctx.accounts.user_account.credit as i128;
    let new_debt = current_credit
        .checked_add(amount as i128)
        .ok_or(ErrorCode::Overflow)?;

    let collateral_value = collateral
        .checked_mul(normalized_price)
        .and_then(|v| v.checked_div(LAMPORTS_PER_SOL)) // тот же decimals-фикс
        .ok_or(ErrorCode::Overflow)?;

    let max_borrow = collateral_value
        .checked_mul(ctx.accounts.config.ltv as i128)
        .and_then(|v| v.checked_div(100))
        .ok_or(ErrorCode::Overflow)?;

    require!(new_debt <= max_borrow, ErrorCode::BorrowTooLarge);

    let bump = ctx.bumps.config;
    let config_seed: &[u8] = b"config";
    let bump_bytes = [bump];
    let signer_seeds: &[&[u8]] = &[config_seed, &bump_bytes];
    let binding = [signer_seeds];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info(); // было .key()
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &binding);
    token::transfer(cpi_ctx, amount)?;

    ctx.accounts.user_account.credit = new_debt as u64;
    Ok(())
}
