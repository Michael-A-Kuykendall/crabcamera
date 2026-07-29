# CrabCamera 用戶手冊 — 繁體中文版

<div align="center">

**繁體中文** · [简体中文](USER_MANUAL.zh-CN.md) · [English](../../README.md)

</div>

---

## 目錄

1. [簡介](#簡介)
2. [安裝](#安裝)
3. [快速上手](#快速上手)
4. [Headless 模式](#headless-模式)
5. [相機控制](#相機控制)
6. [錄製功能](#錄製功能)
7. [音訊錄製](#音訊錄製)
8. [對焦堆棧](#對焦堆棧)
9. [品質分析](#品質分析)
10. [Tauri 插件集成](#tauri-插件集成)
11. [CLI 工具](#cli-工具)
12. [範例程式](#範例程式)
13. [平台差異](#平台差異)
14. [常見問題](#常見問題)

---

## 簡介

CrabCamera 是一個生產級 Rust 庫，為桌面應用提供統一相機訪問能力。它支援：

- **跨平台** — Windows（MediaFoundation）、macOS（AVFoundation）、Linux（V4L2）
- **零配置** — 自動發現可用相機
- **專業控制** — 曝光、對焦、白平衡、ISO、增益等
- **同步 A/V 錄製** — 音訊視頻同步編碼為 MP4
- **自動品質驗證** — 模糊檢測、曝光分析、構圖評分

CrabCamera 可以作為：
- **獨立 Rust 庫** — 任何 Rust 項目都可以使用（默認模式）
- **Tauri 插件** — 為 Tauri 應用提供相機集成（可選功能）

---

## 安裝

### 作為獨立庫

```toml
[dependencies]
crabcamera = "0.9"
```

### 作為 Tauri 插件

```toml
[dependencies]
crabcamera = { version = "0.9", features = ["tauri"] }
tauri = { version = "2.11" }
```

### 可選功能

| 功能 | 用途 |
|------|------|
| `tauri` | 啟用 Tauri 插件集成（`crabcamera::init()`） |
| `headless` | 啟用獨立 HeadlessSession API（默認啟用） |
| `recording` | 啟用 MP4 錄製功能（H.264 + Muxide） |
| `audio` | 啟用音訊捕獲和 OPUS 編碼 |
| `full-recording` | 同時啟用 `recording` 和 `audio` |

---

## 快速上手

### 列出可用相機

```rust
use crabcamera::headless::list_devices;

let devices = list_devices()?;
for device in &devices {
    println!("{} — {}", device.device_id, device.friendly_name);
}
```

### 捕獲一幀

```rust
use crabcamera::headless::{HeadlessSession, CaptureConfig};
use crabcamera::headless::types::CameraFormat;

let config = CaptureConfig::new(
    "0".to_string(),                       // 設備 ID
    CameraFormat::standard(),               // 標準格式
);
let session = HeadlessSession::open(config)?;
session.start()?;
let frame = session.get_frame()?;         // 獲取一幀
session.close()?;
```

### 查看支援的格式

```rust
use crabcamera::headless::list_formats;

let formats = list_formats("0")?;         // "0" 為設備 ID
for fmt in &formats {
    println!("{}x{} @ {}fps", fmt.width, fmt.height, fmt.fps);
}
```

---

## Headless 模式

Headless 模式是 CrabCamera 的核心模式，適用於伺服器、CLI 工具和背景處理。

### HeadlessSession

`HeadlessSession` 是 headless 模式的主入口：

```rust
use crabcamera::headless::HeadlessSession;

let session = HeadlessSession::open(config)?;
session.start()?;

// 處理幀
while let Some(frame) = session.get_frame(duration::from_secs(1))? {
    // 處理 frame
}

session.close()?;
```

### 相機控制

```rust
// 獲取所有可用控制
let controls = session.list_controls()?;

// 設定控制值
session.set_control(ControlId::ExposureTime, ControlValue::Float(50.0))?;
session.set_control(ControlId::Gain, ControlValue::Float(1.5))?;
session.set_control(ControlId::FocusAbsolute, ControlValue::Float(5.0))?;
```

### 音訊流

當啟用 `audio` 功能時，可以獲取音訊封包：

```rust
while let Some(audio) = session.get_audio_packet(duration::from_secs(1))? {
    // audio 是 PCM 格式的音訊數據
}
```

---

## 相機控制

CrabCamera 支援以下相機控制：

### 曝光控制
- `ExposureTime` — 曝光時間（毫秒）
- `AutoExposureMode` — 自動曝光模式
- `ExposureCompensation` — 曝光補償
- `Brightness` — 亮度
- `Contrast` — 對比度
- `Saturation` — 飽和度
- `Sharpness` — 銳度
- `Gain` — 增益
- `Gamma` — 伽馬

### 對焦控制
- `FocusAbsolute` — 絕對對焦位置
- `FocusRelative` — 相對對焦調整
- `FocusAuto` — 自動對焦觸發

### 白平衡
- `WhiteBalance` — 白平衡色溫（K）
- `WhiteBalanceAuto` — 自動白平衡

### 其他控制
- `Pan` — 水平平移
- `Tilt` — 垂直傾斜
- `Zoom` — 縮放
- `Roll` — 旋轉

### 範例

```rust
use crabcamera::headless::types::{ControlId, ControlValue};

// 設定曝光時間為 50ms
session.set_control(ControlId::ExposureTime, ControlValue::Float(50.0))?;

// 設定 ISO 為 800
session.set_control(ControlId::Gain, ControlValue::Integer(800))?;

// 啟用自動曝光
session.set_control(ControlId::AutoExposureMode, ControlValue::Integer(1))?;
```

---

## 錄製功能

當啟用 `recording` 功能時，CrabCamera 提供同步 A/V 錄製功能。

### 錄製配置

```rust
use crabcamera::recording::RecordingStartOptions;
use crabcamera::headless::types::RecordQuality;

let options = RecordingStartOptions {
    title: "my_recording".to_string(),
    quality: RecordQuality::High,
    ..Default::default()
};
```

### 錄製品質預設

| 預設 | 解析度 | 碼率 | 用例 |
|------|--------|------|------|
| Low | 640x480 | ~2 Mbps | 快速測試 |
| Medium | 1280x720 | ~5 Mbps | 螢幕共享 |
| High | 1920x1080 | ~10 Mbps | 演示錄製 |
| Ultra | 3840x2160 | ~20 Mbps | 專業製作 |

### 錄製流程

```rust
session.start()?;
session.start_recording(options)?;
// ... 捕獲幀和音訊 ...
session.stop_recording()?;
session.close()?;
```

錄製文件預設保存在當前工作目錄，文件名為 `{title}_{timestamp}.mp4`。

---

## 音訊錄製

當啟用 `audio` 功能時，CrabCamera 支援 CPAL 驅動的低延遲音訊捕獲。

### 音訊設備

```rust
use crabcamera::headless::list_audio_devices;

let devices = list_audio_devices()?;
for device in &devices {
    println!("{} — {}", device.device_id, device.friendly_name);
}
```

### 音訊配置

```rust
use crabcamera::headless::types::AudioMode;

config.audio_mode = AudioMode::Enabled;
config.audio_device = Some("default".to_string());
```

### 音訊格式

- 採樣率：48000 Hz
- 通道：立體聲
- 編碼：OPUS（VBR）
- 容器：MP4

---

## 對焦堆棧

對焦堆棧是多幀融合技術，用於獲得全清晰圖像。

```rust
use crabcamera::focus_stack::FocusStackConfig;

let focus_config = FocusStackConfig::default();
// focus_config.focus_range = FocusRange::new(0.5, 2.0)?;
// focus_config.num_steps = 10;
// focus_config.blend_levels = BlendLevels::default();
```

### 捕獲對焦序列

```rust
session.capture_focus_brackets(&focus_config)?;
// 自動捕獲多張不同對齊位置的幀，然後融合
```

### 融合模式

- **Laplacian Pyramid** — 拉普拉斯金字塔融合
- **Weighted Average** — 加權平均融合
- **Multi-Scale** — 多尺度融合

---

## 品質分析

CrabCamera 內建自動品質驗證：

### 模糊檢測

```rust
use crabcamera::quality::BlurDetector;

let blur = BlurDetector::new();
let report = blur.analyze(&frame);
println!("模糊等級: {:?}", report.blur_level);
```

### 曝光分析

```rust
use crabcamera::quality::ExposureAnalyzer;

let analyzer = ExposureAnalyzer::new();
let report = analyzer.analyze(&frame);
println!("曝光評分: {}", report.exposure_score);
println!("是否曝光正確: {}", report.well_exposed);
```

### 智能觸發器

自動在場景品質滿足條件時觸發捕獲：

```rust
use crabcamera::quality::smart_trigger::SmartTrigger;

let mut trigger = SmartTrigger::new(SmartTriggerConfig::default());
// 當評分超過閾值時，trigger.wait_for_good_frame() 返回 true
```

---

## Tauri 插件集成

### 在 Tauri 應用中註冊插件

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .plugin(crabcamera::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 從 JavaScript 調用

```javascript
import { invoke } from '@tauri-apps/api/core';

// 初始化相機系統
await invoke('plugin:crabcamera|initialize_camera_system');

// 列出相機
const devices = await invoke('plugin:crabcamera|list_devices');

// 捕獲單張照片
const frame = await invoke('plugin:crabcamera|capture_single_photo', {
    deviceId: "0",
    format: null
});

// 開始錄製
await invoke('plugin:crabcamera|start_recording', {
    title: "my_video"
});
```

### Tauri API 前綴

所有命令使用 `plugin:crabcamera|` 前綴：

- `plugin:crabcamera|initialize_camera_system`
- `plugin:crabcamera|list_devices`
- `plugin:crabcamera|capture_single_photo`
- `plugin:crabcamera|start_recording`
- `plugin:crabcamera|stop_recording`
- `plugin:crabcamera|get_recording_status`
- `plugin:crabcamera|capture_focus_stack_legacy`
- `plugin:crabcamera|apply_camera_settings`

### Tauri 配置

在 `tauri.conf.json` 中：

```json
{
  "plugins": {
    "crabcamera": {
      "defaultDevice": "0",
      "defaultQuality": "high"
    }
  }
}
```

---

## CLI 工具

CrabCamera 附帶一個 CLI 二進制工具 `crabcamera-cli`。

### 使用

```bash
# 列出相機
crabcamera list

# 捕獲照片
crabcamera capture --device 0 --output photo.jpg

# 開始錄製
crabcamera record --device 0 --title my_video

# 運行快速測試
crabcamera quicktest
```

---

## 範例程式

| 範例 | 用途 | 功能需求 |
|------|------|----------|
| `quick_test` | 列出相機、預熱、拍照 | 無 |
| `camera_preview` | 啟動預覽流並捕獲幀 | 無 |
| `record_video` | 完整錄製 MP4 | `recording` |
| `live_av_recording` | 音訊視頻同步錄製 | `recording`, `audio` |
| `test_encoder_output` | 驗證編碼器輸出 | `recording` |
| `hardware_audit` | 測試所有命令 | 無 |
| `functional_test` | 完整捕獲 + 錄製流程 | `recording` |

```bash
cargo run --example quick_test
cargo run --example record_video --features recording
cargo run --example live_av_recording --features "recording,audio"
```

---

## 平台差異

### Windows
- 後端：MediaFoundation
- 相機 enumerarion：`GetDeviceID` / `IMFActivate`
- 音訊：WASAPI 獨佔模式
- 支援零 shutter 延遲捕獲

### macOS
- 後端：AVFoundation
- 相機 enumerarion：`AVCaptureDevice`
- 音訊：AVAudioEngine
- 需要用戶授權（相機 + 麥克風權限）

### Linux
- 後端：V4L2（相機）+ webkit2gtk（Webview）
- 相機 enumerarion：`ioctl(VIDIOC_ENUM_FMT)`
- 音訊：ALSA / PulseAudio
- 需要安裝系統依賴：`libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libasound2-dev`

---

## 常見問題

### Q: 為什麼 `cargo test` 需要 `--features recording`？
A: 與錄製相關的測試需要 `recording` 功能編譯。請使用 `cargo test --features recording`。

### Q: audio 測試在 CI 上被忽略怎麼辦？
A: 音訊測試需要真實的音訊設備。它們在無頭 CI 環境中被 `#[ignore]` 標記，需要有音訊硬體的機器上手動運行。

### Q: Windows 上相機列表為空？
A: 確保沒有其他應用正在獨佔訪問相機。MediaFoundation 不支援共享訪問。

### Q: Linux 上預覽無法工作？
A: 需要安裝系統依賴：
```bash
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev
```

### Q: 如何選擇正確的相機格式？
A: 使用 `list_formats(device_id)` 查看所有支援的 `CameraFormat`。選擇匹配目標應用場景的格式（解析度 + FPS + 編碼）。

### Q: Headless 模式和 Tauri 模式的區別？
A: Headless 模式提供純 Rust API，不需要 Tauri。Tauri 模式在此基礎上添加了 `crabcamera::init()` 和 Tauri 命令橋接，讓前端 JavaScript 可以調用。

---

## 許可證

[MIT](../../LICENSE-MIT) — 永久免費。

---

*Made with Rust 🦀*
