import io
import pytest
from unittest.mock import MagicMock
from PIL import Image
from analyze import VALID_CATEGORIES, _normalize_category, analyze_screenshot


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


def test_analyze_screenshot_returns_app_field():
    client = make_mock_client('{"summary": "在 VS Code 写代码", "category": "coding", "app": "VS Code"}')
    result = analyze_screenshot(make_png(), client)
    assert result['app'] == 'VS Code'


def test_analyze_screenshot_handles_missing_app():
    client = make_mock_client('{"summary": "黑屏", "category": "other"}')
    result = analyze_screenshot(make_png(), client)
    assert result.get('app') is None


class TestNormalizeCategory:
    def test_valid_category_passes_through(self):
        for cat in VALID_CATEGORIES:
            assert _normalize_category(cat, None) == cat

    def test_invented_category_falls_back_to_other(self):
        assert _normalize_category('system', None) == 'other'
        assert _normalize_category('media', None) == 'other'

    def test_invented_category_rescued_by_app_name(self):
        assert _normalize_category('system', 'Meshy') == '3d'
        assert _normalize_category('system', 'Finder') == 'app'
        assert _normalize_category('system', 'Slack') == 'slack'

    def test_case_insensitive(self):
        assert _normalize_category('Coding', None) == 'coding'
        assert _normalize_category(' BROWSER ', None) == 'browser'

    def test_app_map_is_case_insensitive(self):
        assert _normalize_category('unknown', 'CURSOR') == 'coding'
        assert _normalize_category('unknown', 'DBeaver') == 'data'


def test_analyze_coerces_invalid_category():
    client = make_mock_client('{"summary": "系统桌面", "category": "system", "app": "Finder"}')
    result = analyze_screenshot(make_png(), client)
    assert result['category'] in VALID_CATEGORIES
