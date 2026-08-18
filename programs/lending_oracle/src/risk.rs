use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use crate::{
    errors::ErrorCode,
    math::{
        collateral_value_usd,
        health_factor,
        max_borrow,
        WAD,
    },
    price::{normalized_price, read_price_info},
    state::{GlobalAccount, UserAccount},
};
use anchor_lang::prelude::*;

pub fn validate_price(
    config: &GlobalAccount,
    price_update: &Account<PriceUpdateV2>,
) -> Result<u128> {
    require_keys_eq!(
        price_update.key(),
        config.price_update,
        ErrorCode::InvalidPriceFeed
    );

    let price_info = read_price_info(
        price_update,
        &config.price_feed_id,
        config.max_price_age,
    )?;

    normalized_price(&price_info)
}

pub fn calculate_collateral_value(
    user: &UserAccount,
    price_wad: u128,
) -> Result<u128> {
    Ok(collateral_value_usd(
        user.collateral,
        price_wad,
    )?)
}

pub fn calculate_max_borrow(
    user: &UserAccount,
    config: &GlobalAccount,
    price_wad: u128,
) -> Result<u128> {
    let collateral_value =
        calculate_collateral_value(user, price_wad)?;

    Ok(max_borrow(
        collateral_value,
        config.ltv_bps,
    )?)
}

pub fn calculate_health_factor(
    user: &UserAccount,
    config: &GlobalAccount,
    price_wad: u128,
) -> Result<u128> {
    let collateral_value =
        calculate_collateral_value(user, price_wad)?;

    Ok(health_factor(
        collateral_value,
        user.debt,
        config.liquidation_threshold_bps,
    )?)
}

pub fn is_healthy(
    user: &UserAccount,
    config: &GlobalAccount,
    price_wad: u128,
) -> Result<bool> {
    Ok(
        calculate_health_factor(
            user,
            config,
            price_wad,
        )? >= WAD
    )
}

pub fn validate_borrow(
    user: &UserAccount,
    config: &GlobalAccount,
    price_wad: u128,
    borrow_amount: u64,
) -> Result<()> {
    let max_borrow_amount =
        calculate_max_borrow(
            user,
            config,
            price_wad,
        )?;

    let new_debt = (user.debt as u128)
        .checked_add(borrow_amount as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    require!(
        new_debt <= max_borrow_amount,
        ErrorCode::BorrowTooLarge
    );

    Ok(())
}

pub fn validate_withdraw(
    user: &UserAccount,
    config: &GlobalAccount,
    price_wad: u128,
    withdraw_amount: u64,
) -> Result<()> {
    require!(
        withdraw_amount <= user.collateral,
        ErrorCode::WithdrawTooLarge
    );

    let collateral_after = user
        .collateral
        .checked_sub(withdraw_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    let collateral_value =
        collateral_value_usd(
            collateral_after,
            price_wad,
        )?;

    let max_borrow_after =
        max_borrow(
            collateral_value,
            config.ltv_bps,
        )?;

    require!(
        (user.debt as u128) <= max_borrow_after,
        ErrorCode::WithdrawalWouldMakePositionUnhealthy
    );

    Ok(())
}

pub fn validate_liquidation(
    user: &UserAccount,
    config: &GlobalAccount,
    price_wad: u128,
) -> Result<()> {
    require!(
        user.debt > 0,
        ErrorCode::PositionNotLiquidatable
    );

    let health_factor =
        calculate_health_factor(
            user,
            config,
            price_wad,
        )?;

    require!(
        health_factor < WAD,
        ErrorCode::PositionHealthy
    );

    Ok(())
}
pub fn require_not_paused(
    config: &GlobalAccount,
) -> Result<()> {
    require!(
        !config.paused,
        ErrorCode::ProtocolPaused
    );

    Ok(())
}