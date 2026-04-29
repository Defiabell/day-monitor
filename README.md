# Day Monitor

> macOS 菜单栏小工具：自动追踪你一天的工作内容，本地分析后生成可视化日报。

每 ~20 秒截一次屏 → Claude Haiku 识别活动类型 → SQLite 记录 → 仪表板展示时间轴/分类/趋势/AI 日报。

[简体中文](#简体中文) · [English](#english)

---

## 简体中文

### 截图预览

（截图占位 - 实际使用时菜单栏弹出 popover 显示今日总时长 + 分类占比；点 Dashboard 进入完整窗口看 6 个视图）

### 这个项目能做什么

- 🕐 **自动追踪时间** — 不需要手动开关 timer
- 🤖 **AI 识别活动** — Claude 看截图直接告诉你"在 VS Code 写代码"或"在微信回消息"
- 📊 **可视化** — 时间轴、分类饼图、跨日趋势、应用排行
- 💰 **可控成本** — 内置 token / 费用统计 + 月度预算上限，避免 API bill 失控
- 🔒 **隐私本地** — 截图不入库（只存 hash 和文字描述），数据全在本机

### 系统要求

- **macOS 12+** (Apple Silicon Mac，arm64-only 当前版本)
- **Anthropic API Key** ([注册](https://console.anthropic.com)，新用户有 $5 免费额度)

### 快速开始

**最简单：[下载预编译 .app](dist/Day-Monitor-0.1.0-arm64.zip)**（见 [INSTALL.md](dist/INSTALL.md)）

**自己编译：**

```bash
git clone https://github.com/yourname/day-monitor
cd day-monitor

# Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 前端依赖
cd app && yarn install

# 开发模式
yarn tauri dev

# 打包发布
yarn tauri build
# 产物：app/src-tauri/target/release/bundle/macos/Day Monitor.app
```

### 架构

```
┌────────────────────────────────────────────────┐
│  Day Monitor.app  (Tauri 2 / Rust + React)      │
│                                                   │
│  Menu Bar ──click──> Popover (200×300)          │
│      │                                           │
│      ├──> Dashboard (1100×700, 6 views)         │
│      └──> Settings (interval, budget, etc.)     │
│                                                   │
│  Rust Backend                                    │
│   ├─ tokio loop: capture → hash → analyze       │
│   ├─ /usr/sbin/screencapture (native, no Python)│
│   ├─ Claude Haiku via reqwest                   │
│   └─ rusqlite (~/.day-monitor/monitor.db)       │
└────────────────────────────────────────────────┘
```

### 项目结构

```
day-monitor/
├── app/                          # Tauri app
│   ├── src/                       # React + TypeScript frontend
│   │   ├── popover/               # 菜单栏 popover
│   │   ├── dashboard/             # 6 个视图
│   │   ├── settings/              # 设置窗口
│   │   └── lib/                   # 共享类型 + invoke 包装
│   └── src-tauri/src/             # Rust backend
│       ├── lib.rs                 # 入口、tray、窗口管理
│       ├── loop_runner.rs         # tokio 监控循环
│       ├── capture.rs             # 截屏 + 感知哈希
│       ├── analyze.rs             # Claude API 调用
│       ├── db.rs                  # SQLite 读写
│       ├── stats.rs               # 数据聚合
│       ├── report.rs              # AI 日报生成
│       ├── settings.rs            # 用户配置
│       └── commands.rs            # Tauri commands
├── dist/                          # 发布包（zip + 安装脚本 + 用户文档）
├── docs/                          # 设计文档
├── LICENSE                         # MIT
├── PRIVACY.md                      # 隐私政策
└── README.md                       # 本文件
```

### 配置项

进 Settings 窗口可调：

| 项 | 默认 | 说明 |
|----|------|------|
| 采集间隔 | 20s | 越短数据越细，越长越省钱 |
| 截图分辨率 | 640px | 影响 token 成本（直接 ~50%）|
| 去重阈值 | 12 | 屏幕几乎没变就跳过，0~20 |
| 数据保留 | 30 天 | 老数据自动清理 |
| 月度预算 | 0 (无限) | 超限自动暂停 API 调用 |

### 隐私

详见 [PRIVACY.md](PRIVACY.md)。简要：

- 截图**只在 RAM 中处理**，分析完立即丢弃
- 数据库存的是：时间戳 + 感知哈希 + 一句中文描述 + 分类，**没有图片本身**
- 截图发给 Anthropic 进行分析，不发给本项目作者
- 屏幕关闭/锁屏时自动跳过

### 贡献

欢迎 PR。关注的方向：

- Intel Mac (x86_64) 支持（universal binary）
- Linux / Windows 支持（需要替换 screencapture）
- 多 LLM 后端（OpenAI、ollama 本地模型）
- i18n（当前 prompt 和 UI 都是中文）
- 更细粒度设置（工作时段、低电量暂停）

### 许可证

MIT License — 见 [LICENSE](LICENSE)。

---

## English

### What it does

A macOS menu-bar app that takes a screenshot every ~20 seconds, sends it to Claude Haiku to identify what you're doing, and stores structured activity data locally for daily reports and visualizations.

### Requirements

- macOS 12+ (Apple Silicon, arm64)
- Anthropic API key

### Build

```bash
git clone https://github.com/yourname/day-monitor
cd day-monitor/app
yarn install
yarn tauri build
```

### License

MIT.
