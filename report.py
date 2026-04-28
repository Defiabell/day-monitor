from pathlib import Path

from storage import get_events_for_date

MODEL = 'claude-haiku-4-5-20251001'


def generate_report(conn, date_str: str, output_dir: Path, client) -> str:
    events = get_events_for_date(conn, date_str)
    if not events:
        return None

    events_text = '\n'.join(
        f"{e['timestamp']} ({e['duration_s']}s): {e['summary']} [{e['category']}]"
        for e in events
    )

    prompt = (
        f'你是一个工作日报生成助手。以下是用户 {date_str} 的工作记录（每条包含时间、持续秒数、活动描述、分类）：\n\n'
        f'{events_text}\n\n'
        f'请生成一份工作日报，格式如下（严格使用 Markdown）：\n\n'
        f'# 工作日报 {date_str}\n\n'
        f'## 时间轴\n'
        f'| 时间 | 时长 | 活动 |\n'
        f'|------|------|------|\n'
        f'（合并相邻相似活动为连续时间块，时间格式 HH:MM – HH:MM，时长用 Xm 或 XhYm）\n\n'
        f'## 分类统计\n'
        f'| 类别 | 时长 | 占比 |\n'
        f'|------|------|------|\n'
        f'（统计 coding/meeting/browsing/reading/communication/design/other 各自总时长和百分比）\n\n'
        f'## 今日小结\n'
        f'（2-3句总结，中文）'
    )

    response = client.messages.create(
        model=MODEL,
        max_tokens=2000,
        messages=[{'role': 'user', 'content': prompt}],
    )

    content = response.content[0].text
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    output_path = output_dir / f'{date_str}.md'
    output_path.write_text(content, encoding='utf-8')
    return str(output_path)
