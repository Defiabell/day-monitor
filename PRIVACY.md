# 隐私政策 / Privacy Policy

**Day Monitor** 是一个本地工具，但因为它截屏并把截图发给第三方 AI 服务做识别，
有几个隐私关键点必须明确告知用户。

This is a local tool, but it screenshots and sends images to a third-party AI
service. Read this before using.

---

## 简体中文

### 截图怎么处理

1. **截屏**：每 ~20 秒（可配置）调用 macOS 系统命令 `screencapture` 截一次主屏幕
2. **本地处理**：截图在 RAM 里完成 perceptual hash 计算和分辨率压缩，临时文件立即删除
3. **发送给 Anthropic**：压缩后的截图（PNG，默认 640px 宽）通过 HTTPS 发送到 Anthropic Claude Haiku API
4. **写入本地 DB**：API 返回的"一句中文描述 + 分类"被写入 `~/.day-monitor/monitor.db`
5. **图片不入库**：截图本身**绝不会**写入数据库或日志文件，只保留它的感知哈希（用于去重）

### 截图发给谁

只发给 [Anthropic](https://www.anthropic.com)（Claude Haiku 模型）。
不发给本项目作者，也不发给任何其他第三方。

Anthropic 的隐私政策：https://www.anthropic.com/privacy

注意 Anthropic 可能会保留 API 请求一段时间用于安全审计。

### 不要在敏感场景使用

强烈建议**不要**在以下场景下打开本工具：

- 密码管理器、银行 / 支付页面、加密钱包
- 公司机密文档、未公开的财务数据
- 医疗记录、个人证件 / 身份信息
- 私密聊天、感情对话

如果手机/电脑同屏镜像了上述内容，截图会带上去。

### 自动跳过场景

工具会在以下情况下**主动跳过**截图：

- 屏幕关闭（macOS Display sleep）
- 屏幕锁定
- 用户手动 Pause

但识别这些状态有延迟（最多一个间隔周期）。最稳妥的方式是看到敏感内容时立刻 Pause。

### 本地数据控制

| 路径 | 内容 | 你的控制 |
|------|------|---------|
| `~/.day-monitor/.env` | API Key | 自己输入，可随时删除 |
| `~/.day-monitor/monitor.db` | 事件 + token 用量 | 默认保留 30 天，可设永久或更短 |
| `~/.day-monitor/config.json` | 设置 | 删除即恢复默认 |
| `~/Documents/day-monitor/*.md` | AI 日报 | 完全本地文件 |

随时 `rm -rf ~/.day-monitor` 即可删光所有本地数据。

### 网络请求

本工具只对 `https://api.anthropic.com` 发起请求。**不发任何 telemetry 给本项目作者**。

### 开源审查

代码完全开源（MIT），你可以审计：

- 截图发送逻辑：`app/src-tauri/src/analyze.rs`
- 数据库 schema：`app/src-tauri/src/db.rs`
- 设置存储：`app/src-tauri/src/settings.rs`

---

## English

### How screenshots are handled

1. Every ~20s (configurable), `screencapture` takes a screenshot of the primary display
2. The image is hashed and downscaled in RAM; the temp PNG file is deleted immediately
3. The downscaled PNG is sent over HTTPS to Anthropic Claude Haiku
4. The model's response (a one-line summary + category) is written to a local SQLite DB
5. **The image is never persisted** — only its perceptual hash (for dedup)

### Where the screenshot goes

Only to [Anthropic](https://www.anthropic.com). Not to this project's author or any other third party.

See Anthropic's privacy policy: https://www.anthropic.com/privacy

### Do not use with sensitive content visible

Avoid running while the following is on-screen:

- Password managers, banking, payment, crypto wallets
- Confidential documents, unreleased financials
- Medical records, ID documents
- Private conversations

### Automatic skips

Capture is skipped when:

- Display is off
- Screen is locked
- User has paused via tray menu

### Local data

All data lives under `~/.day-monitor/` and `~/Documents/day-monitor/`. `rm -rf ~/.day-monitor` wipes everything.

### Open source

Full source under MIT. Audit the network code in `app/src-tauri/src/analyze.rs`.
