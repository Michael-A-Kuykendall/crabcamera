# CrabCamera 用户手册 — 简体中文版

<div align="center">

**简体中文** · [繁體中文](USER_MANUAL.zh-TW.md) · [English](../../README.md)

</div>

---

## 目录

1. [简介](#简介)
2. [安装](#安装)
3. [快速上手](#快速上手)
4. [Headless 模式](#headless-模式)
5. [相机控制](#相机控制)
6. [录制功能](#录制功能)
7. [音频录制](#音频录制)
8. [对焦堆栈](#对焦堆栈)
9. [质量分析](#质量分析)
10. [Tauri 插件集成](#tauri-插件集成)
11. [CLI 工具](#cli-工具)
12. [示例程序](#示例程序)
13. [平台差异](#平台差异)
14. [常见问题](#常见问题)

---

## 简介

CrabCamera 是一个生产级 Rust 库，为桌面应用提供统一的相机访问能力。它支持：

- **跨平台** — Windows（MediaFoundation）、macOS（AVFoundation）、Linux（V4L2）
- **零配置** — 自动发现可用相机
- **专业控制** — 曝光、对焦、白平衡、ISO、增益等
- **同步 A/V 录制** — 音视频同步编码为 MP4
- **自动质量验证** — 模糊检测、曝光分析、构图评分

CrabCamera 可以作为：
- **独立 Rust 库** — 任何 Rust 项目都可以使用（默认模式）
- **Tauri 插件** — 为 Tauri 应用提供相机集成（可选功能）

---

## 安装

### 作为独立库

```toml
[dependencies]
crabcamera = "0.9"
```

### 作为 Tauri 插件

```toml
[dependencies]
crabcamera = { version = "0.9", features = ["tauri"] }
tauri = { version = "2.11" }
```

### 可选功能

| 功能 | 用途 |
|------|------|
| `tauri` | 启用 Tauri 插件集成（`crabcamera::init()`） |
| `headless` | 启用独立 HeadlessSession API（默认启用） |
| `recording` | 启用 MP4 录制功能（H.264 + Muxide） |
| `audio` | 启用音频捕获和 OPUS 编码 |
| `full-recording` | 同时启用 `recording` 和 `audio` |

---

## 快速上手

### 列出可用相机

```rust
use crabcamera::headless::list_devices;

let devices = list_devices()?;
for device in &devices {
    println!("{} — {}", device.device_id, device.friendly_name);
}
```

### 捕获一帧

```rust
use crabcamera::headless::{HeadlessSession, CaptureConfig};
use crabcamera::headless::types::CameraFormat;

let config = CaptureConfig::new(
    "0".to_string(),                       // 设备 ID
    CameraFormat::standard(),               // 标准格式
);
let session = HeadlessSession::open(config)?;
session.start()?;
let frame = session.get_frame()?;         // 获取一帧
session.close()?;
```

### 查看支持的格式

```rust
use crabcamera::headless::list_formats;

let formats = list_formats("0")?;         // "0" 为设备 ID
for fmt in &formats {
    println!("{}x{} @ {}fps", fmt.width, fmt.height, fmt.fps);
}
```

---

## Headless 模式

Headless 模式是 CrabCamera 的核心模式，适用于服务器、CLI 工具和后台处理。

### HeadlessSession

`HeadlessSession` 是 headless 模式的主入口：

```rust
use crabcamera::headless::HeadlessSession;

let session = HeadlessSession::open(config)?;
session.start()?;

// 处理帧
while let Some(frame) = session.get_frame(duration::from_secs(1))? {
    // 处理 frame
}

session.close()?;
```

### 相机控制

```rust
// 获取所有可用控制
let controls = session.list_controls()?;

// 设置控制值
session.set_control(ControlId::ExposureTime, ControlValue::Float(50.0))?;
session.set_control(ControlId::Gain, ControlValue::Float(1.5))?;
session.set_control(ControlId::FocusAbsolute, ControlValue::Float(5.0))?;
```

### 音频流

当启用 `audio` 功能时，可以获取音频包：

```rust
while let Some(audio) = session.get_audio_packet(duration::from_secs(1))? {
    // audio 是 PCM 格式的音频数据
}
```

---

## 相机控制

CrabCamera 支持以下相机控制：

### 曝光控制
- `ExposureTime` — 曝光时间（毫秒）
- `AutoExposureMode` — 自动曝光模式
- `ExposureCompensation` — 曝光补偿
- `Brightness` — 亮度
- `Contrast` — 对比度
- `Saturation` — 饱和度
- `Sharpness` — 锐度
- `Gain` — 增益
- `Gamma` — 伽马

### 对焦控制
- `FocusAbsolute` — 绝对对焦位置
- `FocusRelative` — 相对对焦调整
- `FocusAuto` — 自动对焦触发

### 白平衡
- `WhiteBalance` — 白平衡色温（K）
- `WhiteBalanceAuto` — 自动白平衡

### 其他控制
- `Pan` — 水平平移
- `Tilt` — 垂直倾斜
- `Zoom` — 缩放
- `Roll` — 旋转

### 示例

```rust
use crabcamera::headless::types::{ControlId, ControlValue};

// 设置曝光时间为 50ms
session.set_control(ControlId::ExposureTime, ControlValue::Float(50.0))?;

// 设置 ISO 为 800
session.set_control(ControlId::Gain, ControlValue::Integer(800))?;

// 启用自动曝光
session.set_control(ControlId::AutoExposureMode, ControlValue::Integer(1))?;
```

---

## 录制功能

当启用 `recording` 功能时，CrabCamera 提供同步 A/V 录制功能。

### 录制配置

```rust
use crabcamera::recording::RecordingStartOptions;
use crabcamera::headless::types::RecordQuality;

let options = RecordingStartOptions {
    title: "my_recording".to_string(),
    quality: RecordQuality::High,
    ..Default::default()
};
```

### 录制质量预设

| 预设 | 分辨率 | 码率 | 用例 |
|------|--------|------|------|
| Low | 640x480 | ~2 Mbps | 快速测试 |
| Medium | 1280x720 | ~5 Mbps | 屏幕共享 |
| High | 1920x1080 | ~10 Mbps | 演示录制 |
| Ultra | 3840x2160 | ~20 Mbps | 专业制作 |

### 录制流程

```rust
session.start()?;
session.start_recording(options)?;
// ... 捕获帧和音频 ...
session.stop_recording()?;
session.close()?;
```

录制文件默认保存在当前工作目录，文件名为 `{title}_{timestamp}.mp4`。

---

## 音频录制

当启用 `audio` 功能时，CrabCamera 支持 CPAL 驱动的低延迟音频捕获。

### 音频设备

```rust
use crabcamera::headless::list_audio_devices;

let devices = list_audio_devices()?;
for device in &devices {
    println!("{} — {}", device.device_id, device.friendly_name);
}
```

### 音频配置

```rust
use crabcamera::headless::types::AudioMode;

config.audio_mode = AudioMode::Enabled;
config.audio_device = Some("default".to_string());
```

### 音频格式

- 采样率：48000 Hz
- 通道：立体声
- 编码：OPUS（VBR）
- 容器：MP4

---

## 对焦堆栈

对焦堆栈是多帧融合技术，用于获得全清晰图像。

```rust
use crabcamera::focus_stack::FocusStackConfig;

let focus_config = FocusStackConfig::default();
// focus_config.focus_range = FocusRange::new(0.5, 2.0)?;
// focus_config.num_steps = 10;
// focus_config.blend_levels = BlendLevels::default();
```

### 捕获对焦序列

```rust
session.capture_focus_brackets(&focus_config)?;
// 自动捕获多张不同对焦位置的帧，然后融合
```

### 融合模式

- **Laplacian Pyramid** — 拉普拉斯金字塔融合
- **Weighted Average** — 加权平均融合
- **Multi-Scale** — 多尺度融合

---

## 质量分析

CrabCamera 内建自动质量验证：

### 模糊检测

```rust
use crabcamera::quality::BlurDetector;

let blur = BlurDetector::new();
let report = blur.analyze(&frame);
println!("模糊等级: {:?}", report.blur_level);
```

### 曝光分析

```rust
use crabcamera::quality::ExposureAnalyzer;

let analyzer = ExposureAnalyzer::new();
let report = analyzer.analyze(&frame);
println!("曝光评分: {}", report.exposure_score);
println!("是否曝光正确: {}", report.well_exposed);
```

### 智能触发器

自动在场景质量满足条件时触发捕获：

```rust
use crabcamera::quality::smart_trigger::SmartTrigger;

let mut trigger = SmartTrigger::new(SmartTriggerConfig::default());
// 当评分超过阈值时，trigger.wait_for_good_frame() 返回 true
```

---

## Tauri 插件集成

### 在 Tauri 应用中注册插件

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .plugin(crabcamera::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 从 JavaScript 调用

```javascript
import { invoke } from '@tauri-apps/api/core';

// 初始化相机系统
await invoke('plugin:crabcamera|initialize_camera_system');

// 列出相机
const devices = await invoke('plugin:crabcamera|list_devices');

// 捕获单张照片
const frame = await invoke('plugin:crabcamera|capture_single_photo', {
    deviceId: "0",
    format: null
});

// 开始录制
await invoke('plugin:crabcamera|start_recording', {
    title: "my_video"
});
```

### Tauri API 前缀

所有命令使用 `plugin:crabcamera|` 前缀：

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

CrabCamera 附带一个 CLI 二进制工具 `crabcamera-cli`。

### 使用

```bash
# 列出相机
crabcamera list

# 捕获照片
crabcamera capture --device 0 --output photo.jpg

# 开始录制
crabcamera record --device 0 --title my_video

# 运行快速测试
crabcamera quicktest
```

---

## 示例程序

| 示例 | 用途 | 功能需求 |
|------|------|----------|
| `quick_test` | 列出相机、预热、拍照 | 无 |
| `camera_preview` | 启动预览流并捕获帧 | 无 |
| `record_video` | 完整录制 MP4 | `recording` |
| `live_av_recording` | 音视频同步录制 | `recording`, `audio` |
| `test_encoder_output` | 验证编码器输出 | `recording` |
| `hardware_audit` | 测试所有命令 | 无 |
| `functional_test` | 完整捕获 + 录制流程 | `recording` |

```bash
cargo run --example quick_test
cargo run --example record_video --features recording
cargo run --example live_av_recording --features "recording,audio"
```

---

## 平台差异

### Windows
- 后端：MediaFoundation
- 相机枚举：`GetDeviceID` / `IMFActivate`
- 音频：WASAPI 独占模式
- 支持零 shutter 延迟捕获

### macOS
- 后端：AVFoundation
- 相机枚举：`AVCaptureDevice`
- 音频：AVAudioEngine
- 需要用户授权（相机 + 麦克风权限）

### Linux
- 后端：V4L2（相机）+ webkit2gtk（Webview）
- 相机枚举：`ioctl(VIDIOC_ENUM_FMT)`
- 音频：ALSA / PulseAudio
- 需要安装系统依赖：`libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libasound2-dev`

---

## 常见问题

### Q: 为什么 `cargo test` 需要 `--features recording`？
A: 录制相关的测试需要 `recording` 功能编译。请使用 `cargo test --features recording`。

### Q: audio 测试在 CI 上被忽略怎么办？
A: 音频测试需要真实的音频设备。它们在无头 CI 环境中被 `#[ignore]` 标记，需要有音频硬件的机器上手动运行。

### Q: Windows 上相机列表为空？
A: 确保没有其他应用正在独占访问相机。MediaFoundation 不支持共享访问。

### Q: Linux 上预览无法工作？
A: 需要安装系统依赖：
```bash
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev
```

### Q: 如何选择正确的相机格式？
A: 使用 `list_formats(device_id)` 查看所有支持的 `CameraFormat`。选择匹配目标应用场景的格式（分辨率 + FPS + 编码）。

### Q: Headless 模式和 Tauri 模式的区别？
A: Headless 模式提供纯 Rust API，不需要 Tauri。Tauri 模式在此基础上添加了 `crabcamera::init()` 和 Tauri 命令桥接，让前端 JavaScript 可以调用。

---

## 许可证

[MIT](../../LICENSE-MIT) — 永久免费。

---

*Made with Rust 🦀*
