#[cfg(test)]
mod commands_permissions_tests {
    use crabcamera::commands::permissions::{
        check_camera_permission_status, request_camera_permission,
    };

    #[tokio::test]
    async fn test_request_camera_permission_success() {
        let result = request_camera_permission().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Permission granted");
    }

    #[tokio::test]
    async fn test_check_camera_permission_status_granted() {
        let result = check_camera_permission_status().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Granted");
    }

    #[tokio::test]
    async fn test_permission_functions_are_consistent() {
        // Test multiple calls to ensure consistent behavior
        for _ in 0..3 {
            let request_result = request_camera_permission().await;
            let status_result = check_camera_permission_status().await;

            assert!(request_result.is_ok());
            assert!(status_result.is_ok());

            assert_eq!(request_result.unwrap(), "Permission granted");
            assert_eq!(status_result.unwrap(), "Granted");
        }
    }

    #[tokio::test]
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
            assert_eq!(result.unwrap(), "Permission granted");
        }
    }

    #[tokio::test]
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
            assert_eq!(result.unwrap(), "Granted");
        }
    }
}
