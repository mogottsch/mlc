#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_update_limits() {
        let mut limits = Limits::new();

        limits.add_category("shop");

        assert_eq!(limits.update_limit("shop", 0, 60), true);
        assert_eq!(limits.update_limit("shop", 0, 70), false);
        assert_eq!(limits.update_limit("shop", 100, 30), true);
        assert_eq!(limits.update_limit("shop", 50, 70), false);
        assert_eq!(limits.update_limit("shop", 50, 50), true);
        assert_eq!(limits.update_limit("shop", 200, 10), true);
    }

    #[test]
    fn test_update_limits_single_category() {
        let mut limits = Limits::new();

        limits.add_category("shop");

        limits.update_limit("shop", 0, 60);
        limits.update_limit("shop", 100, 30);
        limits.update_limit("shop", 50, 50);
        limits.update_limit("shop", 200, 10);

        println!("{:?}", limits);
        assert_eq!(limits.is_limit_exceeded(0, 70), true);
        assert_eq!(limits.is_limit_exceeded(0, 60), true);
        assert_eq!(limits.is_limit_exceeded(0, 50), false);

        assert_eq!(limits.is_limit_exceeded(100, 40), true);
        assert_eq!(limits.is_limit_exceeded(100, 30), true);
        assert_eq!(limits.is_limit_exceeded(100, 20), false);

        assert_eq!(limits.is_limit_exceeded(50, 70), true);
        assert_eq!(limits.is_limit_exceeded(50, 50), true);
        assert_eq!(limits.is_limit_exceeded(50, 40), false);
    }

    #[test]
    fn test_update_limits_multi_category() {
        let mut limits = Limits::new();

        limits.add_category("shop");

        limits.update_limit("shop", 0, 60);
        limits.update_limit("shop", 100, 30);
        limits.update_limit("shop", 50, 50);
        limits.update_limit("shop", 200, 10);

        limits.add_category("grocery");

        limits.update_limit("grocery", 0, 100);
        limits.update_limit("grocery", 200, 5);

        assert_eq!(limits.is_limit_exceeded(0, 110), true); // border determined by (0, 100)
        assert_eq!(limits.is_limit_exceeded(0, 100), true);
        assert_eq!(limits.is_limit_exceeded(0, 90), false);

        assert_eq!(limits.is_limit_exceeded(200, 20), true); // border determined by (200, 10)
        assert_eq!(limits.is_limit_exceeded(200, 10), true);
        assert_eq!(limits.is_limit_exceeded(200, 5), false);

        assert_eq!(limits.is_limit_exceeded(100, 110), true); // border determined by (0, 100)
        assert_eq!(limits.is_limit_exceeded(100, 100), true);
        assert_eq!(limits.is_limit_exceeded(100, 90), false);
    }
}
