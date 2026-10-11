# PQ-21: Generator via Message Batches

BUILDPLAN.md T-16 (M3).

The generator: Claude Opus 5 via Message Batches, structured outputs into the §3.1 shape, count/topics/difficulty honoured or reported, talk mode, per-candidate cost and wall-clock, re-runnable

Criteria: AC-1–5
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14
BUILDPLAN notes: Human track: API key, spend cap. Serialized on `pyproject.toml`. The generator's brief carries the room's capacity — source lines at the floor, each option a single line, at most 29 characters (`SPEC.md` §7.1, D-15). The brief also carries the style rules and the §11.1 patterns (D-23).

Orchestrator notes: Human track H-4. Buildable and testable on fixtures without the key; the first paid run is HC-2.

HELD: needs H-4 an Anthropic API key and spend cap. Do not dispatch until the client releases it.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-10-10)

Fixtures only. No request reaches api.anthropic.com from anything in this ticket.

## 1. Module layout and the seam

- `pipeline/src/popquiz/generate.py` — the whole generator: request model, brief, batch
  request builder, result classifier, run directory (state, atomic writes, resume), report,
  CLI (`python -m popquiz.generate`). Split into `generate_*.py` only if it passes ~900 lines.
- The seam, `BatchClient` (Protocol), three methods, mirroring D-18's `Runner`:
  - `submit(requests: list[dict]) -> str` (batch id)
  - `poll(batch_id) -> str` (`processing_status`; done when `"ended"`)
  - `results(batch_id) -> Iterable[dict]` (each `{custom_id, result: {type, message|error}}`)
- `ReplayBatchClient(fixture_path, delay_s=0.0)` — reads one recorded batch from
  `pipeline/tests/fixtures/generate/<name>.json`; records the requests it was handed (tests
  inspect the built brief through it); counts `submit` calls (resume test asserts 0).
- `AnthropicBatchClient(api_key, ...)` — wraps the SDK; docstring says "unexercised until the
  first paid run, HC-2". It refuses in its constructor when no key is given (before importing
  `anthropic`, before any I/O); `anthropic` is imported lazily inside it, so `just test` never
  imports the SDK on the default path. The key is read from the environment only by the CLI on
  a non-fixture run, passed explicitly as `api_key=` (so a stray `ant auth` profile is never
  used silently). Results are normalized with the SDK models' `model_dump(mode="json")` so one
  classifier serves both clients.

SDK calls (from the claude-api skill, python/claude-api/batches.md, tool-use.md "Raw Schema"):
`client.messages.batches.create(requests=[Request(custom_id=..., params=MessageCreateParamsNonStreaming(...))])`,
`client.messages.batches.retrieve(id).processing_status`, `client.messages.batches.results(id)`.
Params per request: `model`, `max_tokens` (configured, default 16000), `system` (the brief),
`messages=[{role:user, content: <this candidate's ask>}]`, `thinking={"type":"adaptive"}`,
`output_config={"effort": "high", "format": {"type": "json_schema", "schema": SCHEMA}}`.
Result types handled: `succeeded` (read `message.content` text block, `stop_reason`,
`usage.{input_tokens, output_tokens, cache_creation_input_tokens?, cache_read_input_tokens?}`),
`errored`, `canceled`, `expired`; `stop_reason` `refusal` / `max_tokens` → reported. Keyed by
`custom_id` (`c01`…), never by position. No server-side `fallbacks` (a refusal is reported, and
the organizer re-runs); noted for HC-2.

## 2. The structured output (what the model is asked for)

Schema (all objects `additionalProperties: false`, no length/array constraints — unsupported —
validated client-side): `source`, `options[]` of `{text, kind ∈ output|does_not_compile|ub|panic,
why_tempting}`, `hint`, `explains {what, takeaway}`, `expect ∈ ran|does_not_compile|ub`, and in
talk mode `concept {name, kind ∈ std|core_language}`.

- **No `correct`, no "expected output", no letter, no position** field exists in the schema. The
  model is never asked which option it believes is right.
- `why_tempting` is required on **all five** options, so the absence of one cannot mark the
  model's believed answer. (bank.Option's docstring says the correct option's is None; that is a
  convention, not validated. Deviation flagged: the correct option's beat is surplus text the
  review step drops after verification.)
- `expect` is verify's declared class (`verify.Expect`, "the generator passes it"); it is a
  category the verifier checks, not an output. It goes in the run report only — never into the
  candidate file, which is §3.1 and nothing else.
- **`trace` is emitted empty** (`bank.Trace()`, legal: "the honest state for a question whose
  walk-through has not been written"). Its last step must name `stdout` with a value: that is a
  model claim of what the program prints in a field the wall reads, and no gate checks it
  (AC-73's `quoted_outputs` reads explains/why_tempting, not trace). Deviation flagged.
- `explains.what` / `takeaway` / `hint` are kept: §3.1 names the generator as writer, and
  AC-73 (T-18) blocks acceptance on a quoted span that disagrees with `verified.stdout`.
- Option order is the model's; bank order is canonical and means nothing (bank.py docstring,
  `schedule.arrange` draws the night's order from the date). The generator never shuffles,
  balances, or records a position. A structural test asserts generate*.py imports neither
  `popquiz.slot` nor `popquiz.schedule`.

## 3. Capacity (ruling 5) — the canonical source

`popquiz.audit` is the Python mirror of the wall's type model (`Room`, `floor_px`, `LINE_HEIGHT`,
`READING_AREA`, `OPTION_MAX_CHARS`, `fit`, `option_fits`). Facts:
1. source lines at the floor = `floor(READING_AREA[1] / (floor_px(room) * LINE_HEIGHT))` → 7 at
   the default room;
2. each option one line (a multi-line output is rewritten to print on one line, never joined);
3. at most `OPTION_MAX_CHARS` (29) characters.
Plus, from the same source, the widest source line that fits at the floor (stated so the check
the model is held to is the one it was told). Room flags as bank-audit's
(`--screen-width-ft`, `--screen-height-ft`, `--back-row-ft`). Candidate check:
`audit.fit(source, READING_AREA, room).fits` and `audit.option_fits` on every option.

## 4. The brief (rulings 5, D-23)

System prompt: the task; the three capacity facts (and width); the shape rules (five options,
exactly one does_not_compile, unique texts); the style brief verbatim from SPEC §7.1 (short
sentences meant to be read aloud, contractions, each claim once, no reassurance, no flattery, no
signposting); every §11.1 pattern from `copylint.TROPES` and the Forbidden row from
`copylint.FORBIDDEN`, rendered from the module (never a copied list). User message per request:
topic, difficulty (1–5, assumption: the bank's existing values are 2–3; no scale is written
down), or in talk mode the title + abstract and the instruction to name one std/core concept.

## 5. Run directory, atomicity, resume (ruling 9, AC-2)

Default out: `bank/candidates/<run_id>/` (the path PR #59 introduced for candidates awaiting
verify; never `bank/questions/`; a run directory per run, so q9…q12 at `bank/candidates/` top
level are never touched). Override with `--out`. `run_id` defaults to `g` + UTC
`YYYYMMDDHHMMSS`; `--run-id` resumes. Files in the run dir:
- `run.json` — the state: request, model, prices, batch id, submitted_at. Written atomically
  right after `submit` returns.
- `<run_id>-NN.json` — one §3.1 candidate each (via `bank.question_to_dict`, validated with
  `bank.validate_question` before writing).
- `report.json` — the run report, written last.
Every write: `.<name>.partial` in the same dir, flush+fsync, `os.replace`. A killed run leaves
complete files or a `.partial`; the next run overwrites/removes its own `.partial` files. Resume
with the same run id: refuses if the stored request differs; never resubmits when a batch id is
stored (no double spend); re-reads results and re-writes candidates (deterministic, idempotent).
Known window: a kill between `submit` returning and `run.json` landing loses the batch id; a
re-run then submits again. Documented; the cap bounds it per run.

## 6. Spend cap (ruling 4)

`--cap-usd` with **no default**. Prices are a per-model table of list-price estimates (D-D:
`claude-opus-5` $5/$25 per MTok), overridable with `--price-in`/`--price-out`; a model with no
price refuses. (Default model per the 2026-10-10 ruling: `claude-opus-5-5`, $4/$20.) Worst case = requests × (max_tokens × out + request UTF-8 bytes × in) / 1e6 ×
0.5 (batch discount; bytes bound tokens from above). A non-fixture run refuses **before
constructing the client and before submit**, one sentence each:
- no cap: "Refusing to submit: no spend cap is configured (--cap-usd); a paid run needs one."
- over: "Refusing to submit: the worst case for N requests is $X, over the $Y cap."
Fixture runs (`--replay`) skip the cap unless one is given. Resume never re-checks (no submit).

## 7. Report, never trim (rulings 6, 10)

Per candidate: custom_id, id (if emitted), topic, difficulty, expect, status
(`emitted`|`reported`), reasons (all found: errored/expired/canceled/refusal/max_tokens/
invalid JSON/shape/capacity with the specific fact), input/output tokens (+cache fields if
present), cost (configured prices × 0.5), wall_clock_s (submit → seen ended, measured locally;
one batch, so every candidate shares it — stated). Totals. `honoured`: count (emitted == n),
topics (each requested topic has ≥1 emitted; n/a in talk mode), difficulty (≥1 emitted, each
tagged d; whether it is at d is AC-88's, at review). Fixture runs: `"numbers": "fixture"` and
the label printed. Nothing is trimmed, wrapped, joined.

## 8. Fixtures (`pipeline/tests/fixtures/generate/`), named for what they prove

`clean.json` (3 candidates, all emitted), `one-errored.json`, `one-expired.json`,
`over-capacity.json` (one 9-line source, one 30-char option, one two-line option),
`invalid-shape.json` (two does_not_compile; a truncated JSON with stop_reason max_tokens),
`talk.json` (talk-mode candidates with concepts), `README.md`: these stand in for model output in
the SDK's documented result shape; none is a recording of a real batch; no answer is claimed;
nothing in them is verified.

## 9. Tests (`pipeline/tests/test_generate.py`) and their mutations

Autouse fixture: `ANTHROPIC_API_KEY` deleted, `socket.socket.connect` and `create_connection`
patched to raise — any network attempt fails the test.
- no-network: CLI without `--replay` and without key refuses before constructing a client;
  `AnthropicBatchClient(api_key=None)` raises before importing anthropic (`"anthropic" not in
  sys.modules` checked in-process). Mutation: drop the key check → test fails.
- brief: each capacity fact (7, single line, 29) and every TROPES/FORBIDDEN source string in the
  built system prompt. Mutation: skip one rule / hardcode 8 lines → fails.
- cap: no cap → refusal sentence, `submit` never called; over cap → refusal; under → submits.
  Mutation: check after submit / `>=` vs `>` boundary.
- AC-3: clean run → n candidates tagged T, d; short run → report names count/topics.
- report-never-trim: over-capacity/invalid/errored/expired each reported with reason, absent
  from emitted, files not written; emitted files validate. Mutation: truncate long option → fails.
- AC-4: talk fixture → every emitted candidate's topic is its named concept, kind std/core.
- AC-5: per-candidate tokens/cost/wall-clock and totals, labelled fixture; cost arithmetic.
- AC-2: fault injection in the atomic writer (raise mid-temp-write, and after temp before
  rename) → no unparsable candidate, no report; re-run same run id completes, `submit` count 0,
  no `.partial` left. Runs in `just test`, hence also in `just test-full` (test-full depends on
  test; the justfile is cleared for one recipe only and `test_runner.py` forbids a subprocess
  in tests, so a real process kill is not available — fault injection at the write is the
  kill model). Deviation flagged.
- shape: no answer field: candidate JSON has no `verified`, `review`, `correct`, `expect`; no
  imports of slot/schedule; `Verified` never constructed (existing test).

## 10. Commits (signed, small)

1. `uv add anthropic` (pyproject + lock). 2. Seam + replay client + real client. 3. Brief +
schema + capacity. 4. Classifier + report. 5. Run dir, atomic writes, resume, CLI. 6. Fixtures +
tests (may interleave with 2–5). 7. README, justfile `generate` recipe, `.env.example` line.

## 11. Contract tensions and the side taken

- Model ID: **amended by RULING 2026-10-10 (client: "use opus5.5")** — default `claude-opus-5-5`,
  one configured value (`DEFAULT_MODEL`). Prices from the claude-api skill: `claude-opus-5-5`
  $4 in / $20 out per MTok, labelled list-price estimates (the table keeps `claude-opus-5` at
  D-D's $5/$25 so `--model claude-opus-5` still estimates). Opus 5.5's default effort is
  `medium`, so `output_config.effort` is set explicitly (`high`); thinking stays
  `{"type": "adaptive"}` (disabling is a 400 on 5.5); no forced `tool_choice` (structured
  outputs instead). BUILDPLAN untouched; the model-ID question is dropped from contract notes.
- `trace` empty; `why_tempting` on all five; topic := concept in talk mode (no §3.1 concept
  field); the source-width fact added to the three; difficulty scale 1–5 assumed;
  generator-reads-rejection-reasons (§3.1 reader) not built here.
- AC-2's kill runs in `test` via fault injection, not a separate test-full recipe.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: general-purpose / sonnet, 2026-10-10.

1. (Critical, kill model) **Accepted with the side taken.** Kill = a `BaseException` (KeyboardInterrupt) injected at three points — mid-temp-write, after temp before rename, and while iterating batch results — then a re-run in the same process. A real process kill is unavailable (test_runner.py forbids subprocess/multiprocessing in tests; justfile cleared for one recipe). Runs in `test`, so in `test-full` too. Deviation listed in DONE; not escalated (delegator decides).
2. (Critical, resume window) **Accepted.** `_run/state.json` is written atomically with `status: submitting` and the request before `submit`; then rewritten with the batch id. A re-run that finds `submitting` with no batch id refuses in one sentence ("a batch may already exist for this run; check the Console before re-running with a new run id") — no silent second submit. A cumulative cross-run ledger is **rejected**: ruling 4 is a per-run cap; H-4's monthly cap is the client's org-level limit.
3. (Critical, cap bypasses) **Accepted.** `--price-in/--price-out` dropped: prices come from the per-model table only; a model not in it refuses. `BATCH_DISCOUNT = 0.5` is a constant. The estimate reads `max_tokens` from the built request params (one source). No `cache_control` is set anywhere, so there is no cache-write premium; a test asserts no request carries `cache_control`.
4. (Major, kind-derived answers) **Accepted.** The shape check rejects more than one option of kind `ub` or `panic` (and the existing exactly-one `does_not_compile`), and rejects `expect: ub` without exactly one `ub` option.
5. (Major, why_tempting on five) **Kept, deviation stated.** A quote-shape lint is rejected for the reason bank.quoted_outputs gives (no syntactic rule separates output from identifiers; a blocking false positive costs a question). Instead the report lists each emitted candidate's `bank.quoted_outputs` spans for AC-73 at review.
6. (Major, ledger lint) **Accepted.** No non-docstring string in generate*.py names the MVP ledger; `just test` runs audit's lints over the new module.
7. (Major, capacity source) **Accepted.** `popquiz.audit.Room` + type model is the room configuration's Python form (bank-audit takes the same flags); stated in the module docstring. The structural test checks generate*.py's direct imports only.
8. (Major, run files beside candidates) **Accepted.** Candidates at `<out>/<run_id>/<id>.json`; state and report under `<out>/<run_id>/_run/` (`state.json`, `report.json`).
9. (Minor) **Accepted:** guard also patches `socket.getaddrinfo`; deletes `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`.
10. (Minor) **Accepted:** PR says `uv.lock` gains anthropic's transitive packages.
11. (Minor) **Accepted:** plain `import anthropic` inside the real client.
12. (Minor) **Accepted:** comment beside `DEFAULT_MODEL` and the price table cites the 2026-10-10 ruling and the claude-api skill (Opus 5.5 default effort is `medium` per the skill, so `high` is set explicitly).
13. (Minor) **Accepted:** `--replay` is the only switch to a fixture client; test: `--replay` with a key set still never builds `AnthropicBatchClient`.

## Reset 2026-10-11 by agent:delegator-pq21

## Plan amendment (option b) — delegator, 2026-10-11

Rulings: ev_01M4NPBK4XWXEMH22W9AEN04DJ (option b, b1–b6) and addendum ev_01M4NPCE4EAVN2SAJJPKYBMN76.
New commits on top of 91875ad; never rebase; origin/main by merge only. Supersedes §11's "trace empty".

**b1 — the model drafts the steps.** `output_schema` gains `trace: {steps: [{lines: [int], focus: [int],
note, values: [{name, was, now}], pivot: bool}]}` (closed objects, all required; counts checked after).
The brief says: at least two steps; `lines`/`focus` are 1-based source lines; the last step is where the
program finishes; never name `stdout` in `values` and never state or quote what the program prints in a
note — a machine adds the output to the last step. `_read_candidate` builds `bank.Trace` and reports
(never trims) as `trace: …` reasons: fewer than two steps; a `lines` entry or `focus` outside
1..source-lines; `focus` not `[a, b]` with a ≤ b; more than one pivot; a value without a name. Module
docstring: the steps are authored content the organizer checks at HC-2, not verified facts.

**b2 — stdout is never the model's.** Every `values` row whose name, stripped and lower-cased, is
`stdout` is discarded from every step, and the discard is *reported* on the candidate
(`discarded: ["step 4: a stdout value"]` in the report row — the value itself is not echoed). The
candidate is still emitted. A `does_not_compile` candidate therefore carries no stdout row either.
**Side taken (deviation, stated in DONE):** the stdout row is created by code *at verification*, with its
value, rather than written by the generator with an empty value. `ValueDelta.now` is a required `str`
in bank.py and `now: String` in room/src/question.rs, so an entry "with no value" would need either a
placeholder string (forbidden by b4) or making `now` optional (a bank + room type change outside the
trace rule). Leaving the row absent until the verifier writes it gives the same guarantee — code, not
the model, owns it — and b4 then needs no bank.py change at all.

**b3 — the verifier fills it.** `verify.with_record` keeps replacing `verified` whole (D-16) and, when the
new record's `receipt_class` is `ran`, the trace is non-empty and `stdout` is observed, sets the last
step's `stdout` row: overwrite `now` on an existing row (re-verify — `was` and every other field and
step untouched), else append `ValueDelta("stdout", "—", …)`. Spelling: `bank.normalized_output(stdout)`,
because that is the spelling `migrate_mvp._trace_for` used for every bank trace and the spelling option
texts use (bank.normalized_output docstring); raw would put a trailing newline into a one-line wall row.
A rejected verdict still writes nothing (`with_record` raises, CLI exits 1 before writing). A
does-not-compile record gets no row. The CLI path already goes through `with_record`, so it is covered.

**b4 — no draft passes as finished.** Unchanged `bank.validate_question` (bank.py:585) already fails a
`ran` record with a non-empty trace whose last step names no stdout; `question_from_dict`,
`save_question` and `check_provenance` all call it. So a candidate given a `verified` by anything other
than `with_record` fails load/save/provenance. No placeholder string is introduced; bank.py's trace rule
is not changed.

**b5 — notes.** `bank.quoted_outputs` also scans every trace step's `note` (one `texts.extend`, plus its
docstring). The generator report's `quoted_spans` therefore covers notes for AC-73 at review.
`test_migration`'s q3 assertion (`== ()`) still holds: no bank note contains a quoted span (checked with rg).

**b6 — fixtures and tests.** New fixture `trace-drafts.json` (4 results): c01 q3's bank source verbatim,
expect ran, a 4-step drafted trace whose last step offers a stdout row holding a model guess; c02 q8's
bank source verbatim, expect does_not_compile, a 2-step trace with no stdout; c03 a one-step trace
(reported); c04 a step pointing past the last source line (reported). Sources are reused so the stub
Runner's recordings (tests/fixtures/verify/recordings, keyed q3/q8) are honest about what ran; the test
replaces only the generated id with the program id. Every existing generate fixture's candidates gain a
valid ≥2-step trace with no stdout so their existing assertions are unchanged.
Tests (test_generate.py unless noted):
- T1 c01 emitted, its trace has no stdout row anywhere, the guess string appears nowhere in the candidate
  file, and the report row says a stdout value was discarded. *Mutation M1: keep stdout rows → caught.*
- T2 c01 → `verify` with `StubRunner` (id→q3) → `with_record`: last step's stdout `now` ==
  `normalized_output(recording run:1 stdout)`; every other step and value identical to the draft;
  record round-trips `question_from_dict`. *M2: with_record skips the fill → caught. M3: raw stdout → caught.*
- T3 c02 → verify (q8, does_not_compile) → no stdout row anywhere; round-trips.
- T4 (test_verify) re-verify q3 whose stdout row `now` was changed: only that value changes; `was`, other
  rows and steps equal. *M4: append a second row instead of overwriting → caught.*
- T5 (test_verify) a rejected verdict leaves the trace untouched (with_record raises; CLI file bytes
  unchanged). (existing test covers the raise; add CLI byte check.)
- T6 (test_bank) a ran record with a drafted trace and no stdout row fails `question_from_dict`/`save`.
- T7 c03/c04 reported with `trace:` reasons, not emitted. *M5: accept one-step traces → caught.*
- T8 (test_bank) `quoted_outputs` returns a span quoted in a note. *M6: drop notes → caught.*
- T9 the brief carries the no-stdout instruction and the schema has `trace`. *M7: drop it → caught.*
Docs: generate.py docstring, fixtures README table row, pipeline/README *Generating* line.
Commits: (1) bank quoted_outputs covers notes; (2) verify fills the stdout row; (3) generator drafts the
trace + fixtures + tests; (4) docs. Then merge origin/main (no rebase), gates, review, push, PR body.

## Amendment plan-review resolutions (AUTHORITATIVE — overrides the amendment on conflict)

Reviewer: general-purpose / sonnet, 2026-10-11. Verdict: sound after Major 1.

1. (Major, stale row on a ran → does-not-compile re-verify) **Accepted.** `with_record`, when the new
   record is not `ran`, removes any `stdout` row from the last step (SPEC 3.2: nothing ran, so nothing
   printed). Test T4b + mutation M8 (fill/keep on a non-ran record → caught).
2. (Major, b4 is name-only) **Accepted as a stated limit.** `validate_question` checks the row exists, not
   its value; `with_record` is the only writer of `verified` in `pipeline/src` (confirmed by reviewer), so
   the value is the verifier's by construction. Said in the verify docstring and in DONE. T6 covers both
   `question_from_dict` and `save_question`.
3. (Minor, lexical filter) **Accepted as a stated limit.** b2 removes rows named `stdout` (strip +
   lower-case); a model row named `output`, or a note stating output in prose, is authored content the
   organizer checks at HC-2 (b1); b5 catches quoted spans in notes. DONE says this plainly.
4. (Minor) c01's offered row is named `" Stdout "` so T1 proves the normalization.
5. (Minor) T2 asserts the row exists and is the only `stdout` row; T4 asserts the row count is unchanged.
6. (Minor) The fixture README names the drafted steps as hand-written, not verified.
