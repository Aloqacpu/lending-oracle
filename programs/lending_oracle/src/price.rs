use anchor_lang::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct PriceInfo {
    pub price: i64,
    pub expo: i32,
}

pub fn read_price_info(account: &AccountInfo) -> Result<PriceInfo> {
    let data = account.data.borrow();

    if data.len() < 12 {
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

    Ok(PriceInfo { price, expo })
}
