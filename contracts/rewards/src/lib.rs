#![cfg_attr(not(test), no_std)]

pub fn calculate_reward_share(total_rewards: u128, user_weight: u128, total_weight: u128) -> u128 {
    if total_weight == 0 {
        return 0;
    }

    total_rewards.saturating_mul(user_weight) / total_weight
}

/// Calculates a user's pending reward share:
/// `(user_balance * total_rewards) / total_deposits`.
///
/// Returns `0` when total deposits is zero, when any input is not positive, or
/// when checked arithmetic overflows. This helper never panics.
pub fn calculate_pending_rewards(
    user_balance: i128,
    total_deposits: i128,
    total_rewards: i128,
) -> i128 {
    if total_deposits <= 0 || user_balance <= 0 || total_rewards <= 0 {
        return 0;
    }

    user_balance
        .checked_mul(total_rewards)
        .and_then(|product| product.checked_div(total_deposits))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_reward_share() {
        assert_eq!(calculate_reward_share(400, 100, 400), 100);
    }

    #[test]
    fn returns_zero_when_total_weight_is_zero() {
        assert_eq!(calculate_reward_share(400, 100, 0), 0);
    }

    #[test]
    fn pending_rewards_are_proportional_to_deposit_share() {
        assert_eq!(calculate_pending_rewards(500, 1000, 200), 100);
        assert_eq!(calculate_pending_rewards(250, 1000, 200), 50);
    }

    #[test]
    fn pending_rewards_are_zero_when_total_deposits_are_zero() {
        assert_eq!(calculate_pending_rewards(500, 0, 200), 0);
    }

    #[test]
    fn pending_rewards_are_zero_when_user_balance_is_zero() {
        assert_eq!(calculate_pending_rewards(0, 1000, 200), 0);
    }

    #[test]
    fn pending_rewards_return_zero_on_overflow() {
        assert_eq!(calculate_pending_rewards(i128::MAX, 1, 2), 0);
    }
}
