# PQ-85: Reason-copy lint follow-ups from the PQ-84 review

Follow-ups the PQ-84 delegator and its Codex review left not-done at PR #130 (merged 2026-11-05 at 7dda474). Held until the client releases it. Not-done: 1. A let-bound format! in a reason path still evades the reason_copy lint (popquiz/tests/reason_copy.rs); the const rule landed in #130, let bindings did not. 2. The Python twin of the moved host/organizer copy at pipeline/src/popquiz/bank.py:736 was not touched. 3. Pre-existing cargo clippy --all-targets -D warnings failures in popquiz (wiring.rs:234/285, src/cli/burst_report.rs:291/298, tests/burst_report.rs:142/173, canary.rs:741); neither CI nor just runs clippy on popquiz. Five Minors Codex deferred, listed in the #130 body: an attributed reason: field escapes the scan; a closure parameter named reason is read as a field; unqualified const-name collisions can false-positive; REASON_FNS pinned by name, not by file; a let-bound format! in phase.rs escapes. Scope when released: close the lint gaps with planted RED cases first (AC-98, copy stays verbatim), decide whether clippy joins just test or stays out, touch the Python twin only if its copy diverged. Source artifacts: art_01M6MYX25HFEEXN0Q7DNE9G2NX (Codex), art_01M6MYX29WBMPPGCHEB7Q8HSFP (Validation).

# Plan (delegator, 2026-11-05)

Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-85-reason-lint-followups`, branch `ai-c11-cc/pq-85-reason-lint-followups` from `origin/main` @ `7dda474`. Only `popquiz/tests/reason_copy.rs` changes for gaps 1–5; covered source files change only inside reverted plants. No sentence changes (AC-98).

Method per gap: plant the shape in the real covered file, run `cargo test --offline --locked --test reason_copy` as a background job killed at 15 minutes, record the result on the old lint, add the rule, rerun, revert the plant, rerun green. Each gap also gets a permanent text case in `reason_copy.rs`, like the existing M5b3 cases, so the rule's reach is pinned without a reverted edit.

## Gap 1: an attributed `reason:` field escapes

- Plant (phase.rs:386–390): `Err(Refused { from: None, command, #[allow(unused)] reason: "There is no room yet.".into() })`.
- Today: the struct-field rule needs `{` or `,` right before `reason`; here it is `]`, so the lint passes the plant.
- Rule: before checking the preceding token, step back over any `#[ … ]` outer attributes.
- Expected after: `no_route_sends_a_sentence_that_is_not_in_the_copy_module` fails naming `quiz_session/models/phase.rs … reason:`.

## Gap 2: a closure parameter named `reason` is read as a field (false positive)

- Plant (phase.rs, beside `let refuse`): `let _note = |from: Phase, reason: &str| format!("{from:?}: {reason}");`.
- Today: `reason` follows `,`, so it reads as a struct field and its span runs into the closure body, flagging `"{from:?}: {reason}"`.
- Rule: a `reason:` is a struct field only when its enclosing bracket, walking back at depth 0, is `{` and an even number of `|` lie between that `{` and the field (so `|a, reason: T|` is a parameter list; `{ cb: |a| a, reason: … }` stays a field).
- Expected after: the plant passes.

## Gap 3: unqualified const-name match (false positive)

- Plant (lifecycle.rs, module level): `#[allow(dead_code)] const HOST_ENDED_EXPIRED: &str = "expired";`. `host_reason` sends `copy::HOST_ENDED_EXPIRED`.
- Today: the name in `copy::HOST_ENDED_EXPIRED` resolves to the local constant, so `"expired"` is flagged.
- Rule: a name resolves to a same-file `const`/`static` only when unqualified or qualified by `self::`/`Self::`; a name after any other `path::` is another module's item.
- Expected after: the plant passes.

## Gap 4: `REASON_FNS` pinned by name, not by file

- Plant: rename `Affirmation::refusal` (affirmation.rs:140) to `refusal_for`, and its one caller (answers.rs:684).
- Today: `every_reason_fn_is_still_a_function_the_scan_can_see` is satisfied by `Ended::refusal` (lifecycle.rs:92) and `PublicQuestion::refusal` (answers.rs:682); the renamed body leaves the scan silently.
- Rule: `REASON_FNS` becomes (file, name) pairs, and a body is read only in its own file: `lifecycle.rs host_reason`, `affirmation.rs refusal`, `affirmation.rs check_shape`, `answers.rs refusal`. The pin test checks each pair in its file. `Ended::refusal` returns a `JoinRefusal` enum, not a sentence, so it is no longer read (decision, recorded in DONE); the floor becomes "every pinned body was read".
- Expected after: the pin test fails with `fn refusal( is gone from quiz_session/models/affirmation.rs`.

## Gap 5: a `let`-bound `format!` escapes

- Plant (phase.rs:429–431): `let why = format!("{phase:?} does not step the trace."); return Err(refuse(why));`.
- Today: phase.rs is not a `FORMAT_SOURCES` file, and `why` is a variable, so `refuse(why)` reads no literal; the lint passes.
- Rule: like PQ-84's constants, a `let` binding whose initializer holds a letter-bearing literal is read as that literal where a span reads its name, within the binding's block and after it (nearest preceding binding wins, so shadowing by a clean `let` clears it). Applies to every scanned file, as the const rule does.
- Expected after: the no-route test fails naming `"{phase:?} does not step the trace."`.
- Check: the lint must still pass on `main`'s sources; any binding it newly flags there is investigated, not allow-listed.

## Item 6: the Python twin

No divergence. `bank.py:736` raises `f"{question.id}: review.affirmed_digest is not sha256:<64 hex>"`; Rust's `copy::LOAD_DIGEST_MALFORMED` is `"review.affirmed_digest is not sha256:<64 hex>"`, sent through `LOAD_REFUSED` (`"‹id›: ‹why›"`) by `answers.rs:360`. `bank.py:739–740` is `"review.affirmed_digest is recorded without " "affirmed_by and affirmed_at"`, equal to `copy::LOAD_DIGEST_WITHOUT_WHO`. The other room-only sentences (the host refusals, `HOST_ENDED_*`, `LOAD_AFFIRMATION_*`) have no Python twin. `bank.py` is not touched.

## Item 7: clippy (reserved for the client)

Re-verified at `7dda474` with `cargo clippy --offline --locked --all-targets --keep-going -- -D warnings`: **11 failures**, not 7. The earlier list came from a run that stops at the first failing targets; `--keep-going` shows them all.

- Named in the ticket (7): `tests/wiring.rs:234`, `:285` (`drop` of a non-`Drop` value); `src/cli/burst_report.rs:291`, `:298` (negated `<`/`>` on a float); `tests/burst_report.rs:142`, `:173` (very complex type); `tests/canary.rs:741` (manual checked division).
- Not named (4): `tests/room_record.rs:69` (byte string), `:259` (`drop` of a non-`Drop` value); `tests/twins.rs:571` (`.err().expect()`); `tests/admin.rs:928` (redundant `&` in `format!`).

All 11 look fixable without a behaviour change, a string change, a new dependency or an `#[allow]` on production code. The 4 extra sites are in test files outside the cleared list, so A or B needs those three files cleared as well.

Warm `just test` on this machine: 28.4 s and 29.0 s before. With clippy added to `test-room` (a scratch copy of the justfile, failures ignored so the timing covers a full clippy pass): 31.2 s warm, and 64.2 s on the first run while clippy's cache was cold. The 60 s budget holds warm, so the recommendation is **A**.

## Commits

One per gap or shared rule (1 and 2 share the field predicate), `popquiz/README.md` only if it documents the lint (it does not), then clippy per the ruling.

## Reset 2026-11-05 by agent:delegator-pq85

## Reset 2026-11-05 by agent:delegator-pq85

## Reset 2026-11-06 by agent:delegator-pq85

## Reset 2026-11-06 by agent:delegator-pq85
