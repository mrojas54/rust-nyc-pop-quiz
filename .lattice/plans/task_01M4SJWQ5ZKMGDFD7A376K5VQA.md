# PQ-77: ADR-0005 item 9: the room uses only affirmed questions and records who affirmed them

ADR-0005 Action Item 9, under the q3 RULING on PQ-23: an affirmation without a bound digest counts nowhere (no legacy exception). The room serves only affirmed questions and records who affirmed each.

# Plan (delegator, 2026-10-18)

Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/room-affirmed`, branch
`ai-c11-cc/room-affirmed`, base `origin/main` @ `8098174`. Closes GAP-25 (PQ-63).

**The outcome an organizer sees.** `just schedule` keeps refusing a question that
is not affirmed for its current text, as it does today. New: the room refuses it
too, at the push and again when a room is created, and it names the reason. When
the room releases a question, the used record says who affirmed it, and
`just sync` copies that name into the bank and prints it in the ledger report.

**Acceptance check.** A local room accepts the push of an affirmed fixture and
refuses an unaffirmed one, a stale-digest one and a no-digest one, each with its
reason. After release, `GET /admin/used` shows `affirmed_by`, and `sync` writes it
to the bank. `schedule reserve` prints 1 ready both before and after (baseline
taken 2026-10-18: `reserve (nyc): 1 ready`).

## Criteria

- **AC-72** (USER_STORIES:101, EVALUATION:78): an unaffirmed question cannot be
  scheduled; who and when are recorded; enforced, not advisory.
- **G-12** (SPEC:69): every scheduled question has an organizer's affirmation.
  **GAP-25** (PQ-63): the room accepts a pushed question with no affirmation
  (`answers.rs:100` ignores `review`; `admin.rs` admits it).
- **AC-92 / G-10** (SPEC:67, EVALUATION:140): the used record is written at
  release and by nothing else. The new field rides on that one writer.
- **AC-75**: the reserve count stays as it is (no change to the pipeline's rule).
- **AC-95**: affirm's other blockers are unchanged (review.py is not touched).
- **AC-56 / AC-57 / G-4**: nothing about a *participant* outlives the room.
  `affirmed_by` names an organizer, never a participant. See Contract notes.
- **AC-43 / ruling 7**: affirmation is a person's sign-off, not a signature.
- **q3 RULING on PQ-23** (2026-10-12): an affirmation without a digest counts
  nowhere, with no legacy exception.
- The ADR's item 9 section is the scope: *what it buys / costs / how it could fail*.

## What the code does today (read on this worktree)

- The rule: `bank.affirmation_digest` (sha256 over compact, key-sorted JSON of
  `source`, `options`, `hint`, `explains`, `trace`, built with `_dump`, which drops
  `None`, `pivot: false` and `legacy: false`), `affirmation_state`
  (`none | bound | stale | unbound`) and `is_affirmed` (`bound` only).
- `schedule.refusal` already checks affirmation at `just schedule` time with that
  rule (stale, unbound, missing who/when), and `pipeline/tests/test_schedule.py`
  already covers it (lines 226-337). Ruling 3 already holds in the pipeline.
- **The push sends the arranged record.** `schedule.arrange` moves the correct
  option to the date's slot and shuffles the rest. The digest covers `options` in
  the bank's order. So a digest computed over the pushed record is wrong for every
  arranged question, unless the room knows the bank order. Every design below has
  to deal with this.
- The room stores scheduled records in memory only (`AppState.questions`). A record
  cannot change between push and room creation; only a new push replaces it.
- `room/tests/common` builds most test records from `bank/questions/q3.json`
  and edits their text (canary plants, a five-step trace and others). Each edit makes
  q3's digest stale.

## Ruling 2: three designs (the halt)

### (a) A Rust twin of the digest, pinned to Python by shared fixtures. **Recommended.**

- The new file `room/src/models/affirmation.rs` holds a twin of `affirmation_digest`,
  `affirmation_state` and `Review.affirmed()`. It builds the same payload from the
  parsed `Record`, with the same drops as `_dump`, and serializes it with
  `serde_json` (its `Map` is a `BTreeMap` here because `preserve_order` is off, so
  keys come out sorted; the output is compact; the escapes match `json.dumps(...,
  ensure_ascii=False)`: `\" \\ \b \f \n \r \t`, `\u00xx` lower-case for other
  control characters, everything else raw UTF-8).
- **The bank order.** `schedule.push` adds one key to the push body:
  `"bank_order": [i0..i4]`, meaning wall slot *k* holds the option at bank index
  *i_k*. The room checks it is a permutation of 0..4, puts the options back in
  bank order to compute the digest, then drops the field. It is never stored,
  never returned and never logged. When the key is absent (a hand push of a bank
  file: `just demo`, the README's harness ids `smoke-q3` / `burst-q3`), the order
  is taken as the bank's own. The id is not an input to the digest, so q3's record
  under a harness id stays bound.
- Shared fixtures in `bank/fixtures/affirmation/*.json`, one case per file:
  `{record, bank_order?, digest, state}`. `pipeline/tests/test_affirmation_twin.py`
  asserts that `bank.affirmation_digest` and `affirmation_state` give each
  fixture's values, and `room/tests/twins.rs` asserts the room gives the same. Both
  sides also check `bank/questions/*.json` directly (q3 → `bound`, q4/q7/q8 →
  `none`). The fixtures are written once by a small generator test that is skipped
  unless asked for (`POPQUIZ_REGEN_AFFIRMATION=1`), as the receipts fixtures are.
  Cases: q3 as is; escapes (quote, backslash, `\n`, `\t`, U+0001, U+2028, an emoji,
  non-ASCII); `why_tempting` absent vs `""`; `pivot` true/absent; `legacy` present;
  `values` absent; an empty trace; an edited hint (stale); a digest with no
  who/when (refused at load, as `validate_question` does); no digest (unbound);
  empty `affirmed_by` (none); no review (none); a malformed digest (refused at
  load); and a shuffled `bank_order`.
- **SHA-256.** The room has no hash crate of its own. `ring 0.17.14` is already in
  `room/Cargo.lock` and already compiled into the room as rustls's provider. The
  plan promotes it to a direct dependency (`ring = "0.17"`, default features off),
  following the Cargo.toml precedent "already in the lock, no crate added".
  No download is needed. *Fallback, if you don't want the Cargo.toml change:*
  reach SHA-256 through rustls's public hash provider
  (`rustls::crypto::ring::cipher_suite::TLS13_AES_128_GCM_SHA256 … hash_provider`).
  That needs no Cargo change, but it is an odd path to read.
- **Failure modes.** (1) The twins drift when someone changes `_dump` or the digest
  inputs. The shared fixtures fail in both languages, and this is the cost the ADR
  already accepts ("a change to the affirmation digest becomes a room change
  too"). (2) The room is stricter than Python on badly typed input (`pivot: 1`,
  a float line number). The room then refuses something review would accept. That
  only fails closed, and review never writes such values. (3) `bank_order` is wrong
  or missing on an arranged push. The digest misses and the room refuses the push
  as *stale*: fails closed, and the `schedule` tests cover it.
- **The afternoon hint edit.** The organizer edits the hint through `popquiz.review
  edit`, which clears the affirmation (`review._cleared`), so `just schedule`
  refuses: *not affirmed*. They affirm again, schedule again, and the push replaces
  the morning record. A hand-edited bank file pushed by `just demo` is refused by
  the room as *stale*. A refused push never removes the record already scheduled,
  so the morning's affirmed text stays in the room, matching the static
  `<id>.html` written with it. The refusal tells the organizer to affirm again.

### (b) `just schedule` decides; the room checks only what it can see

- `schedule.push` adds `"affirmation": {"state": "bound", "by", "at", "digest"}`,
  computed by the Python rule. The room requires `state == "bound"`, non-empty
  `by` and `at`, and a well-formed digest. It does no hashing.
- **Failure modes.** The room cannot tell a stale digest from a current one: it
  trusts the claim. That does not meet ruling 1's "digest stale" refusal at the
  room. A hand push (`just demo`, harness ids) carries no `affirmation` key, so it
  is refused, unless the bash recipe learns to call Python. Anyone holding the
  admin token can write `"state": "bound"`, but under ruling 7 that is not a
  security property anyway. Simplest change: no Rust hashing and no Cargo change.
- **The afternoon hint edit.** Same as (a) through `just schedule`. A hand-edited
  file pushed by hand is refused only because the key is missing, never because
  its text is stale.

### (c) A digest spec file both implementations read

- `bank/schema/affirmation-digest.json` names the inputs and the canonical form.
  `bank.py` and the room both interpret it.
- **Failure modes.** It is still two implementations of the canonical JSON, plus an
  interpreter in each language. It rewrites `bank.affirmation_digest`, and any
  difference in the rewrite makes q3's stored digest stale: q3 stops scheduling
  (ruling 5) until someone re-affirms it. It adds a format to maintain and gains
  nothing over (a)'s fixtures. The afternoon edit behaves as in (a).

**Recommendation: (a).** It is the only design where the room itself refuses a
stale digest (ruling 1) while keeping one rule: Python stays the reference, and the
room is a twin held to it by shared data, exactly as `correct_index` and the
receipt already are (`room/tests/twins.rs`). `bank.py` and `review.py` do not
change, so q3's digest and the reserve count cannot move.

## Refusal points and reasons (ruling 1)

`answers::load` stays structural. It parses `review` into a typed struct (all eight
`Review` fields, `deny_unknown_fields` like the rest of `Record`), checks the digest's
shape like `validate_question` does, checks `bank_order`, and stores the computed
`Affirmation` in `Scheduled`: `Bound { by }`, or `Stale`, `Unbound`, `None`. The
`vault` and the public half do not change. `Affirmation::refusal(id)` is the one
place the reason strings live. Two callers use it:

1. **At the push.** `AppState::schedule_for` (the one way into `questions`, which
   `admin::put_question` and the tests both use) refuses any state other than
   `Bound`. The admin route answers `409 {"reason"}`, which `schedule.push` already
   relays verbatim.
2. **At room creation.** `AppState::create_for` refuses again, for any state other
   than `Bound`, inside the same critical section.

Honest note: the room keeps nothing on disk, and a stored record never changes,
so the creation check guards the invariant rather than catching drift. It is
cheap, and ruling 1 asks for it.

Reasons. They reach the CLI and the admin JSON only, follow the existing
`"{id}: …"` `LoadError` pattern, and quote no question prose (not `copy.rs`):

| State | Reason |
|---|---|
| none | `{id}: is not affirmed; affirm it at review, then schedule it again.` |
| unbound | `{id}: its affirmation records who and when but not what was affirmed; affirm it again at review.` |
| stale | `{id}: its text changed after it was affirmed; affirm it again at review.` |
| bad `bank_order` | `{id}: bank_order is not an arrangement of the five options.` |
| malformed digest | `{id}: review.affirmed_digest is not sha256:<64 hex>.` |

At creation the host sees what the room returns today for a refusal. No new
participant- or organizer-visible copy is proposed, so nothing in AC-98 needs a
copy ruling.

## Ruling 3: the check at `just schedule` time

This is already in `schedule.refusal`, using the Python rule. The plan adds the
`bank_order` computation (`schedule.bank_order(question, arranged)`, by option text,
which `validate_question` keeps unique) and passes it to `push`. The
`refusal` → `push` order is unchanged. The static `<id>.html` fallback is unchanged.

## Ruling 4: `affirmed_by`, its writer and its reader

- **Room.** `UsedRecord` gains `affirmed_by: String`, and the two compile-time
  field patterns are updated with it. **Writer:** the release transition in
  `AppState::act` (`rooms/state.rs:300`), the used record's one writer (G-10). It
  reads the room's `Scheduled::affirmed_by()`, which exists only for `Bound`.
  **Production reader:** `GET /admin/used` → `schedule.pull_used`.
- **Bank.** `bank.Used` gains `affirmed_by: str | None = None`. It is absent on
  records from before this change (absent is absent, F-30). `question_from_dict`
  reads it. **Writer:** `schedule.merge_used`, through `question_from_dict` as
  today. **Production reader:** `schedule.ledger_report`, whose line becomes
  `used: q3 by nyc on 2026-10-14, room R, affirmed by queenrose54`, with the suffix
  left out when the field is absent. `bank/schema/question.schema.json` gains the
  property, since the `used` item has `additionalProperties: false`.
- `room/tests/used.rs` already parses `bank.Used`'s fields against `UsedRecord`'s,
  so the two cannot drift apart.

## Files: needs clearance (rides on the halt)

1. `room/src/rooms/state.rs`: the `schedule_for` refusal and the used-record writer
   above. There is no way to write `affirmed_by` at release without touching it.
2. `bank/schema/question.schema.json`: one property, `used[].affirmed_by`.
3. `room/Cargo.toml` + `room/Cargo.lock`: `ring` as a direct dependency (design
   (a) only; the fallback avoids it).
4. `justfile`: comment only, if the `demo` recipe's note needs a line. Otherwise
   left alone.

Also touched, all on the cleared list: `room/src/models/{answers,used,affirmation(new),mod}.rs`,
`room/src/server/admin.rs` (the module doc table), `room/src/rooms/create.rs`,
`room/tests/**` (twins, admin, used, common), `pipeline/src/popquiz/schedule.py`
and `bank.py` (only `Used.affirmed_by` and its reader; the rule is untouched),
`pipeline/tests/**`, `bank/fixtures/affirmation/**`, `room/README.md`,
`pipeline/README.md`, `docs/RUNBOOK.md` (one failure row: *the room refused the
push: not affirmed / text changed*), `docs/adr/0005-*.md` (the item 9 tick only).

## Tests and their mutations (ruling 9)

| Check | Test | Mutation it must catch |
|---|---|---|
| unaffirmed push refused | `room/tests/admin.rs`: PUT q3 with `review` removed → 409 *is not affirmed*; nothing scheduled | `schedule_for` accepts any state → test fails |
| stale push refused | admin.rs: q3 with the hint edited → 409 *text changed* | twin ignores the digest comparison (`Stale` treated as `Bound`) → fails |
| unbound refused | admin.rs: q3 with `affirmed_digest` removed → 409 | unbound treated as bound → fails |
| creation refuses | `room/tests/lifecycle.rs` or `one_room.rs`: an unaffirmed `Scheduled` put in through the test seam, create → refused | creation check removed → fails |
| arranged push accepted | admin.rs: q3 arranged with a non-identity `bank_order` → 201; the wrong `bank_order` → 409 stale | `bank_order` ignored → the first fails |
| twin = rule | twins.rs + `test_affirmation_twin.py` over `bank/fixtures/affirmation/` and `bank/questions/` | any change to escapes, drops or the inputs on either side → fails |
| `affirmed_by` written | `room/tests/used.rs`: release q3 → `/admin/used` entry `used.affirmed_by == "queenrose54"` | the writer sets `""` or the field is removed → fails |
| `affirmed_by` synced | `test_schedule.py`: `merge_used` with a room entry carrying it → the bank `Used` holds it and `ledger_report` prints it | `merge_used` drops it → fails |
| schedule-time check | existing `test_ac72_…` / `test_gap24_…`, plus `bank_order` in `test_ac101_the_push_carries…` | `_schedule` skips `refusal` → the existing tests fail (shown in DONE) |

Test fixtures: `common::load` and the helpers that edit q3 (`planted`,
`planted_dnc`, `five_steps` …) re-affirm the edited value through one helper,
`common::affirmed(v)`, which writes the twin's digest. That is circular only for the
twin itself, and the shared fixtures prove the twin. Each mutation runs as a
background job killed at 15 minutes. `just test` stays hermetic and under 60 s
warm; `just test-full` runs once.

## Commit order

1. The shared fixtures plus the Python twin test (green: pins today's rule).
2. Room: `affirmation.rs`, the twin, the typed `review` and `bank_order` in
   `answers::load`, and the twins tests.
3. Room: refuse at `schedule_for` and `create_for`; the test helpers re-affirm;
   the admin tests.
4. Room: `UsedRecord.affirmed_by` at release; used.rs tests.
5. Pipeline: `bank_order` in the push; `Used.affirmed_by` with its schema property
   and ledger line; tests.
6. Docs: room/README, pipeline/README, RUNBOOK row, ADR-0005 item 9 `[x]`.

## Contract notes (to repeat in DONE)

- SPEC §3.1's `used` row lists `{meetup_date, room_id, released_at, fit}`. Ruling 4
  adds `affirmed_by`. SPEC is not mine to edit, so this is routed upstream.
- `used.rs` says the used record "carries nothing about a person (AC-56, AC-57)".
  Both criteria are about participants. The doc becomes "nothing about a
  participant", and `affirmed_by` names the organizer who signed off, as
  `review.affirmed_by` already does in the bank.
- EVALUATION:185 calls the MVP bank "already affirmed by use". The PQ-23 ruling
  supersedes that, and this change follows the ruling.

## Deferred (not this ticket)

- `just demo` pushes the bank file in stored order. That still works (`bank_order`
  absent means identity), but its comment could say the room now checks
  affirmation.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Review: Sonnet subagent, 2026-10-18, 7 findings (2 Critical, 4 Major, 1 Minor).

1. **C1, the digest twin and empty lists. Accepted.** `_dump` drops only `None`, so
   Python always hashes `"values":[]` for each step and `{"steps":[]}` for an empty
   trace, whether the key was absent or `[]` in the file. The twin always emits
   both. The fixtures gain a pair (`values` absent / `values: []`, `trace` absent /
   `{"steps": []}`), and both languages assert that each pair gives **one** digest.
   This replaces the fixture line "`values` absent" above.
2. **C2, the `affirmed_by` writer. Rejected, with the evidence.** `Room` holds the
   record itself: `question: Arc<Scheduled>` (`room/src/rooms/room.rs:77`), which is
   kept from creation to release, so a later replacement in `questions` cannot
   change it. The writer at `state.rs:300` reads `room.question.affirmed_by()`.
   No extra field is needed on `Room`.
3. **M1, `just demo`. Accepted in part.** The room will now refuse `just demo q4`,
   `q7` and `q8` (all `none`). That is the point of the change, not a regression.
   The recipe prints only `PUT answered 409`. With `justfile` clearance (item 4 of
   *needs clearance*), it also prints the room's `reason` from the body. Without
   clearance, it stays as it is and goes in DONE as deferred. q3 still demos
   (bound, stored order, no `bank_order`).
4. **M2, tests that edit q3. Accepted; this was already the plan, now with the
   call sites named.** `common::affirmed(v)` re-affirms each edited fixture.
   Sites: `room/tests/common/mod.rs` (`load`, `planted`, `planted_dnc`,
   `five_steps` and the other editors), `clubs.rs:412,451`,
   `common/discord_mock.rs:474`, `twins.rs:290`, and any others the first
   `just test` run names. The circularity is limited to the twin, and the shared
   fixtures prove the twin against Python. `twins.rs:290`'s refusal cases test
   `load`, which stays structural, so they need no affirmation.
5. **M3, smoke and burst. Checked: no change needed.** `cli/smoke.rs:77`
   `include_str!`s `bank/questions/q3.json` unedited. It reads secrets from it and
   does not edit its text, so q3's record under `smoke-q3` / `burst-q3` stays
   bound.
6. **M4, `bank_order` against `deny_unknown_fields`. Accepted.** `Record` keeps
   mirroring the bank schema and gains no `bank_order` field. `admin::put_question`
   parses the body as a JSON object, removes `bank_order` (if present), and calls
   `answers::load_arranged(record_json, bank_order: Option<[usize; 5]>)`.
   `answers::load(json)` remains as `load_arranged(json, None)` for every existing
   caller and test.
7. **m1, `ring`. Already flagged.** It needs the client's word (item 3). The
   fallback is rustls's hash provider, with no Cargo change.

The design recommended is unchanged: **(a)**.

## Client rulings (BINDING, 2026-10-18 14:2xZ, via the Orchestrator)

- Design **(a)**: the Rust twin of the digest, pinned to Python by shared fixtures; the push carries `bank_order`.
- SHA-256 via **ring** as a direct dependency (`ring = "0.17"`, default features off, already in Cargo.lock; no download).
- Clearance granted: `room/src/rooms/state.rs`; `bank/schema/question.schema.json` (`used[].affirmed_by`);
  `room/Cargo.toml` + `Cargo.lock` (ring only); `justfile` (`demo` prints the room's reason).
- No new copy.

## Reset 2026-10-18 by agent:delegator-pq77
