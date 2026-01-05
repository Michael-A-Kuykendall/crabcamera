use crate::headless::errors::HeadlessError;
use crate::types::WhiteBalance;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ControlId {
    AutoFocus,
    FocusDistance,
    AutoExposure,
    ExposureTime,
    IsoSensitivity,
    WhiteBalance,
    Aperture,
    Zoom,
    Brightness,
    Contrast,
    Saturation,
    Sharpness,
    NoiseReduction,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ControlKind {
    Bool,
    F32,
    U32,
    WhiteBalance,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ControlValue {
    Bool(bool),
    F32(f32),
    U32(u32),
    WhiteBalance(WhiteBalance),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ControlInfo {
    pub id: ControlId,
    pub kind: ControlKind,
    pub min_f32: Option<f32>,
    pub max_f32: Option<f32>,
    pub min_u32: Option<u32>,
    pub max_u32: Option<u32>,
}

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
