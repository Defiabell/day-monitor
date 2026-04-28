import io
import pytest
from unittest.mock import MagicMock
from PIL import Image
from analyze import analyze_screenshot


def make_png() -> bytes:
    img = Image.new('RGB', (100, 100), color=(128, 128, 128))
    buf = io.BytesIO()
    img.save(buf, format='PNG')
    return buf.getvalue()


def make_mock_client(response_text: str):
    mock_content = MagicMock()
    mock_content.text = response_text
    mock_response = MagicMock()
    mock_response.content = [mock_content]
    mock_client = MagicMock()
    mock_client.messages.create.return_value = mock_response
    return mock_client


def test_analyze_screenshot_returns_dict():
    client = make_mock_client('{"summary": "在 VS Code 写代码", "category": "coding"}')
    result = analyze_screenshot(make_png(), client)
    assert result['summary'] == '在 VS Code 写代码'
    assert result['category'] == 'coding'


def test_analyze_screenshot_strips_markdown_codeblock():
    client = make_mock_client('```json\n{"summary": "开 Zoom 会议", "category": "meeting"}\n```')
    result = analyze_screenshot(make_png(), client)
    assert result['category'] == 'meeting'


def test_analyze_screenshot_calls_haiku_model():
    client = make_mock_client('{"summary": "浏览网页", "category": "browser"}')
    analyze_screenshot(make_png(), client)
    call_kwargs = client.messages.create.call_args
    assert call_kwargs.kwargs['model'] == 'claude-haiku-4-5-20251001'


def test_analyze_screenshot_sends_image_and_text():
    client = make_mock_client('{"summary": "阅读文档", "category": "reading"}')
    analyze_screenshot(make_png(), client)
    messages = client.messages.create.call_args.kwargs['messages']
    content = messages[0]['content']
    types = [c['type'] for c in content]
    assert 'image' in types
    assert 'text' in types
