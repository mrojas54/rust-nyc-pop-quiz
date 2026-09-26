# PQ-20: Verifier with Miri and the pin

BUILDPLAN.md T-15b (M3).

The verifier in code: inherit `verify.py`'s procedure after audit; **add Miri** (strict provenance, Tree Borrows for UB-intended), the flag set, target triple, the §3.2 record, **the pin comparison and stale rejection, and the `Runner` seam with its stub and recorded fixtures** (`SPEC.md` §7.2, D-17, D-18); `verify <program>`

Criteria: AC-6–11, AC-13, AC-87, G-2
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14, T-15a
BUILDPLAN notes: Audit ticket for `verify.py`. Serialized on `pyproject.toml` with T-16, T-17

Orchestrator notes: `mvp/tools/verify.py` never calls Miri (confirmed by grep); this ticket adds it. The pin is read from T-15a's definition, never re-declared. Audit ticket: read `verify.py` before inheriting.

Workflow mode: sub-agent-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-22) — DRAFT, paused before plan review

**Status at pause.** The client asked for a clean close-out mid-plan; planning resumes next session.
- Branch `ai-c11-cc/verifier`: one commit, the signed merge of `origin/main` @ `238146e`, which is `9a758cd`. Not pushed.
- Ticket status: `in_planning`.
- Nothing is implemented.
- Not yet done: the plan-review subagent and the `## Plan-review resolutions` block.
- Baseline: `just test` takes 2.4 s warm; the pipeline suite is 191 passed in 0.71 s. It needs the sandbox bypass for the uv cache.
- The pinned image `popquiz-sandbox:1.98.1-2026-09-19-25a914853547` (arm64) is built locally and Docker is up.

**Open questions to the Orchestrator** (comment `ev_01M33MG66HM7TY3MVC8HH05AG1`, unanswered at pause):
1. May `sandbox.py` gain a `toolchain` mode? The fallback is a probe program through the `run` mode; it was tried on the image and works (~2 s).
2. The PENDING line's `verify:T-15b` entry. I leave it and flag it in DONE.

**Empirical checks on the real image, exploration only, not fixtures:**
- `-Zmiri-strict-provenance`, `-Zmiri-tree-borrows` and `-Zmiri-seed=0` are accepted.
- Stacked Borrows flags a write through `(&mut v[0] as *mut u8).add(1)`; Tree Borrows does not. This is the "models disagree" fixture.
- A write through a raw parent `p`, followed by a write through child `r = &mut *p`, is flagged by both models. This is the "declared UB" fixture. Print only *before* the UB.
- `cfg!(miri)` gives different stdout with a clean Miri run (the AC-10 fixture).
- `std::env::current_dir()` gives `unsupported operation` with exit 1 under isolation.
- A panic exits 101 both natively and under Miri.

## Shape
`verify(question, runner, *, expect, pin=None, runs=RUNS, now=utcnow) -> Verdict`. Steps run in this order: pin → toolchain probe → compile → N native runs → Miri SB → Miri TB (only when `expect="ub"`) → `bank.Verified`.

- **Refusals** (`PinMismatch`, `ToolchainUnreadable` ⊂ `VerifyError`) are exceptions and count nothing about the candidate.
- **Rejections** are `Verdict`s with a code and a plain-words reason, and nothing is written.
- **Every execution** is `runner.run(kind, question.id)`.
- **Other exports:** `is_stale`, `expect_from_record` (T-20), `check_provenance` (AC-7/G-2), and a `main()` CLI: `python -m popquiz.verify <candidate.json> [--expect ran|does_not_compile|ub]`.
  - `--expect` is required unless it derives from an existing record.
  - Exit codes: 0 accepted (record written back), 1 rejected, 2 refused.

**Kinds:** `toolchain`, `compile`, `run:<i>` (i = 1…N), `miri:<stacked_borrows|tree_borrows>:<seed>`.
**`RunStep` gains:** `exit_code: int | None`, `stopped_by: tuple[str, ...]` (TIMEOUT/MEMORY_LIMIT), and optional `wall_clock_s`. `StubRunner` ignores `_`-prefixed keys.

## Decisions → stub fixture → test-full twin
Pins are always altered with `dataclasses.replace(read_pin(), …)`; no version is ever typed.

| # | Decision (criterion) | Stub (`just test`) | Twin (`test-full`) |
|---|---|---|---|
| 1 | Pin unreadable or unrecorded → refuse, no step asked (D-17) | tmp pin with a field blanked | same |
| 2 | Probe missing release, commit-hash or Miri line → refuse | recorded probe with a line removed | stub-only |
| 3 | release or commit-hash ≠ pin → refuse; only `toolchain` asked (AC-6) | q3 `toolchain` step vs altered pin | same on image |
| 4 | Miri version ≠ `pin.miri_version`, whole string → refuse | same, altered `miri_version` | same |
| 5 | Declared dnc; fails with ≥1 `error[Exxxx]` → accept; codes deduped in compiler order (AC-11) | q8 (E0502). Codes asserted against the replayed stderr and legacy q8, never typed | real == stub |
| 6 | Declared dnc; compiles → reject | q3 | same |
| 7 | Declared dnc; fails with no code → reject | syntax-error | same |
| 8 | Declared ran or ub; fails to compile → reject | q8 under `ran` | same |
| 9 | Any step `stopped_by` a limit → reject | loops-forever | shortened-timeout pin |
| 10 | N runs disagree on (stdout, exit) → reject; report distinct count (AC-8) | hashmap-order (recorder asserts the outputs differ) | verdict only |
| 11 | Identical nonzero exit (panic) → accept, exit_code recorded. MVP's nonzero-exit reject dropped | panics | real == stub |
| 12 | SB `error: Undefined Behavior:` and not declared → reject (AC-9) | ub-both under `ran` | same |
| 13 | SB clean but Miri stdout or exit ≠ native → reject (AC-10) | miri-differs (`cfg!(miri)`) | same |
| 14 | Miri failed without the UB marker → reject "Miri could not run it"; leaks get their own code | miri-unsupported | same |
| 15 | SB clean and matching, declared ran → accept; configs [stacked_borrows] | q3 | real == stub |
| 16 | Declared ub; SB clean → reject | q3 under `ub` | same |
| 17 | Declared ub; SB and TB both UB, outputs **equal** → accept; clean=false; both configs (AC-9) | ub-both under `ub` | real == stub |
| 18 | Declared ub; SB UB, TB clean → reject (models disagree) | ub-sb-only | same |
| 19 | Accepted record replaces a legacy one whole (D-16) | q3 from the bank | same |
| 20 | Record keys exactly §3.2; round-trips `question_from_dict`; receipt renders the right list (AC-13, AC-87) | records from 5, 11, 15, 17 | — |
| 21 | `is_stale` (AC-6): match false, mismatch true, legacy exempt, dnc ignores Miri, unparseable rustc true. Compares release, commit-hash, edition, flags, and miri.version when present | row-15 record vs altered pins; bank q3 and q8 | real record not stale |
| 22 | `check_provenance` refuses (AC-7, G-2): option text hand-edited; `correct` key added; verified_at or verifier_version stripped | q3 record edited in test | — |
| 23 | Static: only verify.py, migrate_mvp.py and bank.py's reader construct `Verified(` | AST test | — |
| 24 | No pin string and no fixture stdout literal in verify/runner/tests | scan test | — |

**Fixture programs** go under `pipeline/tests/fixtures/verify/programs/`:
- `q3` and `q8`: copied from the bank, and a test asserts equality with the bank source.
- `hashmap-order`, `syntax-error`, `loops-forever` (reuses `tests/sandbox/programs/loops_forever.rs`), `panics`, `ub-both`, `ub-sb-only`, `miri-differs`, `miri-unsupported`.

**The recorder.** Fixtures are recorded by `tests/fixtures/verify/record.py`, a RecordingRunner over RealRunner that records on a miss and replays on a hit.
- It drives `verify` over `cases.toml` (program, expect, expected verdict code — shared by both suites).
- It asserts each program's named property before writing.
- It writes `_recorded` (image, pin, recorded_at) and `_source_sha256` into each fixture. Tests fail with "re-record" on drift.

## Files
**Create:**
- `pipeline/src/popquiz/verify.py`
- `pipeline/tests/test_verify.py`
- `pipeline/tests/fixtures/verify/**` (programs, recordings, `cases.toml`, `record.py`)
- `pipeline/tests/sandbox/verify/{conftest.py,test_verify_full.py}`. The conftest ignores collection unless `POPQUIZ_VERIFY_FULL`, so `test-sandbox` doesn't run it, and a bare `pytest` still refuses via the parent conftest.

**Change:**
- `runner.py`:
  - RealRunner(sources, *, pin, process_runner) maps kinds onto `run_in_sandbox` and caches `toolchain`.
  - The seam is this ticket's deliverable; it is not on the shared list, which is noted as a deviation.
- `sandbox.py`, within the Miri clearance:
  - `MIRI_CONFIGS`.
  - `miri_config` and `miri_seed` params.
  - `export MIRIFLAGS=…` inside the script, never `-e`.
  - `[profile.dev]` in the generated Cargo.toml written from `pin.flags`. This one is flagged: read as part of "the Miri flag set".
- `test_sandbox.py`, for those changes.
- `test_runner.py`:
  - The placeholder tests are replaced.
  - The provenance-refusal test uses a tmp fixture with no program output.
  - RealRunner mapping tests with a fake process runner.
- Delete `fixtures/scaffold-placeholder.json`.
- `fixtures/README.md`.
- `bank/README.md`: one section on verification.
- `justfile`:
  - Add `test-verify-full: sandbox-build`, which runs `POPQUIZ_SANDBOX_SUITE=1 POPQUIZ_VERIFY_FULL=1 … pytest tests/sandbox/verify -vv`.
  - `test-full` gains it.
  - Nothing else is touched: not `PENDING`, and not `verify *ARGS`.

**Record fields:**

| Field | Source |
|---|---|
| `rustc` | the probe's full `-Vv` |
| `target_triple` | its `host:` line |
| `edition`, `flags` | the pin (the runner's inputs) |
| `runs` | {N, true} |
| `stdout`, `exit_code` | run 1 |
| `miri.version` | the probe |
| `seeds` | `MIRI_SEEDS=(0,)` |
| `verified_at` | UTC `Z` |
| `verifier_version` | `popquiz-verify <sha256(verify.py+runner.py+sandbox.py)[:12]>`: derived; no importlib |

## Audit of `mvp/tools/verify.py`
**Kept:**
- `RUNS=5` (L22).
- The compile-then-run split (L171–216).
- `error[` code parsing (L180–186), keeping compiler order rather than sorted.
- Accept a failure only when it was declared (L191); reject a declared failure that compiled (L195–198).
- The set-identity run comparison (L209), widened to (stdout, exit).
- Run 1's output as the answer (L213).

**Dropped:**
- The "PINNED" claim (L5), which is false (D-17).
- The no-Miri caveat (L12–13), superseded.
- `subprocess` and `tempfile` (L16–20): the seam replaces them.
- The inline questions (L26–157): content lives in the bank.
- `rustc --version` from PATH (L160–163): replaced by the sandbox `-Vv` compared against the pin.
- The literal "2021" and missing `-C` flags (L172, L239): these come from the pin now.
- The `{**q}` merge (L177): the record is replaced whole instead.
- `stderr_head` (L188–190): not in §3.2.
- The `answer` string (L192): the answer is derived.
- Host execution with timeout=10 (L202): the sandbox runs it now.
- Nonzero exit rejects (L203–206): `panic` is an option kind.
- The bool plus `reject_reason` (L191, L197): replaced by a coded `Verdict`.
- The aggregate `verified.json` batch (L219–250).

## Choices and contract tensions (side taken)
1. **Declared answer.** The contract has no field for it, so it is an argument (`ran|does_not_compile|ub`). It is not stored, and T-16 passes it. `expect_from_record` recovers it for re-verification.
2. **Nightly date vs Miri version.** The pin's nightly is 2026-09-19 but the recorded Miri build date is 2026-09-18, so comparing dates would always refuse. Compare the whole `miri_version` string instead; the `+nightly-<pin>` toolchain name enforces the date.
3. **`output_matched` means exact equality, including for declared UB.** A byte-prefix rule would record `true` for outputs that differ. A UB question must therefore print everything before its UB, which constrains the generator (T-16). Reported, not relaxed.
4. **`bank.correct_index` cannot derive a UB or panic answer,** and could mark a distractor correct when stdout equals its text. `bank.py` is T-14's, and this amends §3.1, so it is **not** fixed here: it is reported to the Orchestrator as a contract defect. `check_provenance` fails closed instead: a "ran" record must derive an `output`-kind option with a zero exit (when recorded) and `miri.clean` not false.
5. **G-2 limit.** A hand edit of `verified.stdout` inside a genuine record passes. The docstring states it, and a digest field is proposed via tone-architect.
6. **Staleness is widened** from §7.2's `rustc` to the whole `[pin]` table, which is what `pin.toml` says a record is compared against.
7. **Justfile:** PENDING and `verify *ARGS` stay stale; the Orchestrator should fix them after PQ-24.
8. **"Byte-identical" is decoded text,** because `sandbox.py` uses `text=True`. Flagged; `sandbox.py` is not cleared for this.
9. **Tree Borrows runs only for declared UB,** per the ticket. The synthetic `complete-ran` receipt fixture lists both configs; the ticket wins.
10. **Leaks reject** under their own code; there is no `-Zmiri-ignore-leaks`.
11. **An `abort()` exit (134)** reads as MEMORY_LIMIT in T-15a's inference, so it rejects. Conservative; noted.
12. **Architecture:** the twin compares everything except `rustc`'s host line and `target_triple`, and asserts the triple is in `pin.target_triples`.
13. **Legacy q3/q4/q7/q8 are left alone.** `test_migration` asserts byte equality with `migrate_mvp`'s output. Re-verification is a T-20 or organizer step.
14. **The rejection tally across a batch** belongs to T-16's run report; `verify` reports per-candidate codes.

## Next session, in order
0. **PR #12 merged at 2026-09-22T05:51Z** (main `696b3ce`), and `ai-c11-cc/sandbox-image` was deleted. Run `git fetch origin && git merge origin/main`; the branch then descends cleanly from main. Drop the "Based on #12" PR-body line. Never rebase.
1. Read `lattice comments PQ-20` for the Orchestrator's answers.
2. Run the plan-review subagent (Sonnet, contract paths only).
3. Append `## Plan-review resolutions`.
4. `lattice status PQ-20 planned`.
5. Implement per the arc in the boot prompt.

# Resume amendments (delegator, 2026-09-26) — override the draft above on conflict

Branch now: `0156b13` = signed merge of origin/main `696b3ce` onto `9a758cd`; upstream unset; PR opens against `main` with no "Based on" line.

**A1 — toolchain mode (ruling 1).** `sandbox.MODES` gains `toolchain`: script `rustc -Vv` then `cargo +nightly-<pin.nightly date> miri --version`, same docker argv and limits, no `-e`, no mount. The probe-program fallback is dropped. RealRunner maps kind `toolchain` onto it (cached per runner). Tests on the fake docker callable, like the other modes. `sandbox.py` clearance = Miri flag set (MIRIFLAGS per config, seed) + this mode; nothing else. Draft item "`[profile.dev]` from `pin.flags` in the generated Cargo.toml" stays, read as part of the Miri flag set (Miri must run the same profile the native run used); flagged under deviations.

**A2 — PENDING (ruling 2).** When `just verify` becomes a real recipe (`verify *ARGS` → `uv run python -m popquiz.verify {{ARGS}}`), remove the single token `verify:T-15b` from `PENDING`; touch nothing else on that line. Supersedes draft choice 7.

**A3 — correct_index (ruling 3, F-19).** Minimal change in `bank.correct_index`, by option kind, never text:
- ran, `exit_code` present and non-zero → the single option of kind `panic`.
- ran, `miri.clean is False` and `miri.configs` contains both `stacked_borrows` and `tree_borrows` → the single option of kind `ub`.
- otherwise the existing output/dnc rules. Two matches still raise `BankError`.
A legacy UB record (no `configs`) still derives `None`, so `test_audit.py:627` (PQ-24's, asserts None) and `audit.answer_index`'s fallback are untouched. Tests in `test_bank.py`: panic → panic option; UB with both configs → ub option; UB with SB only → None; two panic options → BankError; a panic whose stdout equals an output option's text → the panic option, not the output. Supersedes draft choice 4; `check_provenance` now derives through `correct_index` for all three answer kinds instead of failing closed on UB/panic.

**A4 — merged tree.** `popquiz/slot.py` and `audit.py` do not construct `Verified` or touch the verifier seam; `tests/test_audit.py` constructs `Verified(` in test code, so the row-23 static test scans `src/popquiz` only.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)
Review: Sonnet subagent, 2026-09-26, 2 Critical / 2 Major / 3 Minor.

**C1 — A3 rule order: a genuine UB record with a nonzero native exit could derive `panic`.** *Accepted.* Order becomes: (1) does-not-compile → the single `does_not_compile` option; (2) ran, `miri.clean is False` and `configs` ⊇ {stacked_borrows, tree_borrows} → the single `ub` option; (3) ran, `exit_code` present and non-zero → the single `panic` option; (4) ran → the output option equal to `normalized_output(stdout)`. Each rule that fires is final: if its kind has no option it returns `None` and never falls through to text equality (nothing is guessed); two matches raise `BankError`. The UB verdict outranks the exit code because Miri's UB finding is what the question is about, whatever the native binary happened to do. New `test_bank.py` case: both configs UB, `exit_code` 101, options include a `panic` and a `ub` → the `ub` index. Also: a panic record with no `panic` option → `None` even when an output option equals stdout.

**C2 — `check_provenance` has no test for the panic/UB path.** *Accepted.* Row 22 gains:
- accepts a genuine panic record and a genuine both-configs UB record (derives through `correct_index`);
- refuses an option whose `kind` was hand-edited (e.g. the `panic` option relabelled `output`, or a second `ub` option added) — `correct_index` then yields `None` or raises, and the check refuses in plain words;
- refuses a record whose fields contradict each other: `miri.clean is False` with fewer than both configs, a `configs` entry outside the known set, `exit_code` present without `runs`, a does-not-compile record carrying `runs`/`stdout`/`exit_code`/`miri`.
Hand edits *inside* a genuine record that stay internally consistent (e.g. `exit_code` 101→0 with a matching output option) are the same gap as C5/M3 below and are stated in the docstring.

**M3 — a hand edit of `verified.stdout` passes `check_provenance`, against AC-7.** *Partly accepted; side taken.* AC-7 reads "a hand-edited answer in a candidate file fails the build's provenance check". The answer is not stored (§3.1, G-2), so the edits that can change the correct option are: option text/kind (caught), an added answer field (caught), and forging the machine record itself. Closing the last one needs a field §3.2 does not list (a digest or signature over the record), and "`verified.json` gains no field code did not write" plus rule 7 (no contract edits) keep it off this ticket. The AC-7 test proves the option-side edits; the record-forgery gap goes to the Orchestrator as a contract finding in DONE, with the digest proposal. Listed under deviations.

**M4 — Files ledger omits `bank.py` and `test_bank.py`.** *Accepted.* Change list gains `pipeline/src/popquiz/bank.py` (`correct_index` only, per ruling 3) and `pipeline/tests/test_bank.py` (the A3/C1 cases). Also `pipeline/src/popquiz/audit.py` is **not** changed: its UB fallback becomes redundant for complete records but stays correct and is PQ-24's; noted for the Orchestrator.

**m5 — `[profile.dev]` from `pin.flags` stretches the sandbox.py clearance.** *Kept, as a deviation.* Without it the Miri run and the native run each rely on cargo's/rustc's default rather than the pin's flags, so the record's `flags` would describe inputs nobody passed. The profile goes into the Cargo.toml the `run` and `miri` modes already generate; no other sandbox.py change.

**m6 — `verify *ARGS` omits `--offline --no-sync`.** *Accepted.* Recipe: `cd pipeline && uv run --offline --no-sync python -m popquiz.verify {{ ARGS }}`, matching `bank-audit` (justfile:147).

**m7 — `_recorded` / `_source_sha256` keys would show in StubRunner's "it has:" message.** *Accepted.* StubRunner skips `_`-prefixed keys both when looking up a kind and when listing what a fixture has; a test asserts the message lists step kinds only. Provenance still requires `_recorded` (FixtureError when absent).
