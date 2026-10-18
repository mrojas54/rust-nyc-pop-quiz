"""Exercise the real binary over a PTY, without extra Python dependencies.

Run from the repository root after building organizer-preview. The legacy case
never answers the keyboard capability query, so repeat bytes decode as Press.
The enhanced case emulates capability reporting and repeat/release event types.
No scheduling actions or network requests are performed.
"""
import codecs
import fcntl
import os
import pty
import re
import select
import struct
import subprocess
import termios
import time
from collections.abc import Callable
from pathlib import Path

BINARY = Path(__file__).resolve().parents[1] / 'target/debug/organizer-preview'
ANSI = re.compile(r'\x1b\[[0-9;:?><=]*[A-Za-z]')
STATE_TIMEOUT = 3.0


def check(enhanced: bool, before_reveal: Callable[[int], None] | None = None) -> None:
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 50, 120, 0, 0))
    child = subprocess.Popen([str(BINARY), '--question', 'q3'], stdin=slave,
                             stdout=slave, stderr=slave,
                             env={**os.environ, 'TERM': 'xterm-256color'})
    os.close(slave)
    raw = ''
    decoder = codecs.getincrementaldecoder('utf-8')()
    replied = False

    def pump(seconds: float) -> None:
        nonlocal raw, replied
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            if select.select([master], [], [], max(0, min(.05, end - time.monotonic())))[0]:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    return
                if not chunk:
                    return
                raw += decoder.decode(chunk)
                if enhanced and not replied and '\x1b[?u' in raw:
                    os.write(master, b'\x1b[?0u\x1b[?1;2c')
                    replied = True

    def send(data: bytes) -> None:
        os.write(master, data)

    def wait_for(predicate: Callable[[], bool], state: str) -> None:
        started = time.monotonic()
        deadline = started + STATE_TIMEOUT
        while not predicate():
            remaining = deadline - time.monotonic()
            if remaining > 0:
                pump(min(.02, remaining))
                if predicate():
                    return
            if remaining <= 0 or child.poll() is not None:
                # Exit can satisfy the predicate between the two polls.
                if predicate():
                    return
                mode = 'enhanced' if enhanced else 'legacy'
                elapsed = time.monotonic() - started
                raise AssertionError(
                    f'{mode}: waiting for {state} after {elapsed:.3f}s; '
                    f'child exit={child.poll()}; terminal output: {raw!r}'
                )

    def wait_text(text: str) -> None:
        wait_for(lambda: text in ANSI.sub('', raw), text)

    try:
        wait_text('Question preview')
        if enhanced:
            wait_for(lambda: replied and '\x1b[>3u' in raw, 'enhanced keyboard protocol')
        else:
            assert '\x1b[>3u' not in raw, raw
        send(b'\x1b[114;1:1u' if enhanced else b'r')
        wait_text('Press Enter to reveal')
        for _ in range(20):
            send(b'\x1b[114;1:2u' if enhanced else b'r')
            # Negative assertions need an observation window, not an early return.
            pump(.12)
            plain = ANSI.sub('', raw)
            assert 'Correct answer' not in plain, plain
            assert 'How we know' not in plain, plain
        assert 'Press Enter to reveal' in ANSI.sub('', raw), raw
        if before_reveal is not None:
            before_reveal(child.pid)
        send(b'\x1b[13;1:1u' if enhanced else b'\r')
        wait_text('Correct answer')
        send(b'\x1b[113;1:1u' if enhanced else b'q')
        if enhanced:
            wait_for(lambda: '\x1b[<1u' in raw, 'keyboard protocol restoration')
        wait_for(lambda: child.poll() is not None, 'clean exit')
        assert child.returncode == 0
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)
    print(('Enhanced' if enhanced else 'Legacy') + ': repeats stay hidden; Enter reveals; clean exit')


if __name__ == '__main__':
    check(False)
    check(True)
