use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use crate::{
    errors::ErrorCode,
    risk::{
        require_not_paused,
        validate_price,
        validate_withdraw,
    },
    state::{GlobalAccount, UserAccount},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

    pub price_update: Account<'info, PriceUpdateV2>,
}

pub fn withdraw(
    ctx: Context<Withdraw>,
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

    validate_withdraw(
        &ctx.accounts.user_account,
        &ctx.accounts.config,
        price_wad,
        amount,
    )?;

    let collateral_after = ctx
        .accounts
        .user_account
        .collateral
        .checked_sub(amount)
        .ok_or(ErrorCode::WithdrawTooLarge)?;

    let user_account_info =
        ctx.accounts.user_account.to_account_info();

    let user_info =
        ctx.accounts.user.to_account_info();

    require!(
        user_account_info.lamports() >= amount,
        ErrorCode::WithdrawTooLarge
    );

    **user_account_info.try_borrow_mut_lamports()? -= amount;

    **user_info.try_borrow_mut_lamports()? += amount;

    ctx.accounts.user_account.collateral =
        collateral_after;
        
    emit!(crate::events::WithdrawEvent {
        user: ctx.accounts.user.key(),
        amount,
    });

    Ok(())
}