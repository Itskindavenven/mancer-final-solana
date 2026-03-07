use anchor_lang::prelude::*;

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
