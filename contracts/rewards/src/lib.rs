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
    fn zero_user_weight_returns_zero_reward() {
        assert_eq!(calculate_reward_share(400, 0, 400), 0);
    }

    #[test]
    fn zero_total_rewards_returns_zero_reward() {
        assert_eq!(calculate_reward_share(0, 100, 400), 0);
    }

    #[test]
    fn large_reward_share_values_do_not_overflow() {
        // saturating_mul clamps the product; division still completes without
        // panicking, and the result is deterministic.
        assert_eq!(calculate_reward_share(u128::MAX, 2, 3), u128::MAX / 3);
        assert_eq!(calculate_reward_share(u128::MAX, u128::MAX, 1), u128::MAX);
        assert_eq!(
            calculate_reward_share(u128::MAX, 2, 3),
            calculate_reward_share(u128::MAX, 2, 3)
        );
    }

    #[test]
    fn pending_rewards_are_zero_when_total_rewards_are_zero() {
        assert_eq!(calculate_pending_rewards(500, 1000, 0), 0);
    }

    #[test]
    fn pending_rewards_are_zero_for_negative_inputs() {
        assert_eq!(calculate_pending_rewards(-500, 1000, 200), 0);
        assert_eq!(calculate_pending_rewards(500, -1000, 200), 0);
        assert_eq!(calculate_pending_rewards(500, 1000, -200), 0);
    }

    #[test]
    fn large_pending_rewards_values_are_deterministic() {
        // i128::MAX * 1 does not overflow; floor division truncates without panic.
        assert_eq!(calculate_pending_rewards(i128::MAX, 2, 1), i128::MAX / 2);
        assert_eq!(
            calculate_pending_rewards(i128::MAX, 2, 1),
            calculate_pending_rewards(i128::MAX, 2, 1)
        );
    }

    #[test]
    fn pending_rewards_truncate_fractional_shares() {
        // 5 * 1 / 10 truncates to 0; 15 * 1 / 10 truncates to 1. Never rounds up.
        assert_eq!(calculate_pending_rewards(5, 10, 1), 0);
        assert_eq!(calculate_pending_rewards(15, 10, 1), 1);
    }

    #[test]
    fn pending_rewards_return_zero_on_overflow() {
        // The product overflows i128, so the checked policy returns 0.
        assert_eq!(calculate_pending_rewards(i128::MAX, 1, 2), 0);
        assert_eq!(calculate_pending_rewards(i128::MAX, 2, i128::MAX), 0);
    }
}
