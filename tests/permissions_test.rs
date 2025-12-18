#[cfg(test)]
mod permissions_tests {
    use crabcamera::permissions::{check_permission, PermissionStatus};

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_check_permission_returns_status() {
        let result = check_permission();
        // Should return one of the valid statuses
        match result {
            PermissionStatus::Granted | PermissionStatus::Denied | 
            PermissionStatus::NotDetermined | PermissionStatus::Restricted => {
                // Valid status
            }
        }
    }

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_check_permission_is_consistent() {
        // Test multiple calls to ensure consistent behavior
        let first = check_permission();
        for _ in 0..5 {
            let result = check_permission();
            assert_eq!(result, first, "Permission status should be consistent");
        }
    }

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_check_permission_concurrent() {
        // Test concurrent permission checks
        let handles: Vec<_> = (0..10)
            .map(|_i| {
                std::thread::spawn(move || {
                    check_permission()
                })
            })
            .collect();

        for handle in handles {
            let _result = handle.join().unwrap();
            // Just verify no panic
        }
    }

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_check_permission_performance() {
        // Test that permission check is fast
        // Note: 2000ms threshold accounts for system variability (CI, cold cache, etc.)
        let start = std::time::Instant::now();

        for _ in 0..1000 {
            let _ = check_permission();
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 2000,
            "1000 permission checks should complete in under 2s, took {}ms",
            duration.as_millis()
        );
    }

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_permission_function_exists() {
        // Verify the function exists and is callable
        let _result: PermissionStatus = check_permission();
        // If we get here, the function exists and returns PermissionStatus
    }

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_permission_no_panic() {
        // Test that permission check doesn't panic under normal conditions
        let result = std::panic::catch_unwind(check_permission);
        assert!(result.is_ok(), "Permission check should not panic");
    }

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_permission_in_loop() {
        // Test repeated calls in tight loop
        for _ in 0..100 {
            let _ = check_permission();
            // Just verify no panic
        }
    }
}

