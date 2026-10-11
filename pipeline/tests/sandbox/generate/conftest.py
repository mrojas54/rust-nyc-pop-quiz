"""AC-2's real kill-and-rerun (PQ-68 F3) - `just test-generate-full`.

This directory sits under `tests/sandbox`, so the parent conftest already refuses
it unless `POPQUIZ_SANDBOX_SUITE` is set, and `just test` never reaches it: the
test here starts real processes, which `just test` does not. It has its own
switch as well, `POPQUIZ_GENERATE_FULL`, so `test-sandbox` stays the AC-12
containment suite: without the switch this directory collects nothing, and a
recipe that runs only it with the switch unset collects no tests, which pytest
reports as a failure rather than a pass.

No real `claude` is reached: the generator is given the fake one explicitly.
"""

from __future__ import annotations

import os

ASKED_FOR = "POPQUIZ_GENERATE_FULL"

if not os.environ.get(ASKED_FOR):
    collect_ignore_glob = ["*.py"]
