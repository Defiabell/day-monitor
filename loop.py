import os
import signal
import sys
import time
from datetime import datetime

from analyze import analyze_screenshot, is_unidentifiable
from capture import compute_hash, get_active_app, hash_distance, is_black_screen, is_screen_active, resize_for_api, take_screenshot
from storage import get_last_event, increment_last_duration, insert_event

DEDUP_THRESHOLD = 12

# Remember the parent at startup. If it disappears (PPID becomes 1, init/launchd),
# the parent app died and we should self-terminate to avoid becoming a zombie monitor.
_INITIAL_PPID = os.getppid()


def parent_alive() -> bool:
    return os.getppid() == _INITIAL_PPID


class MonitorLoop:
    def __init__(self, conn, client, interval: int = 20):
        self.conn = conn
        self.client = client
        self.interval = interval
        self._running = True
        self._paused = False
        # Hash of the last frame the model could not identify (black screen
        # etc.). Kept in memory only, so similar frames skip the API call
        # without ever being stored.
        self._last_skipped_hash = None

    def _tick(self) -> None:
        if not is_screen_active():
            return

        raw = take_screenshot()
        resized = resize_for_api(raw)

        if is_black_screen(resized):
            return

        new_hash = compute_hash(resized)

        if self._last_skipped_hash and \
                hash_distance(new_hash, self._last_skipped_hash) < DEDUP_THRESHOLD:
            return

        last = get_last_event(self.conn)
        if last and hash_distance(new_hash, last['image_hash']) < DEDUP_THRESHOLD:
            increment_last_duration(self.conn, self.interval)
            return

        active_app = get_active_app()
        result = analyze_screenshot(resized, self.client, active_app)
        if result['category'] == 'other' and is_unidentifiable(result.get('summary')):
            self._last_skipped_hash = new_hash
            return
        self._last_skipped_hash = None
        ts = datetime.now().strftime('%Y-%m-%dT%H:%M:%S')
        insert_event(
            self.conn, ts, new_hash,
            result['summary'], result['category'],
            result.get('app') or active_app or None,
        )

    def run(self) -> None:
        signal.signal(signal.SIGTERM, self._handle_sigterm)
        signal.signal(signal.SIGUSR1, self._handle_pause)
        signal.signal(signal.SIGUSR2, self._handle_resume)
        while self._running:
            if not parent_alive():
                # Parent app exited — bail out instead of becoming an orphan.
                self._running = False
                break
            try:
                if not self._paused:
                    self._tick()
            except Exception as e:
                print(f'[day-monitor] error: {e}', file=sys.stderr)
            if self._running:
                time.sleep(self.interval)

    def _handle_sigterm(self, signum, frame) -> None:
        self._running = False

    def _handle_pause(self, signum, frame) -> None:
        self._paused = True

    def _handle_resume(self, signum, frame) -> None:
        self._paused = False
