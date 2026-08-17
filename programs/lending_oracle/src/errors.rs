use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Borrow amount exceeds maximum allowed")]
    BorrowTooLarge,
    #[msg("Position is still healthy and cannot be liquidated")]
    PositionHealthy,
    #[msg("Withdraw amount exceeds deposited amount")]
    WithdrawTooLarge,
    #[msg("Repay amount exceeds borrowed amount")]
    RepayTooLarge,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid price feed")]
    InvalidPriceFeed,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Price feed data is stale")]
    StalePrice,
    
}