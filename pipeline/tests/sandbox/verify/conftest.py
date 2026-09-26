"""The verifier's cases on the real toolchain (T-15b, D-18) - `just test-verify-full`.

This directory sits under `tests/sandbox`, so the parent conftest already refuses
it unless `POPQUIZ_SANDBOX_SUITE` is set, and `just test` never reaches it. It has
its own switch as well, `POPQUIZ_VERIFY_FULL`, so that `test-sandbox` stays the
AC-12 containment suite it says it is: without the switch this directory collects
nothing, and a recipe that runs only it with the switch unset collects no tests,
which pytest reports as a failure rather than a pass.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

ASKED_FOR = "POPQUIZ_VERIFY_FULL"

if not os.environ.get(ASKED_FOR):
    collect_ignore_glob = ["*.py"]

# `tests/` on the path, for the shared case list in tests/fixtures/verify.
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
