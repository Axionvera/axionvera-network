pub fn calculate_reward_share(total_rewards: u128, user_weight: u128, total_weight: u128) -> u128 {
    if total_weight == 0 {
        return 0;
    }

    total_rewards.saturating_mul(user_weight) / total_weight
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
}
