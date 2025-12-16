//! Tests for the recording module

#[cfg(test)]
mod recording_tests {
    use crate::recording::{Recorder, RecordingConfig, RecordingQuality};
    use std::env::temp_dir;

    #[test]
    fn test_quality_presets() {
        assert_eq!(RecordingQuality::Low.resolution(), (1280, 720));
        assert_eq!(RecordingQuality::Medium.resolution(), (1920, 1080));
        assert_eq!(RecordingQuality::High.resolution(), (1920, 1080));
    }

    #[test]
    fn test_config_from_quality() {
        let config = RecordingConfig::from_quality(RecordingQuality::High);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.fps, 30.0);
    }

    #[test]
    fn test_config_with_title() {
        let config = RecordingConfig::from_quality(RecordingQuality::Medium)
            .with_title("My Recording");
        assert_eq!(config.title, Some("My Recording".to_string()));
    }

    #[test]
    fn test_recording_workflow() {
        let output = temp_dir().join("test_workflow.mp4");
        let config = RecordingConfig::new(320, 240, 15.0)
            .with_title("Integration Test");

        let mut recorder = Recorder::new(&output, config)
            .expect("Failed to create recorder");

        // Record 15 frames (1 second at 15fps)
        for _ in 0..15 {
            let rgb = vec![100u8; 320 * 240 * 3];
            recorder.write_rgb_frame(&rgb, 320, 240)
                .expect("Failed to write frame");
        }

        assert_eq!(recorder.frame_count(), 15);
        assert!(recorder.is_recording());

        let stats = recorder.finish().expect("Failed to finish");

        assert_eq!(stats.video_frames, 15);
        assert!(stats.bytes_written > 0);
        
        // Verify file exists
        assert!(std::fs::metadata(&output).is_ok());

        // Clean up
        let _ = std::fs::remove_file(&output);
    }
}
