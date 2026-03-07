use anchor_lang::prelude::*;

#[account]
pub struct Contribution {
    pub donor: Pubkey,
    pub campaign: Pubkey,
    pub amount: u64,
    pub refunded: bool,
    pub bump: u8,
}
