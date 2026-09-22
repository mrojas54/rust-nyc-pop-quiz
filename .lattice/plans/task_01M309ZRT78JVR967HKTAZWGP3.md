# PQ-22: Dedupe

BUILDPLAN.md T-17 (M3).

Dedupe: exact hash, normalized-AST fingerprint, near-duplicates by token-bigram Jaccard at the configured threshold (D-13) to the review queue, persistent history with visible size

Criteria: AC-14–18
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14
BUILDPLAN notes: Serialized on `pyproject.toml`

Orchestrator notes: Writes `review.near_duplicate_of` (F-8 ruling); T-18 reads it.

Workflow mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-21)

Branch `ai-c11-cc/dedupe` off `origin/ai-c11-cc/bank-format` @ b4d946c (PR #10).
Baseline: `just test` green, 142 pipeline tests, 1.1 s warm.

## Files

| File | Change |
|---|---|
| `pipeline/src/popquiz/dedupe.py` | the module (replaces the scaffold stub) |
| `pipeline/tests/test_dedupe.py` | new: AC-14…AC-18 |
| `bank/README.md` | one new section, *Dedupe* (nothing else in the file changes) |
| `bank/history.json` | **not changed in this PR** — see D-5 |

Standard library only. No `pyproject.toml`, `justfile`, `bank.py` or other edits.

## The module

1. **Lexer** (`tokenize(source) -> list[Token]`). Hand-written, standard library.
   Skips whitespace, `//` line comments (doc comments included), nested `/* */`
   block comments. Emits: identifiers (incl. raw `r#x` and non-ASCII), keywords
   (strict + reserved set of the 2021 edition), lifetimes/labels (`'a`), char
   literals (`'x'`, `'\n'`, `'\u{..}'`, `b'x'`), string literals (`"…"`,
   `r#"…"#`, `b"…"`, `br"…"`, `c"…"`, with escapes), numbers (dec/hex/oct/bin,
   `_`, floats, exponents, suffixes; `1..2` is not a float, `t.0` stays an index),
   and **single-character punctuation**. Operators are not joined: whether `>>`
   is a shift or two closing generics is a parse decision, and not joining makes
   `Vec<Vec<i32> >` and `Vec<Vec<i32>>` the same stream. An unterminated literal
   or comment raises `DedupeError` (a candidate that cannot be lexed is reported,
   not silently fingerprinted).
2. **Declared names** (`declared_names(tokens)`). Token-level heuristics, no
   parser. A name is *declared by the program* if it appears in a binding or item
   position: `let`/`if let`/`while let` patterns (up to a depth-0 `=`, `:`, `;`,
   `else`); `for PAT in`; fn params (pattern before the depth-1 `:`); closure
   params `|…|` (open `|` recognised by the previous token being start, `(` `,`
   `=` `{` `;` `=>` `move` `return` `[` `:`); match-arm patterns (back from each
   `=>` to the arm start, stopping at a guard `if`); `@` bindings; `fn`, `struct`,
   `enum` (+ variants), `union`, `trait`, `type`, `const`, `static`, `mod`,
   `macro_rules!` names; struct and struct-variant field names; generic params and
   lifetimes. In a pattern an identifier is a binding only if it starts lowercase
   or `_`, is not followed by `(` `{` `::` `!`, and is not a `field:` inside braces.
   Two sub-sets are kept apart: **members** (fields and fn names — reachable after
   `.`) and **macros**.
3. **Normalization** (`normalize(source) -> list[str]`). Alpha-renames declared
   names to placeholders `$1, $2, …` in order of first appearance. **Undeclared
   names are always kept** — std methods, types, macros, paths — so two programs
   that call *different* library functions can never be made equal by renaming
   (that would be a false *rejection*, the expensive failure). Context rules:
   after `.` rename only members; after `::` rename only if the previous segment
   was renamed or is `Self`/`self`/`crate`/`super`; `name!(` renames only declared
   macros; a declared member whose name is in a small std keep-list (`len`,
   `push`, `iter`, `new`, …) is kept after `.`/`::`. Inside the format string of
   a formatting macro (`println!`, `format!`, `write!`, `panic!`, `assert*!`, …),
   inline args `{x}` / `{x:?}` are renamed with the same map (not `{{`). Two
   comma rules for rustfmt's moves: drop `,` before `]`/`}`, and before `)` only
   when the group already has a depth-level comma (so `(x,)` stays a 1-tuple);
   drop `,` after `}` when the next token is not a closer (block match arms).
   Literals, numbers, attributes and keywords are kept verbatim.
4. **The three stores** per question, in T-14's `History` shape:
   `exact_hashes[qid] = bank.source_hash(source)` (reuse, don't re-implement);
   `ast_fingerprints[qid] = sha256(json.dumps(normalized tokens))`;
   `token_bigrams[qid] = sorted distinct bigrams` of the normalized stream, each
   `"a b"`, **with every renamed identifier abstracted to one placeholder `$`**
   (see D-3). Token texts carry no unquoted space, so the join is unambiguous.
5. **The check** (`check(source, history, *, threshold, own_id=None) -> Verdict`).
   Pure. Order: exact hash → `exact_duplicate`; fingerprint → `normalized_duplicate`;
   max Jaccard over recorded bigram sets ≥ threshold → `near_duplicate`
   (of the most similar qid; ties by id); else `none_found`. `own_id` excludes
   that id's entry (for T-18's edit path — an edited question is not a duplicate
   of itself). `Verdict` = `{kind, duplicate_of?, similarity?, threshold,
   statement, detail}`. `statement` is exactly `UNIQUENESS_STATEMENT = "no exact
   or normalized duplicate found"` for `none_found` and `near_duplicate`, and
   `None` for the two rejections; nothing else ever states uniqueness.
   Threshold: `NEAR_DUPLICATE_THRESHOLD = 0.6`, named and documented as
   *uncalibrated*; a function parameter and a CLI flag `--threshold`; validated
   `0 < t ≤ 1`.
6. **The history** (`record`, `sync_with_bank`, `size`). `record` returns a new
   `History` with a question's three entries; refuses to overwrite an existing id
   whose entries differ (ids are never reused; T-18's edit path passes
   `replace=True`). `sync_with_bank(history, bank)` adds any bank question missing
   from the history (*backfilled*) and refreshes an entry whose recorded exact hash
   no longer matches the bank's source (*refreshed*), reporting both. The history
   therefore always describes the bank and is rebuildable from it — a history
   reset by re-running the migration heals on the next run. `size(history)` = the
   number of programs recorded (plus the per-store sizes) for the review surface's
   first screen.
7. **The run** (`run(bank_dir, candidates, *, threshold, write=True) -> RunReport`).
   Loads bank + history, syncs, then checks each candidate **in order**, recording
   each admitted one before the next is checked (two near-identical candidates in
   one batch are caught). Admitted = `none_found` or `near_duplicate`: appended to
   the bank with `bank.append_question` with `review.status` unset — that is the
   **review queue** (D-2) — and for a near-duplicate with
   `review.near_duplicate_of = <qid>` (F-8), an existing `review.reason` kept.
   Rejected candidates are written nowhere. All ids are checked up front (in the
   bank, in the history, or repeated within the batch → `DedupeError`, nothing
   written). Question files are written before the history, and the history once
   at the end; a crash between the two is healed by the next run's sync.
   `write=False` is a dry run: same report, nothing on disk. `RunReport` =
   verdicts, `history_before`, `backfilled`, `refreshed`, `admitted`,
   `history_after`, `growth = after − before`, threshold.
8. **CLI** `uv run python -m popquiz.dedupe [CANDIDATE.json …] [--bank DIR]
   [--threshold T] [--dry-run] [--json]`. Each file holds one candidate in the
   §3.1 shape, loaded with `bank.question_from_dict` (so it is validated). Default
   bank is `<repo>/bank` resolved from the module path (as `migrate_mvp` does), so
   the CLI works from any cwd. Prints one line per candidate
   (`<id>: rejected — exact duplicate of q3` / `<id>: no exact or normalized
   duplicate found` / `<id>: no exact or normalized duplicate found; near-duplicate
   of q3 (similarity 0.72 ≥ 0.60, threshold uncalibrated) — sent to review`) and a
   history line (`history: 4 → 6 programs (+2; 4 backfilled…)`). `--json` prints
   the report as JSON. No candidates = sync + report (the status view). Exit 0 if
   every candidate was admitted, 1 if any was rejected, 2 on a usage/bank error.

## Tests (`pipeline/tests/test_dedupe.py`), by criterion

- **Lexer** (supporting): comments/nesting/doc comments vanish; lifetimes vs chars;
  raw strings with `#`; `1..2`, `t.0`, suffixes; unterminated literal raises;
  every migrated bank question lexes.
- **AC-14**: a byte-identical resubmission of each migrated question (q3, q4, q7,
  q8) under a new id is `exact_duplicate` of it; one changed byte is not exact.
- **AC-15**: fixtures, inline Rust strings derived from q3/q8 by hand: (a) every
  binding renamed, (b) reformatted (line breaks, spacing, trailing commas, block
  match arm comma, comments added), (c) both, (d) inline format arg renamed
  (`{v:?}`→`{w:?}`) — each `normalized_duplicate`. Negative controls that must
  **not** be normalized duplicates: a different std method (`dedup`→`sort`), a
  changed literal, swapped operands (`a - b` vs `b - a`), `(x,)` vs `(x)`, two
  programs differing only in an *undeclared* name. Renaming is a bijection (two
  distinct declared names never share a placeholder).
- **AC-16**: a near-duplicate (a statement added/changed) at ≥ threshold and not
  a normalized duplicate → `near_duplicate` of the right qid; through `run()` it
  lands in the bank with `review.near_duplicate_of = q`, `review.status` unset,
  not affirmed (neither accepted nor dropped); an unrelated program is
  `none_found`; the threshold is a parameter (the same pair flips at a threshold
  above its similarity) and the default is 0.6; out-of-range thresholds refused.
- **AC-17**: two dedupe passes run as **separate subprocesses in two separate
  working directories** (`sys.executable -m popquiz.dedupe`), the second on a
  copy of the first's bank (as a clone would carry it): the history persists and
  grows (sizes 4 → 5 → 6 including backfill of the four migrated questions), and
  the run report states before/after/growth; `size()` is the first-screen number;
  a dry run writes nothing; a history wiped to T-14's empty shape is rebuilt by
  sync; the committed `bank/history.json` still loads in T-14's shape.
- **AC-18**: scan `dedupe.py`'s source for `original` (case-insensitive); run the
  CLI (text and `--json`) and `run()` over a corpus covering all four verdict
  kinds and assert every uniqueness string equals the statement exactly and no
  output contains `original` or its synonyms (`unique`, `novel`, `new question`).
- Speed: all in-process except the two AC-17 subprocess runs; target < 2 s.

## Decisions and deviations (to be carried into the DONE comment)

- **D-1 (deviation, required by the ticket).** AC-15's "normalized AST" is a
  **token-level approximation**: no Rust parser in the standard library. Misses:
  reordered items/statements, `a + b` vs `b + a`, different but equivalent
  expressions, a declared name that shadows a std name kept by the keep-list,
  shadowed rebinds (`let x = 1; let x = x + 1;` gets one placeholder — correct as
  a bijection, noted), macros from `macro_rules!` bodies. Those fall through to the
  near-duplicate check and a human. Stated in the module docstring and README.
- **D-2 (decision).** The *review queue* is the bank's records with no
  `review.status`; dedupe admits a candidate by appending it there. Nothing else
  in the contract defines the queue, and the migrated records already sit there.
- **D-3 (decision).** Near-duplicate bigrams abstract every renamed identifier to
  one placeholder; the fingerprint keeps them numbered. Numbered placeholders
  shift when one statement is inserted, which would collapse Jaccard for exactly
  the near-duplicates AC-16 exists to catch.
- **D-4 (decision).** The history is compared against every question in the bank
  whatever its review status: a question an organizer rejected coming back is
  still a resubmission.
- **D-5 (contract tension).** `pipeline/tests/test_migration.py:90` (T-14's)
  asserts `bank/history.json` equals what `migrate_mvp` writes — the empty
  shape — and `migrate_mvp.main()` rewrites it empty. So committing a filled
  history (the four migrated questions) turns `just test` red, and the first real
  dedupe run in the repo will too. Side taken: leave `bank/history.json` as
  committed; every run syncs from the bank in memory, so no check ever misses a
  bank question. Reported to the Orchestrator for T-14's owner (narrow the
  assertion to the shape); `test_migration.py` is not cleared for this ticket.
- **D-6 (decision).** Growth has no persisted reference point on the review
  surface: T-14's `History` shape has no run log and `bank.py` refuses unknown
  keys. Growth is in the run report; the first screen gets `size()`; T-18 chooses
  its reference (e.g. the last run report, `--json`).
- **D-7 (decision).** Threshold is configuration by parameter and CLI flag with a
  named default; no env var (that would touch `.env.example`, not cleared).

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: general-purpose subagent (sonnet), 2026-09-21. 0 Critical, 3 Major, 4 Minor.

1. **Major — abstracted bigrams may push unrelated short programs over 0.6 on
   boilerplate alone.** Concern accepted as a real risk; the fix is measurement,
   not a guess. Resolution: once implemented, measure the pairwise similarity of the
   four migrated questions (distinct programs, at bank length) under both bigram
   variants (abstracted `$` vs numbered). Choose the variant that keeps distinct bank
   programs below the threshold while a one-statement edit stays above it, and
   record the measured numbers in the README section and the DONE comment as the
   first calibration data. The AC-16 "unrelated" negative control uses those real
   bank programs, not a contrived fixture. The default stays 0.6 (D-13 says start
   there); if distinct bank pairs clear it anyway, that is reported as a finding for
   HC-2, not silently re-tuned.
2. **Major — no direct tests for `record`'s overwrite refusal or `check`'s own-id
   exclusion.** Accepted. Added tests: `record` raises on an existing id with
   different entries; `replace=True` succeeds; `record` of identical entries is a
   no-op; `check(..., excluding=q)` does not match `q`'s own entry but still
   matches every other. (The keyword is `excluding`, not `own_id`.)
3. **Major — D-5 will bite on the first real run.** Accepted that "reported" is
   not "fixed". I cannot confirm or create a follow-up ticket from here; the
   finding was posted as a lattice comment on PQ-22 (planning phase), and the DONE
   comment and PR body carry it as a **must-fix-before-the-first-real-run** note
   with the exact line and the suggested change. No code workaround is honest:
   the CLI must write `bank/history.json` for AC-17.
4. **Minor — `size()` vs `History.total()`.** Accepted. The function is
   `history_size(history)` = the number of distinct question ids recorded across
   the three stores (a program count), with a comment that it is not
   `History.total()` (the sum of the three stores, 3× once synced).
5. **Minor — `_` handling unstated.** Accepted. `_` is in the lexer's keyword set,
   so it is never a declared name and never renamed. Explicit test on q7's
   `|&(_, k)| k`.
6. **Minor — dedupe writing `review.near_duplicate_of` vs SPEC 3.1's writer
   table.** Accepted. **D-8:** SPEC 3.1 lists the review surface as `review`'s
   writer; the Orchestrator's ruling F-8 (PQ-22 ticket notes, "Writes
   `review.near_duplicate_of` (F-8 ruling); T-18 reads it") makes dedupe the writer
   of that one subfield. Dedupe writes nothing else under `review`, and refuses a
   candidate that already carries a status, an affirmation, a mark or `used`.
7. **Minor — AC-18 synonyms are a widening.** Accepted. The output scan forbids
   `original` (the contract's word) and additionally `unique` and `novel`; called
   out as a deliberate widening in the DONE comment.

Also settled while drafting (no reviewer finding): the ticket's "(keywords and std
names kept)" is implemented as *every name the program does not declare is kept*
— a closed list of std names can never be complete, and an unlisted std method
renamed in two programs would make `v.a()` and `v.b()` equal. No separate
keep-list.
