use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
declare_id!("86nmyVnDib9yccgyUMkxY47Cre15mdzW3Anhz9uEtkGz");

#[program]
pub mod vaults {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>)-> Result<()>{
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<DepositeAndWithdraw>, amount: u64)-> Result<()>{
        ctx.accounts.deposit(amount)
    } 

    pub fn withdraw(ctx: Context<DepositeAndWithdraw>, amount: u64)-> Result<()>{
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>)-> Result<()>{
        ctx.accounts.close_vault()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer=user,
        space= 8 + VaultState::INIT_SPACE,
        seeds=[b"state", user.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds=[b"vault", vault_state.key().as_ref()],
        bump, 
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositeAndWithdraw<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds=[b"vault", vault_state.key().as_ref()],
        bump= vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds=[b"state", user.key().as_ref()],
        bump= vault_state.state_bump
    )] 
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>
    
}

// closing the vaul
#[derive(Accounts)]
pub struct Close<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds= [b"vault", vault_state.key().as_ref()],
        bump= vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,


    #[account(
        mut,
        seeds= [b"state", user.key().as_ref()],
        bump= vault_state.state_bump,
        close= user
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>, 

}



impl <'info> Initialize <'info>{
    /// We are going to initialize the vault and state accounts.
    /// This function will be called when the user wants to create a new vault.
    /// for this example, the vault is just a system account, but in a real application, 
    /// it could be a token account or any other type of account.
    /// Since the vault is a sytem account, all we need to initialize it is
    /// sending enough lamports to make rent exempt.
    pub fn initialize(&mut self, bumps: &InitializeBumps)-> Result<()>{
        let rent_exempt= Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());

        let cpi_program= self.system_program.to_account_info();

        let cpi_accounts=Transfer{
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, rent_exempt)?;
        self.vault_state.vault_bump=bumps.vault;
        self.vault_state.state_bump=bumps.vault_state;
        Ok(())
   
    }
}

impl <'info> DepositeAndWithdraw<'info>{
    pub fn deposit(&mut self, amount: u64)-> Result<()>{
        let cpi_program= self.system_program.to_account_info();

        let cpi_accounts= Transfer{
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx= CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)?;
        
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64)-> Result<()>{
        let cpi_program= self.system_program.to_account_info();

        let cpi_accounts= Transfer{
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds=[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];

        let signer_seeds= &[&seeds[..]];

        let cpi_ctx= CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

impl <'info> Close<'info> {
    pub fn close_vault(&mut self)-> Result<()>{
        let cpi_program= self.system_program.to_account_info();
        let cpi_accounts=Transfer{
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds=[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];

        let signer_seeds=&[&seeds[..]];

        let cpi_ctx= CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds) ;

        transfer(cpi_ctx, self.vault.lamports())?;
        Ok(())
    }
}


//store the vault bumps so that we don't have to recompute them
#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState {
    const INIT_SPACE: usize = 1 + 1; // vault_bump + state_bump
}


