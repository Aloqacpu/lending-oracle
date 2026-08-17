use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub user: Pubkey,
    pub deposit: u64,
    pub credit: u64,
}
