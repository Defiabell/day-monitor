import base64
import json
import re

PROMPT = (
    '你是一个工作活动识别助手。观察这张 macOS 截图，用一句中文描述用户当前在做什么（15字以内），'
    '并从以下类别中选一个最合适的：\n\n'
    '- coding     : VS Code、Cursor、终端、Xcode、任何 IDE 或代码编辑器\n'
    '- meeting    : Zoom、Google Meet、腾讯会议等视频/语音会议\n'
    '- slack      : Slack\n'
    '- wechat     : 微信（WeChat）\n'
    '- feishu     : 飞书（Lark）\n'
    '- email      : 邮件客户端（Mail、Outlook、Gmail 网页版）\n'
    '- browser    : 浏览器（Safari、Chrome、Firefox）中的网页浏览\n'
    '- reading    : 阅读文档、PDF、技术文章、Notion 文档\n'
    '- design     : Figma、Sketch 等设计工具\n'
    '- app        : 其他桌面应用（如 Meshy、产品测试、系统设置等）\n'
    '- other      : 以上均不符合\n\n'
    '返回 JSON：{"summary": "...", "category": "..."}'
)
MODEL = 'claude-haiku-4-5-20251001'


def analyze_screenshot(image_bytes: bytes, client) -> dict:
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
                {'type': 'text', 'text': PROMPT},
            ],
        }],
    )
    text = response.content[0].text.strip()
    text = re.sub(r'^```(?:json)?\s*', '', text)
    text = re.sub(r'\s*```$', '', text)
    return json.loads(text)
