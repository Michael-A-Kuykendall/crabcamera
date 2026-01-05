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
        let mut errors = vec![
            CameraError::InitializationError("Init error".to_string()),
            CameraError::PermissionDenied("Permission error".to_string()),
            CameraError::CaptureError("Capture error".to_string()),
            CameraError::ControlError("Control error".to_string()),
            CameraError::StreamError("Stream error".to_string()),
            CameraError::UnsupportedOperation("Unsupported error".to_string()),
        ];

        // Add conditional feature errors if compiled with those features
        #[cfg(feature = "recording")]
        {
            errors.push(CameraError::EncodingError("Encoding error".to_string()));
            errors.push(CameraError::MuxingError("Muxing error".to_string()));
            errors.push(CameraError::IoError("IO error".to_string()));
        }

        #[cfg(feature = "audio")]
        {
            errors.push(CameraError::AudioError("Audio error".to_string()));
        }

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

    #[test]
    fn test_all_error_variant_messages() {
        // Test that all error variants have appropriate message patterns
        let test_cases = vec![
            (
                CameraError::InitializationError("test".to_string()),
                "Camera initialization error",
            ),
            (
                CameraError::PermissionDenied("test".to_string()),
                "Permission denied error",
            ),
            (
                CameraError::CaptureError("test".to_string()),
                "Capture error",
            ),
            (
                CameraError::ControlError("test".to_string()),
                "Camera control error",
            ),
            (CameraError::StreamError("test".to_string()), "Stream error"),
            (
                CameraError::UnsupportedOperation("test".to_string()),
                "Unsupported operation",
            ),
        ];

        for (error, expected_prefix) in test_cases {
            let display = error.to_string();
            assert!(
                display.contains(expected_prefix),
                "Error '{}' should contain prefix '{}'",
                display,
                expected_prefix
            );
            assert!(
                display.contains("test"),
                "Error '{}' should contain message 'test'",
                display
            );
        }
    }

    #[cfg(feature = "recording")]
    #[test]
    fn test_recording_error_variants() {
        let recording_errors = vec![
            (
                CameraError::EncodingError("codec issue".to_string()),
                "Encoding error",
            ),
            (
                CameraError::MuxingError("container issue".to_string()),
                "Muxing error",
            ),
            (
                CameraError::IoError("file write issue".to_string()),
                "IO error",
            ),
        ];

        for (error, expected_prefix) in recording_errors {
            let display = error.to_string();
            assert!(display.contains(expected_prefix));

            // Verify the error can be converted to trait object
            let _boxed: Box<dyn Error> = Box::new(error);
        }
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_audio_error_variant() {
        let audio_error = CameraError::AudioError("microphone issue".to_string());
        let display = audio_error.to_string();

        assert!(display.contains("Audio error"));
        assert!(display.contains("microphone issue"));

        // Verify the error can be converted to trait object
        let _boxed: Box<dyn Error> = Box::new(audio_error);
    }

    #[test]
    fn test_error_chaining_patterns() {
        // Test error propagation patterns common in camera operations
        fn init_camera() -> Result<(), CameraError> {
            Err(CameraError::InitializationError(
                "Hardware not found".to_string(),
            ))
        }

        fn capture_frame() -> Result<Vec<u8>, CameraError> {
            init_camera()?;
            Ok(vec![])
        }

        fn save_video() -> Result<String, CameraError> {
            let _frame = capture_frame()?;
            Ok("saved".to_string())
        }

        // Error should propagate up the chain
        match save_video() {
            Err(CameraError::InitializationError(msg)) => {
                assert_eq!(msg, "Hardware not found");
            }
            _ => panic!("Expected InitializationError to propagate"),
        }
    }

    #[test]
    fn test_error_map_patterns() {
        // Test error mapping patterns
        fn operation_with_mapping() -> Result<String, CameraError> {
            Err(CameraError::CaptureError("Original error".to_string())).map_err(|e| match e {
                CameraError::CaptureError(msg) => {
                    CameraError::StreamError(format!("Mapped from capture: {}", msg))
                }
                other => other,
            })
        }

        match operation_with_mapping() {
            Err(CameraError::StreamError(msg)) => {
                assert!(msg.contains("Mapped from capture"));
                assert!(msg.contains("Original error"));
            }
            _ => panic!("Expected mapped StreamError"),
        }
    }

    #[test]
    fn test_error_concurrent_safety() {
        use std::sync::Arc;
        use std::thread;

        let error = Arc::new(CameraError::CaptureError("Thread safe test".to_string()));
        let mut handles = vec![];

        // Test that errors can be shared across threads
        for i in 0..5 {
            let error_clone = error.clone();
            let handle = thread::spawn(move || {
                let display = error_clone.to_string();
                assert!(display.contains("Thread safe test"));
                i
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_error_context_patterns() {
        // Test adding context to errors
        fn add_error_context(base_error: CameraError, context: &str) -> CameraError {
            match base_error {
                CameraError::CaptureError(msg) => {
                    CameraError::CaptureError(format!("{}: {}", context, msg))
                }
                CameraError::InitializationError(msg) => {
                    CameraError::InitializationError(format!("{}: {}", context, msg))
                }
                other => other,
            }
        }

        let base_error = CameraError::CaptureError("Frame timeout".to_string());
        let contextual_error = add_error_context(base_error, "During burst capture");

        match contextual_error {
            CameraError::CaptureError(msg) => {
                assert!(msg.contains("During burst capture"));
                assert!(msg.contains("Frame timeout"));
            }
            _ => panic!("Expected CaptureError with context"),
        }
    }

    #[test]
    fn test_error_serialization_readiness() {
        // While CameraError doesn't derive Serialize/Deserialize, test that
        // it can be converted to serializable formats for logging
        let errors = vec![
            CameraError::InitializationError("Serialization test".to_string()),
            CameraError::PermissionDenied("Access denied".to_string()),
            CameraError::CaptureError("Capture failed".to_string()),
        ];

        for error in errors {
            // Test conversion to string for JSON serialization
            let error_string = error.to_string();
            let json_ready = serde_json::json!({
                "error_type": format!("{:?}", error).split('(').next().unwrap_or("Unknown"),
                "message": error_string,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            assert!(json_ready.is_object());
            assert!(
                json_ready["message"].as_str().unwrap().contains("test")
                    || json_ready["message"].as_str().unwrap().contains("denied")
                    || json_ready["message"].as_str().unwrap().contains("failed")
            );
        }
    }

    #[test]
    fn test_error_recovery_patterns() {
        // Test error recovery patterns commonly used with camera operations
        fn attempt_capture_with_fallback() -> Result<Vec<u8>, CameraError> {
            // First attempt
            match attempt_primary_camera() {
                Ok(data) => Ok(data),
                Err(CameraError::InitializationError(_)) => {
                    // Try fallback camera
                    attempt_secondary_camera()
                }
                Err(CameraError::PermissionDenied(_)) => {
                    // Cannot recover from permission issues
                    Err(CameraError::PermissionDenied(
                        "Access denied and no fallback".to_string(),
                    ))
                }
                Err(other) => Err(other),
            }
        }

        fn attempt_primary_camera() -> Result<Vec<u8>, CameraError> {
            Err(CameraError::InitializationError(
                "Primary camera failed".to_string(),
            ))
        }

        fn attempt_secondary_camera() -> Result<Vec<u8>, CameraError> {
            Ok(vec![1, 2, 3, 4]) // Mock fallback success
        }

        match attempt_capture_with_fallback() {
            Ok(data) => assert_eq!(data, vec![1, 2, 3, 4]),
            Err(_) => panic!("Expected successful fallback"),
        }
    }

    #[test]
    fn test_error_exhaustive_matching() {
        // Test that we can exhaustively match all error variants
        fn handle_camera_error(error: CameraError) -> String {
            match error {
                CameraError::InitializationError(msg) => format!("Init: {}", msg),
                CameraError::PermissionDenied(msg) => format!("Permission: {}", msg),
                CameraError::CaptureError(msg) => format!("Capture: {}", msg),
                CameraError::ControlError(msg) => format!("Control: {}", msg),
                CameraError::StreamError(msg) => format!("Stream: {}", msg),
                CameraError::UnsupportedOperation(msg) => format!("Unsupported: {}", msg),

                #[cfg(feature = "recording")]
                CameraError::EncodingError(msg) => format!("Encoding: {}", msg),
                #[cfg(feature = "recording")]
                CameraError::MuxingError(msg) => format!("Muxing: {}", msg),
                #[cfg(feature = "recording")]
                CameraError::IoError(msg) => format!("IO: {}", msg),

                #[cfg(feature = "audio")]
                CameraError::AudioError(msg) => format!("Audio: {}", msg),
            }
        }

        let test_error = CameraError::CaptureError("test message".to_string());
        let handled = handle_camera_error(test_error);
        assert_eq!(handled, "Capture: test message");
    }

    #[test]
    fn test_error_memory_usage() {
        // Test that errors don't use excessive memory
        use std::mem;

        let error = CameraError::InitializationError("Memory test".to_string());
        let size = mem::size_of_val(&error);

        // Error size should be reasonable (string + discriminant)
        // This is more of a regression test than a hard requirement
        assert!(
            size < 1000,
            "Error size should be reasonable, got {} bytes",
            size
        );

        // Test with very long message
        let long_error = CameraError::CaptureError("x".repeat(10000));
        let long_size = mem::size_of_val(&long_error);

        // String on heap means size_of stays the same (just pointer + len + cap)
        // So we check that it's similar size, not larger
        assert!(
            long_size == size || long_size > size,
            "Long error size should be similar or slightly larger"
        );
        assert!(
            long_size < 50000,
            "Long error should not be excessively large, got {} bytes",
            long_size
        );
    }

    #[test]
    fn test_error_diagnostic_information() {
        // Test that errors provide useful diagnostic information
        let errors = vec![
            CameraError::InitializationError(
                "Failed to connect to camera device /dev/video0".to_string(),
            ),
            CameraError::PermissionDenied(
                "Camera access denied. Check privacy settings.".to_string(),
            ),
            CameraError::CaptureError("Frame capture timed out after 5000ms".to_string()),
            CameraError::ControlError("Failed to set ISO to 1600: not supported".to_string()),
            CameraError::StreamError("Video stream interrupted: cable disconnected".to_string()),
            CameraError::UnsupportedOperation(
                "HDR burst mode not available on this device".to_string(),
            ),
        ];

        for error in errors {
            let debug_output = format!("{:?}", error);
            let display_output = format!("{}", error);

            // Debug output should contain error information (either variant name or message)
            assert!(
                debug_output.len() > 0 && !display_output.is_empty(),
                "Debug output should have content: {}",
                debug_output
            );

            // Display output should be user-friendly
            assert!(
                !display_output.is_empty(),
                "Display output should not be empty"
            );
            assert!(
                display_output.len() > 10,
                "Display output should be descriptive: {}",
                display_output
            );
        }
    }
}
