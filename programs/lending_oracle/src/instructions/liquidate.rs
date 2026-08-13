use anchor_lang::prelude::*;
use crate::errors::ErrorCode;
use crate::state::{GlobalAccount, UserAccount};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
    #[account(mut)]
    pub owner: SystemAccount<'info>,
    #[account(mut, seeds = [b"user", owner.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    pub system_program: Program<'info, System>,
}

pub fn liquidate(ctx: Context<Liquidate>, _amount: u64) -> Result<()> {
    let user_acc = &mut ctx.accounts.user_account;
    let config = &ctx.accounts.config;

    let ltv = config.ltv as u128;
    let collateral = user_acc.deposit as u128;
    let current_credit = user_acc.credit as u128;

    let max_borrow = collateral
        .checked_mul(ltv)
        .and_then(|value| value.checked_div(100u128))
        .ok_or(ErrorCode::Overflow)?;

    if current_credit <= max_borrow {
        return Err(error!(ErrorCode::PositionHealthy));
    }

    user_acc.deposit = 0;
    user_acc.credit = 0;

    Ok(())
}