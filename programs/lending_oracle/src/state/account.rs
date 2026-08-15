use anchor_lang::prelude::*;
use anchor_lang::prelude::Pubkey;
#[account]
#[derive(InitSpace)]
pub struct GlobalAccount {
    pub admin:Pubkey,
    pub price_feed:Pubkey,
    pub ltv:u64,
    pub liquidation:u64
}


