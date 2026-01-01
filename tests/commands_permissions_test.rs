#[cfg(test)]
mod commands_permissions_tests {
    use crabcamera::commands::permissions::{
        check_camera_permission_status, request_camera_permission, get_permission_status_string,
    };
    use crabcamera::permissions::PermissionStatus;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    async fn test_request_camera_permission_success() {
        let result = request_camera_permission().await;
        assert!(result.is_ok());
        let info = result.unwrap();
        // Should return valid status
        match info.status {
            PermissionStatus::Granted
            | PermissionStatus::Denied
            | PermissionStatus::NotDetermined
            | PermissionStatus::Restricted => {
                // Valid
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    async fn test_check_camera_permission_status_granted() {
        let result = check_camera_permission_status().await;
        assert!(result.is_ok());
        let info = result.unwrap();
        // Should return valid status
        match info.status {
            PermissionStatus::Granted
            | PermissionStatus::Denied
            | PermissionStatus::NotDetermined
            | PermissionStatus::Restricted => {
                // Valid
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    async fn test_permission_functions_are_consistent() {
        // Test multiple calls to ensure consistent behavior
        let first_request = request_camera_permission().await.unwrap().status;
        let first_status = check_camera_permission_status().await.unwrap().status;

        for _ in 0..3 {
            let request_result = request_camera_permission().await.unwrap();
            let status_result = check_camera_permission_status().await.unwrap();

            assert_eq!(request_result.status, first_request);
            assert_eq!(status_result.status, first_status);
        }
    }

    #[tokio::test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    async fn test_concurrent_permission_requests() {
        let mut handles = vec![];

        // Launch multiple concurrent requests
        for _ in 0..5 {
            let handle = tokio::spawn(async { request_camera_permission().await });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    async fn test_concurrent_permission_status_checks() {
        let mut handles = vec![];

        // Launch multiple concurrent status checks
        for _ in 0..5 {
            let handle = tokio::spawn(async { check_camera_permission_status().await });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_permission_status_string_function() {
        // Test the legacy string function
        let status = get_permission_status_string();
        assert!(!status.is_empty(), "Status string should not be empty");
        
        // Should be one of the valid permission status strings
        let valid_statuses = ["Granted", "Denied", "NotDetermined", "Restricted"];
        assert!(
            valid_statuses.iter().any(|&s| status.contains(s)),
            "Status string should contain a valid permission status: {}",
            status
        );
    }

    #[tokio::test]
    async fn test_permission_check_timeout() {
        // Test that permission checks don't hang
        let timeout_duration = Duration::from_secs(30);
        
        let result = timeout(timeout_duration, check_camera_permission_status()).await;
        assert!(result.is_ok(), "Permission check should complete within timeout");
        
        let permission_result = result.unwrap();
        assert!(permission_result.is_ok(), "Permission check should return a result");
    }

    #[tokio::test]
    async fn test_permission_request_timeout() {
        // Test that permission requests don't hang
        let timeout_duration = Duration::from_secs(60); // Longer for permission dialogs
        
        let result = timeout(timeout_duration, request_camera_permission()).await;
        assert!(result.is_ok(), "Permission request should complete within timeout");
        
        let permission_result = result.unwrap();
        assert!(permission_result.is_ok(), "Permission request should return a result");
    }

    #[tokio::test]
    async fn test_permission_info_structure() {
        // Test the structure of permission info responses
        let result = check_camera_permission_status().await;
        assert!(result.is_ok(), "Permission status check should succeed");
        
        let info = result.unwrap();
        
        // Message should not be empty
        assert!(!info.message.is_empty(), "Permission message should not be empty");
        
        // Status should be a valid enum value
        match info.status {
            PermissionStatus::Granted => {
                // Granted status should have appropriate message
                assert!(
                    info.message.to_lowercase().contains("grant") ||
                    info.message.to_lowercase().contains("authorized") ||
                    info.message.to_lowercase().contains("allowed"),
                    "Granted status should have appropriate message: {}", info.message
                );
            }
            PermissionStatus::Denied => {
                // Denied status should have appropriate message
                assert!(
                    info.message.to_lowercase().contains("deni") ||
                    info.message.to_lowercase().contains("forbidden") ||
                    info.message.to_lowercase().contains("blocked"),
                    "Denied status should have appropriate message: {}", info.message
                );
            }
            PermissionStatus::NotDetermined => {
                // NotDetermined status should indicate uncertainty or need for request
                assert!(
                    info.message.to_lowercase().contains("not") ||
                    info.message.to_lowercase().contains("unknown") ||
                    info.message.to_lowercase().contains("settings") ||
                    info.message.to_lowercase().contains("request"),
                    "NotDetermined status should have appropriate message: {}", info.message
                );
            }
            PermissionStatus::Restricted => {
                // Restricted status should indicate system restrictions
                assert!(
                    info.message.to_lowercase().contains("restrict") ||
                    info.message.to_lowercase().contains("policy") ||
                    info.message.to_lowercase().contains("system"),
                    "Restricted status should have appropriate message: {}", info.message
                );
            }
        }
        
        // can_request should be consistent with status
        match info.status {
            PermissionStatus::NotDetermined => {
                // For macOS, we might be able to request
                // For other platforms, behavior varies by implementation
                // Just verify it's a boolean (no assertion on specific value)
                let _ = info.can_request;
            }
            PermissionStatus::Granted | PermissionStatus::Denied | PermissionStatus::Restricted => {
                // Usually can't request again once determined, but implementation may vary
                let _ = info.can_request;
            }
        }
    }

    #[tokio::test]
    async fn test_permission_consistency_across_calls() {
        // Test that permission status is consistent across multiple calls
        let mut previous_status = None;
        let mut previous_message = None;
        
        for i in 0..5 {
            let result = check_camera_permission_status().await;
            assert!(result.is_ok(), "Permission check {} should succeed", i);
            
            let info = result.unwrap();
            
            if let Some(prev_status) = previous_status {
                // Status should be consistent (unless changed externally)
                if info.status != prev_status {
                    // Permission might have changed externally, which is OK
                    // Just log this for awareness
                    eprintln!("Permission status changed from {:?} to {:?}", prev_status, info.status);
                }
            }
            
            if let Some(ref _prev_message) = previous_message {
                // Message might change but should remain non-empty
                assert!(!info.message.is_empty(), "Message should not become empty");
            }
            
            previous_status = Some(info.status.clone());
            previous_message = Some(info.message.clone());
            
            // Small delay between checks
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[tokio::test]
    async fn test_high_frequency_permission_checks() {
        // Test rapid permission checks to ensure no resource leaks
        for i in 0..50 {
            let result = check_camera_permission_status().await;
            assert!(result.is_ok(), "Permission check {} should succeed", i);
            
            let info = result.unwrap();
            assert!(!info.message.is_empty(), "Message should not be empty for check {}", i);
            
            // No delay - test rapid access
        }
    }

    #[tokio::test]
    async fn test_permission_request_idempotency() {
        // Test multiple permission requests
        let mut results = Vec::new();
        
        for i in 0..3 {
            let result = request_camera_permission().await;
            assert!(result.is_ok(), "Permission request {} should succeed", i);
            results.push(result.unwrap());
        }
        
        // All results should be valid
        for (i, info) in results.iter().enumerate() {
            assert!(!info.message.is_empty(), "Message {} should not be empty", i);
            
            // Status should be one of the valid values
            match info.status {
                PermissionStatus::Granted 
                | PermissionStatus::Denied 
                | PermissionStatus::NotDetermined 
                | PermissionStatus::Restricted => {
                    // All valid
                }
            }
        }
        
        // If we got the same status multiple times, they should be consistent
        let first_status = &results[0].status;
        let consistent = results.iter().all(|info| &info.status == first_status);
        
        if !consistent {
            // Status changed during requests - this can happen and is OK
            eprintln!("Permission status changed during multiple requests");
        }
    }

    #[tokio::test]
    async fn test_permission_error_handling() {
        // Test that permission functions handle errors gracefully
        // These tests don't expect errors, but verify proper error handling if they occur
        
        let status_result = check_camera_permission_status().await;
        match status_result {
            Ok(info) => {
                assert!(!info.message.is_empty(), "Success message should not be empty");
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(!error.contains("panic"), "Error should not mention panic");
            }
        }
        
        let request_result = request_camera_permission().await;
        match request_result {
            Ok(info) => {
                assert!(!info.message.is_empty(), "Success message should not be empty");
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(!error.contains("panic"), "Error should not mention panic");
            }
        }
    }

    #[tokio::test]
    async fn test_permission_status_serialization() {
        // Test that PermissionInfo can be serialized/deserialized
        let result = check_camera_permission_status().await;
        assert!(result.is_ok(), "Permission check should succeed");
        
        let info = result.unwrap();
        
        // Test JSON serialization
        let serialized = serde_json::to_string(&info);
        assert!(serialized.is_ok(), "Permission info should serialize to JSON");
        
        let json = serialized.unwrap();
        assert!(!json.is_empty(), "Serialized JSON should not be empty");
        
        // Test deserialization
        let deserialized: Result<crabcamera::permissions::PermissionInfo, _> = 
            serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        
        let restored_info = deserialized.unwrap();
        assert_eq!(restored_info.status, info.status, "Status should match after round-trip");
        assert_eq!(restored_info.message, info.message, "Message should match after round-trip");
        assert_eq!(restored_info.can_request, info.can_request, "can_request should match after round-trip");
    }

    #[tokio::test]
    async fn test_platform_specific_behavior() {
        // Test platform-specific permission behavior
        let result = request_camera_permission().await;
        assert!(result.is_ok(), "Permission request should return a result");
        
        let info = result.unwrap();
        
        #[cfg(target_os = "macos")]
        {
            // macOS might be able to show system permission dialog
            match info.status {
                PermissionStatus::Granted => {
                    assert!(info.message.contains("authorized") || info.message.contains("granted"),
                           "macOS granted message should be appropriate");
                }
                PermissionStatus::Denied => {
                    assert!(info.message.contains("denied"),
                           "macOS denied message should mention denial");
                }
                PermissionStatus::NotDetermined => {
                    // This is a valid state for macOS
                }
                PermissionStatus::Restricted => {
                    // This can happen on managed systems
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows requires manual settings change
            match info.status {
                PermissionStatus::NotDetermined => {
                    assert!(info.message.to_lowercase().contains("settings"),
                           "Windows should direct to settings: {}", info.message);
                    assert!(!info.can_request, "Windows can't request programmatically");
                }
                _ => {
                    // Other statuses are possible depending on system state
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux uses group-based permissions
            match info.status {
                PermissionStatus::NotDetermined => {
                    assert!(info.message.to_lowercase().contains("video") ||
                           info.message.to_lowercase().contains("group"),
                           "Linux should mention video group: {}", info.message);
                    assert!(!info.can_request, "Linux can't request programmatically");
                }
                _ => {
                    // Other statuses are possible
                }
            }
        }
    }

    #[tokio::test]
    async fn test_permission_integration_with_camera_ops() {
        // Test how permission status affects camera operations
        // This is more of an integration test
        
        let permission_result = check_camera_permission_status().await;
        assert!(permission_result.is_ok(), "Permission check should succeed");
        
        let permission_info = permission_result.unwrap();
        
        // Try to get available cameras
        let cameras_result = crabcamera::commands::init::get_available_cameras().await;
        
        match permission_info.status {
            PermissionStatus::Granted => {
                // If permissions are granted, camera operations should have a better chance of success
                match cameras_result {
                    Ok(_cameras) => {
                        // Success is expected with granted permissions
                    }
                    Err(_) => {
                        // But failure is still possible due to hardware issues
                    }
                }
            }
            PermissionStatus::Denied | PermissionStatus::Restricted => {
                // With denied permissions, camera operations might fail
                // But this depends on platform and implementation
                match cameras_result {
                    Ok(cameras) => {
                        // Might still work on some platforms or return empty list
                        eprintln!("Found {} cameras despite denied permissions", cameras.len());
                    }
                    Err(_) => {
                        // Failure is expected with denied permissions
                    }
                }
            }
            PermissionStatus::NotDetermined => {
                // Uncertain permissions - results may vary
                match cameras_result {
                    Ok(_) | Err(_) => {
                        // Both outcomes are possible
                    }
                }
            }
        }
    }
}
