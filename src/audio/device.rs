//! Audio device enumeration
//!
//! # Spell: AudioDeviceEnumerate
//!
//! Intent: expose stable, cross-platform enumeration of audio input devices
//!
//! ## Features
//!
//! - `system_inputs -> Vec<AudioDevice>`
//! - includes(id, name, sample_rate, channels, is_default)
//! - input_devices_only
//! - deterministic_ordering
//! - no starting_audio_capture
//! - no inferring_missing_fields

use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};

use crate::errors::CameraError;

/// Audio input device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
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

/// List all available audio input devices
///
/// Returns devices in deterministic order (default device first, then alphabetically).
///
/// # Errors
/// Returns error if audio host is unavailable.
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, CameraError> {
    let host = cpal::default_host();
    let default_device_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let mut devices: Vec<AudioDevice> = host
        .input_devices()
        .map_err(|e| CameraError::AudioError(format!("Failed to enumerate audio devices: {}", e)))?
        .enumerate()
        .filter_map(|(index, device)| {
            let name = device.name().ok()?;
            let config = device.default_input_config().ok()?;
            
            // Generate synthetic ID: cpal doesn't expose unique device IDs on all platforms,
            // so we combine index with name hash to create a stable-ish identifier.
            // Format: "audio_{index}_{hash}" where hash is first 8 chars of name hash
            let name_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                format!("{:08x}", hasher.finish() & 0xFFFFFFFF)
            };
            let id = format!("audio_{}_{}", index, name_hash);
            
            Some(AudioDevice {
                id,
                name: name.clone(),
                sample_rate: config.sample_rate().0,
                channels: config.channels(),
                is_default: default_device_name.as_ref() == Some(&name),
            })
        })
        .collect();

    // Deterministic ordering: default first, then alphabetically
    devices.sort_by(|a, b| {
        match (a.is_default, b.is_default) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(devices)
}

/// Get the default audio input device
///
/// # Errors
/// Returns error if no default device is available.
pub fn get_default_audio_device() -> Result<AudioDevice, CameraError> {
    let host = cpal::default_host();
    
    let device = host
        .default_input_device()
        .ok_or_else(|| CameraError::AudioError("No default audio input device".to_string()))?;

    let name = device
        .name()
        .map_err(|e| CameraError::AudioError(format!("Failed to get device name: {}", e)))?;

    let config = device
        .default_input_config()
        .map_err(|e| CameraError::AudioError(format!("Failed to get device config: {}", e)))?;

    // Generate synthetic ID for default device (index 0)
    let name_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        format!("{:08x}", hasher.finish() & 0xFFFFFFFF)
    };
    let id = format!("audio_0_{}", name_hash);

    Ok(AudioDevice {
        id,
        name,
        sample_rate: config.sample_rate().0,
        channels: config.channels(),
        is_default: true,
    })
}

/// Find an audio device by ID or name
///
/// If `device_id` is "default" or empty, returns the default device.
pub fn find_audio_device(device_id: &str) -> Result<AudioDevice, CameraError> {
    if device_id.is_empty() || device_id == "default" {
        return get_default_audio_device();
    }

    let devices = list_audio_devices()?;
    devices
        .into_iter()
        .find(|d| d.id == device_id || d.name == device_id)
        .ok_or_else(|| CameraError::AudioError(format!("Audio device not found: {}", device_id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_audio_devices_no_panic() {
        // Should not panic even if no devices
        let _ = list_audio_devices();
    }

    #[test]
    fn test_default_device_is_first() {
        if let Ok(devices) = list_audio_devices() {
            if !devices.is_empty() {
                // If there's a default, it should be first
                let has_default = devices.iter().any(|d| d.is_default);
                if has_default {
                    assert!(devices[0].is_default);
                }
            }
        }
    }

    #[test]
    fn test_find_device_default() {
        if let Ok(device) = find_audio_device("default") {
            assert!(device.is_default);
        }
    }

    #[test]
    fn test_find_device_empty_string() {
        if let Ok(device) = find_audio_device("") {
            assert!(device.is_default);
        }
    }
}
