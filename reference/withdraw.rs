use anchor_lang::prelude::*;
use crate::errors::ErrorCode;
use crate::state::{GlobalAccount, UserAccount};
use pyth_sdk_solana::load_price_feed_from_account_info;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,

    pub system_program: Program<'info, System>,
    pub price_feed: AccountInfo<'info>,
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // 1) запрещаем нулевой вывод
    require!(amount > 0, ErrorCode::InvalidAmount);

    // 2) проверяем, что feed вообще тот, что в config
    require_keys_eq!(
        ctx.accounts.price_feed.key(),
        ctx.accounts.config.price_feed,
        ErrorCode::InvalidPriceFeed
    );

    let user_acc = &mut ctx.accounts.user_account;

    // 3) проверяем, что пользователь реально has enough deposit
    require!(user_acc.deposit >= amount, ErrorCode::WithdrawTooLarge);

    // 4) получаем цену из oracle
    let price_feed = load_price_feed_from_account_info(&ctx.accounts.price_feed)?;
    let price = price_feed.get_current_price().ok_or(ErrorCode::InvalidPrice)?;

    let sol_price = price.price as i128;
    let expo = price.expo;

    // 5) нормализуем цену по expo
    let normalized_price = if expo < 0 {
        sol_price
            .checked_div(10i128.pow(expo.abs() as u32))
            .ok_or(ErrorCode::Overflow)?
    } else {
        sol_price
            .checked_mul(10i128.pow(expo as u32))
            .ok_or(ErrorCode::Overflow)?
    };

    // 6) считаем, сколько будет deposit после вывода
    let collateral_after = user_acc
        .deposit
        .checked_sub(amount)
        .ok_or(ErrorCode::WithdrawTooLarge)?;

    // 7) считаем value после вывода
    let collateral_value = (collateral_after as i128)
        .checked_mul(normalized_price)
        .ok_or(ErrorCode::Overflow)?;

    // 8) получаем ltv из config
    let ltv = ctx.accounts.config.ltv as i128;

    // 9) считаем максимальный long debt, который ещё можно держать
    let max_borrow = collateral_value
        .checked_mul(ltv)
        .and_then(|v| v.checked_div(100))
        .ok_or(ErrorCode::Overflow)?;

    // 10) главное условие: после снятия долг не должен быть больше максимума
    require!(
        user_acc.credit as i128 <= max_borrow,
        ErrorCode::BorrowTooLarge
    );

    // 11) переводим деньги пользователю
    let bump = ctx.bumps.user_account;
    let signer_seeds: &[&[u8]] = &[b"user", ctx.accounts.user.key.as_ref(), &[bump]];

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.user_account.to_account_info(),
        to: ctx.accounts.user.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        cpi_accounts,
        &[signer_seeds],
    );

    anchor_lang::system_program::transfer(cpi_ctx, amount)?;

    // 12) только после успешного перевода обновляем state
    user_acc.deposit = collateral_after;

    Ok(())
}