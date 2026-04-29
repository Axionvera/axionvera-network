//! Event definitions and emission helpers for the Vault contract.
//!
//! Events are used for off-chain monitoring, indexing, and front-end updates.
//! Each event is identified by a specific symbol and carries relevant data.

use soroban_sdk::{symbol_short, Address, Env, Symbol};

/// Event topics for the vault contract.
pub struct Topics;

impl Topics {
    /// Topic emitted when the contract is initialized.
    pub const INITIALIZE: Symbol = symbol_short!("init");
    /// Topic emitted when a user deposits tokens.
    pub const DEPOSIT: Symbol = symbol_short!("deposit");
    /// Topic emitted when a user withdraws tokens.
    pub const WITHDRAW: Symbol = symbol_short!("withdraw");
    /// Topic emitted when rewards are distributed by the admin.
    pub const DISTRIBUTE: Symbol = symbol_short!("distrib");
    /// Topic emitted when a user claims their accrued rewards.
    pub const CLAIM: Symbol = symbol_short!("claim");
}

/// Emits an event indicating the contract has been successfully initialized.
///
/// # Arguments
/// * `e` - The environment.
/// * `admin` - The address set as the contract administrator.
/// * `deposit_token` - The address of the accepted deposit token.
/// * `reward_token` - The address of the distributed reward token.
pub fn emit_initialize(e: &Env, admin: Address, deposit_token: Address, reward_token: Address) {
    e.events().publish(
        (Topics::INITIALIZE,),
        (admin, deposit_token, reward_token),
    );
}

/// Emits an event when a user makes a deposit.
///
/// # Arguments
/// * `e` - The environment.
/// * `user` - The address of the user who deposited.
/// * `amount` - The amount of tokens deposited.
/// * `new_balance` - The user's new total balance after the deposit.
pub fn emit_deposit(e: &Env, user: Address, amount: i128, new_balance: i128) {
    e.events().publish(
        (Topics::DEPOSIT, user),
        (amount, new_balance),
    );
}

/// Emits an event when a user withdraws their stake.
///
/// # Arguments
/// * `e` - The environment.
/// * `user` - The address of the user who withdrew.
/// * `amount` - The amount of tokens withdrawn.
/// * `remaining_balance` - The user's remaining balance in the vault.
pub fn emit_withdraw(e: &Env, user: Address, amount: i128, remaining_balance: i128) {
    e.events().publish(
        (Topics::WITHDRAW, user),
        (amount, remaining_balance),
    );
}

/// Emits an event when new rewards are distributed.
///
/// # Arguments
/// * `e` - The environment.
/// * `amount` - The total amount of reward tokens distributed.
/// * `new_index` - The global reward index after the distribution.
pub fn emit_distribute_rewards(e: &Env, amount: i128, new_index: i128) {
    e.events().publish(
        (Topics::DISTRIBUTE,),
        (amount, new_index),
    );
}

/// Emits an event when a user claims their accrued rewards.
///
/// # Arguments
/// * `e` - The environment.
/// * `user` - The address of the user who claimed rewards.
/// * `amount` - The amount of reward tokens claimed.
pub fn emit_claim_rewards(e: &Env, user: Address, amount: i128) {
    e.events().publish(
        (Topics::CLAIM, user),
        (amount,),
use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

/// Protocol identifier for all Axionvera Vault events (Topic 1)
const PROTOCOL: Symbol = symbol_short!("AxionVault");

/// Event action types (Topic 2)
const ACT_INIT: Symbol = symbol_short!("Initialize");
const ACT_DEPOSIT: Symbol = symbol_short!("Deposit");
const ACT_WITHDRAW: Symbol = symbol_short!("Withdraw");
const ACT_DISTRIBUTE: Symbol = symbol_short!("Distribute");
const ACT_CLAIM: Symbol = symbol_short!("Claim");

/// Initialize event payload: contract setup with protocol parameters
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializeEvent {
    pub admin: Address,
    pub deposit_token: Address,
    pub reward_token: Address,
    pub timestamp: u64,
}

/// Deposit event payload: user deposits tokens into vault
/// Data payload contains user_address, amount, and timestamp.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    pub user_address: Address,
    pub amount: i128,
    pub timestamp: u64,
}

/// Withdraw event payload: user withdraws tokens from vault
/// Data payload contains user_address, amount, and timestamp.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawEvent {
    pub user_address: Address,
    pub amount: i128,
    pub timestamp: u64,
}

/// Distribute event payload: admin distributes reward tokens
/// Data payload contains caller address, amount distributed, and timestamp.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributeEvent {
    pub caller: Address,
    pub amount: i128,
    pub timestamp: u64,
}

/// Claim event payload: user claims accrued rewards
/// Data payload contains user_address, amount claimed, and timestamp.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimEvent {
    pub user_address: Address,
    pub amount: i128,
    pub timestamp: u64,
}

/// Emits a standardized Initialize event.
///
/// Topics:
/// - Topic 1: "AxionVault" (protocol identifier)
/// - Topic 2: "Initialize" (action)
///
/// Data payload contains admin, deposit_token, reward_token, and timestamp.
pub fn emit_initialize(e: &Env, admin: Address, deposit_token: Address, reward_token: Address) {
    e.events().publish(
        (PROTOCOL, ACT_INIT),
        InitializeEvent {
            admin,
            deposit_token,
            reward_token,
            timestamp: e.ledger().timestamp(),
        },
    );
}

/// Emits a standardized Deposit event.
///
/// Topics:
/// - Topic 1: "AxionVault" (protocol identifier)
/// - Topic 2: "Deposit" (action)
///
/// Data payload contains user_address, amount, and timestamp.
pub fn emit_deposit(e: &Env, user: Address, amount: i128) {
    e.events().publish(
        (PROTOCOL, ACT_DEPOSIT),
        DepositEvent {
            user_address: user,
            amount,
            timestamp: e.ledger().timestamp(),
        },
    );
}

/// Emits a standardized Withdraw event.
///
/// Topics:
/// - Topic 1: "AxionVault" (protocol identifier)
/// - Topic 2: "Withdraw" (action)
///
/// Data payload contains user_address, amount, and timestamp.
pub fn emit_withdraw(e: &Env, user: Address, amount: i128) {
    e.events().publish(
        (PROTOCOL, ACT_WITHDRAW),
        WithdrawEvent {
            user_address: user,
            amount,
            timestamp: e.ledger().timestamp(),
        },
    );
}

/// Emits a standardized Distribute event.
///
/// Topics:
/// - Topic 1: "AxionVault" (protocol identifier)
/// - Topic 2: "Distribute" (action)
///
/// Data payload contains caller, amount, and timestamp.
pub fn emit_distribute(e: &Env, caller: Address, amount: i128) {
    e.events().publish(
        (PROTOCOL, ACT_DISTRIBUTE),
        DistributeEvent {
            caller,
            amount,
            timestamp: e.ledger().timestamp(),
        },
    );
}

/// Emits a standardized Claim event.
///
/// Topics:
/// - Topic 1: "AxionVault" (protocol identifier)
/// - Topic 2: "Claim" (action)
///
/// Data payload contains user_address, amount, and timestamp.
pub fn emit_claim(e: &Env, user: Address, amount: i128) {
    e.events().publish(
        (PROTOCOL, ACT_CLAIM),
        ClaimEvent {
            user_address: user,
            amount,
            timestamp: e.ledger().timestamp(),
        },
    );
}

pub fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address) {
    e.events().publish(
        (EVT_ADMIN_PROPOSED,),
        AdminTransferProposedEvent {
            current_admin,
            pending_admin,
            timestamp: e.ledger().timestamp(),
        },
    );
}

pub fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address) {
    e.events().publish(
        (EVT_ADMIN_ACCEPTED,),
        AdminTransferAcceptedEvent {
            previous_admin,
            new_admin,
            timestamp: e.ledger().timestamp(),
        },
    );
}
