#![no_std]

use soroban_sdk::{contracterror, contracttype, Address, BytesN, Env, Symbol, Val, Vec};

/// Trait that all event emitters must implement.
/// Ensures each action emits a well-formed event with the standard two-topic pattern.
pub trait VaultEventEmitter {
    fn emit_initialize(e: &Env, admin: Address, deposit_token: Address, reward_token: Address);
    fn emit_deposit(e: &Env, user: Address, amount: i128);
    fn emit_withdraw(e: &Env, user: Address, amount: i128, remaining_balance: i128);
    fn emit_distribute(e: &Env, caller: Address, amount: i128);
    fn emit_claim_rewards(e: &Env, user: Address, amount: i128);
    fn emit_lock(e: &Env, user: Address, amount: i128, unlock_timestamp: u64);
    fn emit_unlock(e: &Env, user: Address, amount: i128);
    fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address);
    fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address);
    fn emit_upgrade(e: &Env, admin: Address, new_wasm_hash: BytesN<32>);
    fn emit_pause(e: &Env, admin: Address);
    fn emit_unpause(e: &Env, admin: Address);
    fn emit_asset_added(e: &Env, asset: Address);
    fn emit_asset_deposit(e: &Env, user: Address, asset: Address, amount: i128);
    fn emit_asset_withdraw(
        e: &Env,
        user: Address,
        asset: Address,
        amount: i128,
        remaining_balance: i128,
    );
    fn emit_asset_distribute(e: &Env, caller: Address, asset: Address, amount: i128);
    fn emit_asset_claim_rewards(e: &Env, user: Address, asset: Address, amount: i128);
}

/// A single operation inside a cross-contract execution plan.
///
/// `depends_on` lists operation ids that must appear earlier in the same plan.
/// `rollback` contains zero or one compensating calls that are scheduled if
/// this operation completed before a later step failed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationOperation {
    pub id: u32,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
    pub depends_on: Vec<u32>,
    pub rollback: Vec<RollbackOperation>,
}

/// A compensating call for an executed operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackOperation {
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
}

/// A deterministic execution plan for coordinating multiple contract calls.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    pub id: BytesN<32>,
    pub caller: Address,
    pub operations: Vec<OrchestrationOperation>,
}

/// State recorded for a single operation in an execution receipt.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    Pending,
    Executed,
    RolledBack,
}

/// Final state of an execution plan.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    Succeeded,
    Failed,
    RolledBack,
}

/// Per-operation receipt data persisted by the orchestrator.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationReceipt {
    pub operation_id: u32,
    pub status: OperationStatus,
}

/// Receipt persisted after every attempted orchestration run.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionReceipt {
    pub plan_id: BytesN<32>,
    pub caller: Address,
    pub status: ExecutionStatus,
    pub executed: Vec<OperationReceipt>,
    pub rollback: Vec<OperationReceipt>,
    pub failed_operation: Option<u32>,
    pub timestamp: u64,
}

/// Errors returned by orchestration validation and execution.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum OrchestrationError {
    EmptyPlan = 1,
    TooManyOperations = 2,
    DuplicateOperationId = 3,
    InvalidTarget = 4,
    InvalidDependency = 5,
    DependencyNotOrdered = 6,
    OperationFailed = 7,
    RollbackFailed = 8,
}

/// Interface implemented by contracts that coordinate execution plans.
pub trait TransactionOrchestrator {
    fn validate_plan(e: Env, plan: ExecutionPlan) -> Result<(), OrchestrationError>;
    fn execute_plan(e: Env, plan: ExecutionPlan) -> Result<ExecutionReceipt, OrchestrationError>;
    fn execution_receipt(e: Env, plan_id: BytesN<32>) -> Option<ExecutionReceipt>;
}

// ---------------------------------------------------------------------------
// Treasury management shared types
// ---------------------------------------------------------------------------

pub const TREASURY_BPS_DENOMINATOR: u32 = 10_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationRule {
    pub recipient: Address,
    pub share_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationStrategy {
    pub id: BytesN<32>,
    pub rules: Vec<AllocationRule>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationTransfer {
    pub recipient: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryDistributionReceipt {
    pub distribution_id: BytesN<32>,
    pub strategy_id: BytesN<32>,
    pub asset: Address,
    pub total_amount: i128,
    pub transfers: Vec<AllocationTransfer>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryFeeRecord {
    pub fee_id: BytesN<32>,
    pub payer: Address,
    pub asset: Address,
    pub amount: i128,
    pub treasury_balance: i128,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    EmptyStrategy = 5,
    TooManyRules = 6,
    InvalidShare = 7,
    InvalidShareTotal = 8,
    DuplicateRecipient = 9,
    StrategyNotFound = 10,
    DuplicateDistribution = 11,
    InsufficientBalance = 12,
    DuplicateFee = 13,
}

pub trait TreasuryAllocator {
    fn initialize(e: Env, admin: Address, asset: Address) -> Result<(), TreasuryError>;
    fn configure_strategy(
        e: Env,
        admin: Address,
        strategy: AllocationStrategy,
    ) -> Result<(), TreasuryError>;
    fn record_fee(
        e: Env,
        caller: Address,
        fee_id: BytesN<32>,
        payer: Address,
        amount: i128,
    ) -> Result<TreasuryFeeRecord, TreasuryError>;
    fn distribute(
        e: Env,
        admin: Address,
        distribution_id: BytesN<32>,
        strategy_id: BytesN<32>,
        amount: i128,
    ) -> Result<TreasuryDistributionReceipt, TreasuryError>;
    fn strategy(e: Env, strategy_id: BytesN<32>) -> Option<AllocationStrategy>;
    fn fee_record(e: Env, fee_id: BytesN<32>) -> Option<TreasuryFeeRecord>;
    fn distribution_receipt(
        e: Env,
        distribution_id: BytesN<32>,
    ) -> Option<TreasuryDistributionReceipt>;
    fn recipient_distributed(e: Env, recipient: Address) -> i128;
    fn total_fees_recorded(e: Env) -> i128;
    fn total_distributed(e: Env) -> i128;
    fn asset_balance(e: Env) -> i128;
}
