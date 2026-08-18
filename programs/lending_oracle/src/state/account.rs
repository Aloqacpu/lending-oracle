use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalAccount {
    pub admin: Pubkey,
    pub liquidation_bonus_bps: u64,
    /// Pyth PriceUpdateV2 account.
    pub price_update: Pubkey,

    /// Pyth feed ID.
    pub price_feed_id: [u8; 32],

    /// Loan-to-value ratio in basis points.
    ///
    /// 70% = 7000.
    pub ltv_bps: u64,

    /// Liquidation threshold in basis points.
    ///
    /// 
    /// 80% = 8000.
    pub liquidation_threshold_bps: u64,

    pub paused: bool,
    /// Maximum accepted oracle age.
    pub max_price_age: u64,

    /// Maximum accepted confidence ratio in basis points.
    ///
    /// Example:
    /// 100 bps = 1%.
    pub max_confidence_bps: u64,
}