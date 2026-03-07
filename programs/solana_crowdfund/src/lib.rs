pub mod error;
pub mod event;
pub mod instruction;
pub mod state;

use anchor_lang::prelude::*;
use instruction::*;

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
        instruction::create_campaign(ctx, campaign_id, goal, deadline)
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        instruction::contribute(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instruction::withdraw(ctx)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instruction::refund(ctx)
    }
}
