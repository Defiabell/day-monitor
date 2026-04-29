import io
from unittest.mock import MagicMock, patch
from PIL import Image
from storage import init_db, get_last_event
from loop import MonitorLoop


def make_png(color=(128, 128, 128)) -> bytes:
    img = Image.new('RGB', (100, 100), color=color)
    buf = io.BytesIO()
    img.save(buf, format='PNG')
    return buf.getvalue()


def make_mock_client(summary='写代码', category='coding'):
    mock_content = MagicMock()
    mock_content.text = f'{{"summary": "{summary}", "category": "{category}"}}'
    mock_response = MagicMock()
    mock_response.content = [mock_content]
    mock_client = MagicMock()
    mock_client.messages.create.return_value = mock_response
    return mock_client


def test_loop_inserts_event_on_first_screenshot():
    conn = init_db(':memory:')
    client = make_mock_client()
    png = make_png()

    loop = MonitorLoop(conn=conn, client=client, interval=0)
    with patch('loop.is_screen_active', return_value=True), \
         patch('loop.take_screenshot', return_value=png), \
         patch('loop.resize_for_api', return_value=png):
        loop._tick()

    last = get_last_event(conn)
    assert last is not None


def test_loop_increments_duration_on_similar_screenshot():
    conn = init_db(':memory:')
    client = make_mock_client()
    png = make_png()

    loop = MonitorLoop(conn=conn, client=client, interval=10)
    with patch('loop.is_screen_active', return_value=True), \
         patch('loop.take_screenshot', return_value=png), \
         patch('loop.resize_for_api', return_value=png):
        loop._tick()
        loop._tick()

    assert client.messages.create.call_count == 1
    assert get_last_event(conn)['duration_s'] == 20


def test_loop_calls_api_on_different_screenshot():
    conn = init_db(':memory:')
    client = make_mock_client()

    import random
    random.seed(1)
    pixels1 = [(random.randint(0, 255), random.randint(0, 255), random.randint(0, 255))
               for _ in range(100 * 100)]
    random.seed(999)
    pixels2 = [(random.randint(0, 255), random.randint(0, 255), random.randint(0, 255))
               for _ in range(100 * 100)]

    def make_noise_png(pixels) -> bytes:
        img = Image.new('RGB', (100, 100))
        img.putdata(pixels)
        buf = io.BytesIO()
        img.save(buf, format='PNG')
        return buf.getvalue()

    png1 = make_noise_png(pixels1)
    png2 = make_noise_png(pixels2)

    loop = MonitorLoop(conn=conn, client=client, interval=10)
    with patch('loop.is_screen_active', return_value=True), \
         patch('loop.take_screenshot', side_effect=[png1, png2]), \
         patch('loop.resize_for_api', side_effect=lambda x, **kw: x):
        loop._tick()
        loop._tick()

    assert client.messages.create.call_count == 2


def test_loop_skips_tick_when_screen_inactive():
    conn = init_db(':memory:')
    client = make_mock_client()

    loop = MonitorLoop(conn=conn, client=client, interval=10)
    with patch('loop.is_screen_active', return_value=False):
        loop._tick()

    assert client.messages.create.call_count == 0
    assert get_last_event(conn) is None


def test_loop_paused_by_default_is_false():
    loop = MonitorLoop(conn=init_db(':memory:'), client=make_mock_client(), interval=10)
    assert loop._paused is False


def test_handle_pause_sets_paused_true():
    loop = MonitorLoop(conn=init_db(':memory:'), client=make_mock_client(), interval=10)
    loop._handle_pause(0, None)
    assert loop._paused is True


def test_handle_resume_sets_paused_false():
    loop = MonitorLoop(conn=init_db(':memory:'), client=make_mock_client(), interval=10)
    loop._paused = True
    loop._handle_resume(0, None)
    assert loop._paused is False


def test_run_skips_tick_when_paused(monkeypatch):
    """run() should not call _tick when _paused is True."""
    conn = init_db(':memory:')
    client = make_mock_client()
    loop = MonitorLoop(conn=conn, client=client, interval=0)
    loop._paused = True

    tick_calls = []
    monkeypatch.setattr(loop, '_tick', lambda: tick_calls.append(1))

    sleep_calls = []
    def fake_sleep(s):
        sleep_calls.append(s)
        if len(sleep_calls) >= 2:
            loop._running = False
    monkeypatch.setattr('loop.time.sleep', fake_sleep)

    loop.run()
    assert tick_calls == []


def test_run_resumes_ticks_after_unpause(monkeypatch):
    """run() should call _tick again once _paused is cleared."""
    conn = init_db(':memory:')
    client = make_mock_client()
    loop = MonitorLoop(conn=conn, client=client, interval=0)
    loop._paused = True

    tick_calls = []
    monkeypatch.setattr(loop, '_tick', lambda: tick_calls.append(1))

    sleep_calls = []
    def fake_sleep(s):
        sleep_calls.append(s)
        # After 2 paused iterations, simulate a SIGUSR2 by clearing the flag
        if len(sleep_calls) == 2:
            loop._paused = False
        if len(sleep_calls) >= 4:
            loop._running = False
    monkeypatch.setattr('loop.time.sleep', fake_sleep)

    loop.run()
    # Iterations 1-2: paused (no tick); iterations 3-4: resumed (tick called twice)
    assert len(tick_calls) == 2


def test_loop_passes_app_name_to_insert_event():
    """When analyze returns 'app', loop should store it as app_name."""
    conn = init_db(':memory:')
    # Mock client returns app field
    mock_content = MagicMock()
    mock_content.text = '{"summary": "coding", "category": "coding", "app": "VS Code"}'
    mock_response = MagicMock()
    mock_response.content = [mock_content]
    client = MagicMock()
    client.messages.create.return_value = mock_response

    loop = MonitorLoop(conn=conn, client=client, interval=10)
    with patch('loop.is_screen_active', return_value=True), \
         patch('loop.take_screenshot', return_value=make_png()), \
         patch('loop.resize_for_api', return_value=make_png()):
        loop._tick()

    # Verify the row in DB has app_name
    rows = conn.execute("SELECT app_name FROM events").fetchall()
    assert len(rows) == 1
    assert rows[0][0] == 'VS Code'


def test_loop_coerces_empty_app_to_null():
    """Model returning app: '' should be stored as NULL, not empty string."""
    conn = init_db(':memory:')
    mock_content = MagicMock()
    mock_content.text = '{"summary": "x", "category": "other", "app": ""}'
    mock_response = MagicMock()
    mock_response.content = [mock_content]
    client = MagicMock()
    client.messages.create.return_value = mock_response

    loop = MonitorLoop(conn=conn, client=client, interval=10)
    with patch('loop.is_screen_active', return_value=True), \
         patch('loop.take_screenshot', return_value=make_png()), \
         patch('loop.resize_for_api', return_value=make_png()):
        loop._tick()

    rows = conn.execute("SELECT app_name FROM events").fetchall()
    assert len(rows) == 1
    assert rows[0][0] is None
