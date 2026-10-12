"""Exercise the real binary over a PTY, without extra Python dependencies.

Run from the repository root after building organizer-preview. The legacy case
never answers the keyboard capability query, so repeat bytes decode as Press.
The enhanced case emulates capability reporting and repeat/release event types.
No scheduling actions or network requests are performed.
"""
import codecs
import fcntl
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import termios
import time

BINARY = Path(__file__).resolve().parents[1] / 'target/debug/organizer-preview'
ANSI = re.compile(r'\x1b\[[0-9;:?><=]*[A-Za-z]')


def check(enhanced):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 50, 120, 0, 0))
    child = subprocess.Popen([str(BINARY), '--question', 'q3'], stdin=slave,
                             stdout=slave, stderr=slave,
                             env={**os.environ, 'TERM': 'xterm-256color'})
    os.close(slave)
    raw = ''
    decoder = codecs.getincrementaldecoder('utf-8')()
    replied = False

    def pump(seconds):
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

    def send(data):
        os.write(master, data)
        pump(.12)

    try:
        pump(3)
        assert 'Question preview' in ANSI.sub('', raw), raw
        if enhanced:
            assert replied and '\x1b[>3u' in raw, raw
        else:
            assert '\x1b[>3u' not in raw, raw
        send(b'\x1b[114;1:1u' if enhanced else b'r')
        for _ in range(20):
            send(b'\x1b[114;1:2u' if enhanced else b'r')
            plain = ANSI.sub('', raw)
            assert 'Correct answer' not in plain, plain
            assert 'How we know' not in plain, plain
        assert 'Press Enter to reveal' in ANSI.sub('', raw), raw
        send(b'\x1b[13;1:1u' if enhanced else b'\r')
        assert 'Correct answer' in ANSI.sub('', raw), raw
        send(b'\x1b[113;1:1u' if enhanced else b'q')
        child.wait(timeout=3)
        pump(.1)
        assert child.returncode == 0
        if enhanced:
            assert '\x1b[<1u' in raw, 'keyboard protocol was not restored'
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)
    print(('Enhanced' if enhanced else 'Legacy') + ': repeats stay hidden; Enter reveals; clean exit')


if __name__ == '__main__':
    check(False)
    check(True)
