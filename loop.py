import signal
import sys
import time
from datetime import datetime

from analyze import analyze_screenshot
from capture import compute_hash, hash_distance, is_screen_active, resize_for_api, take_screenshot
from storage import get_last_event, increment_last_duration, insert_event

DEDUP_THRESHOLD = 12


class MonitorLoop:
    def __init__(self, conn, client, interval: int = 20):
        self.conn = conn
        self.client = client
        self.interval = interval
        self._running = True
        self._paused = False

    def _tick(self) -> None:
        if not is_screen_active():
            return

        raw = take_screenshot()
        resized = resize_for_api(raw)
        new_hash = compute_hash(resized)

        last = get_last_event(self.conn)
        if last and hash_distance(new_hash, last['image_hash']) < DEDUP_THRESHOLD:
            increment_last_duration(self.conn, self.interval)
            return

        result = analyze_screenshot(resized, self.client)
        ts = datetime.now().strftime('%Y-%m-%dT%H:%M:%S')
        # `or None` coerces "" / null to NULL so app_name stays clean
        insert_event(
            self.conn, ts, new_hash,
            result['summary'], result['category'], result.get('app') or None,
        )

    def run(self) -> None:
        signal.signal(signal.SIGTERM, self._handle_sigterm)
        signal.signal(signal.SIGUSR1, self._handle_pause)
        signal.signal(signal.SIGUSR2, self._handle_resume)
        while self._running:
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
