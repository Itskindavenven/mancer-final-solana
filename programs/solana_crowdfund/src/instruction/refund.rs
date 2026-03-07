use anchor_lang::prelude::*;
use crate::{state::*, error::*, event::*};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump = contribution.bump,
        has_one = donor @ CrowdfundError::Unauthorized,
        has_one = campaign @ CrowdfundError::InvalidCampaign,
        close = donor // Return rent to donor and zero out state securely
    )]
    pub contribution: Account<'info, Contribution>,

    /// CHECK: PDA used as a vault to hold funds securely
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub donor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn refund(ctx: Context<Refund>) -> Result<()> {
    let clock = Clock::get()?;
    let campaign = &ctx.accounts.campaign;
    let contribution = &mut ctx.accounts.contribution;

    require!(clock.unix_timestamp >= campaign.deadline, CrowdfundError::DeadlineNotReached);
    require!(campaign.raised < campaign.goal, CrowdfundError::GoalReached);
    require!(contribution.amount > 0, CrowdfundError::ZeroContribution);
    require!(!contribution.refunded, CrowdfundError::AlreadyRefunded);

    let amount = contribution.amount;
    let vault = &ctx.accounts.vault;
    require!(vault.lamports() >= amount, CrowdfundError::VaultDrained);

    // Mark refunded before transfer (Checks-Effects-Interactions)
    contribution.refunded = true;
    contribution.amount = 0;

    let donor = &ctx.accounts.donor;
    let campaign_key = campaign.key();
    let seeds = &[
        b"vault",
        campaign_key.as_ref(),
        &[ctx.bumps.vault],
    ];
    let signer = &[&seeds[..]];

    // Transfer funds back to donor
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::transfer(
            &vault.key(),
            &donor.key(),
            amount,
        ),
        &[
            vault.to_account_info(),
            donor.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer,
    )?;

    emit!(Refunded {
        campaign: campaign.key(),
        donor: donor.key(),
        amount,
    });

    // Anchor successfully returns rent to donor and closes due to `close = donor` attribute below
    Ok(())
}
