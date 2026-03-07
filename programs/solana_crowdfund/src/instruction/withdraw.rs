use anchor_lang::prelude::*;
use crate::{state::*, error::*, event::*};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        has_one = creator @ CrowdfundError::Unauthorized
    )]
    pub campaign: Account<'info, Campaign>,

    /// CHECK: PDA used as a vault to hold funds securely
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let clock = Clock::get()?;
    let campaign = &mut ctx.accounts.campaign;

    require!(campaign.raised >= campaign.goal, CrowdfundError::GoalNotReached);
    require!(clock.unix_timestamp >= campaign.deadline, CrowdfundError::DeadlineNotReached);
    require!(!campaign.claimed, CrowdfundError::AlreadyClaimed);

    let vault = &ctx.accounts.vault;
    let amount = vault.lamports();
    require!(amount >= campaign.raised, CrowdfundError::VaultDrained);

    // Mark claimed before transfer (Checks-Effects-Interactions)
    campaign.claimed = true;

    let creator = &ctx.accounts.creator;
    let campaign_key = campaign.key();
    let seeds = &[
        b"vault",
        campaign_key.as_ref(),
        &[ctx.bumps.vault],
    ];
    let signer = &[&seeds[..]];

    // Perform the CPI transfer from Vault
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::transfer(
            &vault.key(),
            &creator.key(),
            amount,
        ),
        &[
            vault.to_account_info(),
            creator.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer,
    )?;

    emit!(Withdrawn {
        campaign: campaign.key(),
        amount,
    });
    Ok(())
}
