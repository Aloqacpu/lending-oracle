use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Borrow amount exceeds maximum allowed")]
    BorrowTooLarge,
    #[msg("Position is still healthy and cannot be liquidated")]
    PositionHealthy,
}