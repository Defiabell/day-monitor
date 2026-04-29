//! Claude Haiku screenshot analysis — direct Rust replacement for `analyze.py`.

use base64::Engine;
use serde::{Deserialize, Serialize};

const MODEL: &str = "claude-haiku-4-5-20251001";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";

const PROMPT: &str = "你是一个工作活动识别助手。观察这张 macOS 截图，用一句中文描述用户当前在做什么（15字以内），\
并从以下类别中选一个最合适的：\n\n\
- coding     : VS Code、Cursor、终端、Xcode、任何 IDE 或代码编辑器\n\
- meeting    : Zoom、Google Meet、腾讯会议等视频/语音会议\n\
- slack      : Slack\n\
- wechat     : 微信（WeChat）\n\
- feishu     : 飞书（Lark）\n\
- email      : 邮件客户端（Mail、Outlook、Gmail 网页版）\n\
- browser    : 浏览器中的网页浏览（非文档阅读）\n\
- reading    : 阅读文档 / PDF / 技术文章 / Notion 中的长文阅读\n\
- writing    : 笔记 / 文档写作（Notion、Obsidian、Bear 等专注写作场景）\n\
- design     : 2D 设计（Figma、Sketch、Photoshop、Illustrator）\n\
- 3d         : 3D 建模 / 浏览（Blender、Maya、ZBrush、3D 模型查看器、AI 3D 工具）\n\
- media      : 视频/音频剪辑或观看（Premiere、Final Cut、QuickTime、YouTube 全屏观看）\n\
- data       : 数据分析（Excel、Numbers、看板、BI 工具、Grafana、Databricks）\n\
- system     : 系统设置 / Finder / 文件管理 / 启动台\n\
- other      : 以上均不符合\n\n\
同时识别正在使用的应用名（如 \"VS Code\"、\"Slack\"、\"Blender\"、\"Chrome\" 等的具体应用名）。\
识别不出来时省略 app 字段。\n\n\
返回 JSON：{\"summary\": \"...\", \"category\": \"...\", \"app\": \"VS Code\"}";

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

pub async fn analyze_screenshot(
    image_bytes: &[u8],
    api_key: &str,
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
                MsgContent::Text { text: PROMPT },
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
    let mut text = parsed
        .content
        .into_iter()
        .map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    // Strip markdown code fences if present
    let trimmed = text.trim();
    if trimmed.starts_with("```") {
        let after_first = trimmed.trim_start_matches("```");
        let after_lang = after_first.trim_start_matches("json").trim_start_matches('\n');
        let before_close = after_lang.rsplit_once("```").map(|(s, _)| s).unwrap_or(after_lang);
        text = before_close.trim().to_string();
    }

    let p: ParsedJson =
        serde_json::from_str(&text).map_err(|e| format!("parse: {e}; raw: {text}"))?;
    Ok(AnalyzeResult {
        summary: p.summary,
        category: p.category,
        app: p.app,
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
    })
}
