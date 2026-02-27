#[cfg(all(test, target_os = "windows"))]
mod tests {
    use crabcamera::platform::windows::controls::MediaFoundationControls;

    #[test]
    fn test_media_foundation_initialization() {
        // This test attempts to initialize MediaFoundation controls for the default camera (index 0).
        // It validates that our new implementation logic runs without crashing.
        // Whether it finds a camera or not depends on the hardware, but it shouldn't panic or error out with "Not Implemented".
        
        println!("Attempting to initialize MediaFoundation controls...");
        match MediaFoundationControls::new(0) {
            Ok(controls) => {
                println!("Successfully initialized controls for device 0.");
                
                // Test capabilities query
                match controls.get_capabilities() {
                    Ok(caps) => println!("Capabilities: {:?}", caps),
                    Err(e) => println!("Error getting capabilities: {:?}", e),
                }

                // Test getting current controls
                match controls.get_controls() {
                    Ok(current) => println!("Current controls: {:?}", current),
                    Err(e) => println!("Error getting controls: {:?}", e),
                }
            },
            Err(e) => {
                println!("Could not initialize controls (expected if no camera or locked): {:?}", e);
                // Verify it's NOT the old "Not implemented" error
                let critical_failure = format!("{:?}", e).contains("not yet implemented");
                assert!(!critical_failure, "The implementation is still returning 'not yet implemented'!");
            }
        }
    }
}
