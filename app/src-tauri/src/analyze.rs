//! Claude Haiku screenshot analysis — direct Rust replacement for `analyze.py`.

use base64::Engine;
use serde::{Deserialize, Serialize};

const MODEL: &str = "claude-haiku-4-5-20251001";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";

const PROMPT: &str = "你是一个工作活动识别助手。观察这张 macOS 截图，判断用户当前在做什么。\n\n\
## 规则\n\
1. summary：一句中文，15字以内\n\
2. category：**必须且只能**从下方列表中选择一个，禁止自创类别\n\
3. app：识别正在使用的应用名，识别不出时省略\n\n\
## 类别列表（唯一合法值）\n\
- coding   : IDE、代码编辑器、终端（VS Code、Cursor、Xcode、Terminal、iTerm）\n\
- meeting  : 视频/语音会议（Zoom、Google Meet、腾讯会议）\n\
- slack    : Slack\n\
- wechat   : 微信（WeChat）\n\
- feishu   : 飞书（Lark）\n\
- email    : 邮件（Mail、Outlook、Gmail 网页版）\n\
- browser  : 浏览器中的一般网页浏览（不属于其他更具体类别时才选此项）\n\
- reading  : 阅读文档、PDF、技术文章、Notion\n\
- writing  : 写作（Google Docs、Obsidian、Linear 写文档）\n\
- design   : 设计工具（Figma、Sketch）\n\
- data     : 数据工具（Grafana、DataGrip、DBeaver、Google Sheets、数据看板）\n\
- 3d       : 3D 相关应用（Meshy、Blender、Midjourney）\n\
- app      : 其他桌面应用（Discord、Finder、系统设置、预览等）\n\
- other    : 以上均不符合时才选此项\n\n\
返回 JSON：{\"summary\": \"...\", \"category\": \"...\", \"app\": \"...\"}\n\
只输出 JSON，不要输出其他内容。";

#[derive(Debug, Deserialize)]
struct ParsedJson {
    summary: String,
    category: String,
    #[serde(default)]
    app: Option<String>,
}

#[derive(Debug)]
pub struct AnalyzeResult {
    pub summary: String,
    pub category: String,
    pub app: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Claude Haiku pricing as of 2026-04 ($/Mtoken).
pub const PRICE_INPUT_PER_MTOK: f64 = 0.80;
pub const PRICE_OUTPUT_PER_MTOK: f64 = 4.00;

pub fn estimate_cost_usd(input_tokens: u32, output_tokens: u32) -> f64 {
    (input_tokens as f64 * PRICE_INPUT_PER_MTOK
        + output_tokens as f64 * PRICE_OUTPUT_PER_MTOK)
        / 1_000_000.0
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Msg<'a>>,
}

#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: Vec<MsgContent<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum MsgContent<'a> {
    #[serde(rename = "image")]
    Image { source: ImageSource<'a> },
    #[serde(rename = "text")]
    Text { text: &'a str },
}

#[derive(Serialize)]
struct ImageSource<'a> {
    #[serde(rename = "type")]
    type_: &'a str,
    media_type: &'a str,
    data: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

/// Build the full prompt, optionally including a frontmost-app hint.
fn build_prompt(app_hint: Option<&str>) -> String {
    match app_hint {
        Some(app) => format!(
            "{PROMPT}\n\n（系统检测到当前前台应用为：{app}，请结合截图内容综合判断）"
        ),
        None => PROMPT.to_string(),
    }
}

/// CLI first, API fallback.
pub async fn analyze_screenshot(
    image_bytes: &[u8],
    api_key: &str,
    app_hint: Option<&str>,
) -> Result<AnalyzeResult, String> {
    let prompt = build_prompt(app_hint);
    match cli_analyze(image_bytes, &prompt).await {
        Ok(result) => Ok(result),
        Err(cli_err) => {
            eprintln!("[day-monitor] CLI failed ({cli_err}), falling back to API");
            api_analyze(image_bytes, api_key, &prompt).await
        }
    }
}

async fn cli_analyze(image_bytes: &[u8], prompt: &str) -> Result<AnalyzeResult, String> {
    let tmp = std::env::temp_dir().join("day-monitor-capture.png");
    tokio::fs::write(&tmp, image_bytes)
        .await
        .map_err(|e| format!("write tmp: {e}"))?;

    let output = tokio::process::Command::new("claude")
        .arg("-p")
        .arg(prompt)
        .arg(tmp.to_str().unwrap_or_default())
        .arg("--model")
        .arg(MODEL)
        .arg("--output-format")
        .arg("text")
        // Strip inherited routing/auth overrides (e.g. from a claude-aigw shell):
        // the CLI must always run on subscription against the official API,
        // or a gateway without this model turns every call into a 400 + alert.
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_BASE_URL")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .env_remove("ANTHROPIC_MODEL")
        .env_remove("ANTHROPIC_SMALL_FAST_MODEL")
        .env_remove("ANTHROPIC_CUSTOM_MODEL_OPTION")
        .output()
        .await
        .map_err(|e| format!("spawn: {e}"))?;

    let _ = tokio::fs::remove_file(&tmp).await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("exit {}: {stderr}", output.status));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // CLI uses subscription — tokens/cost not counted against API budget
    parse_response(&text, 0, 0)
}

async fn api_analyze(
    image_bytes: &[u8],
    api_key: &str,
    prompt: &str,
) -> Result<AnalyzeResult, String> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
    let req = AnthropicRequest {
        model: MODEL,
        max_tokens: 100,
        messages: vec![Msg {
            role: "user",
            content: vec![
                MsgContent::Image {
                    source: ImageSource {
                        type_: "base64",
                        media_type: "image/png",
                        data: b64,
                    },
                },
                MsgContent::Text { text: prompt },
            ],
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
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("anthropic {status}: {body}"));
    }

    let parsed: AnthropicResponse = resp.json().await.map_err(|e| format!("decode: {e}"))?;
    let usage = parsed.usage.unwrap_or(Usage {
        input_tokens: 0,
        output_tokens: 0,
    });
    let text = parsed
        .content
        .into_iter()
        .map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    parse_response(&text, usage.input_tokens, usage.output_tokens)
}

fn parse_response(
    raw: &str,
    input_tokens: u32,
    output_tokens: u32,
) -> Result<AnalyzeResult, String> {
    let mut text = raw.trim().to_string();
    if text.starts_with("```") {
        let after_first = text.trim_start_matches("```");
        let after_lang = after_first
            .trim_start_matches("json")
            .trim_start_matches('\n');
        let before_close = after_lang
            .rsplit_once("```")
            .map(|(s, _)| s)
            .unwrap_or(after_lang);
        text = before_close.trim().to_string();
    }

    let p: ParsedJson =
        serde_json::from_str(&text).map_err(|e| format!("parse: {e}; raw: {text}"))?;
    let category = normalize_category(&p.category, p.app.as_deref());
    Ok(AnalyzeResult {
        summary: p.summary,
        category,
        app: p.app,
        input_tokens,
        output_tokens,
    })
}

/// Summaries the model produces when the screen content is not recognizable
/// (black screen, blank screen). These events are noise — mostly idle/asleep
/// displays — and should not be stored.
const UNIDENTIFIABLE_MARKERS: &[&str] =
    &["黑屏", "黑暗", "全黑", "黑色", "空白", "未响应", "无法识别"];

pub fn is_unidentifiable(summary: &str) -> bool {
    !summary.is_empty() && UNIDENTIFIABLE_MARKERS.iter().any(|m| summary.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_screen_summaries_are_unidentifiable() {
        for s in [
            "屏幕黑屏，无法识别具体活动",
            "屏幕黑暗，无法识别具体活动",
            "屏幕为黑色，无法识别具体操作",
            "屏幕全黑，无法识别具体活动",
            "屏幕空白，无明显工作内容",
            "屏幕黑屏或应用未响应",
        ] {
            assert!(is_unidentifiable(s), "{s}");
        }
    }

    #[test]
    fn normal_summaries_are_identifiable() {
        for s in ["在 VS Code 写代码", "开 Zoom 会议", "浏览技术文档", "回复 Slack 消息", ""] {
            assert!(!is_unidentifiable(s), "{s}");
        }
    }
}

const VALID_CATEGORIES: &[&str] = &[
    "coding", "meeting", "slack", "wechat", "feishu", "email",
    "browser", "reading", "writing", "design", "data", "3d", "app", "other",
];

fn normalize_category(raw: &str, app_name: Option<&str>) -> String {
    let cat = raw.trim().to_lowercase();
    if VALID_CATEGORIES.contains(&cat.as_str()) {
        return cat;
    }
    if let Some(app) = app_name {
        let app_lower = app.trim().to_lowercase();
        let mapped = match app_lower.as_str() {
            "code" | "visual studio code" | "cursor" | "xcode"
            | "terminal" | "iterm2" | "warp" | "alacritty" => "coding",
            "zoom" | "google meet" => "meeting",
            "slack" => "slack",
            "wechat" | "微信" => "wechat",
            "lark" | "飞书" | "feishu" => "feishu",
            "mail" | "outlook" => "email",
            "figma" | "sketch" => "design",
            "datagrip" | "dbeaver" => "data",
            "meshy" | "blender" => "3d",
            "discord" | "finder" => "app",
            _ => "other",
        };
        return mapped.to_string();
    }
    "other".to_string()
}
