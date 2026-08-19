pub fn health_status() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_is_ok() {
        assert_eq!(health_status(), "ok");
    }
}
