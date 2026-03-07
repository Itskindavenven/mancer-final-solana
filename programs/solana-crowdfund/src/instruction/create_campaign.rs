use anchor_lang::prelude::*;
use crate::{state::*, error::*, event::*};

#[derive(Accounts)]
#[instruction(campaign_id: u64)]
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + 32 + 8 + 8 + 8 + 1 + 8 + 1,
        seeds = [b"campaign", creator.key().as_ref(), campaign_id.to_le_bytes().as_ref()],
        bump
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

pub fn create_campaign(
    ctx: Context<CreateCampaign>,
    campaign_id: u64,
    goal: u64,
    deadline: i64,
) -> Result<()> {
    require!(goal > 0, CrowdfundError::InvalidGoal);
    let clock = Clock::get()?;
    require!(deadline > clock.unix_timestamp, CrowdfundError::DeadlinePassed);

    let campaign = &mut ctx.accounts.campaign;
    campaign.creator = ctx.accounts.creator.key();
    campaign.goal = goal;
    campaign.raised = 0;
    campaign.deadline = deadline;
    campaign.claimed = false;
    campaign.campaign_id = campaign_id;
    campaign.bump = ctx.bumps.campaign;

    emit!(CampaignCreated {
        campaign_id,
        creator: campaign.creator,
        goal,
        deadline,
    });
    Ok(())
}
