use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

declare_id!("HezVxzdFxE8hfLJGp24nLQ31M6jjzJD8Uyj1QCxJGJ45");

#[program]
pub mod token_swapper {
    use super::*;

    pub fn make_offer(
        ctx: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_wanted_amount: u64,
    ) -> Result<()> {
        msg!("Greetings from: {:?} make_offer invoked!!!", ctx.program_id);

        let transfer_options = TransferChecked {
            from: ctx.accounts.maker_token_account_a.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            mint: ctx.accounts.token_mint_a.to_account_info(),
            authority: ctx.accounts.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_options,
        );

        transfer_checked(
            cpi_ctx,
            token_a_offered_amount,
            ctx.accounts.token_mint_a.decimals,
        )?;
        msg!("vault: {}", ctx.accounts.vault.key());
        msg!("offer: {}", ctx.accounts.offer.key());
        msg!(
            "maker_token_account_a: {}",
            ctx.accounts.maker_token_account_a.key()
        );

        ctx.accounts.offer.set_inner(Offer {
            id,
            maker: ctx.accounts.maker.key(),
            token_mint_a: ctx.accounts.token_mint_a.key(),
            token_mint_b: ctx.accounts.token_mint_b.key(),
            token_b_wanted_amount,
            bump: ctx.bumps.offer,
        });

        // let _ = ctx.accounts.offer.reload()?;
        let maker_key = ctx.accounts.maker.key();
        let id_bytes = ctx.accounts.offer.id.to_le_bytes();
        let bump = ctx.accounts.offer.bump;

        // Manually derive the PDA inside the instruction
        let (derived_offer_pda, derived_bump) = Pubkey::find_program_address(
            &[b"offer", maker_key.as_ref(), &id_bytes],
            ctx.program_id,
        );

        // Log the individual components and the derived PDA
        msg!("id: {:?}", id);
        msg!("maker.key(): {:?}", maker_key);
        msg!("offer.id.to_le_bytes(): {:?}", id_bytes);
        msg!("bump: {:?}", bump);
        msg!("Derived PDA: {:?}", derived_offer_pda);
        msg!("Expected PDA: {:?}", ctx.accounts.offer.key());
        msg!("Derived bump: {:?}", derived_bump);

        Ok(())
    }

    pub fn take_offer(ctx: Context<TakeOffer>) -> Result<()> {
        // Logging each account as before
        msg!("Taker: {:?}", ctx.accounts.taker.key());
        msg!("Maker: {:?}", ctx.accounts.maker.key());
        msg!("Token Mint A: {:?}", ctx.accounts.token_mint_a.key());
        msg!("Token Mint B: {:?}", ctx.accounts.token_mint_b.key());

        // Condition on Taker Token Account A
        if ctx.accounts.taker_token_account_a.amount > 0 {
            msg!(
                "Taker Token Account A has a positive balance: {}",
                ctx.accounts.taker_token_account_a.amount
            );
        } else {
            msg!("Taker Token Account A balance is zero.");
        }

        // Condition on Taker Token Account B
        if ctx.accounts.taker_token_account_b.amount > 0 {
            msg!(
                "Taker Token Account B has a positive balance: {}",
                ctx.accounts.taker_token_account_b.amount
            );
        } else {
            msg!("Taker Token Account B balance is zero.");
        }

        // Condition on Maker Token Account B
        if ctx.accounts.maker_token_account_b.amount > 0 {
            msg!(
                "Maker Token Account B has a positive balance: {}",
                ctx.accounts.maker_token_account_b.amount
            );
        } else {
            msg!("Maker Token Account B balance is zero.");
        }

        // Condition on Vault Account
        if ctx.accounts.vault.amount > 0 {
            msg!(
                "Vault Account has a positive balance: {}",
                ctx.accounts.vault.amount
            );
        } else {
            msg!("Vault Account balance is zero.");
        }

        Ok(())
    }

    pub fn take_offer_latesr(ctx: Context<TakeOffer>) -> Result<()> {
        msg!("take_offer invoked!");

        // move tokens from taker's account to maker account
        let tranfer_options = TransferChecked {
            from: ctx.accounts.taker_token_account_b.to_account_info(),
            to: ctx.accounts.maker_token_account_b.to_account_info(),
            mint: ctx.accounts.token_mint_b.to_account_info(),
            authority: ctx.accounts.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            tranfer_options,
        );

        transfer_checked(
            cpi_ctx,
            ctx.accounts.offer.token_b_wanted_amount,
            ctx.accounts.token_mint_b.decimals,
        )?;

        // move tokens from vault to taker's account, might be intialised the account
        let seeds = &[
            b"offer",
            ctx.accounts.maker.to_account_info().key.as_ref(),
            &ctx.accounts.offer.id.to_le_bytes()[..],
            &[ctx.accounts.offer.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let tranfer_options = TransferChecked {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.taker_token_account_a.to_account_info(),
            authority: ctx.accounts.offer.to_account_info(),
            mint: ctx.accounts.token_mint_a.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            tranfer_options,
            signer_seeds,
        );

        transfer_checked(
            cpi_ctx,
            ctx.accounts.vault.amount,
            ctx.accounts.token_mint_a.decimals,
        )?;

        // close the vault the pay the rent back to maker
        let close_account_options = CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.taker.to_account_info(),
            authority: ctx.accounts.offer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_account_options,
            signer_seeds,
        );

        close_account(cpi_ctx)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    // it must already be present with authority field set to taker
    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    // #[account(
    //     init_if_needed,
    //     payer = taker,
    //     associated_token::mint = token_mint_a,
    //     associated_token::authority = taker,
    //     associated_token::token_program = token_program,
    // )]
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
        seeds = [b"offer", maker.key().as_ref(), &offer.id.to_le_bytes().as_ref()],
        bump = offer.bump,
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
#[instruction(id:u64,token_a_offered_amount: u64,token_b_wanted_amount: u64)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = 8 + Offer::INIT_SPACE,
        seeds = [b"offer",maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub id: u64,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_b_wanted_amount: u64,
    pub bump: u8,
}
