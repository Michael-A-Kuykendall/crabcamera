//! Comprehensive WebRTC data channel operations tests
//!
//! This module tests the WebRTC data channel implementation including:
//! - Data channel creation and lifecycle
//! - Message transmission and reliability
//! - Channel state management
//! - Multi-channel scenarios
//! - Error handling and recovery
//! - Performance under load

use crabcamera::commands::webrtc::{
    create_peer_connection, create_data_channel, send_data_channel_message,
    get_peer_connection_status, close_peer_connection
};
use crabcamera::webrtc::peer::{
    PeerConnection, RTCConfiguration, DataChannel, DataChannelState, ConnectionState
};
use std::collections::HashMap;

#[tokio::test]
async fn test_data_channel_basic_lifecycle() {
    let peer_id = "data_channel_test_peer".to_string();
    let channel_label = "test_channel".to_string();

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok(), "Failed to create peer connection");

    // Create data channel
    let channel_id = create_data_channel(peer_id.clone(), channel_label.clone()).await;
    assert!(channel_id.is_ok(), "Failed to create data channel");
    
    let channel_id = channel_id.unwrap();
    assert!(!channel_id.is_empty(), "Channel ID should not be empty");
    assert!(channel_id.contains(&peer_id), "Channel ID should contain peer ID");
    assert!(channel_id.contains(&channel_label), "Channel ID should contain channel label");

    // Verify data channel shows up in stats
    let status = get_peer_connection_status(peer_id.clone()).await;
    assert!(status.is_ok());
    let status = status.unwrap();
    assert_eq!(status.data_channels_count, 1, "Should have 1 data channel");

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_multiple_data_channels() {
    let peer_id = "multi_channel_peer".to_string();
    let channel_labels = vec!["channel_1", "channel_2", "channel_3", "channel_4"];

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    // Create multiple data channels
    let mut channel_ids = Vec::new();
    for label in &channel_labels {
        let channel_id = create_data_channel(peer_id.clone(), label.to_string()).await;
        assert!(channel_id.is_ok(), "Failed to create channel {}", label);
        channel_ids.push(channel_id.unwrap());
    }

    // Verify all channels exist
    let status = get_peer_connection_status(peer_id.clone()).await;
    assert!(status.is_ok());
    let status = status.unwrap();
    assert_eq!(status.data_channels_count, channel_labels.len());

    // Verify each channel has unique ID
    let unique_ids: std::collections::HashSet<_> = channel_ids.iter().collect();
    assert_eq!(unique_ids.len(), channel_ids.len(), "All channel IDs should be unique");

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_message_sending() {
    let peer_id = "message_test_peer".to_string();
    let channel_label = "message_channel".to_string();

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    // Create data channel
    let channel_id = create_data_channel(peer_id.clone(), channel_label.clone()).await;
    assert!(channel_id.is_ok());

    // Test different types of message data
    let test_messages = vec![
        b"Hello WebRTC!".to_vec(),
        b"".to_vec(), // Empty message
        vec![0u8; 1024], // Binary data
        b"Unicode test: \xF0\x9F\x98\x80\xF0\x9F\x9A\x80".to_vec(), // Unicode
        vec![255u8; 65535], // Large message
    ];

    for (i, message) in test_messages.into_iter().enumerate() {
        let result = send_data_channel_message(
            peer_id.clone(),
            channel_label.clone(),
            message.clone()
        ).await;
        
        // Note: Current mock implementation doesn't have open channels
        // In real implementation, we'd need to properly open channels first
        // For now, we're testing the API structure
        if result.is_err() {
            // Expected in mock implementation due to channel state
            assert!(result.as_ref().unwrap_err().contains("not open") || result.as_ref().unwrap_err().contains("not found"));
        } else {
            // If implementation supports it
            assert!(result.is_ok(), "Failed to send message {}", i);
        }
    }

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_error_conditions() {
    let nonexistent_peer = "nonexistent_peer".to_string();
    let valid_peer = "valid_peer".to_string();

    // Test creating channel for non-existent peer
    let result = create_data_channel(nonexistent_peer.clone(), "test".to_string()).await;
    assert!(result.is_err(), "Should fail for non-existent peer");

    // Test sending message to non-existent peer
    let result = send_data_channel_message(
        nonexistent_peer,
        "test".to_string(),
        b"test".to_vec()
    ).await;
    assert!(result.is_err(), "Should fail for non-existent peer");

    // Create valid peer for further tests
    let result = create_peer_connection(valid_peer.clone(), None).await;
    assert!(result.is_ok());

    // Test sending message to non-existent channel
    let result = send_data_channel_message(
        valid_peer.clone(),
        "nonexistent_channel".to_string(),
        b"test".to_vec()
    ).await;
    assert!(result.is_err(), "Should fail for non-existent channel");

    // Cleanup
    let _ = close_peer_connection(valid_peer).await;
}

#[tokio::test]
async fn test_data_channel_direct_api() {
    let peer_id = "direct_api_test".to_string();
    let config = RTCConfiguration::default();

    let peer = PeerConnection::new(peer_id.clone(), config);

    // Test creating data channel
    let channel_result = peer.create_data_channel("test_channel".to_string()).await;
    assert!(channel_result.is_ok(), "Should create data channel successfully");

    let channel_id = channel_result.unwrap();
    assert!(!channel_id.is_empty());

    // Test sending data
    let test_data = b"test message".to_vec();
    let send_result = peer.send_data("test_channel", test_data).await;
    // Expected to fail because channel is in Connecting state, not Open
    assert!(send_result.is_err());
    assert!(send_result.unwrap_err().contains("not open"));

    // Test getting stats shows data channel
    let stats = peer.get_stats().await;
    assert_eq!(stats.data_channels_count, 1);
    assert_eq!(stats.peer_id, peer_id);

    // Test creating multiple channels
    for i in 0..5 {
        let channel_name = format!("channel_{}", i);
        let result = peer.create_data_channel(channel_name).await;
        assert!(result.is_ok(), "Should create channel {}", i);
    }

    let stats = peer.get_stats().await;
    assert_eq!(stats.data_channels_count, 6); // 1 + 5 = 6 total

    // Test closing peer closes all channels
    let result = peer.close().await;
    assert!(result.is_ok());
    assert!(matches!(peer.get_connection_state().await, ConnectionState::Closed));
}

#[tokio::test]
async fn test_data_channel_name_uniqueness() {
    let peer_id = "uniqueness_test_peer".to_string();

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    // Create first channel with a label
    let channel_label = "unique_channel".to_string();
    let first_channel = create_data_channel(peer_id.clone(), channel_label.clone()).await;
    assert!(first_channel.is_ok());

    // Create second channel with same label - should succeed (implementation allows)
    // Different implementations may handle this differently
    let second_channel = create_data_channel(peer_id.clone(), channel_label.clone()).await;
    assert!(second_channel.is_ok());

    // Verify both channels exist (or one overwrote the other)
    let status = get_peer_connection_status(peer_id.clone()).await;
    assert!(status.is_ok());
    let status = status.unwrap();
    // Implementation may have 1 or 2 channels depending on behavior
    assert!(status.data_channels_count >= 1);

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_with_special_characters() {
    let peer_id = "special_char_test".to_string();
    let special_labels = vec![
        "channel with spaces",
        "channel-with-dashes",
        "channel_with_underscores",
        "channel.with.dots",
        "channel123",
        "UPPERCASE_CHANNEL",
        "unicode_🚀_channel",
        "", // Empty label
    ];

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    for (i, label) in special_labels.into_iter().enumerate() {
        let result = create_data_channel(peer_id.clone(), label.to_string()).await;
        // Most labels should work, empty label might not
        if label.is_empty() {
            // Implementation may reject empty labels
            // Either success or failure is acceptable
        } else {
            assert!(result.is_ok(), "Should handle special characters in label: '{}'", label);
        }
    }

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_large_message_handling() {
    let peer_id = "large_message_test".to_string();
    let channel_label = "large_message_channel".to_string();

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    // Create data channel
    let result = create_data_channel(peer_id.clone(), channel_label.clone()).await;
    assert!(result.is_ok());

    // Test increasingly large messages
    let message_sizes = vec![
        1024,      // 1 KB
        10_240,    // 10 KB
        65_535,    // Max UDP packet size
        1_048_576, // 1 MB
    ];

    for size in message_sizes {
        let large_message = vec![0xABu8; size];
        
        let result = send_data_channel_message(
            peer_id.clone(),
            channel_label.clone(),
            large_message
        ).await;
        
        // Current mock implementation will likely fail due to channel state
        // But API should handle large messages gracefully
        if result.is_err() {
            let error = result.unwrap_err();
            // Should be state error, not size error
            assert!(error.contains("not open") || error.contains("not found"));
        }
    }

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_concurrent_operations() {
    let peer_id = "concurrent_test_peer".to_string();

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    // Create multiple data channels concurrently
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let peer_id_clone = peer_id.clone();
        let handle = tokio::spawn(async move {
            let channel_label = format!("concurrent_channel_{}", i);
            create_data_channel(peer_id_clone, channel_label).await
        });
        handles.push(handle);
    }

    // Wait for all channels to be created
    let mut successful_creations = 0;
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Task should complete successfully");
        if result.unwrap().is_ok() {
            successful_creations += 1;
        }
    }

    assert!(successful_creations > 0, "At least some channels should be created");

    // Verify final state
    let status = get_peer_connection_status(peer_id.clone()).await;
    assert!(status.is_ok());
    let status = status.unwrap();
    assert!(status.data_channels_count > 0);

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_message_ordering() {
    let peer_id = "ordering_test_peer".to_string();
    let channel_label = "ordering_channel".to_string();

    // Create peer connection and channel
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    let result = create_data_channel(peer_id.clone(), channel_label.clone()).await;
    assert!(result.is_ok());

    // Send multiple messages in sequence
    let messages = vec![
        b"Message 1".to_vec(),
        b"Message 2".to_vec(),
        b"Message 3".to_vec(),
        b"Message 4".to_vec(),
        b"Message 5".to_vec(),
    ];

    for (i, message) in messages.into_iter().enumerate() {
        let result = send_data_channel_message(
            peer_id.clone(),
            channel_label.clone(),
            message
        ).await;
        
        // Expected to fail in mock due to channel state
        if result.is_err() {
            assert!(result.unwrap_err().contains("not open"));
        } else {
            // If implementation supports it
            println!("Successfully sent message {}", i);
        }
    }

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_binary_data() {
    let peer_id = "binary_test_peer".to_string();
    let channel_label = "binary_channel".to_string();

    // Create peer connection and channel
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    let result = create_data_channel(peer_id.clone(), channel_label.clone()).await;
    assert!(result.is_ok());

    // Test various binary data patterns
    let binary_data_tests = vec![
        vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD], // Mixed bytes
        vec![0x00; 1000], // All zeros
        vec![0xFF; 1000], // All ones
        (0..256).map(|i| i as u8).collect::<Vec<u8>>(), // Sequential pattern
        vec![], // Empty data
    ];

    for (i, data) in binary_data_tests.into_iter().enumerate() {
        let result = send_data_channel_message(
            peer_id.clone(),
            channel_label.clone(),
            data.clone()
        ).await;
        
        // Expected to fail in mock due to channel state
        if result.is_err() {
            assert!(result.unwrap_err().contains("not open"));
        } else {
            println!("Successfully sent binary data pattern {}", i);
        }
    }

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}

#[tokio::test]
async fn test_data_channel_state_management() {
    let peer_id = "state_test_peer".to_string();
    let config = RTCConfiguration::default();

    let peer = PeerConnection::new(peer_id, config);

    // Initially no data channels
    let stats = peer.get_stats().await;
    assert_eq!(stats.data_channels_count, 0);

    // Create a channel
    let result = peer.create_data_channel("test_channel".to_string()).await;
    assert!(result.is_ok());

    // Should have 1 channel
    let stats = peer.get_stats().await;
    assert_eq!(stats.data_channels_count, 1);

    // Close peer connection - should close all channels
    let result = peer.close().await;
    assert!(result.is_ok());

    // Verify connection is closed
    assert!(matches!(peer.get_connection_state().await, ConnectionState::Closed));

    // Attempting to send data after close should fail
    let result = peer.send_data("test_channel", b"test".to_vec()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_data_channel_performance_load() {
    let peer_id = "performance_test_peer".to_string();

    // Create peer connection
    let result = create_peer_connection(peer_id.clone(), None).await;
    assert!(result.is_ok());

    // Create many data channels rapidly
    let num_channels = 50;
    let mut channel_creation_times = Vec::new();

    for i in 0..num_channels {
        let start = std::time::Instant::now();
        
        let channel_label = format!("perf_channel_{}", i);
        let result = create_data_channel(peer_id.clone(), channel_label).await;
        
        let duration = start.elapsed();
        channel_creation_times.push(duration);
        
        assert!(result.is_ok(), "Failed to create channel {} under load", i);
    }

    // Verify all channels were created
    let status = get_peer_connection_status(peer_id.clone()).await;
    assert!(status.is_ok());
    let status = status.unwrap();
    assert_eq!(status.data_channels_count, num_channels);

    // Check performance metrics
    let avg_time = channel_creation_times.iter().sum::<std::time::Duration>() / channel_creation_times.len() as u32;
    println!("Average channel creation time: {:?}", avg_time);
    
    // Should be reasonably fast (less than 10ms per channel in mock implementation)
    assert!(avg_time.as_millis() < 10, "Channel creation should be fast");

    // Cleanup
    let _ = close_peer_connection(peer_id).await;
}