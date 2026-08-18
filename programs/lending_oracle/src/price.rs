use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use crate::{
    errors::ErrorCode,
    math::normalize_price,
};
use anchor_lang::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct PriceInfo {
    pub price: i64,
    pub confidence: u64,
    pub exponent: i32,
    pub publish_time: i64,
}

pub fn read_price_info(
    price_update: &PriceUpdateV2,
    feed_id: &[u8; 32],
    max_age: u64,
) -> Result<PriceInfo> {
    let clock = Clock::get()?;

    let price = price_update
        .get_price_no_older_than(
            &clock,
            max_age,
            feed_id,
        )
        .map_err(|_| error!(ErrorCode::StalePrice))?;

    require!(
        price.price > 0,
        ErrorCode::InvalidPrice
    );

    require!(
        price.publish_time <= clock.unix_timestamp,
        ErrorCode::FuturePrice
    );

    Ok(PriceInfo {
        price: price.price,
        confidence: price.conf,
        exponent: price.exponent,
        publish_time: price.publish_time,
    })
}

pub fn normalized_price(info: &PriceInfo) -> Result<u128> {
    Ok(normalize_price(
        info.price,
        info.exponent,
    )?)
}