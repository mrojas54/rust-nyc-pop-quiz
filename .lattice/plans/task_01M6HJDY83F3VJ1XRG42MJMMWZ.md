# PQ-84: Move remaining host refusal and lifecycle reasons into copy.rs

PQ-60 (PR #126, merged bdcf951) moved the answer-route and admin reasons, plus answers.rs, state_manager.rs and host/create.rs (client ruling 'A, three files only'), into the room-only block of copy.rs and added popquiz/tests/reason_copy.rs. Left undone: (1) the refusal text built in Affirmation::refusal (popquiz/src/quiz_session/models/affirmation.rs ~:138), shown to the organizer and host; (2) other host phase/lifecycle reasons Codex named in PQ-60 review pass 1; (3) Minor: a sentence held in a same-file const evades the reason_copy lint. Copy is frozen (AC-98): moves are verbatim; any wording change is the client's call. Acceptance: the named strings live in copy.rs, reason_copy covers their files and the const case, planted strays fail it, just test green. List the exact files and get them cleared by the client before editing.

# Plan (delegator, 2026-11-04)

Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-84-lifecycle-reason-copy`, branch `ai-c11-cc/pq-84-lifecycle-reason-copy`, base `origin/main` @ `0ca67a1`. All line numbers are from that commit.

## 1. What is still written outside copy.rs (confirmed)

The orchestrator's floor held, with two corrections: `phase.rs` has **8** sentences, not 7 (the `format!` at :427–431 builds a fourth host refusal), and `affirmation.rs` has **2 more** (`check_shape`, :67 and :70) that reach the organizer.

### A. `popquiz/src/quiz_session/models/affirmation.rs`: 5 strings

| Line | String (verbatim) | Who sees it, by what route | Proposed key |
|---|---|---|---|
| 141 | `its text changed after it was affirmed; affirm it again at review.` | organizer: `Affirmation::refusal` → `answers.rs:684` → `state_manager.rs:248` → admin `reason` (`admin.rs:196`) → `popquiz schedule`'s error. Host: `host/create.rs:126` → 409 `reason` (`routes/mod.rs:58`) → `host.js:233 statusLine` | `LOAD_AFFIRMATION_STALE` |
| 143 | `its affirmation records who and when but not what was affirmed; affirm it again at review.` | same | `LOAD_AFFIRMATION_UNBOUND` |
| 145 | `is not affirmed; affirm it at review, then schedule it again.` | same | `LOAD_AFFIRMATION_NONE` |
| 147 | framing `format!("{id}: {why}")` | same | no new key: becomes `copy::load_refused(id, why)` over the existing `LOAD_REFUSED` (`‹id›: ‹why›`). `fill` is single-pass, so the output is byte-identical. Keeping the `format!` is not possible once the file is covered: its literal `"{id}: {why}"` carries letters and is a `format` sink. |
| 67 | `review.affirmed_digest is not sha256:<64 hex>` | organizer only: `check_shape` → `answers.rs:361 refuse` → `LoadError` → admin `reason` (`admin.rs:179`) | `LOAD_DIGEST_MALFORMED` |
| 70 | `review.affirmed_digest is recorded without affirmed_by and affirmed_at` | same | `LOAD_DIGEST_WITHOUT_WHO` |

### B. `popquiz/src/quiz_session/models/phase.rs`: 8 strings, host only

Route: `apply` → `Refused { reason }` → `RoomError::Refused` (`errors.rs:49`) → 409 `{"reason"}` (`routes/mod.rs:56–58`) → `host.js:400 statusLine`. `canary.rs:222` already pins one: `Reveal isn't next. Start comes next.`

| Line | String (verbatim) | Proposed key |
|---|---|---|
| 388 | `There is no room yet. Create a room comes first.` | `HOST_REFUSED_NO_ROOM_YET` |
| 403 | `This room already exists. Create a room makes a separate one.` | `HOST_REFUSED_ROOM_EXISTS` |
| 426 | `The room is released. Run it again starts a new room.` | `HOST_REFUSED_RELEASED` |
| 428 | `{} isn't next. {} comes next.` (the action's label, then the next action's label, both already `copy::HOST_ACTION_*`) | `HOST_REFUSED_NOT_NEXT` = `‹action› isn't next. ‹next› comes next.` plus `host_refused_not_next(action, next)`, the `LOAD_OPTION_COUNT` pattern. Same output. |
| 436 | `The trace only steps while walking it through or at the answer.` | `HOST_REFUSED_TRACE_STILL` |
| 449 | `That's the first step.` | `HOST_REFUSED_FIRST_STEP` |
| 454 | `That's the last step before the reveal.` | `HOST_REFUSED_LAST_STEP_BEFORE_REVEAL` |
| 455 | `That's the last step.` | `HOST_REFUSED_LAST_STEP` |

### C. `popquiz/src/quiz_session/lifecycle.rs`: 2 strings, host only

Route: `Ended::host_reason` → 409 `{"refusal", "reason"}` (`routes/mod.rs:64`) → `host.js statusLine` (create, command and resume paths).

| Line | String (verbatim) | Proposed key |
|---|---|---|
| 102 | `This room has ended.` | `HOST_ENDED_EXPIRED` |
| 103 | `This room closed for inactivity.` | `HOST_ENDED_INACTIVE` |

**Total: 15 strings in 3 files.** Codex's PQ-60 pass-1 list ("creation, phase, lifecycle, and affirmation sentences") is all here: its creation item was `host/create.rs`, which PQ-60 moved.

`phase.rs:359` and `lifecycle.rs:98` doc comments call these "API diagnostics … not SPEC §11 copy". The host page shows them anyway, so they are the same kind PQ-60's ruling moved. They go in the room-only block (outside `copy::ALL`), and the two doc comments get one line pointing at `copy.rs`. Their meaning stays.

**Swept and left alone** (not route reasons): `club.rs` `ClubsError` Display (startup config error, operator log), `host/discord/*` (URLs, form keys, log lines), `rooms.rs:139` (URL), `affirmation.rs:120–123` / `phase.rs:175–200` (state names and route slugs), tests, `panic!` / `expect`. The pipeline has a Python twin of `affirmation.rs:67` (`pipeline/src/popquiz/bank.py:736`). It is the pipeline's own error, not this ticket's, and is left alone.

## 2. Lint widening (`popquiz/tests/reason_copy.rs`)

- `REASON_SOURCES` += `quiz_session/models/affirmation.rs`, `quiz_session/models/phase.rs`, `quiz_session/lifecycle.rs`.
- **Struct-field shape** (`Refused { reason: "…".into() }`, phase.rs:388): in a source file, the word `reason` followed by a single `:` opens a field span, read as the JSON keys are. `refuse("…".into())` and `refuse(match … { … => "…".into(), _ => format!(…) })` are already covered: `refuse` is in `SOURCE_SINKS`.
- **Match-arm shape** (`Ended::Expired => "…"`, `Affirmation::Stale => "…"`, `Err("…")` in `check_shape`): new `REASON_FNS = ["host_reason", "refusal", "check_shape"]`. In a source file, every letter-bearing literal in the body of `fn <one of these>` fails. `Ended::refusal` and `answers::refusal` hold no literals, so they stay clean. A new test, `every_reason_fn_is_still_a_function_the_scan_can_see`, checks that each name is still declared in the sources, the same way `every_reason_sink_…` does.
- Floors unchanged (`fields ≥ 10`, `sink_calls ≥ 5`, `source_calls ≥ 15`), plus a floor that the scan read ≥ 3 reason-fn bodies.
- Text tests gain the three new shapes: the struct field, the match arm, and `Err("…")` in `check_shape`. They must flag; the moved forms must pass.

## 3. The const case

**Shown today:** I appended `const PQ84_PLANT: &str = "Turn to your neighbour and volunteer one.";` plus `fn pq84_plant() -> Response { refused(StatusCode::CONFLICT, PQ84_PLANT) }` to `server/admin.rs`. `cargo test --test reason_copy` passed **6/6**. Reverted (`git diff` clean).

**Proposed rule (narrow):** in every covered or source file, collect each `const`/`static` whose initializer holds a letter-bearing string literal. A bare word naming one of them, inside any span the scan already reads (a key's value, a sink's arguments, a reason field, a reason-fn body), is a hit reported at the constant's literal. **Flags no existing constant:** the letter-bearing constants in scope (`admin.rs VAR`, `config.rs TAKE_IT_HOME / CLUBS_VAR / DISCORD_VARS`, `pages.rs SNAPSHOT_SLOT`, `wall.rs CSS / JS / assets`, `host.rs` and `pages.rs` `include_str!` pages, `answers.rs BORROW_MODELS`) are used only outside reason spans.

**Rejected alternative:** every letter-bearing `const` in a covered file fails. That would flag the ~11 constants above and need an allowlist that grows with every new MIME type or env var.

**Residual, named, not proposed:** `let why = "…"; refused(s, why)` evades the same way. The ticket names only `const`, so this goes under not-done unless Michelle wants `let` folded into the same rule.

## 4. Twins

`twins.rs` compares `copy::ALL` with `copy.js` (twins.rs:425, :467). The room-only block sits outside `ALL`, as PQ-60's DONE said, so `copy.js`, `twins.rs` and `copy-freeze.test.js` stay unchanged. No page renders a key from this block by name. The host page shows whatever `reason` the server sends.

## 5. After RESUME

The edited files are the cleared ones plus `copy.rs` and `reason_copy.rs`. There are three commits: (1) the moves, (2) the lint widening, (3) the const rule. Then `just test`, with a checkpoint once it is green.

Each mutation is reverted after it is shown to fail:
- a literal at a `Refused { reason }` in phase.rs, and one at a `refuse(...)`;
- a match-arm literal in lifecycle.rs `host_reason`;
- a `format!` sentence in affirmation.rs `refusal`;
- the same-file `const` plant in admin.rs and in phase.rs.

Each of those must fail `no_route_sends_a_sentence_that_is_not_in_the_copy_module`. Editing one moved string by a single character must fail a test that pins verbatim text. Today `canary.rs:222` pins one phase string and `admin.rs:1073` pins the prefix of `LOAD_AFFIRMATION_NONE`; nothing pins the other 13. I would add one test, `room_only_reasons_are_word_for_word`, that asserts each of the 15 constants equals its PQ-84 text.

Codex review runs over `origin/main...HEAD`, two passes at most. The run stops at `review`.
