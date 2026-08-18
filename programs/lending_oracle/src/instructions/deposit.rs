use crate::errors::ErrorCode;
use crate::state::GlobalAccount;
use crate::state::UserAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use crate::risk::require_not_paused;
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    pub system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require_not_paused(
        &ctx.accounts.config
    )?;

    let user_acc = &mut ctx.accounts.user_account;
    user_acc.collateral = user_acc
        .collateral
        .checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.user.to_account_info(),
        to: ctx.accounts.user_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.key();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, amount)?;
    Ok(())
}