use crate::db::Event;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::analyze::Endpoint;

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

fn weekly_cache_path(week_id: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".day-monitor")
        .join("reports")
        .join(format!("weekly-{week_id}.md"))
}

/// "2026-08-27" → "2026-W35" (ISO week).
pub fn iso_week_id(date: &str) -> Option<String> {
    use chrono::Datelike;
    let d = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let iso = d.iso_week();
    Some(format!("{}-W{:02}", iso.year(), iso.week()))
}

fn fmt_hm(total_s: u32) -> String {
    let h = total_s / 3600;
    let m = (total_s % 3600) / 60;
    if h > 0 {
        format!("{h}h{m:02}m")
    } else {
        format!("{m}m")
    }
}

fn build_daily_prompt(date: &str, events_text: &str) -> String {
    let cats = crate::analyze::VALID_CATEGORIES.join("/");
    format!(
        "你是一个工作日报生成助手。以下是用户 {date} 的工作记录（每条包含时间、持续秒数、活动描述、分类）：\n\n\
{events_text}\n\n\
请生成一份工作日报，格式如下（严格使用 Markdown）：\n\n\
# 工作日报 {date}\n\n\
## 时间轴\n| 时间 | 时长 | 活动 |\n|------|------|------|\n\
（合并相邻相似活动为连续时间块，时间格式 HH:MM – HH:MM，时长用 Xm 或 XhYm）\n\n\
## 分类统计\n| 类别 | 时长 | 占比 |\n|------|------|------|\n\
（统计 {cats} 各自总时长和百分比，跳过时长为 0 的类别）\n\n\
## 今日洞察\n\
- **未完成的线索**：从记录里找出看起来没做完、明天可能要继续的事（1-3 条；找不到就写「无」）\n\
- **潜在待办**：记录中出现过、像待办但没见到完成迹象的事项（0-3 条）\n\
- **焦点与切换**：最长的专注块是什么；上下文切换是否频繁（1-2 句）\n\n\
## 今日小结\n\
（2-3 句总结，中文；如果有 ai-coding 时间，点一句 AI 代理干活与自己动手的比例）"
    )
}

/// Per-day digest fed to the weekly-report prompt. Computed in Rust so the
/// LLM input stays compact regardless of how many raw events a week has.
fn build_weekly_digest(days: &[(String, Vec<Event>)]) -> String {
    use std::collections::HashMap;
    const WEEKDAYS: [&str; 7] = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

    let mut out = String::new();
    for (date, events) in days {
        let weekday = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map(|d| {
                use chrono::Datelike;
                WEEKDAYS[d.weekday().num_days_from_monday() as usize]
            })
            .unwrap_or("");
        if events.is_empty() {
            out.push_str(&format!("### {date} {weekday}\n无记录\n\n"));
            continue;
        }
        let total: u32 = events.iter().map(|e| e.duration_s).sum();

        let mut by_cat: HashMap<&str, u32> = HashMap::new();
        let mut by_app: HashMap<&str, u32> = HashMap::new();
        for e in events {
            *by_cat.entry(e.category.as_str()).or_insert(0) += e.duration_s;
            if let Some(app) = e.app_name.as_deref() {
                *by_app.entry(app).or_insert(0) += e.duration_s;
            }
        }
        let mut cats: Vec<(&str, u32)> = by_cat.into_iter().collect();
        cats.sort_by(|a, b| b.1.cmp(&a.1));
        let cats_str = cats
            .iter()
            .map(|(c, s)| format!("{c} {}", fmt_hm(*s)))
            .collect::<Vec<_>>()
            .join("、");

        let mut apps: Vec<(&str, u32)> = by_app.into_iter().collect();
        apps.sort_by(|a, b| b.1.cmp(&a.1));
        let apps_str = apps
            .iter()
            .take(3)
            .map(|(a, _)| a.to_string())
            .collect::<Vec<_>>()
            .join("、");

        let mut top_events: Vec<&Event> = events.iter().collect();
        top_events.sort_by(|a, b| b.duration_s.cmp(&a.duration_s));
        let sums_str = top_events
            .iter()
            .take(5)
            .map(|e| e.summary.clone())
            .collect::<Vec<_>>()
            .join("；");

        out.push_str(&format!(
            "### {date} {weekday}（共 {}）\n分类: {}\n主要应用: {}\n代表活动: {}\n\n",
            fmt_hm(total),
            cats_str,
            apps_str,
            sums_str
        ));
    }
    out
}

fn build_weekly_prompt(week_id: &str, from: &str, to: &str, digest: &str) -> String {
    format!(
        "你是一个工作周报生成助手。以下是用户 {from} ~ {to} 每天的工作记录摘要（已按天聚合）：\n\n\
{digest}\n\
请生成一份工作周报，格式如下（严格使用 Markdown）：\n\n\
# 工作周报 {week_id}（{from} ~ {to}）\n\n\
## 本周概览\n（总时长、整体节奏与主线，2-3 句）\n\n\
## 每日一行\n| 日期 | 时长 | 主要内容 |\n|------|------|------|\n\n\
## 分类时间分配\n| 类别 | 时长 | 占比 |\n|------|------|------|\n（只列有数据的类别）\n\n\
## AI 协作观察\n（ai-coding 与 coding 的时间比例和观察；本周没有 ai-coding 数据就写「本周无数据」）\n\n\
## 本周洞察\n（工作模式、专注度、和往常不同的变化，2-4 条）\n\n\
## 未完成与下周建议\n（从记录里看出的未完成线索和建议，1-3 条）"
    )
}

/// Weekly report over the given (date, events) days — expected to be the 7
/// days ending on the requested date. Cached per ISO week; a mid-week
/// generation is a partial week, use force=true to refresh later.
pub async fn generate_weekly_report(
    days: Vec<(String, Vec<Event>)>,
    api_key: &str,
    force: bool,
) -> Result<String, String> {
    let (from, to) = match (days.first(), days.last()) {
        (Some((f, _)), Some((t, _))) => (f.clone(), t.clone()),
        _ => return Err("No days given".into()),
    };
    let week_id = iso_week_id(&to).ok_or("bad date")?;
    let cache = weekly_cache_path(&week_id);
    if !force && cache.exists() {
        return std::fs::read_to_string(&cache).map_err(|e| e.to_string());
    }
    if days.iter().all(|(_, e)| e.is_empty()) {
        return Err("No events in that week".into());
    }

    let digest = build_weekly_digest(&days);
    let prompt = build_weekly_prompt(&week_id, &from, &to, &digest);

    let endpoint = Endpoint::from_settings(&crate::settings::get());
    let markdown = if endpoint.custom {
        // See analyze.rs: the CLI would route to the official API regardless.
        api_generate(&prompt, api_key, &endpoint).await?
    } else {
        match cli_generate(&prompt, &endpoint.model).await {
        Ok(text) => text,
        Err(cli_err) => {
            eprintln!("[day-monitor] CLI weekly report failed ({cli_err}), falling back to API");
            api_generate(&prompt, api_key, &endpoint).await?
            }
        }
    };

    if let Some(parent) = cache.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&cache, &markdown);
    Ok(markdown)
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

    let prompt = build_daily_prompt(date, &events_text);

    // CLI first, API fallback
    let endpoint = Endpoint::from_settings(&crate::settings::get());
    let markdown = if endpoint.custom {
        // See analyze.rs: the CLI would route to the official API regardless.
        api_generate(&prompt, api_key, &endpoint).await?
    } else {
        match cli_generate(&prompt, &endpoint.model).await {
        Ok(text) => text,
        Err(cli_err) => {
            eprintln!("[day-monitor] CLI report failed ({cli_err}), falling back to API");
            api_generate(&prompt, api_key, &endpoint).await?
            }
        }
    };

    if let Some(parent) = cache.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&cache, &markdown);
    Ok(markdown)
}

async fn cli_generate(prompt: &str, model: &str) -> Result<String, String> {
    let output = tokio::process::Command::new("claude")
        .arg("-p")
        .arg(prompt)
        .arg("--model")
        .arg(model)
        .arg("--output-format")
        .arg("text")
        // Same env scrub as analyze.rs — see cli_analyze for rationale.
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_BASE_URL")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .env_remove("ANTHROPIC_MODEL")
        .env_remove("ANTHROPIC_SMALL_FAST_MODEL")
        .env_remove("ANTHROPIC_CUSTOM_MODEL_OPTION")
        .output()
        .await
        .map_err(|e| format!("spawn: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("exit {}: {stderr}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(ts: &str, summary: &str, category: &str, dur: u32) -> Event {
        Event {
            id: 0,
            timestamp: ts.to_string(),
            summary: summary.to_string(),
            category: category.to_string(),
            app_name: None,
            duration_s: dur,
        }
    }

    #[test]
    fn daily_prompt_uses_v2_categories_and_asks_for_insights() {
        let p = build_daily_prompt("2026-08-27", "09:00 coding");
        assert!(p.contains("ai-coding"));
        assert!(p.contains("entertainment"));
        assert!(!p.contains("media"), "legacy category leaked into prompt");
        assert!(!p.contains("slack"), "legacy category leaked into prompt");
        assert!(p.contains("未完成"), "insights section missing");
    }

    #[test]
    fn iso_week_id_formats() {
        assert_eq!(iso_week_id("2026-08-27").unwrap(), "2026-W35");
        assert!(iso_week_id("garbage").is_none());
    }

    #[test]
    fn weekly_digest_lists_days_categories_and_empty_marker() {
        let days = vec![
            (
                "2026-08-24".to_string(),
                vec![
                    ev("2026-08-24T09:00:00", "写代码", "coding", 3600),
                    ev("2026-08-24T10:00:00", "回消息", "communicating", 600),
                ],
            ),
            ("2026-08-25".to_string(), vec![]),
        ];
        let d = build_weekly_digest(&days);
        assert!(d.contains("2026-08-24"));
        assert!(d.contains("coding"));
        assert!(d.contains("写代码"));
        assert!(d.contains("2026-08-25"));
        assert!(d.contains("无记录"));
    }
}

async fn api_generate(
    prompt: &str,
    api_key: &str,
    endpoint: &Endpoint,
) -> Result<String, String> {
    let req = AnthropicRequest {
        model: &endpoint.model,
        max_tokens: 2000,
        messages: vec![Msg {
            role: "user",
            content: prompt.to_string(),
        }],
    };

    let client = reqwest::Client::new();
    let mut builder = client
        .post(&endpoint.url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json");
    // Both auth headers on a custom endpoint — see api_analyze for why.
    if endpoint.custom {
        builder = builder.bearer_auth(api_key);
    }
    let resp = builder
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
    Ok(parsed
        .content
        .into_iter()
        .map(|c| c.text)
        .collect::<Vec<_>>()
        .join(""))
}
