use anchor_lang::prelude::*;

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
