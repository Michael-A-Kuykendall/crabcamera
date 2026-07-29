<div align="center">

# CrabCamera 中文文档中心

### 生产级跨平台相机采集与录制库 · 兼容 Tauri 插件与独立运行模式 · 永久免费

**简体中文** · [繁體中文](../zh-TW/README.md) · [English](../../README.md)

[![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/crabcamera?style=social)](https://github.com/Michael-A-Kuykendall/crabcamera/stargazers)
[![Crates.io](https://img.shields.io/crates/v/crabcamera.svg)](https://crates.io/crates/crabcamera)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![💝 赞助项目](https://img.shields.io/badge/💝_赞助项目-ea4aaa?style=flat&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)

</div>

---

## 欢迎使用 CrabCamera

CrabCamera 是一个用纯 **Rust** 编写的跨平台相机采集与录制库。它提供统一的相机访问接口，覆盖 Windows、macOS 和 Linux 平台，支持专业级相机控制、同步音视频录制和零配置快速启动。

CrabCamera 附带可选的 **Tauri 插件集成**，可以让你的 Tauri 应用直接调用原生相机能力。同时它也提供独立的 **Headless 模式**，适用于服务器、CLI 工具或任何纯 Rust 项目。

**免费 forever。MIT 许可证。无任何限制。**

---

## 快速开始

### 从 Rust 使用（独立模式，默认）

```toml
[dependencies]
crabcamera = "0.9"
```

```rust
use crabcamera::headless::HeadlessSession;

let session = HeadlessSession::new(config)?;
session.start()?;
let frame = session.get_frame()?;
```

可选添加功能：
- `recording` — 启用 MP4 录制（H.264 + Muxide）
- `audio` — 启用 OPUS 音频采集与编码

### Tauri 插件集成

如果你正在构建 Tauri 应用：

```toml
[dependencies]
crabcamera = { version = "0.9", features = ["tauri"] }
tauri = { version = "2.11" }
```

---

## 支持的平台

| 平台 | 后端 | 相机访问 | 音频 | 录制 |
|------|------|----------|------|------|
| Windows | MediaFoundation | ✅ | ✅ | ✅ |
| macOS | AVFoundation | ✅ | ✅ | ✅ |
| Linux | V4L2 + webkit2gtk | ✅ | ✅ | ✅ |

---

## 功能特性

- 📷 **零配置相机发现** — 自动识别可用相机并列出支持的格式
- 🎞️ **同步 A/V 录制** — 音视频同步录制为 MP4 文件
- 🎛️ **专业相机控制** — 曝光、对焦、白平衡、ISO 等手动控制
- 🔍 **自动质量验证** — 内建模糊检测、曝光分析和构图评分
- 📐 **对焦堆栈（Focus Stacking）** — 多帧融合获得全清晰图像
- 🖥️ **实时预览流** — 低延迟 JPEG 预览
- 🔊 **音频捕获** — CPAL 驱动的低延迟音频录制
- 🪶 **零 unsafe 代码** — 纯安全 Rust

---

## 支持 CrabCamera 的发展

🚀 **如果 CrabCamera 对你有帮助，欢迎[赞助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有赞助款项 100% 用于保持项目永久免费。**

- **$5/月**：咖啡档 ☕ 永久感谢 + 赞助者徽章
- **$25/月**：Bug 优先处理档 🐛 优先支持 + 名字收录于 [SPONSORS.md](../../SPONSORS.md)
- **$100/月**：企业支持档 🏢 Logo 展示 + 每月答疑
- **$500/月**：基础设施合作伙伴 🚀 直接支持 + 路线图影响力

[**🎯 成为赞助者**](https://github.com/sponsors/Michael-A-Kuykendall) | 查看我们的 [赞助者](../../SPONSORS.md) 🙏

---

## 文档中心

- [快速开始](../../README.md) — 英文快速入门
- [用户手册](USER_MANUAL.zh-CN.md) — 完整中文用户手册
- [API 参考](../../docs/SYSTEM_MAP.md) — 系统架构与 API 概览

---

## 许可证

[MIT](../../LICENSE-MIT) — 永久免费。

> 💝 如果 CrabCamera 节省了你的时间，考虑[每月 $5 赞助](https://github.com/sponsors/Michael-A-Kuykendall)——它让项目保持免费和持续开发。感谢我们的 [赞助者](../../SPONSORS.md) 🙏

---

*Made with Rust 🦀*
