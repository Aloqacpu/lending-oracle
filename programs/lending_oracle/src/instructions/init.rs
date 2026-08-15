use anchor_lang::prelude::*;
use crate::state::UserAccount;  


#[derive(Accounts)]
pub struct InitAccount<'info> {
    #[account(mut)]
    pub user:Signer<'info>,
    #[account(init,payer=user,seeds=[b"user",user.key().as_ref()],bump,space=8+UserAccount::INIT_SPACE)]
    pub user_account:Account<'info,UserAccount>,
    pub system_program:Program<'info,System>,
}


pub fn init_user_account(ctx: Context<InitAccount>) -> Result<()> {
    ctx.accounts.user_account.user = ctx.accounts.user.key();
    Ok(())
}