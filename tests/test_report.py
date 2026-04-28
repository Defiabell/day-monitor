import pytest
from pathlib import Path
from unittest.mock import MagicMock
from storage import init_db, insert_event
from report import generate_report


def make_mock_client(report_text: str):
    mock_content = MagicMock()
    mock_content.text = report_text
    mock_response = MagicMock()
    mock_response.content = [mock_content]
    mock_client = MagicMock()
    mock_client.messages.create.return_value = mock_response
    return mock_client


FAKE_REPORT = """# 工作日报 2026-04-28

## 时间轴
| 时间 | 时长 | 活动 |
|------|------|------|
| 09:00 – 09:10 | 10m | 写代码 |

## 分类统计
| 类别 | 时长 | 占比 |
|------|------|------|
| coding | 10m | 100% |

## 今日小结
今天专注写代码。"""


def test_generate_report_writes_file(tmp_path):
    conn = init_db(':memory:')
    insert_event(conn, '2026-04-28T09:00:00', 'hash1', '写代码', 'coding')
    client = make_mock_client(FAKE_REPORT)

    output_path = generate_report(conn, '2026-04-28', tmp_path, client)

    assert output_path is not None
    assert Path(output_path).exists()
    assert '工作日报' in Path(output_path).read_text()


def test_generate_report_returns_none_for_empty_day(tmp_path):
    conn = init_db(':memory:')
    client = make_mock_client(FAKE_REPORT)

    result = generate_report(conn, '2026-04-28', tmp_path, client)
    assert result is None


def test_generate_report_includes_events_in_prompt(tmp_path):
    conn = init_db(':memory:')
    insert_event(conn, '2026-04-28T09:00:00', 'hash1', '写代码', 'coding')
    insert_event(conn, '2026-04-28T10:00:00', 'hash2', '开会', 'meeting')
    client = make_mock_client(FAKE_REPORT)

    generate_report(conn, '2026-04-28', tmp_path, client)

    prompt_content = client.messages.create.call_args.kwargs['messages'][0]['content']
    assert '写代码' in prompt_content
    assert '开会' in prompt_content


def test_generate_report_filename_is_date(tmp_path):
    conn = init_db(':memory:')
    insert_event(conn, '2026-04-28T09:00:00', 'hash1', '写代码', 'coding')
    client = make_mock_client(FAKE_REPORT)

    output_path = generate_report(conn, '2026-04-28', tmp_path, client)
    assert Path(output_path).name == '2026-04-28.md'
