from __future__ import annotations

import base64
import json
import re

VALID_CATEGORIES = frozenset({
    'coding', 'meeting', 'slack', 'wechat', 'feishu', 'email',
    'browser', 'reading', 'writing', 'design', 'data', '3d', 'app', 'other',
})

PROMPT_TEMPLATE = (
    '你是一个工作活动识别助手。观察这张 macOS 截图，判断用户当前在做什么。\n\n'
    '## 规则\n'
    '1. summary：一句中文，15字以内\n'
    '2. category：**必须且只能**从下方列表中选择一个，禁止自创类别\n'
    '3. app：识别正在使用的应用名，识别不出时省略\n\n'
    '## 类别列表（唯一合法值）\n'
    '- coding   : IDE、代码编辑器、终端（VS Code、Cursor、Xcode、Terminal、iTerm）\n'
    '- meeting  : 视频/语音会议（Zoom、Google Meet、腾讯会议）\n'
    '- slack    : Slack\n'
    '- wechat   : 微信（WeChat）\n'
    '- feishu   : 飞书（Lark）\n'
    '- email    : 邮件（Mail、Outlook、Gmail 网页版）\n'
    '- browser  : 浏览器中的一般网页浏览（不属于其他更具体类别时才选此项）\n'
    '- reading  : 阅读文档、PDF、技术文章、Notion\n'
    '- writing  : 写作（Google Docs、Obsidian、Linear 写文档）\n'
    '- design   : 设计工具（Figma、Sketch）\n'
    '- data     : 数据工具（Grafana、DataGrip、DBeaver、Google Sheets、数据看板）\n'
    '- 3d       : 3D 相关应用（Meshy、Blender、Midjourney）\n'
    '- app      : 其他桌面应用（Discord、Finder、系统设置、预览等）\n'
    '- other    : 以上均不符合时才选此项\n\n'
    '{active_app_hint}'
    '返回 JSON：{{"summary": "...", "category": "...", "app": "..."}}\n'
    '只输出 JSON，不要输出其他内容。'
)

_ACTIVE_APP_HINT = '当前前台应用（macOS 系统报告）：{app}。请优先用此信息辅助分类。\n\n'

_APP_CATEGORY_MAP = {
    'code': 'coding', 'visual studio code': 'coding', 'cursor': 'coding',
    'xcode': 'coding', 'terminal': 'coding', 'iterm2': 'coding',
    'warp': 'coding', 'alacritty': 'coding',
    'zoom': 'meeting', 'google meet': 'meeting',
    'slack': 'slack',
    'wechat': 'wechat', '微信': 'wechat',
    'lark': 'feishu', '飞书': 'feishu', 'feishu': 'feishu',
    'mail': 'email', 'outlook': 'email',
    'figma': 'design', 'sketch': 'design',
    'datagrip': 'data', 'dbeaver': 'data',
    'meshy': '3d', 'blender': '3d',
    'discord': 'app', 'finder': 'app',
}

MODEL = 'claude-haiku-4-5-20251001'


def _normalize_category(category: str, app_name: str | None) -> str:
    cat = category.strip().lower()
    if cat in VALID_CATEGORIES:
        return cat
    if app_name:
        app_lower = app_name.strip().lower()
        if app_lower in _APP_CATEGORY_MAP:
            return _APP_CATEGORY_MAP[app_lower]
    return 'other'


def analyze_screenshot(image_bytes: bytes, client, active_app: str | None = None) -> dict:
    hint = _ACTIVE_APP_HINT.format(app=active_app) if active_app else ''
    prompt = PROMPT_TEMPLATE.format(active_app_hint=hint)
    b64 = base64.standard_b64encode(image_bytes).decode('utf-8')
    response = client.messages.create(
        model=MODEL,
        max_tokens=100,
        messages=[{
            'role': 'user',
            'content': [
                {
                    'type': 'image',
                    'source': {'type': 'base64', 'media_type': 'image/png', 'data': b64},
                },
                {'type': 'text', 'text': prompt},
            ],
        }],
    )
    text = response.content[0].text.strip()
    text = re.sub(r'^```(?:json)?\s*', '', text)
    text = re.sub(r'\s*```$', '', text)
    result = json.loads(text)
    result['category'] = _normalize_category(
        result.get('category', 'other'),
        result.get('app') or active_app,
    )
    return result
