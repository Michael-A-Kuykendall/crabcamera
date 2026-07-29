<div align="center">

# CrabCamera 中文文檔中心

### 生產級跨平台相機採集與錄製庫 · 兼容 Tauri 插件與獨立運行模式 · 永久免費

**繁體中文** · [简体中文](../zh-CN/README.md) · [English](../../README.md)

[![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/crabcamera?style=social)](https://github.com/Michael-A-Kuykendall/crabcamera/stargazers)
[![Crates.io](https://img.shields.io/crates/v/crabcamera.svg)](https://crates.io/crates/crabcamera)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![💝 赞助项目](https://img.shields.io/badge/💝_赞助项目-ea4aaa?style=flat&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)

</div>

---

## 歡迎使用 CrabCamera

CrabCamera 是一個用純 **Rust** 編寫的跨平台相機採集與錄製庫。它提供統一的相機訪問介面，覆蓋 Windows、macOS 和 Linux 平台，支持專業級相機控制、同步音訊錄製和零配置快速啟動。

CrabCamera 附帶可選的 **Tauri 插件集成**，可以讓你的 Tauri 應用直接調用原生相機能力。同時它也提供獨立的 **Headless 模式**，適用於伺服器、CLI 工具或任何純 Rust 項目。

**免費 forever。MIT 許可證。無任何限制。**

---

## 快速開始

### 從 Rust 使用（獨立模式，默認）

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

可選添加功能：
- `recording` — 啟用 MP4 錄製（H.264 + Muxide）
- `audio` — 啟用 OPUS 音訊採集與編碼

### Tauri 插件集成

如果你正在構建 Tauri 應用：

```toml
[dependencies]
crabcamera = { version = "0.9", features = ["tauri"] }
tauri = { version = "2.11" }
```

---

## 支持的平台

| 平台 | 後端 | 相機訪問 | 音訊 | 錄製 |
|------|------|----------|------|------|
| Windows | MediaFoundation | ✅ | ✅ | ✅ |
| macOS | AVFoundation | ✅ | ✅ | ✅ |
| Linux | V4L2 + webkit2gtk | ✅ | ✅ | ✅ |

---

## 功能特性

- 📷 **零配置相機發現** — 自動識別可用相機並列出支持的格式
- 🎞️ **同步 A/V 錄製** — 音訊視頻同步錄製為 MP4 文件
- 🎛️ **專業相機控制** — 曝光、對焦、白平衡、ISO 等手動控制
- 🔍 **自動品質驗證** — 內建模糊檢測、曝光分析和構圖評分
- 📐 **對焦堆棧** — 多幀融合獲得全清晰圖像
- 🖥️ **即時預覽流** — 低延遲 JPEG 預覽
- 🔊 **音訊採集** — CPAL 驅動的低延遲音訊錄製
- 🪶 **零 unsafe 代碼** — 純安全 Rust

---

## 支持 CrabCamera 的發展

🚀 **如果 CrabCamera 对你有帮助，欢迎[赞助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有赞助款项 100% 用于保持项目永久免费。**

- **$5/月**：咖啡档 ☕ 永久感谢 + 赞助者徽章
- **$25/月**：Bug 优先处理档 🐛 优先支持 + 名字收录于 [SPONSORS.md](../../SPONSORS.md)
- **$100/月**：企业支持档 🏢 Logo 展示 + 每月答疑
- **$500/月**：基础设施合作伙伴 🚀 直接支持 + 路线图影响力

[**🎯 成为赞助者**](https://github.com/sponsors/Michael-A-Kuykendall) | 查看我们的 [赞助者](../../SPONSORS.md) 🙏

---

## 文檔中心

- [快速开始](../../README.md) — 英文快速入門
- [用戶手冊](USER_MANUAL.zh-TW.md) — 完整中文用戶手冊
- [API 參考](../../docs/SYSTEM_MAP.md) — 系統架構與 API 概覽

---

## 許可證

[MIT](../../LICENSE-MIT) — 永久免費。

> 💝 如果 CrabCamera 節省了你的時間，考慮[每月 $5 贊助](https://github.com/sponsors/Michael-A-Kuykendall)——它讓項目保持免費和持續開發。感謝我們的 [贊助者](../../SPONSORS.md) 🙏

---

*Made with Rust 🦀*
