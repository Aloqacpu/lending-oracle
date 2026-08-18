use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::risk::require_not_paused;
use crate::errors::ErrorCode;
use crate::state::{GlobalAccount, UserAccount};

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,

    #[account(mut, token::mint = mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut, token::mint = mint, token::authority = config)]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        ctx.accounts.user_account.debt >= amount,
        ErrorCode::RepayTooLarge
    );
    require_not_paused(
        &ctx.accounts.config
    )?;

    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.key();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;

    ctx.accounts.user_account.debt = ctx
        .accounts
        .user_account
        .debt
        .checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(())
}

// QUICK LOGIC:
// 1) amount must be > 0
// 2) user debt must be >= amount
// 3) transfer tokens from user ATA to vault ATA
// 4) reduce credit
// 5) no health check here, because repay is just debt reduction
