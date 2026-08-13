use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::state::GlobalAccount;
use crate::state::UserAccount;


#[derive(Accounts)]
pub struct Repay<'info> {
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
}

pub fn repay(ctx: Context<Repay>,amount:u64) -> Result<()> {
    
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(ctx.accounts.user_account.credit >= amount, ErrorCode::RepayTooLarge);



    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.key();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    ctx.accounts.user_account.credit = ctx.accounts.user_account.credit.checked_sub(amount).ok_or(ErrorCode::Overflow)?;
    if ctx.accounts.user_account.credit == 0 {
        ctx.accounts.user_account.deposit = 0;
    } 

    Ok(())
}