use anchor_lang::prelude::*;
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
}

pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    let user_acc = &mut ctx.accounts.user_account;
    let config = &ctx.accounts.config;

    let ltv = config.ltv as u128;
    let collateral = user_acc.deposit as u128;
    let current_credit = user_acc.credit as u128;
    let new_debt = current_credit + amount as u128;

    let max_borrow = collateral
        .checked_mul(ltv)
        .and_then(|value| value.checked_div(100u128))
        .ok_or(ErrorCode::Overflow)?;

    if new_debt > max_borrow {
        return Err(error!(ErrorCode::BorrowTooLarge));
    }

    user_acc.credit = user_acc.credit.checked_add(amount).ok_or(ErrorCode::Overflow)?;

    Ok(())
}