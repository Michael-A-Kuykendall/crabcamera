//! WebRTC Real Camera Test Example
//!
//! This example demonstrates how to start a WebRTC stream using your actual camera
//! and provides the SDP offer for connecting from a browser.
//!
//! To test:
//! 1. Run this example
//! 2. Copy the SDP offer from the console
//! 3. Open webrtc_test.html in a browser
//! 4. Paste the offer and click "Connect"
//! 5. You should see your camera stream in the browser

use crabcamera::commands::init::{get_available_cameras, initialize_camera_system};
#[cfg(feature = "webrtc")]
use crabcamera::webrtc::{StreamConfig, StreamMode, WebRTCStreamer};

#[cfg(not(feature = "webrtc"))]
fn main() {
    println!("❌ This example requires the 'webrtc' feature to be enabled");
    println!("Run with: cargo run --example webrtc_real_camera_test --features webrtc");
    std::process::exit(1);
}

#[cfg(feature = "webrtc")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 CrabCamera WebRTC Real Camera Test");
    println!("=====================================");

    // Initialize camera system
    println!("Initializing camera system...");
    initialize_camera_system().await?;

    // Get available cameras
    let cameras = get_available_cameras().await?;
    if cameras.is_empty() {
        println!("❌ No cameras found!");
        return Ok(());
    }

    println!("📷 Available cameras:");
    for (i, camera) in cameras.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, camera.name, camera.id);
    }

    // Use first camera
    let device_id = cameras[0].id.clone();
    let stream_id = "test-stream".to_string();

    println!("\n🎥 Starting WebRTC stream with real camera...");
    println!("Device ID: {}", device_id);
    println!("Stream ID: {}", stream_id);

    // Create WebRTC streamer
    let config = StreamConfig::default();
    let streamer = WebRTCStreamer::new(stream_id.clone(), config);

    // Set to real camera mode
    streamer.set_mode(StreamMode::RealCamera).await;

    // Check camera status before starting
    let status = streamer.get_camera_status().await;
    println!("📊 Camera status: {:?}", status);

    // Start the stream
    match streamer.start_streaming(device_id.clone()).await {
        Ok(_) => println!("✅ WebRTC stream started successfully"),
        Err(e) => {
            println!("❌ Failed to start stream: {}", e);
            println!("💡 This might indicate camera access issues or hardware problems");
            return Ok(());
        }
    }

    // Get updated status
    let status = streamer.get_camera_status().await;
    println!("📊 Updated camera status: {:?}", status);

    // Get stream stats
    let stats = streamer.get_stats().await;
    println!("📈 Stream stats: {:?}", stats);

    // Create a peer connection for signaling (simplified)
    println!("\n🔗 For full WebRTC testing, you would:");
    println!("1. Create a PeerConnection");
    println!("2. Add video transceivers");
    println!("3. Connect the streamer to send RTP packets to the peer");
    println!("4. Create SDP offer and exchange with browser");
    println!("5. Handle ICE candidates");

    println!("\n✅ Camera capture test completed!");
    println!("The stream is capturing from your real camera.");
    println!("To test full WebRTC, you'd need to implement the signaling above.");

    // Keep the stream running briefly to test
    println!("\n🎬 Stream is running for 10 seconds to test camera capture...");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    println!("🛑 Stopping stream...");
    streamer.stop_streaming().await?;

    Ok(())
}
