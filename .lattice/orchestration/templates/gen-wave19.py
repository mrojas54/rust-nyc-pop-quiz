"""Wave 19 boot prompt: PQ-38 (fast-track) — machine-verify the re-authored q4/q7/q8 and reconcile
the pipeline tests, off `origin/main` @ 1d721f4 (the client's PR #36, which turned main red).

    python3 gen-wave19.py PQ-38 <origin/main sha>
"""
import importlib.util
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

_w15_spec = importlib.util.spec_from_file_location("w15", pathlib.Path(__file__).parent / "gen-wave15.py")
_w15 = importlib.util.module_from_spec(_w15_spec)
_w15_spec.loader.exec_module(_w15)
PLAN_REVIEW, FAST_TRACK_PLAN = _w15.PLAN_REVIEW, _w15.FAST_TRACK_PLAN

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "main"

TICKETS = {}

TICKETS["PQ-38"] = dict(
    t="fix-it (D-15 follow-through; no BUILDPLAN row)", slug="bank-reverify", mode="fast-track",
    title="Re-authored q4, q7, q8: machine-verify them and reconcile the pipeline tests", tab="Bank Reverify", actor="pq38",
    oneliner="the pinned verifier writes q4's and q7's verified blocks, the migration/fit/audit tests say what is true of re-authored records, and main goes green again.",
    body="""**Why this ticket exists.** `main` is red since the client's own PR #36 (`1d721f4`, *Re-author q4/q7/q8 options for the 29-char wall rule*, D-15). It changed q4's and q7's programs, hand-wrote their `verified` blocks (a `-Vv` line, `runs`, a typed `stdout`, `verified_at`; `legacy` and `miri` dropped; **no `verifier_version`**), replaced one q8 distractor and drafted `why_tempting` beats for all three. The pipeline's `just test` now fails in six places (below). Every PR's CI gate inherits the failure, so this lands first. The house rule (`CLAUDE.md`, AC-7): **never write down what a program prints; a correct answer comes from a verified record the verifier wrote, never a hand.** You fix the provenance, not the client's words.

Read, in this order, before planning: `CLAUDE.md` (house rules); the diff `git diff 1263add 1d721f4 -- bank/questions/` (what PR #36 did, file by file: q4 and q7 programs and records rewritten; q8's program and record untouched, options only); `SPEC.md` **§3.2** (the `verified` record: for a complete record the compiler's full `-Vv` line, edition, target triple, flag set, Miri configuration, `runs`, `stdout`, `verified_at`, `verifier_version`; a `legacy` record carries none of the machine detail — D-16), §3.1 (options, `why_tempting`), §5.2 (the 29-character rule, D-15), §7.6 (`bank-audit`); `sequence/USER_STORIES.md` **AC-6** (a non-legacy record whose pin does not match is stale), **AC-7** (the answer is never hand-written), AC-11, AC-13, AC-24, **AC-100** (fits the configured room); `EVALUATION.md` rows AC-6 (~9–10), AC-7 (~11: *only the verifier's output reader writes `correct`; a hand-edited answer fails the build*), AC-100; `BUILDPLAN.md` T-14 (q4, q7, q8 migrate, then need one-line options and **re-verification — the machine decides the new answer**), T-15b; `sequence/run-state.md` D-15, D-16, D-17, D-22; then the code **on your branch**: `pipeline/src/popquiz/verify.py` (`verify()` ~283; `is_stale` ~507–516: a legacy record is exempt, a non-legacy one must match the pin; ~585–611: *a record `verify` did not write* — a non-legacy record without `verified_at` **and** `verifier_version` is refused; `main` ~636), `pipeline/src/popquiz/migrate_mvp.py` (`MIGRATED = ("q3","q4","q7","q8")` ~65; `legacy=True` ~275/292; `migrate()` ~496 — **does a re-run overwrite an existing record?** read it and say), `pipeline/src/popquiz/bank.py` (`Verified`, `correct_index` by kind, `load_bank`), `pipeline/src/popquiz/audit.py` (`source_metrics`, `fit`, the reserve flags, `main`), `pipeline/sandbox/pin.toml` (the pin: 1.98.1 / 48a229c, nightly-2026-09-19), the `justfile` (`verify` ~236: `just verify path/to/candidate.json --expect ran|does_not_compile|ub`; `sandbox-build` ~127; `test-pipeline` ~50), `pipeline/tests/fixtures/` (how the stub `Runner`'s recordings were made on the image — never typed), and the failing tests: **`pipeline/tests/test_migration.py`** (`MIGRATED`/`NOT_MIGRATED` ~34–35; ~76–95 `test_the_committed_records_are_what_a_fresh_run_writes` — the drift test that now fails for q4, q7 **and q8** (q8's options changed); ~97–110 the set test; ~116–140 `written["legacy"] is True` and the version-line test — both fail for q4/q7; ~144–160 q8; ~162–182 runs/miri vs the MVP; ~205–220 the correct option is the machine's output), **`pipeline/tests/test_fit.py:102–108`** (the worked shapes: q4 `(6, 56)`, q7 `(5, 69)` are the August programs; q4 now measures `(5, 78)`), **`pipeline/tests/test_audit.py:744–760`** (it relied on q7's real 48-character option to prove *a flagged question in the reserve fails the run*; q7 fits now, so the premise is gone). `mvp/**` is read-only and contract-protected.

**The six failures on `main` (`just test`, both CI jobs, run 36673189915):** `test_migration.py:89` q4.json on disk ≠ what the migration writes (also q7, q8); `written["legacy"]` KeyError (q4, q7); `test_migration.py:138` record rustc `1.98.1 …` ≠ the MVP batch's `1.96.1 …` (q4, q7); `verified.stdout ≠ mvp["answer"]` (q4, q7); `test_fit.py:107` q4 `(5, 78) ≠ (6, 56)`; `test_audit.py:756` `audit.main` returns 0, expected 1.

**Deliverables:**

- **q4 and q7, verified by the machine.** `just sandbox-build` (Docker; the client approves it in this tab) then `just verify` on each record with `--expect ran` on the T-15a image, so the verifier writes the whole `verified` block — the `-Vv` line, edition, target triple, flags, Miri under both borrow models, `runs`, `stdout`, `verified_at`, `verifier_version` — and the record passes `verify.is_stale` against `pin.toml` and the *record verify did not write* check. You type none of those values. **If the machine's `stdout` differs from what PR #36 typed, or Miri flags anything, stop:** `lattice status PQ-38 needs_human` plus a c11 flag naming the difference — the content is the client's. The `review.reason` gains one sentence saying the record was re-verified by the pinned verifier on this date (no other wording changes; `why_tempting` and option texts are the client's).
- **q8 stays as it is:** program and legacy record untouched (D-22 and PR #36 agree); only its options changed.
- **The tests say what is true now.** The migration's output is the *origin* of the four records, not their current state: keep the exact drift check for q3; for q4 and q7 assert instead that they are non-legacy, verifier-written (`verified_at` + `verifier_version` present, `is_stale` false against the pin), and that their `review.reason` records the re-authoring; for q8 assert its `verified` block still equals what the migration writes while its options may differ. The `legacy`/version-line tests parametrize over the records that are still legacy. `test_the_correct_option_is_the_machines_output` must hold for all four (the answer is `stdout`, matched by kind). If `migrate()` would overwrite a re-authored record on a re-run, add the smallest guard (skip a record whose `verified` is not legacy) and say so; otherwise leave it.
- **`test_fit.py`:** re-measure q4 and q7 with `audit.source_metrics` and pin the new shapes; both must still `fit` at the default room (AC-100) — if one does not, that is a finding for the client (`needs_human`), never a fixture edit. If SPEC §5.2's worked table names the old August shapes, note it under deviations; the Orchestrator routes it upstream (F-40).
- **`test_audit.py`:** synthesize the flagged reserve question inside the tmp copy (mutate one option of any question to 48 characters and mark it accepted and affirmed) so the test proves the mechanism, not the bank's current state.
- **Green:** `just test-pipeline` (or as much of it as this machine runs; CI runs the whole `just test`), `just bank-audit` on the bank (attach its output), both CI jobs green on your PR head.

**Exit check before DONE:** `git diff origin/main --stat` shows only `bank/questions/q4.json`, `q7.json` (their `verified` blocks and one `review.reason` sentence each), the three test files, and at most the `migrate()` guard; the two `just verify` command lines and the image digest they ran on are in your DONE; both CI jobs SUCCESS on the head; `just bank-audit` passed.

**Siblings running now (off PQ-16's branch, not yours):** PQ-26 owns the `justfile`, `.github/workflows/**`, `room/**`; PQ-27 owns `web/shared/**`, `room/src/copy.rs`, `room/tests/twins.rs`, `pipeline/src/popquiz/copylint.py`, `pipeline/tests/test_copylint.py`, `bank/fixtures/copy-lint/**`. Touch none of those. PQ-16 (PR #37) is at review and waits for you to land first.

**Out of scope:** any option text, `why_tempting` or explanation wording (the client's); the traces (still empty by design until affirm); `mvp/**`; `SPEC.md`; `verify.py`, `bank.py` and `audit.py` *rules* (a fixture is not a rule — a rule change is a QUESTION); the `justfile`; `web/**`; `room/**`.

**Shared files cleared for this ticket:** `bank/questions/q4.json` and `bank/questions/q7.json` (the verifier's writes plus one `review.reason` sentence each), `pipeline/tests/test_migration.py`, `pipeline/tests/test_fit.py`, `pipeline/tests/test_audit.py`, `pipeline/src/popquiz/migrate_mvp.py` (the re-run guard only, if needed), `pipeline/tests/fixtures/**` (only recordings made on the image, never typed). **Not** `bank/questions/q8.json`, `q3.json`, `bank/history.json`, `pipeline/sandbox/pin.toml`, `pyproject.toml`.""",
)


def main():
    pq, parent_sha = sys.argv[1], sys.argv[2]
    assert len(parent_sha) == 40, "pass the full 40-char origin/main sha"
    d = TICKETS[pq]
    wt = f"{WT}/{d['slug']}"
    branch = f"ai-c11-cc/{d['slug']}"
    text = HEADER.format(
        title=d["title"], pq=pq, t=d["t"], mode=d["mode"], parent=PARENT, parent_sha=parent_sha,
        wt=wt, root=ROOT, tab=d["tab"], oneliner=d["oneliner"], actor=d["actor"],
        body=d["body"], planner_extra="", branch=branch,
    )
    branch_note = (
        f"**Base.** Your branch starts from `origin/main` @ `{parent_sha}` — the client's PR #36, the commit that turned `main` red. "
        f"Rebase onto `origin/main` before your first push and never after it. Your PR opens against `main` and **lands before every other open PR** "
        f"(PR #37 and the wave-18 PRs wait on it), so keep it small and exact. Siblings are named in §2; touch none of their files."
    )
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line",
         "and the line"),
        (PLAN_REVIEW.replace("{pq}", pq), FAST_TRACK_PLAN.replace("{pq}", pq)),
    ]
    for a, b in reps:
        assert a in text, (pq, a[:70])
        text = text.replace(a, b)
    assert "#8 " not in text and "#8)" not in text and "#8," not in text, "a #8 reference survived"
    assert "stacked" not in text
    out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(text)
    print(out, len(text.splitlines()), "lines")


if __name__ == "__main__":
    main()
