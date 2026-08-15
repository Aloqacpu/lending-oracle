use anchor_lang::prelude::*;
use anchor_lang::prelude::Pubkey;
#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub user:Pubkey,
    pub deposit:u64,
    pub credit:u64,

}
