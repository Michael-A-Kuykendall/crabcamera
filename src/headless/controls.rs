use crate::headless::errors::HeadlessError;
use crate::types::WhiteBalance;
use std::str::FromStr;

/// Identifiers for supported camera controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ControlId {
    /// Automatic focus control (Boolean).
    AutoFocus,
    /// Manual focus distance (0.0 to 1.0).
    FocusDistance,
    /// Automatic exposure control (Boolean).
    AutoExposure,
    /// Exposure time in seconds.
    ExposureTime,
    /// ISO sensitivity (e.g., 100, 400).
    IsoSensitivity,
    /// White balance mode.
    WhiteBalance,
    /// Aperture (f-stop).
    Aperture,
    /// Digital or optical zoom factor.
    Zoom,
    /// Image brightness.
    Brightness,
    /// Image contrast.
    Contrast,
    /// Color saturation.
    Saturation,
    /// Image sharpness.
    Sharpness,
    /// Noise reduction (Boolean or strength).
    NoiseReduction,
    /// Image stabilization (Boolean).
    ImageStabilization,
}

impl FromStr for ControlId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AutoFocus" => Ok(Self::AutoFocus),
            "FocusDistance" => Ok(Self::FocusDistance),
            "AutoExposure" => Ok(Self::AutoExposure),
            "ExposureTime" => Ok(Self::ExposureTime),
            "IsoSensitivity" => Ok(Self::IsoSensitivity),
            "WhiteBalance" => Ok(Self::WhiteBalance),
            "Aperture" => Ok(Self::Aperture),
            "Zoom" => Ok(Self::Zoom),
            "Brightness" => Ok(Self::Brightness),
            "Contrast" => Ok(Self::Contrast),
            "Saturation" => Ok(Self::Saturation),
            "Sharpness" => Ok(Self::Sharpness),
            "NoiseReduction" => Ok(Self::NoiseReduction),
            "ImageStabilization" => Ok(Self::ImageStabilization),
            _ => Err(()),
        }
    }
}

/// The data type of a control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ControlKind {
    /// Boolean value (on/off).
    Bool,
    /// Floating point value.
    F32,
    /// Unsigned integer value.
    U32,
    /// White balance mode enum.
    WhiteBalance,
}

/// The concrete value for a control setting.
#[derive(Debug, Clone, serde::Serialize)]
pub enum ControlValue {
    /// Boolean value.
    Bool(bool),
    /// Floating point value.
    F32(f32),
    /// Unsigned integer value.
    U32(u32),
    /// White balance preset.
    WhiteBalance(WhiteBalance),
}

/// Metadata describing a camera control's type and valid range.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ControlInfo {
    /// The unique identifier for the control.
    pub id: ControlId,
    /// The data type of the control value.
    pub kind: ControlKind,
    /// Minimum allowed float value, if applicable.
    pub min_f32: Option<f32>,
    /// Maximum allowed float value, if applicable.
    pub max_f32: Option<f32>,
    /// Minimum allowed integer value, if applicable.
    pub min_u32: Option<u32>,
    /// Maximum allowed integer value, if applicable.
    pub max_u32: Option<u32>,
}

/// Returns a list of all supported controls and their specifications.
///
/// This provides a static registry of camera capabilities used for validation
/// and introspection.
#[must_use]
pub fn all_controls() -> Vec<ControlInfo> {
    vec![
        ControlInfo {
            id: ControlId::AutoFocus,
            kind: ControlKind::Bool,
            min_f32: None,
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::FocusDistance,
            kind: ControlKind::F32,
            min_f32: Some(0.0),
            max_f32: Some(1.0),
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::AutoExposure,
            kind: ControlKind::Bool,
            min_f32: None,
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::ExposureTime,
            kind: ControlKind::F32,
            min_f32: Some(0.0),
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::IsoSensitivity,
            kind: ControlKind::U32,
            min_f32: None,
            max_f32: None,
            min_u32: Some(0),
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::WhiteBalance,
            kind: ControlKind::WhiteBalance,
            min_f32: None,
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::Aperture,
            kind: ControlKind::F32,
            min_f32: Some(0.0),
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::Zoom,
            kind: ControlKind::F32,
            min_f32: Some(1.0),
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::Brightness,
            kind: ControlKind::F32,
            min_f32: Some(-1.0),
            max_f32: Some(1.0),
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::Contrast,
            kind: ControlKind::F32,
            min_f32: Some(-1.0),
            max_f32: Some(1.0),
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::Saturation,
            kind: ControlKind::F32,
            min_f32: Some(-1.0),
            max_f32: Some(1.0),
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::Sharpness,
            kind: ControlKind::F32,
            min_f32: Some(-1.0),
            max_f32: Some(1.0),
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::NoiseReduction,
            kind: ControlKind::Bool,
            min_f32: None,
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
        ControlInfo {
            id: ControlId::ImageStabilization,
            kind: ControlKind::Bool,
            min_f32: None,
            max_f32: None,
            min_u32: None,
            max_u32: None,
        },
    ]
}

/// Validates if a given value is appropriate for a specific control.
///
/// checks if the `value` type matches the `id`'s expected type, and if the value
/// is within the allowed numeric range.
///
/// # Errors
///
/// * `HeadlessError::InvalidControl`: If the value is out of bounds or type mismatch.
/// * `HeadlessError::Unsupported`: If the control ID is not recognized.
pub fn validate_control_value(id: ControlId, value: &ControlValue) -> Result<(), HeadlessError> {
    let info = all_controls()
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| HeadlessError::unsupported(format!("control {id:?} not supported")))?;

    match (info.kind, value) {
        (ControlKind::Bool, ControlValue::Bool(_)) => Ok(()),
        (ControlKind::F32, ControlValue::F32(v)) => {
            if let Some(min) = info.min_f32 {
                if *v < min {
                    return Err(HeadlessError::invalid_argument("value below minimum"));
                }
            }
            if let Some(max) = info.max_f32 {
                if *v > max {
                    return Err(HeadlessError::invalid_argument("value above maximum"));
                }
            }
            Ok(())
        }
        (ControlKind::U32, ControlValue::U32(v)) => {
            if let Some(min) = info.min_u32 {
                if *v < min {
                    return Err(HeadlessError::invalid_argument("value below minimum"));
                }
            }
            if let Some(max) = info.max_u32 {
                if *v > max {
                    return Err(HeadlessError::invalid_argument("value above maximum"));
                }
            }
            Ok(())
        }
        (ControlKind::WhiteBalance, ControlValue::WhiteBalance(_)) => Ok(()),
        _ => Err(HeadlessError::invalid_argument(
            "control value kind mismatch",
        )),
    }
}
