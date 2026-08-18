use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Arithmetic overflow")]
    MathOverflow,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Invalid price feed")]
    InvalidPriceFeed,

    #[msg("Invalid price")]
    InvalidPrice,

    #[msg("Price feed data is stale")]
    StalePrice,

    #[msg("Price feed publish time is in the future")]
    FuturePrice,

    #[msg("Price confidence interval is too wide")]
    PriceConfidenceTooWide,

    #[msg("Invalid LTV")]
    InvalidLtv,

    #[msg("Invalid liquidation threshold")]
    InvalidLiquidationThreshold,
    #[msg("Invalid liquidation bonus")]
    InvalidLiquidationBonus,

    #[msg("LTV must be lower than liquidation threshold")]
    InvalidRiskParameters,

    #[msg("Borrow amount exceeds maximum allowed")]
    BorrowTooLarge,

    #[msg("Position is still healthy and cannot be liquidated")]
    PositionHealthy,

    #[msg("Position is not liquidatable")]
    PositionNotLiquidatable,

    #[msg("Withdraw amount exceeds deposited collateral")]
    WithdrawTooLarge,

    #[msg("Withdrawal would make the position unhealthy")]
    WithdrawalWouldMakePositionUnhealthy,

    #[msg("Repay amount exceeds borrowed amount")]
    RepayTooLarge,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Insufficient protocol liquidity")]
    InsufficientLiquidity,

    #[msg("Invalid collateral account")]
    InvalidCollateralAccount,

    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("Protocol is paused")]
    ProtocolPaused,
    
}