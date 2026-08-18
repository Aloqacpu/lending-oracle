use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use crate::{
    errors::ErrorCode,
    risk::{
        require_not_paused,
        validate_borrow,
        validate_price,
    },
    state::{GlobalAccount, UserAccount},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        has_one = user,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, GlobalAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = config
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub price_update: Account<'info, PriceUpdateV2>,
}

pub fn borrow(
    ctx: Context<Borrow>,
    amount: u64,
) -> Result<()> {
    require!(
        amount > 0,
        ErrorCode::InvalidAmount
    );
    require_not_paused(
        &ctx.accounts.config
    )?;

    let price_wad = validate_price(
        &ctx.accounts.config,
        &ctx.accounts.price_update,
    )?;

    validate_borrow(
        &ctx.accounts.user_account,
        &ctx.accounts.config,
        price_wad,
        amount,
    )?;

    let new_debt = ctx
        .accounts
        .user_account
        .debt
        .checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;

    require!(
        ctx.accounts.vault_token_account.amount >= amount,
        ErrorCode::InsufficientLiquidity
    );

    let bump = ctx.bumps.config;

    let signer_seeds: &[&[u8]] = &[
        b"config",
        &[bump],
    ];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };

    let signer_seeds_binding = [signer_seeds];

    let cpi_program = ctx.accounts.token_program.key();
    let cpi_ctx = CpiContext::new_with_signer(
        cpi_program,
        cpi_accounts,
        &signer_seeds_binding,
    );

    token::transfer(
        cpi_ctx,
        amount,
    )?;

    ctx.accounts.user_account.debt = new_debt;
    emit!(crate::events::BorrowEvent {
        user: ctx.accounts.user.key(),
        amount,
        new_debt,
    });

    Ok(())

}