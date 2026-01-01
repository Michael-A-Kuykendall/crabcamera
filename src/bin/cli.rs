use crabcamera::headless::*;
use crabcamera::types::CameraFormat;
use std::env;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: crabcamera-cli <command> [args]");
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "list-devices" => cmd_list_devices(&args),
        "list-formats" => cmd_list_formats(&args),
        "capture" => cmd_capture(&args),
        "list-controls" => cmd_list_controls(&args),
        "set-control" => cmd_set_control(&args),
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
}

fn cmd_list_devices(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let devices = list_devices()?;
    if args.contains(&"--json".to_string()) {
        println!("{}", serde_json::to_string(&devices)?);
    } else {
        for d in devices {
            println!("{}: {}", d.id, d.name);
        }
    }
    Ok(())
}

fn cmd_list_formats(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        eprintln!("Usage: crabcamera-cli list-formats <device_id>");
        std::process::exit(1);
    }
    let device_id = &args[2];
    let formats = list_formats(device_id)?;
    if args.contains(&"--json".to_string()) {
        println!("{}", serde_json::to_string(&formats)?);
    } else {
        for f in formats {
            println!("{}x{}@{} {}", f.width, f.height, f.fps, f.format_type);
        }
    }
    Ok(())
}

fn cmd_capture(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Parse args: capture <device_id> <format> [--frames <n>] [--timeout <ms>] [--json]
    let mut device_id = None;
    let mut format = None;
    let mut frames = 1;
    let mut timeout_ms = 1000;
    let mut json = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--frames" => {
                i += 1;
                frames = args[i].parse()?;
            }
            "--timeout" => {
                i += 1;
                timeout_ms = args[i].parse()?;
            }
            "--json" => json = true,
            _ => {
                if device_id.is_none() {
                    device_id = Some(args[i].clone());
                } else if format.is_none() {
                    format = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    let device_id = device_id.ok_or("device_id required")?;
    let format_str = format.ok_or("format required")?;

    let format = parse_format(&format_str)?;

    // Create config
    let config = CaptureConfig {
        device_id,
        format,
        buffer_policy: BufferPolicy::DropOldest { capacity: 2 },
        audio_mode: AudioMode::Disabled,
        audio_device_id: None,
    };

    // Open session
    let session = HeadlessSession::open(config)?;

    // Start
    session.start()?;

    // Get frames
    for _ in 0..frames {
        let frame = session.get_frame(Duration::from_millis(timeout_ms))?;
        if let Some(f) = frame {
            if json {
                println!("{}", serde_json::to_string(&f)?);
            } else {
                println!("Frame: {}x{} {} seq:{}", f.width, f.height, f.format, f.sequence);
            }
        } else {
            if json {
                println!("null");
            } else {
                println!("Timeout");
            }
        }
    }

    // Stop
    session.stop(Duration::from_millis(1000))?;

    // Close
    session.close(Duration::from_millis(1000))?;

    Ok(())
}

fn cmd_list_controls(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 3 {
        eprintln!("Usage: crabcamera-cli list-controls <device_id>");
        std::process::exit(1);
    }
    let device_id = &args[2];

    // Open session to list controls
    let config = CaptureConfig {
        device_id: device_id.clone(),
        format: CameraFormat { width: 640, height: 480, fps: 30.0, format_type: "MJPEG".to_string() }, // dummy
        buffer_policy: BufferPolicy::DropOldest { capacity: 2 },
        audio_mode: AudioMode::Disabled,
        audio_device_id: None,
    };
    let session = HeadlessSession::open(config)?;
    let controls = session.list_controls()?;
    session.close(Duration::from_millis(100))?;

    if args.contains(&"--json".to_string()) {
        println!("{}", serde_json::to_string(&controls)?);
    } else {
        for (info, value) in controls {
            println!("{:?}: {:?} {:?}", info.id, info.kind, value);
        }
    }

    Ok(())
}

fn cmd_set_control(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 5 {
        eprintln!("Usage: crabcamera-cli set-control <device_id> <control_id> <value>");
        std::process::exit(1);
    }
    let device_id = &args[2];
    let control_id_str = &args[3];
    let value_str = &args[4];

    // Parse control_id
    let control_id = match control_id_str.parse::<ControlId>() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Invalid control_id: {}", control_id_str);
            std::process::exit(1);
        }
    };

    // Parse value based on control
    let value = parse_control_value(control_id, value_str)?;

    // Open session
    let config = CaptureConfig {
        device_id: device_id.clone(),
        format: CameraFormat { width: 640, height: 480, fps: 30.0, format_type: "MJPEG".to_string() }, // dummy
        buffer_policy: BufferPolicy::DropOldest { capacity: 2 },
        audio_mode: AudioMode::Disabled,
        audio_device_id: None,
    };
    let session = HeadlessSession::open(config)?;

    // Set control
    session.set_control(control_id, value)?;

    // Close
    session.close(Duration::from_millis(100))?;

    if args.contains(&"--json".to_string()) {
        println!("{{}}");
    } else {
        println!("OK");
    }

    Ok(())
}

fn parse_control_value(id: ControlId, s: &str) -> Result<ControlValue, Box<dyn std::error::Error>> {
    use ControlValue::*;
    match id {
        ControlId::AutoFocus | ControlId::AutoExposure | ControlId::NoiseReduction | ControlId::ImageStabilization => {
            match s {
                "true" | "1" => Ok(Bool(true)),
                "false" | "0" => Ok(Bool(false)),
                _ => Err(format!("Invalid bool value: {}", s).into()),
            }
        }
        ControlId::FocusDistance | ControlId::ExposureTime | ControlId::Aperture | ControlId::Zoom | ControlId::Brightness | ControlId::Contrast | ControlId::Saturation | ControlId::Sharpness => {
            Ok(F32(s.parse()?))
        }
        ControlId::IsoSensitivity => Ok(U32(s.parse()?)),
        ControlId::WhiteBalance => {
            // For simplicity, only support auto for now
            if s == "auto" {
                Ok(ControlValue::WhiteBalance(crabcamera::types::WhiteBalance::Auto))
            } else {
                Err(format!("Only 'auto' supported for white balance, got: {}", s).into())
            }
        }
    }
}

fn parse_format(s: &str) -> Result<CameraFormat, Box<dyn std::error::Error>> {
    // Simple parse: widthxheight@fps:format_type
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("format should be widthxheight@fps:format_type".into());
    }
    let size_fps = parts[0];
    let format_type = parts[1];
    let size_fps_parts: Vec<&str> = size_fps.split('@').collect();
    if size_fps_parts.len() != 2 {
        return Err("size should be widthxheight@fps".into());
    }
    let size = size_fps_parts[0];
    let fps: u32 = size_fps_parts[1].parse()?;
    let size_parts: Vec<&str> = size.split('x').collect();
    if size_parts.len() != 2 {
        return Err("size should be widthxheight".into());
    }
    let width: u32 = size_parts[0].parse()?;
    let height: u32 = size_parts[1].parse()?;
    Ok(CameraFormat {
        width,
        height,
        fps: fps as f32,
        format_type: format_type.to_string(),
    })
}