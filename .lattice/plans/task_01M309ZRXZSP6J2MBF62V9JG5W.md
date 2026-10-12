# PQ-23: Review surface

BUILDPLAN.md T-18 (M3).

Review surface (local, served by the CLI): one screen per candidate, accept/reject-with-reason/edit-and-reverify, difficulty judged, **affirm** as a blocking gate with who/when, middle-beat and quoted-output checks; §11.1 trope matches highlighted beside `explains`, `why_tempting` and `hint` as warnings that block nothing (`SPEC.md` §7.4, D-23)

Criteria: AC-19–22, AC-72–74, AC-88, AC-95, G-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14, T-15b
BUILDPLAN notes: —

Orchestrator notes: Reads `near_duplicate_of` written by T-17.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator-pq23, 2026-10-12)

Deliverable: one PR against `main` — "Review decisions and the affirm gate in the organizer TUI" — adding a `popquiz.review` CLI (the one writer) and a review screen in `prototypes/organizer-tui`. Acceptance: `just test` green < 60 s warm, `just test-full` green once, `just bank-audit` on the real bank unchanged (nyc reserve 1), `git diff origin/main --stat` inside the cleared files, nothing under `bank/questions/`.

## 1. Record fields (bank.py)

`Review` gains two optional fields (absent is absent, so existing records do not change):

- `affirmed_digest: str | None` — `"sha256:" + hex` of the affirmed text, written by `review affirm` only.
- `quotes_checked: tuple[str, ...] | None` — quoted spans the organizer attested are not a mismatched output claim (AC-73, see §3).

New in bank.py:

- `affirmation_digest(question) -> str`: SHA-256 over canonical JSON (`sort_keys`, `ensure_ascii=False`, compact separators) of `{source, options (text, kind, why_tempting), hint, explains (what, takeaway, legacy), trace}`, each through the existing `_dump` (absent is absent). **Inputs are a superset of ruling 4's three** (explains, why_tempting, trace): source, option text/kind and hint are added so a hand edit of the program or the answer options after affirmation also unbinds it. `verified` and `review` are not inputs (re-verifying the same output keeps the text the organizer read; the trace's `stdout` row *is* an input, so a changed output unbinds).
- `affirmation_state(question) -> "none" | "bound" | "stale" | "legacy" | "unbound"`:
  - `none`: no `affirmed_by`+`affirmed_at`;
  - `bound`: digest present and equal to the current digest; `stale`: present and different;
  - no digest: `legacy` **when `verified.legacy` is true** (the MVP-batch records — today exactly q3), else `unbound`.
- `is_affirmed(question) = state in {bound, legacy}` — the one predicate every reader uses.
- `validate_question` additions (structure only): a digest must match `sha256:[0-9a-f]{64}` and may exist only beside `affirmed_by` and `affirmed_at`; `quotes_checked` must be a list of strings. A **stale** digest does not make a record unloadable (it makes it unaffirmed) — see contract notes.
- `Review.affirmed()` stays as the who-and-when half (tests outside the cleared files call it); its docstring says it is not the gate and points at `is_affirmed`.

## 2. Readers (ruling 4)

- `schedule.refusal`: after the missing-who/when check, `stale` → "‹id› was affirmed, but its text changed after that - affirm it again at review (GAP-24)"; `unbound` → "‹id›'s affirmation records who and when but not what was affirmed - affirm it at review (GAP-24)". Hard errors, nothing sent or written.
- `audit.in_reserve`: `is_accepted(q) and bank.is_affirmed(q) and q.used_by(club) is None`.
- q3: legacy verified, digest-less, accepted → `legacy` → still in reserve; nyc reserve reads 1.

## 3. `python -m popquiz.review` (review.py — the one writer, ruling 1)

`--bank DIR` (default `schedule.DEFAULT_BANK`). Exit 0 done; 1 refused (one plain sentence on stderr, file byte-identical); 2 unreadable input/usage. Every write is `load_question` → `dataclasses.replace` → `bank.save_question` (validates). No subcommand writes `verified`.

| Subcommand | Checks (blocking) | Writes |
|---|---|---|
| `show ID --json` | — | nothing; prints the review report (below) |
| `accept ID [--checked-quote SPAN]…` | verified present; `correct_index` derives one option; AC-73: every quoted span (`bank.quoted_outputs`) is a whole line of `normalized_output(stdout)` (or the whole output), or is in `quotes_checked`/`--checked-quote` | `status=accepted`, `quotes_checked` ∪= given |
| `reject ID --reason TEXT` | reason non-blank | `status=rejected`, `reason=TEXT`; clears `affirmed_by/at/digest` |
| `difficulty ID LEVEL` | 1–5 (`generate.DIFFICULTIES`) | `difficulty_judged` |
| `export ID --out FILE` | — | writes the editable fields (`source`, `options`, `hint`, `explains.what/takeaway`, `trace`) to FILE (a temp file, never the bank) for the organizer's editor |
| `edit ID --from FILE [--expect KIND]` | FILE keys ⊆ editable fields; `explains.legacy` never rewritten | the edit; `status=edited`; clears `affirmed_by/at/digest`. If `source` or any option `text`/`kind` changed: drops `verified`, removes the last step's `stdout` row (nothing observed for the new program yet), saves, then calls **`verify.main([path, "--expect", K])`** — the existing writer of `verified` and the trace's `stdout` row. K = `--expect` or `verify.expect_from_record(old.verified)`; neither → refuse before writing. Prose-only edits do not re-verify (SPEC 7.4: re-verify on source or option change). No change → "No change; nothing written." exit 0 |
| `reverify ID [--expect KIND]` | — | only through `verify.main` |
| `affirm ID [--by NAME]` | identity from `--by` or `POPQUIZ_ORGANIZER`, **no default** → "Affirm needs your name: set POPQUIZ_ORGANIZER or pass --by NAME."; not already affirmed (protects q3's hand record); status ∈ {accepted, edited}; does-not-compile → blocked on F-15; verified present and answer derives; trace ≥ 2 steps; every incorrect option has non-blank `why_tempting` (AC-95); AC-73 as in accept | `affirmed_by`, `affirmed_at` (UTC ISO-8601, seconds), `affirmed_digest` |
| `reasons [--json]` | — | nothing; lists rejected ids and reasons (AC-19 signal for the generator) |

**Runner selection for re-verify.** Default `RealRunner` (pinned rustc + Miri in Docker: `just test-full` and real use). `POPQUIZ_VERIFY_REPLAY=DIR` (test-only) uses `StubRunner(DIR)` **only when** `DIR/<id>.json`'s `_source_sha256` equals SHA-256 of the record's current source; otherwise refuse ("no recording for this source"). Output always says "re-verified against recorded outputs, not the toolchain" when replaying. `verify.py` is called, not changed.

**`show --json` report:** id, topic, source, options (text, kind, why_tempting, correct: bool from `bank.correct_index`), answer_error, hint, explains, trace step count, receipt lines (`receipt.receipt_lines`) + legacy flag, difficulty requested/judged, review status/reason/affirmed_by/affirmed_at, affirmation state, near_duplicate_of, quotes (span, matches, checked), warnings (`copylint.check_prose`, ruling 5: never blocking), accept_blockers, affirm_blockers, blocked_on ("F-15" for does-not-compile).

## 4. TUI (prototypes/organizer-tui)

- `src/pipeline.rs` (new): `python_command(module, args)` extracted from `scheduling.rs` (POPQUIZ_PYTHON or `pipeline/.venv/bin/python`, cwd `pipeline/`, `PYTHONPATH=pipeline/src`, stdin null) plus the background-thread job + `safe_report`. `scheduling.rs` switches to it unchanged in behaviour.
- `src/review.rs` (new): a sub-state like `Schedule`. `v` opens it. Stages: **Notice** (AC-22: "Review shows the answer. You will have seen it; the room has not. Enter continues, Esc goes back.") → **Loading** (`show --json`) → **Screen** → **Reason input** / **Difficulty digit** / **Confirm** → **Running** → **Result** (CLI stdout/stderr verbatim) → reload `show` and the preview's candidate.
- Screen (one scrollable pane, AC-20): status line (review status; affirmation as words: *not affirmed*, *affirmed by X at T*, *affirmed (unbound, before GAP-24)*, *affirmation no longer matches the text*); difficulty requested/judged; near-duplicate note (or *No near-duplicate recorded*); source; options in bank order with `✓ correct (machine record)` in text; each option's why_tempting with `warning (does not block): group "match"` lines beside it; three beats + hint (marked *human-written*), each with its warnings; receipt under *How we know / machine record* with the executed-paths line; quoted output rows (`matches a printed line` / `not a printed line — checked` / `not a printed line — blocks accept`); checks as `READY` / `BLOCKS accept` / `BLOCKS affirm` / `WAITING (F-15)` words. Colour only doubles the words (AC-40).
- Keys on the screen: `a` accept (confirm lists unmatched spans; `y` passes them as `--checked-quote`), `x` reject (reason input, Enter, `y`), `d` then `1`–`5` then `y` difficulty, `e` edit, `A` affirm (confirm shows "Organizer: NAME" from `POPQUIZ_ORGANIZER` or "Organizer: not set"; `y` runs the CLI, which refuses without a name), `j/k/PgUp/PgDn` scroll, `Esc` back. Every write needs an explicit `y`.
- Identity: read from `POPQUIZ_ORGANIZER` once at open, passed as `--by NAME` only when set, and the subprocess gets `env_remove("POPQUIZ_ORGANIZER")`, so the name shown is the name used.
- Edit: `e` runs `review export` to a temp file, the main loop suspends the terminal and opens `$VISUAL`/`$EDITOR` (no default: unset → "Set EDITOR to edit a question."), restores, then a confirm ("Save this edit? A source or choice change re-verifies with rustc and Miri and may take minutes.") → `review edit --from FILE`; the TUI removes the temp file. **The TUI writes no file itself.**
- Footer/help gain `v Review`; existing preview tests stay green.

## 5. Fixtures and tests

Fixtures `pipeline/tests/fixtures/review/questions/`: `q4.json` (copy of q4, review cleared except reason), `q3.json` (q3 with a fixture identity, legacy affirmed), `q8.json` (does-not-compile), `q7.json` (empty trace), `wrongquote.json` (q4 shape, a beat quoting output it did not print), `tropes.json` (a beat with §11.1 matches). Tests copy the directory to `tmp_path`; a module fixture asserts `git status --porcelain bank/` is empty after.

`pipeline/tests/test_review.py`: round-trips for accept / reject-with-reason (read back by `reasons`) / difficulty / edit (AC-19); AC-73 wrong quote blocks accept until checked; affirm refusals (no identity, unaccepted, rejected, missing why_tempting, < 2 steps, F-15, already affirmed) each leave the file byte-identical; affirm writes by/at/digest and the record schedules (`schedule.refusal is None`, `in_reserve`); prose edit clears affirmation and sets `edited`; option edit drops `verified` and re-verifies through replay; source edit with no recording refuses re-verify and leaves no `verified`; warnings never block; `verified` byte-identical across every non-verify write.
`test_bank.py`: digest inputs, states, validation. `test_schedule.py` / `test_audit.py`: stale and unbound refused, legacy kept, gate == reserve; the `affirm` helpers write a digest.
`prototypes/organizer-tui/tests/review.rs` (preview.rs style): notice before answers; screen render from a report fixture (warnings beside text, words not colour, legacy label, F-15 waiting); command args (checked quotes, `--by` only when set, env removed); an end-to-end run through the real CLI on a temp copy of the fixtures (accept → affirm refused without identity → affirm → prose edit clears it → re-affirm); a source scan that `src/` has no `fs::write`/`File::create`/`OpenOptions`.

Mutations (each run, each must kill a named test): M1 identity defaults to a name; M2 affirm skips the status check; M3 edit keeps the affirmation; M4 `is_affirmed` ignores the digest; M5 the TUI writes a `verified` file. Plus M6 accept skips AC-73.

## 6. Commit order

1. bank: bind an affirmation to its text (fields, digest, state, validation) + test_bank.
2. schedule and audit read the bound affirmation + tests/helpers.
3. review CLI: show, accept, reject, difficulty, reasons + fixtures + tests.
4. review CLI: export, edit, reverify, affirm + tests.
5. TUI: shared pipeline subprocess plumbing (refactor of scheduling.rs).
6. TUI: review screen and decisions + tests.
7. TUI: edit through the organizer's editor.
8. Docs: organizer-tui README, pipeline/README and bank/README review sections.

## 7. Contract tensions and the side taken

- **Legacy narrowed** (ruling 4): a digest-less affirmation is legacy-valid only on a record whose `verified` is legacy (q3). Otherwise a hand-typed affirmation (GAP-24's M12b) would keep scheduling on every new record. q3 is unaffected.
- **Digest inputs widened** (ruling 4): source, option text/kind and hint added to explains/why_tempting/trace.
- **Stale ≠ unloadable**: a stale digest makes the record unaffirmed in every reader rather than a `BankError` at load, because an unloadable record cannot be repaired through review and would crash `load_bank` for generate/dedupe/audit. Malformed digests are rejected at load.
- **AC-73 vs `bank.quoted_outputs`**: the helper deliberately does not decide which spans claim output. The check is mechanical (whole printed line or whole output); every other span blocks accept until the organizer attests it, recorded in `review.quotes_checked`.
- **`edited` counts as accepted** (audit.ACCEPTED); affirm accepts `accepted` or `edited` but re-runs every accept check.
- **AC-19 generator read-back**: `review reasons` and `review.rejection_reasons()` expose the signal; wiring them into `generate.py` is outside the cleared files.
- **Re-affirm refused while affirmed**: protects q3's hand record; an edit (which clears) is the way to a new affirmation.
- **Real verifier in tests**: re-verify's real path is `verify.main` with the default `RealRunner`, already exercised by `test-verify-full`; this ticket adds no sandbox test (outside the cleared files).

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Review: general-purpose subagent (sonnet), 2 Critical, 4 Major, 5 Minor. All resolved:

1. **C1 (stdout row forgeable through a trace edit) — fixed.** `trace` is not an editable field. `export`/`edit` expose the trace only as `trace_notes` (one string per step, same count); `lines`, `focus`, `values` (including the `stdout` row) and `pivot` cannot change through review. The only change review makes to a `stdout` row is removing it when the source or an option changes, and then only the verifier writes it back.
2. **C2 (ruling 6, q4's trace) — fixed by 1.** Structure is immutable; note text edits go through AC-73 again at accept/affirm (notes are in `quoted_outputs`). This PR edits no real question.
3. **M3 (schema outside cleared files) — asked.** Clearance for `bank/schema/question.schema.json` `$defs.Review` requested on the ticket 2026-10-12; `question_from_dict` reads both new keys. Work not touching the schema proceeds.
4. **M4 (replay env var a production bypass) — fixed.** No env var. `review.main(argv, *, runner=None)` passes `runner` straight to `verify.main`; only tests inject a `StubRunner`. The CLI always uses the real toolchain.
5. **M5 (half-saved record on failed re-verify) — fixed.** A source/option edit is written to a temp file, verified there through `verify.main`, and the bank file is replaced only when verify accepts. Rejected or refused: the bank file is byte-identical, and the TUI keeps the organizer's edit file and says where it is.
6. **M6 (`verified.legacy` is hand-settable) — accepted residual, stated.** The digest is a binding, not a signature; a determined hand edit can also recompute it. The narrowing still closes M12b for every record the verifier wrote. Named in DONE and the PR.
7. **Minor: refusal order** — stale/unbound branches sit before the generic `in_reserve` fallthrough in `schedule.refusal`.
8. **Minor: digest stability** — a test re-verifies identical output and asserts the affirmation stays bound.
9. **Minor: Rust source scan** — extended to `fs::write`, `File::create`, `OpenOptions`, `fs::copy`, `fs::rename`, `write_all`.
10. **Minor: real-bank safety** — `--bank` is **required** on `popquiz.review` (no default); the TUI always passes its bank. M7 added: the TUI end-to-end test runs against a temp copy (asserted by path).

## Reset 2026-10-12 by agent:delegator-pq23
