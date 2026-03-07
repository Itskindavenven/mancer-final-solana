use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

declare_id!("BqqrNNMCmewqgVWTbzBJyrD2uFhmzZF5ot8i4pZXLZRP");

#[program]
pub mod solana_crowdfund {
    use super::*;

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
}

// ========================
// Account Contexts
// ========================

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

// ========================
// State Structs
// ========================

#[account]
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub claimed: bool,
    pub campaign_id: u64,
    pub bump: u8,
}

#[account]
pub struct Contribution {
    pub donor: Pubkey,
    pub campaign: Pubkey,
    pub amount: u64,
    pub refunded: bool,
    pub bump: u8,
}

// ========================
// Events
// ========================

#[event]
pub struct CampaignCreated {
    pub campaign_id: u64,
    pub creator: Pubkey,
    pub goal: u64,
    pub deadline: i64,
}

#[event]
pub struct ContributionMade {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub total: u64,
}

#[event]
pub struct Withdrawn {
    pub campaign: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Refunded {
    pub campaign: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
}

// ========================
// Errors
// ========================

#[error_code]
pub enum CrowdfundError {
    #[msg("Goal must be greater than zero")]
    InvalidGoal,
    #[msg("The deadline has already passed")]
    DeadlinePassed,
    #[msg("The deadline has not been reached yet")]
    DeadlineNotReached,
    #[msg("The goal has not been reached")]
    GoalNotReached,
    #[msg("The goal has been reached")]
    GoalReached,
    #[msg("The campaign funds have already been claimed")]
    AlreadyClaimed,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Contribution amount must be greater than zero")]
    ZeroContribution,
    #[msg("Funds have already been refunded")]
    AlreadyRefunded,
    #[msg("Mathematical overflow occurred")]
    MathOverflow,
    #[msg("Vault balance is less than expected")]
    VaultDrained,
    #[msg("Invalid Campaign tied to this contribution")]
    InvalidCampaign,
}
