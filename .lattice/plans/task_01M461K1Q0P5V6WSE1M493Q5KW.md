# PQ-60: Room-authored reason strings sit outside the copy module and are unlinted

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-14** (G-5, Minor, new): Room-authored reason strings sit outside the copy module and are unlinted
- Evidence: routes.rs:196,208; M5b3 survived
- What would close it: Move them into copy.rs/copy.js or lint room/src string literals

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-03)

Base `origin/main` @ 1fbc6bd. Re-verified: `JoinRefusal::message()` already returns `copy::JOIN_FAIL_*`; `routes/mod.rs` denials already use `copy::HOST_DENIED_*`.

**Strings to move, verbatim, into `popquiz/src/copy.rs`:**
- `connections.rs` answer route: `"An answer is one of A, B, C, D or E."` (400) and `"Answers open when the question is on the screen."` (409, Idle).
- `admin.rs` reasons (they reach a person: `pipeline/.../schedule.py::_reason` puts them in the `ScheduleError` the organizer reads): not a club name; larger than a record; not UTF-8 JSON; not a JSON object; `{id}: bank_order is not a list…`; `The path names {id}; the record is {}.` The two `format!` ones become ‹placeholder› templates filled by small `copy.rs` functions, so the call sites carry no literal and the filled text is byte-identical.

**Twins (decision):** these go in a separate *room-only* block in `copy.rs`, outside `copy::ALL`. `twins.rs` holds `ALL` to `copy.js`; adding them to `copy.js` would need a `COPY_ROWS` row, and `web/test/copy.test.js` refuses any key in no §11 row ("author them in SPEC.md §11 first") — that would be authoring copy, and the PROPOSED block is one the room may not send. So no `copy.js` change; logged as a deviation.

**Lint:** new `popquiz/tests/reason_copy.rs`. Tokenizes each `.rs` under `src/server/` and `src/connections/` (comments, strings, char literals, lifetimes), then fails on a string literal containing a letter (a) in the value of a `"reason"`/`"message"`/`"refusal"` JSON key, up to the next depth-0 `,` or close bracket, and (b) in the arguments of a call to admin's reason helper `refused(`. Narrowest-rule choice: a `refusal` value that is a bare slug (`^[a-z][a-z_]*$`) passes — slugs are machine-read, and `routes/mod.rs:70,75` (not cleared) send `"wrong_server"`/`"wrong_role"`. In-file cases pin the scanner to M5b3 and the two plants.

**Mutations (bounded, reverted):** M5b3 (the sentence as a literal in the answer route's reason); a planted `"reason"` literal in a `connections.rs` route; one in an `admin.rs` response. Each must fail `reason_copy`.

**Out of scope, noted for DONE:** `RoomError::Refused` reasons from `rooms/create.rs`/`rooms/state.rs` and `answers::LoadError` text reach `reason` through variables; they live outside the cleared files and the ruling's lint scope.

Checks: `just test` green; the three mutations red; Codex review ≤2 passes.

## Reset 2026-11-03 by agent:delegator-pq60

## Reset 2026-11-03 by agent:delegator-pq60
