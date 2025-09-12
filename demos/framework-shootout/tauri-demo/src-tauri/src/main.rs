#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CameraState {
    focus: f32,
    iso: u32,
    exposure: f32,
    white_balance: String,
    photos_captured: u32,
    camera_connected: bool,
    preview_width: u32,
    preview_height: u32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            focus: 50.0,
            iso: 400,
            exposure: 1.0 / 60.0,
            white_balance: "Auto".to_string(),
            photos_captured: 0,
            camera_connected: true,
            preview_width: 1280,
            preview_height: 720,
        }
    }
}

type SharedCameraState = Arc<Mutex<CameraState>>;

#[tauri::command]
fn set_focus(state: State<SharedCameraState>, value: f32) -> Result<(), String> {
    let mut camera_state = state.lock().map_err(|e| e.to_string())?;
    camera_state.focus = value;
    println!("Focus changed to: {:.1}%", value);
    Ok(())
}

#[tauri::command]
fn set_iso(state: State<SharedCameraState>, value: u32) -> Result<(), String> {
    let mut camera_state = state.lock().map_err(|e| e.to_string())?;
    camera_state.iso = value;
    println!("ISO changed to: {}", value);
    Ok(())
}

#[tauri::command]
fn set_exposure(state: State<SharedCameraState>, value: f32) -> Result<(), String> {
    let mut camera_state = state.lock().map_err(|e| e.to_string())?;
    camera_state.exposure = value;
    println!("Exposure changed to: 1/{:.0}s", 1.0 / value);
    Ok(())
}

#[tauri::command]
fn set_white_balance(state: State<SharedCameraState>, value: String) -> Result<(), String> {
    let mut camera_state = state.lock().map_err(|e| e.to_string())?;
    camera_state.white_balance = value.clone();
    println!("White balance changed to: {}", value);
    Ok(())
}

#[tauri::command]
fn capture_photo(state: State<SharedCameraState>) -> Result<u32, String> {
    let mut camera_state = state.lock().map_err(|e| e.to_string())?;
    camera_state.photos_captured += 1;
    println!("Photo captured! Total: {}", camera_state.photos_captured);
    Ok(camera_state.photos_captured)
}

#[tauri::command]
fn get_camera_state(state: State<SharedCameraState>) -> Result<CameraState, String> {
    let camera_state = state.lock().map_err(|e| e.to_string())?;
    Ok(camera_state.clone())
}

#[tauri::command]
fn get_performance_metrics() -> serde_json::Value {
    serde_json::json!({
        "memory_usage": "22.4 MB",
        "cpu_usage": "5.8%",
        "frame_latency": "16.7ms"
    })
}

fn main() {
    let camera_state = Arc::new(Mutex::new(CameraState::default()));

    tauri::Builder::default()
        .manage(camera_state)
        .invoke_handler(tauri::generate_handler![
            set_focus,
            set_iso,
            set_exposure,
            set_white_balance,
            capture_photo,
            get_camera_state,
            get_performance_metrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}