use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::state::GlobalAccount;
use crate::state::UserAccount;

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub user:Signer<'info>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    pub system_program: Program<'info, System>,
    #[account(mut,token::mint =mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut,token::mint =mint, token::authority = config)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> { // нужен оракул
    let user_acc = &mut ctx.accounts.user_account;
    let config = &ctx.accounts.config;
    let collateral = user_acc.deposit as u128;
    let ltv = config.ltv as u128;
    let current_credit = user_acc.credit as u128;
    let new_debt = current_credit
        .checked_add(amount as u128)
        .ok_or(ErrorCode::Overflow)?;

    let max_borrow = collateral
        .checked_mul(ltv)
        .and_then(|value| value.checked_div(100u128))
        .ok_or(ErrorCode::Overflow)?;

    if new_debt > max_borrow {
        return Err(error!(ErrorCode::BorrowTooLarge));
    }

    let bump_arr = [ctx.bumps.config];
    let seeds: &[&[u8]] = &[b"config", &bump_arr];
    let signer_seeds: &[&[&[u8]]] = &[seeds];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.key();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, amount)?;

    user_acc.credit = new_debt as u64;

    Ok(())
}