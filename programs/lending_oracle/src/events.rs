use anchor_lang::prelude::*;

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct BorrowEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub new_debt: u64,
}

#[event]
pub struct RepayEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub remaining_debt: u64,
}

#[event]
pub struct LiquidationEvent {
    pub liquidator: Pubkey,
    pub user: Pubkey,
    pub debt_repaid: u64,
    pub collateral_seized: u64,
}

#[event]
pub struct ConfigUpdatedEvent {
    pub admin: Pubkey,
}

#[event]
pub struct ProtocolPauseEvent {
    pub admin: Pubkey,
    pub paused: bool,
}
#[event]
pub struct RiskParametersUpdatedEvent {
    pub admin: Pubkey,
    pub ltv_bps: u64,
    pub liquidation_threshold_bps: u64,
    pub liquidation_bonus_bps: u64,
}

#[event]
pub struct OracleUpdatedEvent {
    pub admin: Pubkey,
    pub price_update: Pubkey,
    pub price_feed_id: [u8; 32],
}