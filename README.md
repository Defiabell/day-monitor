# day-monitor

macOS 工作活动自动监控工具。每 10 秒截一次屏，用 Claude Haiku（视觉模型）识别你在做什么，自动去重、分类，每天生成时间轴 + 分类统计日报。

---

## 快速开始

**环境要求：** macOS、Python 3.9+、Anthropic API Key

```bash
# 1. 安装依赖
cd personal-projects/day-monitor
pip install -r requirements.txt

# 2. 配置 API Key（只需一次）
echo 'ANTHROPIC_API_KEY=sk-ant-你的key' > .env

# 3. 启动监控
python monitor.py start

# 4. 下班后生成日报
python monitor.py report
```

---

## 命令参考

| 命令 | 说明 |
|------|------|
| `python monitor.py start` | 启动后台监控 |
| `python monitor.py stop` | 停止监控 |
| `python monitor.py status` | 打印今日活动概况（随时可查，不生成文件） |
| `python monitor.py status --date 2026-04-27` | 查看指定日期概况 |
| `python monitor.py report` | 生成今天的完整日报（Markdown 文件） |
| `python monitor.py report --date 2026-04-27` | 生成指定日期的日报 |
| `python monitor.py install` | 安装 launchd，登录时自动启动 |
| `python monitor.py uninstall` | 卸载 launchd 自启 |

---

## 日报格式

报告保存到 `~/Documents/day-monitor/YYYY-MM-DD.md`，包含三个部分：

```markdown
# 工作日报 2026-04-28

## 时间轴
| 时间 | 时长 | 活动 |
|------|------|------|
| 09:02 – 09:47 | 45m  | 在 Slack 处理消息，回复团队问题 |
| 09:47 – 11:30 | 1h43m | 在 VS Code 写 Python，开发 day-monitor |
| 14:00 – 15:20 | 1h20m | Zoom 视频会议 |

## 分类统计
| 类别 | 时长 | 占比 |
|------|------|------|
| coding | 3h20m | 42% |
| meeting | 1h45m | 22% |
| ...   | ...  | ... |

## 今日小结
今天主要时间用于开发工作...
```

---

## 工作原理

```
每 10 秒截图
    ↓
感知哈希去重（屏幕无变化则跳过，累计时长）
    ↓
Claude Haiku 视觉分析（返回活动描述 + 类别）
    ↓
写入 SQLite（~/.day-monitor/monitor.db）
    ↓
按需生成 Markdown 日报
```

**去重机制：** 计算截图的感知哈希（perceptual hash），与上一张比较汉明距离。距离 < 8（屏幕基本没变）则跳过 API 调用，把上一条记录的时长 +10s。实测可跳过 60–70% 的截图。

---

## 活动分类

| 类别 | 触发场景 |
|------|---------|
| `coding` | VS Code、Cursor、终端、Xcode、任何 IDE |
| `meeting` | Zoom、Google Meet、腾讯会议等视频/语音会议 |
| `slack` | Slack |
| `wechat` | 微信（WeChat） |
| `feishu` | 飞书（Lark） |
| `email` | Mail、Outlook、Gmail 网页版 |
| `browser` | 浏览器中的网页浏览 |
| `reading` | 文档、PDF、技术文章、Notion |
| `design` | Figma、Sketch 等设计工具 |
| `app` | 其他桌面应用（Meshy、产品测试、系统设置等） |
| `other` | 以上均不符合 |

---

## 数据与文件

| 路径 | 内容 |
|------|------|
| `.env` | API Key（不提交 git） |
| `~/.day-monitor/monitor.db` | SQLite 数据库 |
| `~/.day-monitor/monitor.pid` | 后台进程 PID |
| `~/.day-monitor/stdout.log` | daemon 标准输出 |
| `~/.day-monitor/stderr.log` | daemon 错误日志 |
| `~/Documents/day-monitor/` | 生成的日报文件 |
| `~/Library/LaunchAgents/com.daymonitor.plist` | launchd 自启配置 |

**数据保留：** 超过 30 天的记录在每次 `start` 时自动清理。

---

## 费用估算

| 场景 | API 调用次数/天 | 费用/天 |
|------|--------------|--------|
| 最坏（无去重） | ~2,880 次（8h） | ~$2.9 |
| 实际（含去重） | ~900 次 | ~$0.9 |

基于 Claude Haiku 价格，含图片 token。

---

## 项目结构

```
day-monitor/
├── monitor.py      # CLI 入口（click）：start / stop / report / install
├── loop.py         # 监控主循环（截图 → 去重 → 分析 → 存储）
├── capture.py      # 截图（screencapture）+ 感知哈希
├── analyze.py      # Claude Haiku 视觉 API 调用
├── storage.py      # SQLite 读写（init / insert / query / cleanup）
├── report.py       # 日报生成（Claude Haiku 聚合 → Markdown）
├── daemon.py       # launchd plist 安装/卸载
├── tests/          # 单元测试（23 个，mock API，纯本地运行）
├── .env            # API Key（gitignored）
├── .gitignore
└── requirements.txt
```

---

## 查看原始数据

**方式一：DB Browser for SQLite（GUI，推荐）**

```bash
brew install --cask db-browser-for-sqlite
open -a "DB Browser for SQLite" ~/.day-monitor/monitor.db
```

打开后点 **Browse Data → events 表** 即可浏览所有记录。

**方式二：命令行**

```bash
# 查看最近 20 条记录
sqlite3 ~/.day-monitor/monitor.db \
  "SELECT timestamp, category, summary, duration_s FROM events ORDER BY timestamp DESC LIMIT 20"

# 统计今日各分类时长（秒）
sqlite3 ~/.day-monitor/monitor.db \
  "SELECT category, SUM(duration_s) FROM events WHERE timestamp LIKE '$(date +%Y-%m-%d)%' GROUP BY category ORDER BY 2 DESC"
```

---

## 开发

```bash
# 运行所有测试（无需 API Key，全部 mock）
python -m pytest -v

# 查看监控日志（daemon 模式）
tail -f ~/.day-monitor/stderr.log
```

---

## 常见问题

**Q：监控已在运行，如何确认？**
```bash
cat ~/.day-monitor/monitor.pid   # 查看 PID
ps aux | grep monitor.py         # 确认进程存在
```

**Q：想随时看今天进展怎么办？**

```bash
python monitor.py status
```

输出示例：
```
今日概况 2026-04-28  （共 3h20m）

分类统计：
  coding             1h23m   42%  ████████
  meeting              55m   27%  █████
  communication        30m   15%  ███
  browsing             22m   11%  ██
  other                10m    5%  █

最近 10 条活动：
  14:30  [coding       ]  在 VS Code 写 Python，修改 monitor.py
  14:31  [communication]  在 Slack 回复消息
  ...
```

**Q：报告显示"No events found"？**

确认监控已运行（`python monitor.py start`）且当天有截图数据。新启动后需等待至少 10 秒才有第一条记录。

**Q：屏幕共享/敏感内容怎么办？**

截图只在本机处理，图片不存储（只保留哈希值和文字描述），原始截图在分析后立即删除。API 调用会将截图发送给 Anthropic 服务器，请注意工作内容的隐私边界。

**Q：launchd 自启后 API Key 从哪里读？**

`monitor.py` 启动时自动从脚本同目录的 `.env` 文件加载，launchd 无需额外配置。
