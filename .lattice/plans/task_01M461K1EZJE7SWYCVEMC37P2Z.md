# PQ-58: G-2 provenance follow-ups: stale pin, record digest, the MVP deck

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-5** (G-2, Minor, new): Scheduling does not refuse a stale pin
- Evidence: verify.is_stale (verify.py:507) has no caller; SPEC 7.2
- What would close it: Call is_stale in schedule.refusal for non-legacy records

**GAP-6** (G-2, Minor, new (CONTRACT NOTE 1)): A self-consistent hand edit to a verified record is undetectable
- Evidence: verify.py:592-596; M2c accepted by check_provenance
- What would close it: Contract: a digest or recording id binding the record to its run

**GAP-7** (G-2, Minor, new): The MVP deck trusts verified.json's answer with no provenance and no test
- Evidence: mvp/tools/build_deck.py:241,266; mvp/tools/verify.py:166-216
- What would close it: A provenance check in build_deck, or retire the MVP deck once the room runs

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-06)

Base `origin/main` c810d0d. Re-verified there: `verify.is_stale` at `verify.py:562` has no caller in `pipeline/src`; `schedule.refusal` at `schedule.py:136`; its one src caller is `_schedule` (`schedule.py:611`).

## GAP-5 — the stale-pin refusal (no ruling needed)

**Placement.** In `schedule.refusal`, after the `in_reserve` line and before `check_question`, so every refusal above reads as it did and the provenance check stays last. Only a record that has passed the reserve checks reaches it, and `is_stale` itself exempts `legacy`.

**Signature.** `refusal(question, club=DEFAULT_CLUB, *, pin: sandbox.Pin | None = None)`. Keyword-only, default `None` = read `pipeline/sandbox/pin.toml` through `sandbox.read_pin()` (the package's only reader), and only when a non-legacy record reaches the check, so a legacy record never touches the pin file. A `PinError` (missing or unrecorded pin) is returned as the refusal in the error's own words, not raised, so `just schedule` still exits 1 with one sentence rather than a traceback (`main` catches only `ScheduleError`/`BankError`).

**Callers.** Every existing call (`_schedule`; `test_review.py` and `test_schedule.py` direct calls) keeps working unchanged: the new parameter is keyword-only with a default. No caller's behaviour changes for a matching or legacy record. The committed bank is not stale (below), so `test_g12_the_gate_agrees_with_the_reserve` and `test_ac72_every_committed_record_…` stay green.

**Sentence (new, one; no other changes by a character):**

> `{qid} was verified under a toolchain that no longer matches the pin ({mismatch}) - re-verify it with \`just verify\` before it is scheduled (AC-6)`

`{mismatch}` names each differing part, `; `-joined, from: `rustc release X, the pin says Y`; `rustc commit-hash X, the pin says Y`; `edition X, the pin says Y`; `flags X, the pin says Y`; `Miri X, the pin says Y`; or `its rustc is not a -Vv report, so it cannot be compared`. The decision is `is_stale`'s alone; the naming helper only describes. (`verify.py` is not cleared before the GAP-6 ruling, so the helper lives in `schedule.py`.)

**Other routes to the same gate.** `fallback.bake` calls `check_question`, not `refusal`, and does not check the pin; its only src callers are `build_html`/`host_sheet`/`write_fallback`, reached from `_schedule` after `refusal`, so a stale record never reaches it there. `reserve_lines` counts `audit.in_reserve`, which does not check the pin: a stale record would still count as *ready* in the reserve line while `schedule` refuses it. `just demo` pushes the stored file after `verify --check` (`check_provenance`), which does not check the pin either. Reported, not fixed (`audit.py`, `fallback.py` beyond a signature change, and `justfile` are not cleared).

**Tests, RED first, in `pipeline/tests/test_schedule.py`** (the suite that covers `refusal`), with a `Pin` built in the test from literal values matching q4's committed record, never from `pin.toml`:
1. q4 affirmed, pin release differs → refused, with the new sentence naming the release.
2. q3 (legacy) affirmed, pin release differs → `None`.
3. q4 affirmed, pin matching → `None`.
4. q4 affirmed (Miri ran), pin Miri version differs → refused, naming Miri.
Plus one CLI test: q4's recorded release edited to `0.0.0` in the copied bank and re-affirmed → `schedule` exits 1, nothing sent or written, needle from the new sentence (exercises the default loader through `_schedule`).

**Docs.** `pipeline/README.md` item 2 of `schedule`: add the stale-pin refusal to the list (one line).

## Stale-bank check (read-only, `is_stale` vs `pipeline/sandbox/pin.toml`)

Pin: release 1.98.1, edition 2021, `miri 0.1.0 (420ed2a0c3 2026-09-18)`. q3 legacy (exempt); q4, q7, q8 non-legacy and **not stale**. **None would be refused.** Nothing written.

## GAP-6 — binding a record to its run (reserved: client's ruling)

- **A — digest field.** `verify` writes `record_digest`: sha256 over canonical JSON (sorted keys, no whitespace) of every other `verified` field plus the question's `source`. `check_question` recomputes and refuses a non-legacy record whose digest is missing or wrong. **Honestly: an unkeyed digest is recomputable** by anyone with the repo, so it catches an edit that forgot to recompute it, not one that did. A keyed HMAC needs a secret the verifier holds; the organizer who edits on the laptop holds it too, and the bank is public, so the key would also have to stay out of the repo and CI could no longer check committed records without it. **Cost:** q4, q7, q8 lack the field, so all three non-legacy reserve questions would be refused until re-verified in the sandbox (`just verify`, docker, Michelle runs it; I never edit `bank/questions/**`). That leaves q3 alone in the reserve until then.
- **B — recording id + replay.** The record names its recording; provenance replays it through the stub runner and compares. The recordings live under `pipeline/tests/fixtures/` and are as hand-editable as the record, so a replay proves record and recording agree, not that the machine produced either. A true replay (re-running the sandbox at schedule time) needs docker + Miri on the night: minutes per question, and a toolchain the room laptop otherwise does not need. A question with no recording is refused. Highest cost.
- **C — leave it a contract note.** No code. The remaining path is an uncommitted edit on the organizer's own laptop; every committed edit already fails CI (`test_every_bank_question_passes_the_provenance_check`, the `test_migration.py` replay). F-55 stays with tone-architect.

**Recommended: C.** A guards only against a careless edit and blocks three of four reserve questions until a re-verify; B costs a toolchain at schedule time; the threat C leaves open is the trusted organizer's own uncommitted edit.

Draft wording if A is chosen (applied verbatim on approval):
- SPEC §3.2, after `verifier_version`: "`record_digest` — sha256 of the record's other fields and the question's `source`, in canonical JSON, written by `verify` last."
- SPEC §7.2, after the stale sentence: "Scheduling and the fallback refuse a non-legacy record whose `record_digest` is missing or does not match. The digest is unkeyed: it catches an edit that did not recompute it, not one that did (G-2: never overstate)."
- EVALUATION AC-7 row, `test` clause: "a record edited after verification without recomputing its digest is refused by `schedule` and `bake`."

Draft wording if B is chosen:
- SPEC §3.2: "`recording` — the path of the runner recording `verify` replayed, written by `verify`."
- SPEC §7.2: "Scheduling replays a non-legacy record's `recording` through the stub runner and refuses a record that disagrees with it or names none. The replay proves record and recording agree, not that a toolchain produced either."

## GAP-6 plan after the ruling (client, 2026-11-06: "recording-id plus true replay behind feature flag, plan first and show me")

**What exists.** `RealRunner` runs each step in the sandbox; `StubRunner` replays a recording (one JSON per program: steps by kind, plus `_recorded` naming the image/toolchain and `_source_sha256`). Recordings exist only as test fixtures (`pipeline/tests/fixtures/verify/recordings/q3,q4,q7,q8.json`, made by `record.py` on the pinned image). `test_migration.py` already stub-replays them in CI. The room's `Verified` struct refuses unknown fields (`popquiz/src/quiz_session/models/answers.rs:178`, `deny_unknown_fields`), and `schedule`/`just demo` push the whole record.

**1. The recording id (always on, no Docker).** `just verify` keeps every step the real runner returned and writes it to `bank/recordings/<qid>.json`. The binding goes in the recording, not the record: the file's `_record_sha256` is the sha256 of the canonical `verified` block that run produced, beside the existing `_source_sha256`. Neither the record format nor the room changes. `schedule.refusal` gets a check, placed before `check_question`: if `bank/recordings/<qid>.json` exists, the record's digest and source must match the file, and replaying `verify` over it through `StubRunner` must give back the same answer facts (stdout, exit code, compile codes, Miri result). Otherwise the record is refused. **Limit, stated plainly:** someone who edits the record, the recording and both digests together gets through. Only part 2 catches that.

**2. True replay (behind the flag).** `just schedule <id> --replay …` re-runs the question's source in the sandbox through `RealRunner` with the record's own `expect` and the current pin, then compares the new record with the stored one on rustc, edition, target triple, flags, runs, stdout, exit code, compile codes and Miri (version, configs, clean, output matched). A legacy record is compared only on the fields it has. Any difference is refused and named. So is a sandbox that cannot run (no Docker, or no pinned image): "… `--replay` needs the sandbox; run `just sandbox-build`, or schedule without `--replay`". With the flag off, scheduling behaves exactly as after GAP-5. **Cost:** Docker running and the pinned image built on the scheduling laptop. The recordings' measured in-container time is q3 11.0 s, q4 3.3 s, q7 2.8 s, q8 0.4 s, plus one container start per step (2 to 8 steps), so a few seconds to about 20 s per question.

**Existing records.** q4, q7 and q8 have no `bank/recordings/` file until they are re-verified with `just verify` (the client runs it; I never edit `bank/**`). Until then part 1 does not apply to them, and `--replay` still works because it re-runs the source.

**Tests (`just test`, no Docker):** the replay takes a runner seam. (a) A recording that agrees passes. (b) A record whose stdout and matching option were hand-edited together is refused by part 1 when a recording exists, and by `--replay`. (c) A recording whose `_record_sha256` or `_source_sha256` does not match is refused. (d) A replay that disagrees on Miri is refused. (e) With no Docker, `--replay` refuses with the sentence and sends nothing. (f) Without the flag, nothing new runs. **`just test-full`:** one real `--replay` of q8 (0.4 s) on the image.

**Files:** `pipeline/src/popquiz/{runner.py (a recording wrapper around RealRunner), verify.py (write the recording), schedule.py (check + --replay)}`, the tests, `bank/recordings/` (new, created by `just verify` only), `bank/README.md`, `pipeline/README.md`, `docs/RUNBOOK.md` (one line), and SPEC §3.2/§7.2 plus EVALUATION's AC-7 row, using wording the client approves.

**Draft contract wording:**
- SPEC §3.2, after the legacy paragraph: "**Recording.** `verify` writes every step the sandbox returned to `bank/recordings/<id>.json`, with the sha256 of the question's `source` and of the `verified` block that run produced. The record is unchanged; the recording names it."
- SPEC §7.2, after the stale sentence: "Scheduling refuses a record whose recording exists but does not name it, or whose replay through the stub runner disagrees with it. With `--replay`, scheduling re-runs the source in the sandbox and refuses a record the run disagrees with. Without the flag, a record edited together with its recording and both digests is not caught (G-2: never overstate)."
- EVALUATION AC-7 row, `test`: "a record whose stdout and matching option were hand-edited together is refused when its recording exists, and by `schedule --replay`; `test-full` replays one question on the image."

**Decisions for the client:** see the PLAN-GAP6 comment.

## GAP-7 — the retired deck (reserved: client's ruling)

**Recommended: A, closed by retirement.** No code. `mvp/tools/build_deck.py` and `mvp/tools/verify.py` are retired tooling nothing runs (`CLAUDE.md` §Shape, 2026-10-11; PQ-46 cancelled that day: with no deck builds, `build_deck.py`'s ledger write cannot fire). The room's only answer path is the sealed module fed by `popquiz schedule` through `check_question` (AC-7, AC-61). B (delete `mvp/tools/**` and the README lines that run them) and C (a check inside a retired tool) are listed in the QUESTION.
