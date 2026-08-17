use anchor_lang::prelude::*;

pub const MAX_PRICE_AGE_SECONDS: i64 = 60;

#[derive(Clone, Copy, Debug)]
pub struct PriceInfo {
    pub price: i64,
    pub expo: i32,
    pub publish_time: i64,
}

pub fn read_price_info(account: &AccountInfo) -> Result<PriceInfo> {
    let data = account.data.borrow();

    if data.len() < 20 {
        return err!(crate::errors::ErrorCode::InvalidPrice);
    }

    let price = i64::from_le_bytes(
        data[0..8]
            .try_into()
            .map_err(|_| error!(crate::errors::ErrorCode::InvalidPrice))?,
    );

    let expo = i32::from_le_bytes(
        data[8..12]
            .try_into()
            .map_err(|_| error!(crate::errors::ErrorCode::InvalidPrice))?,
    );

    let publish_time = i64::from_le_bytes(
        data[12..20]
            .try_into()
            .map_err(|_| error!(crate::errors::ErrorCode::InvalidPrice))?,
    );

    Ok(PriceInfo { price, expo, publish_time })
}

pub fn check_price_fresh(info: &PriceInfo) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let age = now.saturating_sub(info.publish_time);
    require!(
        age >= 0 && age <= MAX_PRICE_AGE_SECONDS,
        crate::errors::ErrorCode::StalePrice
    );
    Ok(())
}