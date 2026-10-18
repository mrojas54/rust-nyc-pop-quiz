"""Real-process regressions for the PTY harness's reveal synchronization."""
import os
import signal
import threading
import unittest
from unittest.mock import patch

from terminal_repeat import check


class RevealWaitTests(unittest.TestCase):
    def test_delayed_child_reveals_in_both_protocols(self) -> None:
        for enhanced in (False, True):
            with self.subTest(enhanced=enhanced):
                timer: threading.Timer | None = None

                def pause(pid: int) -> None:
                    nonlocal timer
                    os.kill(pid, signal.SIGSTOP)
                    _, status = os.waitpid(pid, os.WUNTRACED)
                    self.assertTrue(os.WIFSTOPPED(status))

                    def resume() -> None:
                        try:
                            os.kill(pid, signal.SIGCONT)
                        except ProcessLookupError:
                            pass  # The old harness kills the child before this timer fires.

                    timer = threading.Timer(.35, resume)
                    timer.start()

                try:
                    check(enhanced, before_reveal=pause)
                finally:
                    if timer is not None:
                        timer.join()

    def test_missing_reveal_times_out_with_terminal_evidence(self) -> None:
        write = os.write

        def drop_enter(fd: int, data: bytes) -> int:
            if data == b'\r':
                return len(data)
            return write(fd, data)

        with (
            patch('terminal_repeat.os.write', side_effect=drop_enter),
            self.assertRaisesRegex(
                AssertionError, r'legacy.*Correct answer.*[0-9].*s.*terminal output'
            ),
        ):
            check(False)


if __name__ == '__main__':
    unittest.main()
