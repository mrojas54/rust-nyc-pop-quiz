"""The AC-12 containment suite runs only when it is asked for, and never quietly.

`just test` is Docker-free by contract (`EVALUATION.md`, the harness table), so it
passes `--ignore=tests/sandbox` and never imports this file. Anything else — a
bare `pytest`, an IDE, a future recipe that forgets — lands here and is refused
with a message saying which hook runs it.

**Refused, not skipped.** A skipped Docker suite is indistinguishable from a
passing one in a summary line, and AC-12 is a criterion that must never be able to
look green without a container having actually held something: fail loudly
rather than pass quietly.
"""

from __future__ import annotations

import os

ASKED_FOR = "POPQUIZ_SANDBOX_SUITE"


class ContainmentSuiteNotRequested(RuntimeError):
    """Raised at collection when the suite was reached without being asked for."""


if not os.environ.get(ASKED_FOR):
    raise ContainmentSuiteNotRequested(
        "the AC-12 containment suite needs a real container and was not asked for.\n"
        "  Run it with:      just test-full      (builds the image, then runs it)\n"
        f"  Or directly with: {ASKED_FOR}=1 uv run pytest tests/sandbox\n"
        "\n"
        "`just test` skips this directory on purpose — it is hermetic, offline and "
        "Docker-free, and budgeted at 60 s. This suite is none of those things."
    )
