use anchor_lang::prelude::*;




#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, GlobalAccount>,
    pub system_program: Program<'info, System>,
}



pub fn withdraw(ctx:Context<Withdraw>,amount:u64) -> Result<()> {
    let user_acc = &mut ctx.accounts.user_account;
    require!(user_acc.deposit >= amount, ErrorCode::WithdrawTooLarge);
    user_acc.deposit = user_acc.deposit.checked_sub(amount).ok_or(ErrorCode::Overflow)?;

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.user_account.to_account_info(),
        to: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.system_program.key();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, amount)?;

    user_acc.deposit = user_acc.deposit.checked_sub(amount).ok_or(ErrorCode::Overflow)?;
    
    Ok(())
}