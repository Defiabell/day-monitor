import base64
import json
import re

PROMPT = (
    '你是一个工作活动识别助手。观察这张 macOS 截图，用一句中文描述用户当前在做什么（15字以内），'
    '并从以下类别中选一个：coding / meeting / browsing / reading / communication / design / other。\n\n'
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
