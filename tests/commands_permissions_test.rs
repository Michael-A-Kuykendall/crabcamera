#[cfg(test)]
mod commands_permissions_tests {
    use crabcamera::commands::permissions::{
        check_camera_permission_status, request_camera_permission,
    };
    use crabcamera::permissions::PermissionStatus;

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
}
