use crate::errors::ErrorCode;

pub const BPS_DENOMINATOR: u128 = 10_000;
pub const WAD: u128 = 1_000_000_000_000_000_000;
pub const LAMPORTS_PER_SOL: u128 = 1_000_000_000;

/// Converts a Pyth price into 18-decimal fixed-point representation.
///
/// Example:
/// Pyth price = 150.25
/// result      = 150.25 * 1e18
pub fn normalize_price(price: i64, exponent: i32) -> std::result::Result<u128, ErrorCode> {
    if price <= 0 {
        return Err(ErrorCode::InvalidPrice);
    }

    let price = price as u128;

    let normalized = if exponent >= 0 {
        let multiplier = pow10(exponent as u32)?;
        price
            .checked_mul(multiplier)
            .ok_or(ErrorCode::MathOverflow)?
    } else {
        let divisor = pow10((-exponent) as u32)?;

        price
            .checked_mul(WAD)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(divisor)
            .ok_or(ErrorCode::MathOverflow)?
    };

    Ok(normalized)
}

/// Converts collateral amount in lamports into USD value
/// using an 18-decimal fixed-point price.
///
/// Result is also expressed with 18 decimals.
pub fn collateral_value_usd(
    collateral_lamports: u64,
    price_wad: u128,
) -> std::result::Result<u128, ErrorCode> {
    (collateral_lamports as u128)
        .checked_mul(price_wad)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(LAMPORTS_PER_SOL)
        .ok_or(ErrorCode::MathOverflow)
}

/// Calculates maximum borrow amount using LTV in basis points.
///
/// Example:
/// collateral = $1000
/// LTV = 70% = 7000 bps
/// max borrow = $700
pub fn max_borrow(
    collateral_value: u128,
    ltv_bps: u64,
) -> std::result::Result<u128, ErrorCode> {
    if ltv_bps as u128 > BPS_DENOMINATOR {
        return Err(ErrorCode::InvalidLtv);
    }

    collateral_value
        .checked_mul(ltv_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(ErrorCode::MathOverflow)
}

/// Calculates liquidation threshold in the same fixed-point units.
pub fn liquidation_limit(
    collateral_value: u128,
    liquidation_threshold_bps: u64,
) -> std::result::Result<u128, ErrorCode> {
    if liquidation_threshold_bps as u128 > BPS_DENOMINATOR {
        return Err(ErrorCode::InvalidLiquidationThreshold);
    }

    collateral_value
        .checked_mul(liquidation_threshold_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(ErrorCode::MathOverflow)
}

/// Returns health factor in WAD.
///
/// health factor:
///
///     collateral_value * liquidation_threshold
///     -----------------------------------------
///                    debt
///
/// HF >= 1e18 => healthy
/// HF <  1e18 => liquidatable
pub fn health_factor(
    collateral_value: u128,
    debt: u64,
    liquidation_threshold_bps: u64,
) -> std::result::Result<u128, ErrorCode> {
    if debt == 0 {
        return Ok(u128::MAX);
    }

    let liquidation_value =
        liquidation_limit(collateral_value, liquidation_threshold_bps)?;

    liquidation_value
        .checked_mul(WAD)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(debt as u128)
        .ok_or(ErrorCode::MathOverflow)
}

fn pow10(exponent: u32) -> std::result::Result<u128, ErrorCode> {
    10u128
        .checked_pow(exponent)
        .ok_or(ErrorCode::MathOverflow)
}

/// Calculates how much collateral must be seized
/// to cover a given debt repayment plus liquidation bonus.
///
/// collateral_seized =
///
///     debt_repaid * (1 + bonus)
///     ------------------------
///          collateral_price
pub fn liquidation_collateral(
    debt_repaid: u64,
    price_wad: u128,
    liquidation_bonus_bps: u64,
) -> std::result::Result<u64, ErrorCode> {
    if liquidation_bonus_bps as u128 > BPS_DENOMINATOR {
        return Err(ErrorCode::InvalidLiquidationBonus);
    }

    let bonus_multiplier = BPS_DENOMINATOR
        .checked_add(liquidation_bonus_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    let debt_with_bonus = (debt_repaid as u128)
        .checked_mul(bonus_multiplier)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(ErrorCode::MathOverflow)?;

    let collateral_lamports = debt_with_bonus
        .checked_mul(LAMPORTS_PER_SOL)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(price_wad)
        .ok_or(ErrorCode::MathOverflow)?;

    collateral_lamports
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)
}