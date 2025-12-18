//! Tauri commands for audio device management
//!
//! # Spell: TauriAudioCommands
//!
//! Intent: expose audio device discovery and audio-enabled recording through Tauri commands safely
//!
//! ## Features
//!
//! - ui_request -> recording_action
//! - list_audio_devices_returns_structured_data
//! - start_recording_accepts_audio_device_option
//! - user_safe_error_strings
//! - no leaking_internal_error_types
//! - async_safe_execution

use tauri::command;
use serde::{Deserialize, Serialize};

use crate::audio::{list_audio_devices as enumerate_audio_devices, AudioDevice};

/// Audio device information exposed to Tauri frontend
/// 
/// Per #TauriAudioCommands: ! list_audio_devices_returns_structured_data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDeviceInfo {
    /// Unique device identifier
    pub id: String,
    /// Human-readable device name  
    pub name: String,
    /// Default sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Whether this is the system default input device
    pub is_default: bool,
}

impl From<AudioDevice> for AudioDeviceInfo {
    fn from(device: AudioDevice) -> Self {
        AudioDeviceInfo {
            id: device.id,
            name: device.name,
            sample_rate: device.sample_rate,
            channels: device.channels,
            is_default: device.is_default,
        }
    }
}

/// List all available audio input devices
/// 
/// Per #TauriAudioCommands:
/// - ! list_audio_devices_returns_structured_data
/// - ! user_safe_error_strings  
/// - - leaking_internal_error_types
/// 
/// # Returns
/// List of audio devices, sorted with default device first
#[command]
pub async fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    enumerate_audio_devices()
        .map(|devices| devices.into_iter().map(AudioDeviceInfo::from).collect())
        .map_err(|e| {
            // Per spell: ! user_safe_error_strings, - leaking_internal_error_types
            log::error!("Failed to enumerate audio devices: {:?}", e);
            "Unable to list audio devices. Please check that your audio drivers are installed correctly.".to_string()
        })
}

/// Get the default audio input device
/// 
/// # Returns
/// The default audio device, or an error if none available
#[command]
pub async fn get_default_audio_device() -> Result<AudioDeviceInfo, String> {
    crate::audio::get_default_audio_device()
        .map(AudioDeviceInfo::from)
        .map_err(|e| {
            log::error!("Failed to get default audio device: {:?}", e);
            "No default audio input device available. Please connect a microphone.".to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_device_info_serialization() {
        let device = AudioDeviceInfo {
            id: "device_1".to_string(),
            name: "Test Microphone".to_string(),
            sample_rate: 48000,
            channels: 2,
            is_default: true,
        };
        
        let json = serde_json::to_string(&device).unwrap();
        // Per spell: uses camelCase for frontend
        assert!(json.contains("sampleRate"));
        assert!(json.contains("isDefault"));
        assert!(json.contains("Test Microphone"));
    }

    #[test]
    fn test_audio_device_info_from_audio_device() {
        use crate::audio::AudioDevice;
        
        let internal = AudioDevice {
            id: "mic_1".to_string(),
            name: "Internal Mic".to_string(),
            sample_rate: 44100,
            channels: 1,
            is_default: false,
        };
        
        let info = AudioDeviceInfo::from(internal);
        assert_eq!(info.id, "mic_1");
        assert_eq!(info.name, "Internal Mic");
        assert_eq!(info.sample_rate, 44100);
        assert_eq!(info.channels, 1);
        assert!(!info.is_default);
    }
}
