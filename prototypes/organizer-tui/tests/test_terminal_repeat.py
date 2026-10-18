"""Real-process regressions for the PTY harness's reveal synchronization."""
import os
import signal
import threading
import unittest
from unittest.mock import Mock, patch

from terminal_repeat import check


class RevealWaitTests(unittest.TestCase):
    def test_exit_between_predicate_and_exit_check(self) -> None:
        for exit_code in (0, 7):
            with self.subTest(exit_code=exit_code):
                self.check_exit_race(exit_code)

    def check_exit_race(self, exit_code: int) -> None:
        # The first two polls after q see a running child; the error-path poll
        # observes its exit. Each call owns its callbacks and terminal replay.
        child = Mock()
        child.returncode = None
        quitting = False
        polls = 0
        output = b'Question preview'

        def write(fd: int, data: bytes) -> int:
            nonlocal output, quitting
            if data == b'r':
                output = b'Press Enter to reveal'
            elif data == b'\r':
                output = b'Correct answer'
            elif data == b'q':
                quitting = True
            return len(data)

        def read(fd: int, size: int) -> bytes:
            nonlocal output
            chunk, output = output, b''
            return chunk

        def poll() -> int | None:
            nonlocal polls
            if quitting:
                polls += 1
                if polls >= 3:
                    child.returncode = exit_code
                    return exit_code
            return None

        child.poll.side_effect = poll
        with (
            patch('terminal_repeat.subprocess.Popen', return_value=child),
            patch('terminal_repeat.os.write', side_effect=write),
            patch('terminal_repeat.os.read', side_effect=read),
            patch('terminal_repeat.select.select', return_value=([0], [], [])),
        ):
            if exit_code == 0:
                check(False)
            else:
                with self.assertRaises(AssertionError):
                    check(False)
        self.assertGreaterEqual(polls, 3)
        child.kill.assert_not_called()

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
