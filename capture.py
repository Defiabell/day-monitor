import io
import os
import subprocess
import tempfile
from pathlib import Path

import imagehash
from PIL import Image


def is_screen_active() -> bool:
    """Return False if display is off or screen is locked."""
    try:
        import Quartz
        # Screen locked check
        session = Quartz.CGSessionCopyCurrentDictionary()
        if session and session.get('CGSSessionScreenIsLocked', 0):
            return False
        # Display active check
        if not Quartz.CGDisplayIsActive(Quartz.CGMainDisplayID()):
            return False
    except Exception:
        pass
    return True


def take_screenshot() -> bytes:
    path = tempfile.mktemp(suffix='.png')
    try:
        subprocess.run(['screencapture', '-x', '-t', 'png', path], check=True)
        with open(path, 'rb') as f:
            return f.read()
    finally:
        if os.path.exists(path):
            os.unlink(path)


def resize_for_api(image_bytes: bytes, max_width: int = 640) -> bytes:
    img = Image.open(io.BytesIO(image_bytes))
    if img.width > max_width:
        ratio = max_width / img.width
        new_size = (max_width, int(img.height * ratio))
        img = img.resize(new_size, Image.LANCZOS)
    buf = io.BytesIO()
    img.save(buf, format='PNG')
    return buf.getvalue()


def compute_hash(image_bytes: bytes) -> str:
    img = Image.open(io.BytesIO(image_bytes))
    return str(imagehash.phash(img))


def hash_distance(h1: str, h2: str) -> int:
    return imagehash.hex_to_hash(h1) - imagehash.hex_to_hash(h2)
