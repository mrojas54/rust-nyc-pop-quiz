"""AC-2 with a real kill (PQ-68 F3, EVALUATION.md's AC-2 row).

The generator runs as its own process on the claude-cli backend, against the fake
`claude` (`tests/fixtures/generate/claude/fake_claude.py`, given by
`--claude-bin`; the real binary is never on this test's PATH). It is sent SIGKILL
while its second candidate is in flight, then run again with no cleanup.

Asserted: no partial record is visible after the kill; the rerun completes, with
the first candidate served from what the killed run kept and only the other two
asked for; and the room is unaffected - every file under `bank/` and under `room/`
(build output in `room/target/` aside) and `mvp/answer-history.json` have the
same bytes after as before.

Bounded: every wait has a deadline, and the whole test is reported in seconds.
"""

from __future__ import annotations

import hashlib
import json
import os
import signal
import stat
import subprocess
import sys
import time
from pathlib import Path

import pytest

PIPELINE = Path(__file__).resolve().parents[3]
REPO = PIPELINE.parent
FAKE = PIPELINE / "tests" / "fixtures" / "generate" / "claude"
TOPICS = ("ownership", "iterators", "integers")
SAFE_PATH = "/usr/bin:/bin"
DEADLINE_S = 60.0


def room_digest() -> dict[str, str]:
    """sha256 of every file the room or the bank could be touched through."""
    files = [p for p in (REPO / "bank").rglob("*") if p.is_file()]
    files += [p for p in (REPO / "room").rglob("*") if p.is_file() and "target" not in p.relative_to(REPO / "room").parts]
    files.append(REPO / "mvp" / "answer-history.json")
    return {str(p.relative_to(REPO)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)}


def wait_for(predicate, what: str) -> None:
    deadline = time.monotonic() + DEADLINE_S
    while not predicate():
        if time.monotonic() > deadline:
            raise AssertionError(f"gave up after {DEADLINE_S:g} s waiting for {what}")
        time.sleep(0.05)


def test_ac2_a_generator_killed_mid_candidate_reruns_to_completion(tmp_path, capsys):
    started = time.monotonic()
    before = room_digest()

    exe = tmp_path / "bin" / "claude"
    exe.parent.mkdir()
    exe.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{FAKE / "fake_claude.py"}" "$@"\n', encoding="utf-8")
    exe.chmod(exe.stat().st_mode | stat.S_IXUSR)
    log = tmp_path / "log"
    log.mkdir()
    blocking = json.loads((FAKE / "cli-clean.json").read_text(encoding="utf-8"))
    blocking["responses"][1]["sleep_s"] = 600  # c02 stays in flight until it is killed
    blocking_script = tmp_path / "blocks-on-c02.json"
    blocking_script.write_text(json.dumps(blocking), encoding="utf-8")

    env = {
        k: v for k, v in os.environ.items() if not k.startswith(("ANTHROPIC_", "CLAUDE_CODE_USE_"))
    }
    env.update(PATH=SAFE_PATH, FAKE_CLAUDE_LOG=str(log), FAKE_CLAUDE_SCRIPT=str(blocking_script))
    out = tmp_path / "out"
    command = [
        sys.executable, "-m", "popquiz.generate", "--backend", "claude-cli", "--claude-bin", str(exe),
        "--max-candidates", "3", "--count", "3", *sum((["--topic", t] for t in TOPICS), []),
        "--difficulty", "2", "--out", str(out), "--run-id", "k1",
    ]
    run_dir = out / "k1"

    generator = subprocess.Popen(command, cwd=PIPELINE, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        wait_for(lambda: len(list(log.glob("*.json"))) == 2, "the fake to be asked for c02")
        wait_for(lambda: (run_dir / "_run" / "cli" / "c01.json").is_file(), "c01 to be kept")
        generator.send_signal(signal.SIGKILL)
        generator.wait(timeout=DEADLINE_S)
    finally:
        if generator.poll() is None:
            generator.kill()
            generator.wait()
    assert generator.returncode == -signal.SIGKILL

    # A SIGKILL cannot reach the child; reap the orphaned fake by its process group.
    calls = [json.loads(p.read_text(encoding="utf-8")) for p in sorted(log.glob("*.json"))]
    try:
        os.killpg(calls[1]["pgid"], signal.SIGKILL)
    except ProcessLookupError:
        pass

    # No partial record: every visible file is whole, and no candidate record exists yet.
    for path in run_dir.rglob("*.json"):
        json.loads(path.read_text(encoding="utf-8"))
    assert sorted(p.name for p in run_dir.glob("*.json")) == []
    assert not (run_dir / "_run" / "report.json").exists()

    # The rerun, untouched directory, same run id, the limit gone.
    env["FAKE_CLAUDE_SCRIPT"] = str(FAKE / "cli-clean.json")
    again = subprocess.run(command, cwd=PIPELINE, env=env, capture_output=True, text=True, timeout=DEADLINE_S)
    assert again.returncode == 0, again.stderr
    records = sorted(p.name for p in run_dir.glob("*.json"))
    assert records == ["k1-01.json", "k1-02.json", "k1-03.json"]
    calls = [json.loads(p.read_text(encoding="utf-8")) for p in sorted(log.glob("*.json"))]
    asked = [c["prompt"].splitlines()[0] for c in calls]
    assert asked == ["Topic: ownership", "Topic: iterators", "Topic: iterators", "Topic: integers"], asked
    assert not list(run_dir.rglob("*.partial"))
    report = json.loads((run_dir / "_run" / "report.json").read_text(encoding="utf-8"))
    assert report["resumed"] is True and report["numbers"] == "api-equivalent"

    assert room_digest() == before, "the generator changed a file under bank/, room/ or mvp/answer-history.json"
    with capsys.disabled():
        print(f"\nAC-2 kill-and-rerun wall-clock: {time.monotonic() - started:.2f} s")
