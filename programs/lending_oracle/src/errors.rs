use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Borrow amount exceeds maximum allowed")]
    BorrowTooLarge,
}