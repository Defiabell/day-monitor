# day-monitor

macOS 工作活动监控工具。每 10 秒截一次屏，用 Claude Haiku 识别你在做什么，每天生成时间轴 + 分类统计报告。

## 安装

```bash
cd personal-projects/day-monitor
pip install -r requirements.txt
export ANTHROPIC_API_KEY=sk-ant-...
```

## 使用

```bash
# 启动监控（后台运行）
python monitor.py start

# 停止监控
python monitor.py stop

# 生成今天的日报
python monitor.py report

# 生成指定日期的日报
python monitor.py report --date 2026-04-27

# 设置开机自启（登录时自动 start）
python monitor.py install

# 取消开机自启
python monitor.py uninstall
```

## 报告位置

`~/Documents/day-monitor/YYYY-MM-DD.md`

报告包含：
- **时间轴**：每段连续活动的开始时间、时长、描述
- **分类统计**：coding / meeting / browsing / reading / communication / design / other 各自时长和占比
- **今日小结**：2-3 句中文总结

## 数据位置

| 路径 | 内容 |
|------|------|
| `~/.day-monitor/monitor.db` | SQLite 数据库（30 天自动清理） |
| `~/.day-monitor/monitor.pid` | 后台进程 PID |
| `~/.day-monitor/stdout.log` | daemon 模式标准输出 |
| `~/.day-monitor/stderr.log` | daemon 模式错误日志 |

## 费用估算

约 $0.5–$1 / 工作日（Claude Haiku，含去重优化，预计跳过 ~65% 截图）。

## 活动分类

| 类别 | 触发场景 |
|------|---------|
| `coding` | IDE、编辑器、终端 |
| `meeting` | Zoom、Slack huddle、视频会议 |
| `browsing` | 浏览器（非文档阅读） |
| `reading` | 文档、PDF、文章 |
| `communication` | Slack、邮件、微信 |
| `design` | Figma、设计工具 |
| `other` | 无法归类 |
