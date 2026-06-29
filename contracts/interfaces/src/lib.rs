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
    fn emit_delegation_granted(
        e: &Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
        expiration: u64,
    );
    fn emit_delegation_revoked(e: &Env, delegator: Address, delegatee: Address, operation: Symbol);
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

/// Rule defining a specific permission delegation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationRule {
    pub delegator: Address,
    pub delegatee: Address,
    pub operation: Symbol,
    pub expiration: u64,
}

/// Errors returned by the delegation layer.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DelegationError {
    Unauthorized = 100,
    InvalidExpiration = 101,
    DelegationNotFound = 102,
    AlreadyDelegated = 103,
    PrivilegeEscalation = 104,
}

/// Interface for managing secure contract operation delegation.
pub trait ContractDelegation {
    /// Delegate a specific operation to a trusted address.
    fn delegate(
        e: Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
        expiration: u64,
    ) -> Result<(), DelegationError>;

    /// Revoke a previously granted delegation.
    fn revoke_delegation(
        e: Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
    ) -> Result<(), DelegationError>;

    /// Retrieve the delegation rule for a specific delegator, delegatee, and operation.
    fn get_delegation(
        e: Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
    ) -> Option<DelegationRule>;

    /// Check if a delegatee is authorized to perform an operation on behalf of a delegator.
    fn is_authorized(
        e: Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
    ) -> bool;
}
