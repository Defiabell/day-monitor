use crate::db::Event;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MODEL: &str = "claude-haiku-4-5-20251001";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Msg<'a>>,
}

#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

fn cache_path(date: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".day-monitor")
        .join("reports")
        .join(format!("{date}.md"))
}

pub async fn generate_report(
    events: Vec<Event>,
    date: &str,
    api_key: &str,
    force: bool,
) -> Result<String, String> {
    let cache = cache_path(date);
    if !force && cache.exists() {
        return std::fs::read_to_string(&cache).map_err(|e| e.to_string());
    }

    if events.is_empty() {
        return Err("No events for that date".into());
    }

    let events_text: String = events
        .iter()
        .map(|e| {
            format!(
                "{} ({}s): {} [{}]",
                e.timestamp, e.duration_s, e.summary, e.category
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "你是一个工作日报生成助手。以下是用户 {date} 的工作记录（每条包含时间、持续秒数、活动描述、分类）：\n\n\
{events_text}\n\n\
请生成一份工作日报，格式如下（严格使用 Markdown）：\n\n\
# 工作日报 {date}\n\n\
## 时间轴\n| 时间 | 时长 | 活动 |\n|------|------|------|\n\
（合并相邻相似活动为连续时间块，时间格式 HH:MM – HH:MM，时长用 Xm 或 XhYm）\n\n\
## 分类统计\n| 类别 | 时长 | 占比 |\n|------|------|------|\n\
（统计 coding/meeting/slack/wechat/feishu/email/browser/reading/design/app/other 各自总时长和百分比，跳过时长为 0 的类别）\n\n\
## 今日小结\n（2-3句总结，中文）"
    );

    let req = AnthropicRequest {
        model: MODEL,
        max_tokens: 2000,
        messages: vec![Msg {
            role: "user",
            content: prompt,
        }],
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(ANTHROPIC_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Anthropic API error {status}: {body}"));
    }

    let parsed: AnthropicResponse = resp.json().await.map_err(|e| e.to_string())?;
    let markdown = parsed
        .content
        .into_iter()
        .map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    if let Some(parent) = cache.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&cache, &markdown);
    Ok(markdown)
}
