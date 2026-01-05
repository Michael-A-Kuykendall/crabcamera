use crabcamera::commands::capture::save_frame_compressed;
use crabcamera::types::CameraFrame;

#[tokio::test]
async fn save_frame_compressed_respects_quality() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Use a smooth gradient so JPEG compression has predictable effects.
    let width: u32 = 256;
    let height: u32 = 256;
    let mut data = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            let r = (x & 0xFF) as u8;
            let g = (y & 0xFF) as u8;
            let b = (((x + y) / 2) & 0xFF) as u8;
            data.extend_from_slice(&[r, g, b]);
        }
    }

    let frame = CameraFrame::new(data, width, height, "test_device".to_string());

    let low_path = dir.path().join("low_q10.jpg");
    let high_path = dir.path().join("high_q95.jpg");

    save_frame_compressed(
        frame.clone(),
        low_path.to_string_lossy().to_string(),
        Some(10),
    )
    .await
    .expect("save low quality");

    save_frame_compressed(frame, high_path.to_string_lossy().to_string(), Some(95))
        .await
        .expect("save high quality");

    let low_size = std::fs::metadata(&low_path).expect("metadata low").len();
    let high_size = std::fs::metadata(&high_path).expect("metadata high").len();

    assert!(low_size > 0, "low-quality output should not be empty");
    assert!(high_size > 0, "high-quality output should not be empty");

    // Higher quality should generally produce larger output.
    assert!(
        high_size > low_size,
        "expected high quality JPEG to be larger (low={}, high={})",
        low_size,
        high_size
    );
}
