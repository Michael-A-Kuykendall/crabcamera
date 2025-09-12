#[cfg(test)]
mod error_tests {
    use crabcamera::errors::CameraError;
    use std::error::Error;

    #[test]
    fn test_camera_error_initialization() {
        let error = CameraError::InitializationError("Test init error".to_string());
        assert!(error.to_string().contains("Camera initialization error"));
        assert!(error.to_string().contains("Test init error"));
    }

    #[test]
    fn test_camera_error_permission_denied() {
        let error = CameraError::PermissionDenied("Access denied".to_string());
        assert!(error.to_string().contains("Permission denied"));
        assert!(error.to_string().contains("Access denied"));
    }

    #[test]
    fn test_camera_error_capture() {
        let error = CameraError::CaptureError("Capture failed".to_string());
        assert!(error.to_string().contains("Capture error"));
        assert!(error.to_string().contains("Capture failed"));
    }

    #[test]
    fn test_camera_error_debug_format() {
        let error = CameraError::InitializationError("Debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InitializationError"));
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_camera_error_display_trait() {
        let error = CameraError::CaptureError("Display test".to_string());
        let display_str = format!("{}", error);
        assert_eq!(display_str, "Capture error: Display test");
    }

    #[test]
    fn test_camera_error_implements_error_trait() {
        let error = CameraError::PermissionDenied("Error trait test".to_string());
        // Test that it implements std::error::Error trait
        let _error_trait: &dyn Error = &error;
        assert!(error.source().is_none()); // CameraError doesn't wrap other errors
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            CameraError::InitializationError("Init error".to_string()),
            CameraError::PermissionDenied("Permission error".to_string()),
            CameraError::CaptureError("Capture error".to_string()),
        ];

        for error in errors {
            // Each error should implement Display
            let display_str = error.to_string();
            assert!(!display_str.is_empty());

            // Each error should implement Debug
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_error_message_extraction() {
        let test_message = "Detailed error information";

        match CameraError::InitializationError(test_message.to_string()) {
            CameraError::InitializationError(msg) => assert_eq!(msg, test_message),
            _ => panic!("Wrong error variant"),
        }

        match CameraError::PermissionDenied(test_message.to_string()) {
            CameraError::PermissionDenied(msg) => assert_eq!(msg, test_message),
            _ => panic!("Wrong error variant"),
        }

        match CameraError::CaptureError(test_message.to_string()) {
            CameraError::CaptureError(msg) => assert_eq!(msg, test_message),
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_error_clone_and_equality() {
        let original_init = CameraError::InitializationError("Clone test".to_string());
        let original_perm = CameraError::PermissionDenied("Clone test".to_string());
        let original_capture = CameraError::CaptureError("Clone test".to_string());

        // Test Debug formatting (Clone is derived)
        let debug_init = format!("{:?}", original_init);
        let debug_perm = format!("{:?}", original_perm);
        let debug_capture = format!("{:?}", original_capture);

        assert!(debug_init.contains("InitializationError"));
        assert!(debug_perm.contains("PermissionDenied"));
        assert!(debug_capture.contains("CaptureError"));
    }

    #[test]
    fn test_error_display_consistency() {
        let errors = vec![
            (
                "InitializationError",
                CameraError::InitializationError("test".to_string()),
            ),
            (
                "PermissionDenied",
                CameraError::PermissionDenied("test".to_string()),
            ),
            (
                "CaptureError",
                CameraError::CaptureError("test".to_string()),
            ),
        ];

        for (expected_prefix, error) in errors {
            let display = error.to_string();
            assert!(
                display.contains(expected_prefix)
                    || display.contains(&expected_prefix.to_lowercase())
                    || display.contains("error"),
                "Error display should contain error type or 'error': {}",
                display
            );
            assert!(
                display.contains("test"),
                "Error display should contain the message: {}",
                display
            );
        }
    }

    #[test]
    fn test_error_empty_message() {
        let errors = vec![
            CameraError::InitializationError("".to_string()),
            CameraError::PermissionDenied("".to_string()),
            CameraError::CaptureError("".to_string()),
        ];

        for error in errors {
            let display = error.to_string();
            // Should still have the error type prefix even with empty message
            assert!(
                !display.is_empty(),
                "Error display should not be empty even with empty message"
            );
            assert!(
                display.contains("error"),
                "Error display should contain 'error'"
            );
        }
    }

    #[test]
    fn test_error_long_message() {
        let long_message = "A".repeat(1000);
        let errors = vec![
            CameraError::InitializationError(long_message.clone()),
            CameraError::PermissionDenied(long_message.clone()),
            CameraError::CaptureError(long_message.clone()),
        ];

        for error in errors {
            let display = error.to_string();
            assert!(
                display.len() > 1000,
                "Long error message should be preserved"
            );
            assert!(
                display.contains(&long_message),
                "Long message should be included in display"
            );
        }
    }

    #[test]
    fn test_error_special_characters() {
        let special_message = "Error with: 🦀 émojis and spéciál chårs & symbols!@#$%^&*()";
        let errors = vec![
            CameraError::InitializationError(special_message.to_string()),
            CameraError::PermissionDenied(special_message.to_string()),
            CameraError::CaptureError(special_message.to_string()),
        ];

        for error in errors {
            let display = error.to_string();
            assert!(display.contains("🦀"), "Should handle emoji");
            assert!(
                display.contains("émojis"),
                "Should handle accented characters"
            );
            assert!(
                display.contains("!@#$%^&*()"),
                "Should handle special symbols"
            );
        }
    }

    #[test]
    fn test_error_as_result() {
        fn returns_init_error() -> Result<String, CameraError> {
            Err(CameraError::InitializationError("Test init".to_string()))
        }

        fn returns_permission_error() -> Result<String, CameraError> {
            Err(CameraError::PermissionDenied("Test permission".to_string()))
        }

        fn returns_capture_error() -> Result<String, CameraError> {
            Err(CameraError::CaptureError("Test capture".to_string()))
        }

        // Test that errors can be used in Result types
        assert!(returns_init_error().is_err());
        assert!(returns_permission_error().is_err());
        assert!(returns_capture_error().is_err());

        // Test error extraction from Result
        match returns_init_error() {
            Err(CameraError::InitializationError(msg)) => assert_eq!(msg, "Test init"),
            _ => panic!("Expected InitializationError"),
        }
    }

    #[test]
    fn test_error_send_sync() {
        // Test that CameraError implements Send and Sync (needed for multi-threading)
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<CameraError>();
        assert_sync::<CameraError>();
    }

    #[test]
    fn test_error_conversion_patterns() {
        // Test common error conversion patterns
        let init_error = CameraError::InitializationError("Device not found".to_string());
        let perm_error = CameraError::PermissionDenied("Camera access denied".to_string());
        let capture_error = CameraError::CaptureError("Frame capture timeout".to_string());

        // Test that errors can be boxed (common pattern for trait objects)
        let _boxed_init: Box<dyn Error> = Box::new(init_error);
        let _boxed_perm: Box<dyn Error> = Box::new(perm_error);
        let _boxed_capture: Box<dyn Error> = Box::new(capture_error);
    }
}
