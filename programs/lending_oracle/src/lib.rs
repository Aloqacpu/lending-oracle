pub mod errors;
pub mod state;
pub mod instructions;
use anchor_lang::prelude::*;

declare_id!("3AoFCSHtMUhupmF4wBc9F8pY1J9doiFs2SMkWt5Mrj6y");

#[program]
pub mod lending_oracle {
    use anchor_lang::system_program::{transfer, Transfer};
    use super::*;

    pub fn deposit(ctx:Context<Deposit>,amount:u64) -> Result<()> {
        let cpi_accounts = Transfer{
            from:ctx.accounts.user.to_account_info(),
            to:ctx.accounts.vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.system_program.key();
        let cpi_ctx = CpiContext::new(cpi_program,cpi_accounts);

        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw(ctx:Context<Withdraw>,amount:u64) -> Result<()> {

        let bump = ctx.bumps.vault;
        let user_key = ctx.accounts.user.key();
        let seeds: &[&[u8]] = &[b"vault", user_key.as_ref(), &[bump]];
        let signer_seeds: &[&[&[u8]]] = &[seeds];

        let cpi_accounts = Transfer{
            from:ctx.accounts.vault.to_account_info(),
            to:ctx.accounts.user.to_account_info(),
        };
        let cpi_program =ctx.accounts.system_program.key();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_ctx,amount)?;
        Ok(())
    }
}


#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    user:Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

}
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    user:Signer<'info>,
    #[account(mut,seeds=[b"vault",user.key().as_ref()],bump)]
    vault:SystemAccount<'info>,
    system_program:Program<'info,System>,

}