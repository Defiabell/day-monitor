# Day Monitor — 安装与使用指南

一个 macOS 菜单栏小工具，每 20 秒截一次屏，用 Claude AI 识别你在做什么，自动统计你一天的时间分配，生成可视化日报。

---

## 系统要求

- **macOS 12 (Monterey) 或更高**
- **Apple Silicon (M1/M2/M3/M4) 或 Intel Mac** —— Universal binary，两种都能跑
- **Anthropic API Key** —— 没有的话去 https://console.anthropic.com 注册一个，有 $5 免费额度

---

## 安装步骤

### 1. 解压并放进 Applications

```bash
unzip Day-Monitor-0.1.0-universal.zip
mv "Day Monitor.app" /Applications/
```

或者直接 Finder 双击 zip → 把 `Day Monitor.app` 拖到 Applications 文件夹。

### 2. 解除 Gatekeeper 隔离

因为这个 app **没有花 $99/年的 Apple Developer 证书签名**，macOS 会拦它。需要解除隔离标记：

```bash
xattr -dr com.apple.quarantine "/Applications/Day Monitor.app"
```

> **如果不做这步**：双击 app 时会弹"无法验证开发者，不能打开"。要么按本步操作，要么右键 .app → "打开" → 在弹出的对话框点"打开"。

### 3. 配置 API Key

第一次启动 app 之前，建一个文件 `~/.day-monitor/.env`：

```bash
mkdir -p ~/.day-monitor
echo 'ANTHROPIC_API_KEY=sk-ant-你的key' > ~/.day-monitor/.env
chmod 600 ~/.day-monitor/.env
```

（也可以启动 app 后从菜单栏 → Settings 里输入）

### 4. 启动

双击 `/Applications/Day Monitor.app`，菜单栏右上角会出现一个图标。

### 5. 授权截屏权限

第一次启动会弹「想要录制此电脑的屏幕和音频」：

1. 点 **「打开系统设置」**
2. 在「隐私与安全性 → 录屏与系统录音」里找到 **Day Monitor**
3. 打开开关
4. 系统会提示重启 app —— 退出再打开就好

---

## 使用

### 菜单栏

- **左键点图标** → 弹出小面板，看今日总时长 + 各分类占比 + 当前活动 + Pause/Dashboard 按钮
- **右键点图标** → 系统菜单：
  - Open Dashboard（完整窗口，6 个视图）
  - Settings（API Key、登录自启）
  - Quit（彻底退出）

### Dashboard 6 个视图

| 视图 | 内容 |
|------|------|
| **Timeline** | 今日横向时间条 + 详细时段表 |
| **Categories** | 分类饼图（Today / 7d / 30d 切换）|
| **Trends** | 跨日堆叠柱状图 |
| **Apps** | top 10 应用使用排行 |
| **Events** | 可搜索可过滤的所有截图记录 |
| **AI Report** | Claude 生成的 Markdown 日报 |

### 15 个活动分类

| 类别 | 触发场景 |
|------|---------|
| `coding` | VS Code、Cursor、终端、Xcode、任何 IDE |
| `meeting` | Zoom、Google Meet、腾讯会议 |
| `slack` | Slack |
| `wechat` | 微信 |
| `feishu` | 飞书 |
| `email` | Mail、Outlook、Gmail |
| `browser` | 浏览器中的网页浏览（非文档阅读）|
| `reading` | 阅读文档、PDF、技术文章、Notion 长文 |
| `writing` | 笔记 / 文档写作（Notion / Obsidian / Bear 等专注写作）|
| `design` | 2D 设计（Figma、Sketch、Photoshop） |
| `3d` | 3D 建模 / 浏览（Blender、Maya、3D 模型查看器、AI 3D 工具）|
| `media` | 视频音频剪辑 / 观看（Premiere、Final Cut、QuickTime）|
| `data` | 数据分析（Excel、Numbers、看板、BI、Grafana、Databricks）|
| `system` | 系统设置 / Finder / 文件管理 |
| `other` | 上述都不符合 |

外加每条记录还会识别**应用名**（`VS Code`、`Slack`、`Chrome` 等），用于 Apps 视图的排行。

---

## 隐私与费用

### 截图怎么处理

- 截图**只在本地短暂存在**（写到 /tmp，分析完立刻删）
- **图片不入库**，数据库只存：时间戳 + 感知哈希 + 一句中文描述 + 分类 + 应用名
- 屏幕**未变化时**（感知哈希相近）跳过 API 调用，仅累加时长——平均 60% 截图被跳过
- 屏幕**关闭/锁屏**时跳过截图

### 截图发给谁了

发给 **Anthropic 的 Claude Haiku API**（claude-haiku-4-5），用于识别活动。
Anthropic 的隐私政策：https://www.anthropic.com/privacy

如果担心隐私，**别在敏感屏幕（密码管理器、银行、医疗）开它**。

### 费用

| 场景 | API 调用次数/天 | 费用/天 |
|------|--------------|--------|
| 标准 8h 工作日 | ~1000-1700 次 | **$0.4 – $1** |

按 Claude Haiku 价格估算（$0.80/Mtoken input，$4/Mtoken output）。

---

## 数据位置

| 路径 | 内容 |
|------|------|
| `~/.day-monitor/.env` | 你的 API Key |
| `~/.day-monitor/monitor.db` | SQLite 数据库（30 天自动清理） |
| `~/Documents/day-monitor/` | （可选）AI 日报 Markdown 文件 |
| `~/Library/Application Support/com.jinkunsun.daymonitor/` | Tauri 内部状态 |

### 直接看数据

```bash
# 安装 GUI 浏览器
brew install --cask db-browser-for-sqlite
open -a "DB Browser for SQLite" ~/.day-monitor/monitor.db

# 或命令行查
sqlite3 ~/.day-monitor/monitor.db \
  "SELECT timestamp, category, app_name, summary FROM events ORDER BY id DESC LIMIT 20"
```

---

## 常见问题

### Q：每次开机想自动启动？
Settings → 勾选「Start Day Monitor on login」。

### Q：怎么停掉？
菜单栏图标右键 → Quit。

### Q：监控停掉后还想继续运行老数据？
DB 不会丢，重新打开 app 后所有历史数据都在 Dashboard 里。

### Q：菜单栏图标消失了？
可能是 macOS 菜单栏自己的 bug。运行 `killall SystemUIServer` 刷新一下。

### Q：每次打开都问截屏授权？

这是 **macOS TCC** 对未付费签名应用的限制：

**原理**：
- macOS 把"哪个 app 有截屏权限"绑定在 `(签名身份 + binary CDHash)` 组合上
- 只要 binary 内容变了（升级版本），CDHash 就变，macOS 视为"新 app"再问一次
- 真正不再问的方法是用 Apple Developer ID（$99/年），签名时 macOS 只看 Team ID，CDHash 怎么变都认

**怎么办**：

如果只是**首次启动**问授权，正常授权就行（在录屏列表打开开关）。

如果**升级到新版本后又开始问**，按以下步骤一键清旧记录：

```bash
# 清空 Day Monitor 的所有 TCC 授权记录
tccutil reset ScreenCapture com.jinkunsun.daymonitor
tccutil reset All com.jinkunsun.daymonitor

# 重启 app
osascript -e 'tell application "Day Monitor" to quit'
open -a "Day Monitor"
```

然后给当前版本重新授权一次。下次启动相同版本不会再问。

**手动方式**（不熟命令行）：
1. 系统设置 → 隐私与安全性 → 录屏与系统录音
2. 选中 Day Monitor → 点 `-` 按钮删掉
3. 重启 app
4. 弹授权时点「打开系统设置」重新加进来

### Q：报告里 "other" 占比很高？
旧数据是用旧 prompt 打的标，分类粒度更粗。新数据会更准。或者那段时间确实在做杂事/启动画面/黑屏。

### Q：能改截图频率？
当前固定 20 秒一次。代码里 `loop_runner.rs` 的 `INTERVAL_SECS` 常量可以改，但需要重新打包。

---

## 卸载

```bash
# 退出 app
osascript -e 'tell application "Day Monitor" to quit'

# 删 .app
rm -rf "/Applications/Day Monitor.app"

# 删数据（如果不想保留历史）
rm -rf ~/.day-monitor

# 删 launchd（如果开了开机自启）
rm -f ~/Library/LaunchAgents/com.jinkunsun.daymonitor.plist
```

---

## 报告问题 / 反馈

这是个个人项目，不保证维护。但如果有 bug 或建议，可以告诉作者。

Built with: Tauri 2 (Rust + React/TypeScript) + Anthropic Claude Haiku.
