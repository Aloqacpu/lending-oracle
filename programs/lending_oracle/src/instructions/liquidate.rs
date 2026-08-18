use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use crate::{
    errors::ErrorCode,
    math::liquidation_collateral,
    risk::{
        validate_liquidation,
        validate_price,
    },
    state::{
        GlobalAccount,
        UserAccount,
    },
};

use anchor_lang::prelude::*;

use anchor_spl::token::{
    self,
    Mint,
    Token,
    TokenAccount,
    Transfer,
};


#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(mut)]
    pub owner: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump,
        constraint = user_account.user == owner.key()
            @ ErrorCode::Unauthorized
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
        token::authority = liquidator
    )]
    pub liquidator_token_account: Account<'info, TokenAccount>,

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

pub fn liquidate(
    ctx: Context<Liquidate>,
    repay_amount: u64,
) -> Result<()> {
    require!(
        repay_amount > 0,
        ErrorCode::InvalidAmount
    );

    let price_wad = validate_price(
        &ctx.accounts.config,
        &ctx.accounts.price_update,
    )?;

    let user = &ctx.accounts.user_account;

    require!(
        user.debt > 0,
        ErrorCode::PositionNotLiquidatable
    );

    require!(
        repay_amount <= user.debt,
        ErrorCode::RepayTooLarge
    );

    validate_liquidation(
        &ctx.accounts.user_account,
        &ctx.accounts.config,
        price_wad,
    )?;

    let collateral_to_seize = liquidation_collateral(
        repay_amount,
        price_wad,
        ctx.accounts.config.liquidation_bonus_bps,
    )?;

    require!(
        collateral_to_seize <= user.collateral,
        ErrorCode::WithdrawTooLarge
    );

    require!(
        ctx.accounts.liquidator_token_account.amount >= repay_amount,
        ErrorCode::InsufficientLiquidity
    );

    let cpi_accounts = Transfer {
        from: ctx.accounts.liquidator_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.liquidator.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.key();

    let cpi_ctx = CpiContext::new(
        cpi_program,
        cpi_accounts,
    );

    token::transfer(
        cpi_ctx,
        repay_amount,
    )?;

    let new_debt = user
        .debt
        .checked_sub(repay_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    let new_collateral = user
        .collateral
        .checked_sub(collateral_to_seize)
        .ok_or(ErrorCode::MathOverflow)?;

    let user_account_info =
        ctx.accounts.user_account.to_account_info();

    let liquidator_info =
        ctx.accounts.liquidator.to_account_info();

    require!(
        user_account_info.lamports() >= collateral_to_seize,
        ErrorCode::WithdrawTooLarge
    );

    **user_account_info.try_borrow_mut_lamports()? -=
        collateral_to_seize;

    **liquidator_info.try_borrow_mut_lamports()? +=
        collateral_to_seize;

    let user_account =
        &mut ctx.accounts.user_account;

    user_account.debt = new_debt;
    user_account.collateral = new_collateral;


    emit!(crate::events::LiquidationEvent {
        liquidator: ctx.accounts.liquidator.key(),
        user: ctx.accounts.owner.key(),
        debt_repaid: repay_amount,
        collateral_seized: collateral_to_seize,
    });

    Ok(())
}