import io
from pathlib import Path
import pytest
from unittest.mock import patch
from PIL import Image
from capture import take_screenshot, resize_for_api, compute_hash, hash_distance, is_screen_active


def make_png(width=100, height=100, color=(128, 128, 128)) -> bytes:
    img = Image.new('RGB', (width, height), color=color)
    buf = io.BytesIO()
    img.save(buf, format='PNG')
    return buf.getvalue()


def test_resize_for_api_shrinks_large_image():
    large_png = make_png(width=2560, height=1440)
    result = resize_for_api(large_png, max_width=1280)
    img = Image.open(io.BytesIO(result))
    assert img.width == 1280
    assert img.height == 720


def test_resize_for_api_keeps_small_image():
    small_png = make_png(width=800, height=600)
    result = resize_for_api(small_png, max_width=1280)
    img = Image.open(io.BytesIO(result))
    assert img.width == 800


def test_compute_hash_returns_string():
    png = make_png()
    h = compute_hash(png)
    assert isinstance(h, str)
    assert len(h) > 0


def test_hash_distance_identical_images():
    png = make_png(color=(100, 100, 100))
    h1 = compute_hash(png)
    h2 = compute_hash(png)
    assert hash_distance(h1, h2) == 0


def make_checkerboard_png(invert=False) -> bytes:
    import random
    random.seed(42 if not invert else 99)
    img = Image.new('RGB', (100, 100))
    pixels = [(random.randint(0, 255), random.randint(0, 255), random.randint(0, 255))
              for _ in range(100 * 100)]
    img.putdata(pixels)
    buf = io.BytesIO()
    img.save(buf, format='PNG')
    return buf.getvalue()


def test_hash_distance_different_images():
    png1 = make_checkerboard_png(invert=False)
    png2 = make_checkerboard_png(invert=True)
    h1 = compute_hash(png1)
    h2 = compute_hash(png2)
    assert hash_distance(h1, h2) > 8


def make_quartz_mock(locked=False, display_active=True):
    from unittest.mock import MagicMock, patch
    import sys
    quartz = MagicMock()
    quartz.CGSessionCopyCurrentDictionary.return_value = {'CGSSessionScreenIsLocked': int(locked)}
    quartz.CGDisplayIsActive.return_value = display_active
    quartz.CGMainDisplayID.return_value = 1
    return quartz


def test_is_screen_active_when_display_on_and_unlocked():
    with patch.dict('sys.modules', {'Quartz': make_quartz_mock(locked=False, display_active=True)}):
        assert is_screen_active() is True


def test_is_screen_active_when_display_off():
    with patch.dict('sys.modules', {'Quartz': make_quartz_mock(locked=False, display_active=False)}):
        assert is_screen_active() is False


def test_is_screen_active_when_locked():
    with patch.dict('sys.modules', {'Quartz': make_quartz_mock(locked=True, display_active=True)}):
        assert is_screen_active() is False


def test_take_screenshot_calls_screencapture(tmp_path):
    fake_png = make_png()

    def fake_run(cmd, check):
        Path(cmd[-1]).write_bytes(fake_png)

    with patch('capture.subprocess.run', side_effect=fake_run), \
         patch('capture.tempfile.mktemp', return_value=str(tmp_path / 'sc.png')):
        result = take_screenshot()
    assert result == fake_png
