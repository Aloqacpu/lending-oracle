
use anchor_lang::prelude::*;
use crate::state::GlobalAccount;


#[derive(Accounts)]
pub struct GlobalInitAccount<'info> {
    #[account(mut)]
    pub admin:Signer<'info>,
    #[account(init,payer=admin,seeds=[b"config"],bump,space=8+GlobalAccount::INIT_SPACE)]
    pub config:Account<'info,GlobalAccount>,
    pub system_program:Program<'info,System>,

}

pub fn init_config(ctx: Context<GlobalInitAccount>, ltv: u64, liquidation: u64, price_feed: Pubkey) -> Result<()> {
    ctx.accounts.config.admin = ctx.accounts.admin.key();
    ctx.accounts.config.ltv = ltv;
    ctx.accounts.config.liquidation = liquidation;
    ctx.accounts.config.price_feed = price_feed;
    Ok(())
}