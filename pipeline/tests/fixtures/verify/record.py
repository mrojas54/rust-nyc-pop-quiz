"""Record the StubRunner fixtures by running every case on the T-15a image.

    cd pipeline && uv run --no-sync python tests/fixtures/verify/record.py

This is the only way a file under `recordings/` is made (`tests/fixtures/README.md`).
It needs Docker and the pinned image (`just sandbox-build`), and it rewrites every
recording from scratch: nothing from an earlier recording is kept.

For each case in `cases.toml` it runs `verify` through a recording runner over
`RealRunner`: a step asked for the first time is executed in the sandbox and kept;
a step asked for again (q3 is verified under three declarations) is replayed from
what was kept, so every case of one program sees one consistent set of runs. The
verdict must be the one `cases.toml` names, or nothing is written - a recording
that does not show the property its case is about is not a fixture for that case.

Each file also says how it was made: `_recorded` (the image, the pin, the host,
when) and `_source_sha256` (the program it is a recording of). `test_verify.py`
fails with "re-record" when a program's source no longer matches its recording.
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))  # pipeline/tests

from fixtures.verify.verify_cases import (  # noqa: E402
    RECORDINGS,
    load_cases,
    programs,
    question_for,
    source_of,
    source_sha256,
)

from popquiz import sandbox, verify  # noqa: E402
from popquiz.runner import RealRunner, RunStep  # noqa: E402


class RecordingRunner:
    """Executes a step once, then replays it. Keeps everything it executed."""

    def __init__(self, real: RealRunner, kept: dict[str, RunStep]) -> None:
        self._real = real
        self._kept = kept

    def run(self, kind: str, program_id: str) -> RunStep:
        if kind not in self._kept:
            self._kept[kind] = self._real.run(kind, program_id)
            step = self._kept[kind]
            print(f"    ran {program_id} {kind}: exit {step.exit_code}"
                  f"{' stopped by ' + ', '.join(step.stopped_by) if step.stopped_by else ''}"
                  f" ({step.wall_clock_s:.1f} s)", flush=True)
        return self._kept[kind]


def main() -> int:
    pin = sandbox.read_pin()
    kept: dict[str, dict[str, RunStep]] = {p: {} for p in programs()}
    failures = []

    for case in load_cases():
        print(f"- {case.name}", flush=True)
        question = question_for(case.program)
        real = RealRunner({case.program: source_of(case.program)}, pin=case.pin(pin))
        runner = RecordingRunner(real, kept[case.program])
        verdict = verify.verify(question, runner, expect=case.expect, pin=pin)  # type: ignore[arg-type]
        print(f"  -> {verdict.code}: {verdict.reason}", flush=True)
        if verdict.code != case.verdict:
            failures.append(f"{case.name}: expected {case.verdict}, got {verdict.code}")

    if failures:
        print("\nNOT WRITTEN - these cases did not show what they are for:", file=sys.stderr)
        for line in failures:
            print(f"  {line}", file=sys.stderr)
        return 1

    host = verify.parse_toolchain(
        next(iter(k[verify.TOOLCHAIN] for k in kept.values() if verify.TOOLCHAIN in k)).stdout
    ).host
    recorded_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    RECORDINGS.mkdir(parents=True, exist_ok=True)
    for old in RECORDINGS.glob("*.json"):
        old.unlink()
    for program, steps in kept.items():
        payload: dict[str, object] = {
            "_recorded": {
                "by": "tests/fixtures/verify/record.py",
                "image": pin.image,
                "host": host,
                "rustc_release": pin.release,
                "rustc_commit_hash": pin.commit_hash,
                "miri_version": pin.miri_version,
                "recorded_at": recorded_at,
            },
            "_source_sha256": source_sha256(program),
        }
        payload.update({kind: step.to_dict() for kind, step in steps.items()})
        path = RECORDINGS / f"{program}.json"
        path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"wrote {path.relative_to(Path.cwd()) if path.is_relative_to(Path.cwd()) else path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
