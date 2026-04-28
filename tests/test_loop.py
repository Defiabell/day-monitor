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
