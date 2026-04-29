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
    );
}
