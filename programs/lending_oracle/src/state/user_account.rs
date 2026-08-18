use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub user: Pubkey,

    /// Collateral deposited by the user in lamports.
    pub collateral: u64,

    /// Outstanding debt in the debt token's smallest unit.
    pub debt: u64,
}