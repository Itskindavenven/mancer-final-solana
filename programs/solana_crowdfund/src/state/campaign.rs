use anchor_lang::prelude::*;

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
