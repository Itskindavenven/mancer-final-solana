use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use crate::{state::*, error::*, event::*};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(
        init_if_needed,
        payer = donor,
        space = 8 + 32 + 32 + 8 + 1 + 1,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump
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

pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let campaign = &mut ctx.accounts.campaign;

    require!(clock.unix_timestamp < campaign.deadline, CrowdfundError::DeadlinePassed);
    require!(amount > 0, CrowdfundError::ZeroContribution);
    require!(!campaign.claimed, CrowdfundError::AlreadyClaimed);

    // Transfer funds from donor to vault
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.donor.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        },
    );
    system_program::transfer(cpi_context, amount)?;

    // Update campaign raised securely
    campaign.raised = campaign.raised.checked_add(amount).ok_or(CrowdfundError::MathOverflow)?;

    // Update contribution securely
    let contribution = &mut ctx.accounts.contribution;
    if contribution.amount == 0 && !contribution.refunded {
        contribution.donor = ctx.accounts.donor.key();
        contribution.campaign = campaign.key();
        contribution.refunded = false;
        contribution.bump = ctx.bumps.contribution;
    }
    contribution.amount = contribution.amount.checked_add(amount).ok_or(CrowdfundError::MathOverflow)?;

    emit!(ContributionMade {
        campaign: campaign.key(),
        donor: ctx.accounts.donor.key(),
        amount,
        total: campaign.raised,
    });

    Ok(())
}
