# Guardrail audit, October 2026

An adversarial read of the assembled tree against SPEC §2's twelve guardrails and
SPEC §4's phase invariants. Each guardrail's named enforcement was attacked: a small,
plausible change that breaks the guardrail was made, the test that should catch it was
run, and the change was reverted. A mechanism whose mutation survived is a gap. A
verdict of HOLDS means its mutations were tried and caught.

This audit fixes nothing. Gaps are listed at the end with IDs; the Orchestrator files
the new ones as held tickets.

## Header

| | |
|---|---|
| Audit tree | `origin/main` @ `0c7d60a6fea0a28ba9c9494c9105313167984b33` (merge of PR #48) |
| Date | 2026-10-05 |
| Ticket | Lattice PQ-28 · BUILDPLAN T-23 |
| Method | Four read-only auditors (answer integrity, secrecy, phase and receipt, copy and auth) proposed; every mutation below was run by the delegator, one at a time, reverted with `git restore`, and the tree checked clean (`git diff --quiet && git status --short`) before the next. |

Commands run on the untouched tree. The sandbox was off for each: inside it, uv cannot
open its cache and the Discord mock cannot bind a local port, and `just test` fails
with `Operation not permitted` for those reasons alone.

| Command | Exit | Result |
|---|---|---|
| `just setup` | 0 | venv created, crates fetched |
| `just test` | 0 | room 236 passed, 4 ignored; pipeline 710 passed, 1 warning; web 286 passed, 0 failed. 17.7 s warm |
| `just canary` | 0 | 12 passed |
| `just secret-scan` | 0 | 2 passed |
| `just bank-audit` | 0 | `bank-audit: passed`, with the known `WARN tell: option position [AC-26]: worst rule 'index E': 4 hit(s) vs 0.80 by chance = 5.00x over 4 question(s), tail p=0.0016` |

Not run: `just test-full` (needs Docker and the sandbox image), `just smoke` and
`just burst` against the deployed room. See *What I could not verify*.

## Summary

49 mutations tried: 34 caught, 15 survived. One probe confirmed a leak (G-3). No
Critical. Two planned mutations were not run (see the mutation table).

| Guardrail | Verdict | Critical | Major | Minor | NIT |
|---|---|---|---|---|---|
| G-1 answer position from the date | HOLDS-WITH-GAPS | 0 | 2 | 1 | 1 |
| G-2 no hand-written answer | HOLDS-WITH-GAPS | 0 | 1 | 3 | 0 |
| G-3 nothing pre-reveal joins options to output | HOLDS-WITH-GAPS | 0 | 2 | 0 | 0 |
| G-4 nothing per-person kept | HOLDS | 0 | 0 | 0 | 0 |
| G-5 no obligation to speak | HOLDS-WITH-GAPS | 0 | 2 | 2 | 0 |
| G-6 phase order | HOLDS | 0 | 0 | 0 | 0 |
| G-7 the receipt claims what was proved | HOLDS-WITH-GAPS | 0 | 1 | 0 | 0 |
| G-8 no source or trace on a phone | HOLDS | 0 | 0 | 0 | 0 |
| G-9 every credential path enumerated | HOLDS-WITH-GAPS | 0 | 0 | 2 | 1 |
| G-10 used at release, not at build | HOLDS-WITH-GAPS | 0 | 2 | 1 | 0 |
| G-11 no published distribution | HOLDS-WITH-GAPS | 0 | 0 | 2 | 0 |
| G-12 every scheduled question affirmed | UNENFORCED | 0 | 1 | 1 | 0 |
| Phase invariants (§4) | HOLDS-WITH-GAPS | 0 | 0 | 1 | 0 |
| Cross-cutting | — | 0 | 0 | 1 | 0 |
| **Total (27 gaps)** | | **0** | **11** | **14** | **2** |

A gap that spans two guardrails is counted once, under the guardrail named first in
its row of the gap list.

---

## G-1 — the answer's position is a pure function of the date

> The correct-answer position is a pure function of the date. No history, ledger, bank
> state, or prior position is an input. Balancing, quotas, and *never the same as last
> time* are prohibited. (AC-23, 23a, 23b)

**Enforced by, checked.**

- `slot_for_day(date, n_options=5)`: `pipeline/src/popquiz/slot.py:55-76`. Pure, takes a
  `datetime.date` and the literal 5. M1a (a `history=None` parameter) was caught by
  `bank-audit`'s `slot path and ledger` check (`FAIL … parameter 'history' reaches for
  history`) and by `test_audit.py`'s anchor on the signature. M1b (round-robin:
  `day.toordinal() % 5`, perfectly even and perfectly predictable) was caught by
  `test_slot.py::test_slot_for_day_draws_exactly_what_the_mvp_drew`, and by
  `bank-audit` three times over: `generator … TOO EVEN (chi2=0.000)`, the attendee
  simulation (`least used so far 100.0%`), and the slot lint.
- The static lint: `audit.slot_path_violations` (`audit.py:357`), `slot_call_violations`
  (`:501`) and `ledger_violations` (`:537-581`). They read `slot.py`, every
  `slot_for_day` call site, and every `pipeline/src/popquiz/*.py` for the ledger's
  name, an import of `mvp`, and a function that draws a slot and writes a file.
- `bank-audit` both tails: `audit_generator` and `simulate_attendees`, run in `just test`
  through `test_audit.py::test_the_real_bank_passes_every_bank_level_check`.
- The one arrangement, `schedule.arrange` (`schedule.py:189-208`, added by PR #47):
  M1d (answer kept at E) was caught by four `test_schedule.py::test_ac23_*` tests.

**The attack that survived (M1c).** `arrange` is where tonight's letter is actually
placed, and no lint or simulation reads it. Adding *never the same letter as last
meetup* inside it (read the bank's latest `used.meetup_date`, recompute that night's
slot, bump tonight's by one if they match) passed the whole pipeline suite. The lints look
for the ledger's file name, a slot parameter, and a draw-and-write function; this
needs none of them. The simulation runs `slot_for_day`, not `arrange`. `schedule.py:166`
also carries its own copy of `_rng`, so a draw written against it is invisible to
`slot_call_violations`. This is the exact shape of the fourth regression (PHILOSOPHY §2).
GAP-1.

**The standalone fallback builder (M12d; the Orchestrator's lead on PQ-28, re-run
here).**
- What it does: `python -m popquiz.fallback <id>` (`fallback.py:306-333`) loads the
  record and calls `write_fallback` with no date and no `arrange`. `fallback.py:27-29`
  renders options in the order given, and leaves arranging to the caller.
- What it produced: all four bank records store the answer at index 4.
  - q3: `python -m popquiz.fallback q3 --bake` exited 0 and baked
    E = `[1, 2, 3, 2, 1]`, q3's answer.
  - q4, q7, q8: exit 1 today, because their traces are empty. Each bakes at E once it
    has a trace.
- Who documents it: only `popquiz schedule` (`schedule.py:529-533`) arranges by date
  before it builds. The runbook sends organizers to `just schedule`.
  `room/README.md:595-596` documents the standalone form as a `test-full` step.
- What tests it: `test_fallback.py:288` asserts the standalone command succeeds on q3,
  so the suite pins the behaviour rather than catching it.
- Why it matters: the fallback is what goes on the projector when the room cannot run.
  A file built this way shows the answer at E on every such night. That is the third
  regression in PHILOSOPHY §2 (the same slot every meetup) by another door.
- Severity, Major (the client confirmed this on 2026-10-05):
  - One night built this way discloses nothing on its own. An attendee learns from it
    only after repeated E nights, all built through the standalone path.
  - It still needs fixing soon. `just schedule` refuses every committed question today,
    because none is affirmed (`docs/RUNBOOK.md:54-65`). That leaves the standalone
    builder as the only command that produces a fallback at all.
- New: PQ-48 is about what the AC-26 tell measures in the stored order, and PQ-45 put
  the arrangement in `schedule` only. Neither covers this entry point.

GAP-2.

**Audit demand.**
- *Every PR touching the slot path re-runs the perfect-memory simulation*: met for
  `slot.py`. `ci.yml` runs `just test` on every pull request with no path filter, and
  the simulation is in it. Not met for `arrange` (GAP-1).
- *Audit `build_deck.py` on inheritance*: `slot_for_day` (`build_deck.py:162`) is pure
  and ported verbatim (the parity test proves it over two years). `answer_slots`
  (`:96-109`) balances answer positions *by construction* for the multi-question review
  deck. The segment's `--only` path does not use it. GAP-3. No test covers
  `mvp/tools/`.

**History readers (PHILOSOPHY §2).** Nothing on the tree reads history to choose a
letter today. `refusal` and `reserve_lines` read `used` to refuse and to count;
`merge_used` reads it to refuse duplicates; `build_deck.py:252-258` reads the ledger to
print. The unevenness of the four stored answers (all at E) is not a gap; what the
option-position tell measures is (GAP-26).

**Verdict: HOLDS-WITH-GAPS.** The slot function is held by a parity test, three lints,
both tails of the generator audit and the attendee simulation. The arrangement around
it is held by none of them.

## G-2 — no hand-written answer reaches a deck or a room

> The correct option is derived only from the verifier's recorded output; a provenance
> field names the verifier run; the build refuses a question whose answer field lacks
> it. (AC-7)

**Enforced by, checked.**

- Derived, never stored: `bank.correct_index` (`bank.py:389`) and the room's twin
  `Record::correct_index` (`room/src/answers.rs:224-269`). `question_from_dict` refuses
  unknown fields. M2b (`"correct": 1` added) was refused at load (`unknown field(s)
  ['correct']`, exit 1). M2d (the room's derivation fixed at index 0) was caught by
  eight room tests, including the canary.
- Provenance field: `verifier_version` (a digest of the verifier's code,
  `verify.py:255-261`) and `verified_at`.
- *The build refuses a question whose answer field lacks it*: `verify.check_provenance`
  (`verify.py:577-628`) refuses it, and
  `test_verify.py:425-428` (`test_every_bank_question_passes_the_provenance_check`) runs
  it over every committed bank file in CI. So a committed record without provenance
  fails CI. The build path itself never calls it: `schedule.refusal` and `fallback.bake`
  do not. M2a (a non-legacy record with no `verifier_version` or `verified_at`,
  affirmed by hand in a copied bank) scheduled with exit 0 and wrote its fallback. The
  gap is a bank the organizer schedules from without committing it first. GAP-4.
- M2f (`verified` deleted) was refused by `schedule.refusal`.

**Hand edits inside the record (M2c).** `verified.stdout` was changed so the answer moved
from E to A, in a copied bank. `check_provenance` accepted it and `popquiz schedule`
arranged option A as correct. The same edit made in the tree was caught by
`test_migration.py::test_the_correct_option_is_the_machines_output`, which replays each
question's recording, for q3 and for q4. That replay covers the fixed tuple
`MIGRATED = ("q3", "q4", "q7", "q8")` (`migrate_mvp.py:76`). A fifth question, or an
uncommitted edit on the organizer's laptop, has no replay and would schedule. The
contract has no field that binds a record to its run (CONTRACT NOTE 1). GAP-4, GAP-6.

Why Major and not Critical: every committed edit to the bank's four questions fails CI.
The path to the room needs an edit that never reaches CI.

**Stale pin.** SPEC §7.2 says scheduling refuses a non-legacy record whose `rustc` does
not match the configured pin. `verify.is_stale` (`verify.py:507`) has no caller. GAP-5.

**Audit demand.** *Test that a hand-edited `correct` fails*: met (`test_bank.py`,
`test_verify.py::test_a_stored_correct_field_is_refused`, and M2b). *Audit `verify.py`
and `build_deck.py`*: `mvp/tools/verify.py:166-216` writes `answer` and a bare
`verified: true`; `build_deck.py:241,266` trusts both, with no provenance and no test.
GAP-7.

**Verdict: HOLDS-WITH-GAPS.**

## G-3 — nothing pre-reveal joins the options to the output

> Nothing pre-reveal carries … which option is correct … nor the explanation, the
> receipt, or the trace's resolving step … (AC-47, AC-60, AC-61, AC-79, AC-97; D-10)

**Enforced by, checked.**

- The sealed vault: `answers::vault` (`answers.rs:456-550`), opened only with a
  `RevealWitness` that `Machine::revealed()` returns in `reveal` alone (`phase.rs:302`).
  Pre-reveal builders take a `PublicView` that does not hold the vault
  (`boundary.rs::the_pre_reveal_builders_never_see_the_opened_room`).
- The canary: `canary.rs::every_phase_every_surface_the_answer_stays_sealed_until_reveal`
  and its does-not-compile twin. Caught:
  - M3a, the `stdout` row filter removed from the walk-through (`answers.rs:655`).
  - M3c, the `work` bound let through to M-1 (`phase.rs:258`).
  - M3d, pre-reveal options reversed (`view.rs:305`), which reported *the correct
    option's text is somewhere other than option E's slot*.
- The route table: `canary.rs::every_route_in_routes_rs_is_scanned_or_listed` caught
  M9a (a new route in `routes.rs`).

**The witness can be forged (M3e, M3e2).** Adding `impl RevealWitness { pub fn forge() ->
Self { Self { _machine: PhantomData } } }` to `phase.rs`, or `#[derive(Default)]` on
the struct, passed `boundary.rs` and the lib tests. The boundary test looks for the
literal text `RevealWitness {` and checks a fixed list of sealed types for derives;
`RevealWitness` is not on it. The compile-fail doctest still passes because the
field stays private. The vault's own surface is held: adding a method to it (M3e3)
was caught by `boundary::the_vault_holds_only_the_allowed_functions_and_every_read_takes_the_witness`.
The key that opens it is held by a text search alone. GAP-9.

**Two rooms on one question leak through `/last` (probe).** `create_for`
(`rooms.rs:846-886`) refuses a question already *used*, but not one held by another
live room. At release, the take-it-home snapshot goes into one global slot
(`rooms.rs:925`), served without authentication at `/last` (`routes.rs:639`). A
throwaway test made rooms A and B on q3, put B live, and released A. `/last` then
served q3's options with `"correct": true` on E while B's buzzer was still in `live`
with its hint showing. Reverted.

Severity, Major: once it happens an attendee does learn the answer, but getting there
takes a setup the documented procedure rules out. To release room A, its host has to
take A through all six host actions, A's own reveal included, while B sits before
reveal. The runbook keeps rehearsal questions away from the night's question
(`docs/RUNBOOK.md:43-44`), and it sends smoke runs to a harness id
(`docs/RUNBOOK.md:77-80`). An idle duplicate room leaks nothing until someone walks it
to release. What is missing is a refusal in code, with a test, for something only the
runbook prevents. GAP-8.

**Reachability.** Every route a phone can reach, from `routes.rs`, `admin.rs` and
`ws.rs`:

| Surface | Auth | Carries pre-reveal | Canary |
|---|---|---|---|
| `POST /join` | none | own token, buzzer view | yes |
| `PUT /rooms/{id}/answer` | session token | own letter only | yes |
| `GET /rooms/{id}/buzzer`, `WS /rooms/{id}/ws/buzzer` | none / token | letters, hint (live), split | yes |
| `GET /rooms/{id}/wall`, `WS /rooms/{id}/ws/wall` | none | source, options, split; trace 0..M-2 in work | yes |
| `GET /rooms/{id}/host`, `WS …/ws/host`, host actions | host bearer | step notes in work | yes |
| `PUT /rooms/{id}/fit` | none | nothing (204) | status only |
| `/last` | none | the last *released* question | yes (and GAP-8) |
| `/wall/*`, `/host*`, `/join*`, `/home/*`, `/shared/{file}`, `/{code}` | none | compiled-in bytes; `/shared` is an exact-name list, no disk read | yes |
| `/auth/discord*` | none | redirect | listed unscanned, covered by `auth.rs` |
| `/admin*` | admin token | 401, empty body | yes |

WebSocket inbound is the attach message only (`ws.rs:395-403`). No route serves the
static fallback or host sheet (`no_fallback_route.rs`).

**Audit demand.** *`canary` in `test`*: met. *A ticket that proves AC-61 structurally*:
met by the vault's types, defeated by GAP-9.

**Verdict: HOLDS-WITH-GAPS.**

## G-4 — nothing per-person is stored beyond the room

> Nothing per-person is stored beyond the room, and nothing per-person leaves the phone
> after close. (AC-56, AC-57, AC-58)

**Enforced by, checked.** Totals carry an expiry: they live on the room record and go
with it at the 4 h expiry (`lifecycle.rs:115-117`, `rooms.rs:352`), read here, not
mutated. No per-participant answer table: sessions live in
`SessionMap` and `release` empties it (`rooms.rs:915`). M4b (that call removed) was
caught by `lifecycle.rs::ac56_nothing_per_person_survives_release`. `UsedRecord`
(`used.rs:34-44`) holds `meetup_date`, `room_id`, `released_at`, `fit` and no count,
pinned by exact-JSON and key-equality tests and by a compile-time pattern over its
fields (`used.rs:56`), which stopped M4a (a `totals` field) at compile time. The post-close traffic canary
(`canary_scan/mod.rs:1614-1637`) fails a fetch ending `/answer`, a body containing the
letter, or any socket send but attach.

**Audit demand.** *Schema audit*: met. *`canary` on post-close traffic*: met.

**Verdict: HOLDS.** The auditor noted the post-close check is a denylist for fetch (a
new path with the letter under another key would pass); this was not shown by a
mutation and is not filed.

## G-5 — no participant-facing string obliges anyone to speak

> No participant-facing string obliges anyone to speak or interact. (AC-98)

**Enforced by, checked.**

- The copy module (`web/shared/copy.js`) is linted against SPEC §11's Forbidden row in
  `copylint.test.js`. M5a (*Turn to your neighbour.* in `buzzer_idle`) was caught by
  `no copy module entry matches the Forbidden row`. M5a2 (the same with two spaces) was
  caught by the AC-98 test and the freeze, though not by the Forbidden-row lint.
- `room/src/copy.rs` is not linted. It is pinned equal to `copy.js`
  (`twins.rs::the_copy_module_mirrors_copy_js_in_both_directions`), which caught M5b.
- Literals outside the copy module: `copy-freeze.test.js` caught a sentence added to
  the buzzer page shell (M11c).

**Question prose is not enforced (M5d2).** *Turn to your neighbour and argue it out.*
added to q4's `explains.what` passed all of `just test`. SPEC §11 puts `explains`,
`why_tempting` and `hint` under the Forbidden lint. `copylint.check_prose`
(`copylint.py:183-194`) only warns, and its one call site in `test_copylint.py:191-199`
turns hits into pytest warnings. (The same edit on q3 was caught, but only because q3's
record is pinned to its migration.) The warn-only rule is SPEC §11.1's, for tropes; the
Forbidden row has no such carve-out. GAP-11.

**Also.** `trace.steps[].note` is rendered and is not a lint home (GAP-12, PQ-42 A).
Whitespace and Unicode variants bypass the prose patterns (GAP-13, PQ-42 C). A
room-authored reason string with *Turn to your neighbour and volunteer one.*
(`routes.rs:196`) passed every test (M5b3). The buzzer does not show that reason today.
GAP-14.

**UI that counts or waits.** No element prompts for or waits on a contribution
(`copy-freeze.test.js:434,469,482`). The aggregate *‹n› of us said ‹X›* counts answers,
which SPEC §11 allows.

**Audit demand.** *The lint is a ticket*: met for the copy module. *§11 is the only
place copy is authored*: not met for `routes.rs:196,208` (GAP-14).

**Verdict: HOLDS-WITH-GAPS.**

## G-6 — the phase order, each step a host action

> The phase order is `idle → live → closed → split → work → reveal → released`, each
> transition a host action, none skippable. (AC-45, AC-93, AC-97)

**Enforced by, checked.**

- `phase.rs`: seven phases (`:46-54`); `successor` (`:112-122`); `apply` (`:376-465`)
  advances only on the phase's one `next_action`. `Room::act` (`rooms.rs:447`) is the
  only caller. No timer or expiry changes the phase; `lifecycle::verdict` ends a room
  and never touches the machine.
- M6a (`split → reveal`, a skip) and M6a2 (`reveal → work`, a reversal) were each caught
  by:
  - `phase_table.rs::every_state_and_command_does_exactly_what_the_table_says`
  - `::no_transition_skips_a_phase_and_reveal_needs_the_walk_through`
  - `phase::tests::the_witness_exists_only_in_reveal`
  - the canary
- One phase value: the wall, buzzer and host payloads all read `view.phase`
  (`view.rs:398,419,501,602`), checked by `wiring.rs` and the canary's projection
  agreement.
- `work` carries no ✓, receipt or colour: the canary's rules (`canary_scan/mod.rs:763-829`).
  M6d (colour allowed in `work`, `web/shared/phase.js:37`) was caught by four AC-99
  tests. M6c (`closed` and `split` swapped in `phase.js:34`) was caught by
  `phase.test.js`.

**Audit demand.** *State-machine test*: met. *`canary` that `work` carries no ✓,
receipt or colour*: met.

**Verdict: HOLDS.**

## G-7 — the receipt claims exactly what verification proved

> The receipt lines are generated from the `verified` record by one function, and a
> line renders only if the record holds its step (§7.5). (AC-43, AC-71, AC-87)

**Enforced by, checked.** Two implementations, room `answers.rs:321 receipt_lines` and
pipeline `receipt.py:74`, held to nine shared fixtures (`bank/fixtures/receipts/`) by
`twins.rs::the_receipt_matches_every_shared_fixture` and `test_receipt.py`. The JS wall
and take-it-home print `receipt.lines` verbatim. Claim words are banned
(`test_no_rendered_line_overstates`).

**The Miri line is unguarded (M7a, M7b).** In both implementations, rendering *✓ Miri
ran clean* when the record has no `miri` at all passed every test. No fixture has
`runs` without `miri`. Such a record is reachable: the room's loader keeps `miri` as an
`Option` and refuses nothing (`answers.rs:174`), and the build path runs no provenance
check (GAP-4). A receipt could then claim a Miri run that never happened, which is the
exact overstatement G-7 exists to stop. GAP-16.

**Audit demand.** *String test*: met for the lines the fixtures hold. *HC-2 reads it*: a
human checkpoint, not run here.

**Verdict: HOLDS-WITH-GAPS.**

## G-8 — neither source nor trace on a participant device

> Neither source nor trace is rendered on a participant device. (AC-32)

**Enforced by, checked.** `BuzzerPayload` (`view.rs:155-190`) has no source or trace
field. M8a (the source pushed into the buzzer's `live` lines) was caught by
`buzzer_page.rs::a_phone_through_the_wired_room_shows_what_the_server_holds`. M8b (the
buzzer opening `/ws/wall`) was caught by `buzzer.test.js`.

A phone *can* fetch `/rooms/{id}/wall` with the room id `/join` returns. That is the
projector's own view and nothing renders it on a phone; SPEC G-8 binds what is rendered
and what the participant payload holds, and both hold.

**Audit demand.** *`canary` in `test`*: met.

**Verdict: HOLDS.**

## G-9 — every credential path enumerated and checked by one function

> Hosting by role ID in `roles`, checked at room creation; the pipeline channel by one
> admin token (§8.3); and the M1 stand-in (§8.2), which exists only behind a build
> feature. Nothing else authorizes anything. (AC-64, AC-65, AC-69, AC-101)

**Enforced by, checked.**

| Path | Check | Caught |
|---|---|---|
| Discord role ID at creation | `discord.rs:515` | M9c (any member hosts) caught by `ac64_a_member_without_the_role_is_denied`, `ac65_the_role_id_is_compared_whole`, `ac65_an_administrator_and_owner_without_the_role_id_is_denied` |
| Organizer session | `discord.rs:451`, constant-time | read only |
| Host session, resume link, `/ws/host` | `Room::host_matches` (`rooms.rs:421`), constant-time | read only (`ac68_*`) |
| Admin token | `AdminToken::check` (`admin.rs:85-96`), one `ct_eq` | M9b (prefix accepted) and M9b2 (`==` with a dead `ct_eq` left in) caught by `g9_one_constant_time_compare_and_no_log` |
| The M1 stand-in | deleted; `Cargo.toml` features are `spike`, `smoke`, `burst` | `ac64_no_stand_in_survives_in_src_cargo_dockerfile_or_ci` |

The test-only `TestAuth` lives in `room/tests/common` and is not in the binary.

**The route-table test reads one file (M9a2).** An admin-like route added in
`lib.rs` passed every test; the same route in `routes.rs` (M9a) was caught. The
route-table checks (`canary_scan/mod.rs:566-592`, `admin.rs:292`) parse `routes.rs`
only. GAP-17.

**Also.** `HOST_DEV_TOKEN=` and its comments remain in `.env.example:39-40` and
`fly.toml` (GAP-18, PQ-50). `deployed-burst.yml` stores `POPQUIZ_ORGANIZER_SESSION`
and `POPQUIZ_ADMIN_TOKEN` as repository secrets for a client-triggered run and tells the
operator to delete them afterwards. That is a credential path SPEC §8 does not list,
and SPEC §8.3 says the admin token is never in the repository. GAP-19, Minor:
*every path enumerated* is only partly met.

**Audit demand.** *Auth audit covering all three paths*: met for the three paths SPEC names.

**Verdict: HOLDS-WITH-GAPS.**

## G-10 — used at release, not at build

> The used-question record's only writer is the release transition. Building writes
> nothing. (AC-92)

**Enforced by, checked.**

- The room: `rooms.rs:916` is the ledger's only `append`, inside the release branch of
  `act`. The room tests prove it (`used.rs::g10_the_release_transition_is_the_only_writer`,
  `ac92_nothing_is_written_before_release`).
- The laptop: `schedule.merge_used` (`schedule.py:411`) copies the room's ledger and
  nothing else. M10b (`schedule` writing the bank) was caught by
  `test_ac92_schedule_writes_nothing_under_the_bank`. M10c (a second room accepted for
  one question) was caught by
  `test_sync_refuses_one_question_released_by_two_rooms_in_one_ledger`.

**`fit` is writable after release (M10d).** Removing the `!= Released` guard from
`record_fit` (`rooms.rs:438`) passed the whole room suite. `PUT /rooms/{id}/fit` needs
no credential (`routes.rs:364`) and any attendee has the room id. The ledger's `fit` is
copied at the release transition, so the record itself is safe today. Only an untested
line keeps a released room's state frozen. GAP-10.

**Audit demand.** *Audit and retire `build_deck.py`'s write*: not met.
`build_deck.py:220-234` (`slot_for_meetup`) still writes `mvp/answer-history.json` on
every `--only` build. GAP-20, PQ-46. A `just smoke` or `just burst` run against the
deployed room defaults to `--question q3` and releases the real q3 (GAP-21, PQ-41).

**Verdict: HOLDS-WITH-GAPS.** The room holds. The MVP tool and the harness defaults do
not.

## G-11 — no answer-category distribution where an attendee can read it

> `bank-audit` scans participant-facing artifacts; organizer docs carry the tells.
> (AC-25)

**Enforced by, checked.** `audit.distribution_lint` over `PARTICIPANT_FACING`
(`audit.py:636-647`): `web/shared/copy.js`, `README.md`, three `mvp/` docs and
`web/home/`. M11a (*About 40% of the answers are does not compile…* in `README.md`)
was caught by `test_the_real_participant_facing_files_are_clean`. The tell report is
written under `bank/audit/`, which is gitignored (`bank/audit/.gitignore:4`).

**The list misses files (M11b).** The same sentence in `docs/RUNBOOK.md` passed. The
list also omits `web/README.md`, the wall, buzzer and host shells and `room/src/copy.rs`.
M11c (the sentence in the buzzer's shell) was caught, but by the copy freeze, not this
lint. The repository is private today, which lowers the stakes. SPEC G-11 lets
organizer docs carry tells while EVALUATION AC-25 lints them (CONTRACT NOTE 2). GAP-22.

**Audit demand.** *Ticket for the scan*: met. The scan is not a CI step of its own
(GAP-23, PQ-49); it reaches CI through pytest.

**Verdict: HOLDS-WITH-GAPS.**

## G-12 — every scheduled question has an affirmation

> Scheduling refuses unaffirmed questions; affirmation records who and when.
> (AC-72, AC-95)

**Built, and checked as built.** `schedule.refusal` (`schedule.py:127-158`, PR #47)
refuses a question without `affirmed_by` and `affirmed_at`, and `in_reserve` repeats the
check. M12a (both checks removed) was caught by four `test_ac72_*` tests and
`test_g12_the_gate_agrees_with_the_reserve`.

**Unbuilt.** Nothing writes an affirmation: the review surface, T-18 (PQ-23), is gated
on F-15 with the affirmation path F-52 open. The gate trusts any non-empty strings.
M12b (`"affirmed_by": "x", "affirmed_at": "x"` typed into a copied record) scheduled
with exit 0. The affirmation also names who and when but not what: text edited after
affirmation stays affirmed (CONTRACT NOTE 3). GAP-24.

**Other paths around the gate.**
- `python -m popquiz.fallback` checks nothing (M12d, GAP-2).
- The room's admin `PUT` reads no `review` (`answers.rs:100`), so anything holding the
  token schedules an unaffirmed question. The harnesses do this by design with their
  own ids, and with the real q3 by default (GAP-21). GAP-25.

**Verdict: UNENFORCED.** The schedule-side gate holds as written. The affirmation it
checks has no writer.

## Phase invariants (SPEC §4)

- **Exactly seven phases, host-action transitions, none skippable:** proven by M6a and
  M6a2 (see G-6).
- **None reversible except where §4 says:** *Run it again* returns `Applied::NewRoom`
  and the old room stays released (`phase_table.rs::released_never_re_enters_the_room`).
  The static fallback's `Esc` steps back a phase, as SPEC §12 allows
  (`wall-static.test.js:87`).
- **One phase value on all three surfaces (AC-81):** see G-6.
- **What each surface may show:**
  - `work` shows steps `0..M-2` only (M3c caught).
  - `reveal` enters at M-1 (`phase_table.rs::trace_step_bounds_*`).
  - Colour appears only in `live`, `closed` and `split` (M6d caught).
  - `idle` and `released` render no source (the AC-99 tests).
- **Three literals of the phase order:** `web/shared/phase.js:34` is tested.
  `web/wall/fallback/static.js:40` is tested by `wall-static.test.js:69`.
  `pipeline/src/popquiz/fallback.py:90`, which orders the host sheet, is tested by
  nothing: swapping `closed` and `split` there passed `just test`'s pipeline and web
  suites (M6h). GAP-15.

**Verdict: HOLDS-WITH-GAPS.**

## Cross-cutting checks

**One writer, one reader (SPEC §3).**

| Field | Writer | Reader | Finding |
|---|---|---|---|
| `review.affirmed_by`, `affirmed_at` | none (T-18 unbuilt) | `schedule`, `audit`, `dedupe` | no writer |
| `verified.verified_at`, `verifier_version` | `verify` | `check_provenance` only, which nothing calls | no production reader |
| `verified.miri.seeds` | `verify` | none | no reader |
| `review.reason` | migration | none (its reader is T-16, PQ-21, unbuilt) | no reader |
| `explains.legacy` | migration | `bank.quoted_outputs`, which nothing calls (T-18) | no production reader |
| Room `fit` | the wall's `PUT` | host payload, `used` at release | writable after release (GAP-10) |

GAP-27 lists them. Every room-record field in §3.4 has a writer and a reader.

**New credential and data paths since the guardrail tests.**
- The admin channel (T-25) holds against G-9: one constant-time check, a route table,
  the token absent from payloads and logs (`just secret-scan`, the canary's admin rule).
  Its gap is that it reads no `review` (GAP-25).
- `popquiz schedule` and `sync` (PR #47) hold against G-10. Against G-12 they hold as
  written and rest on an affirmation with no writer (GAP-24). Against G-1 they hold, but
  the arrangement they introduced is unguarded (GAP-1).

**Rules that read history.** None changes tonight's output. The one balancer on the
tree, `build_deck.py:96-109`, shapes only the multi-question review deck (GAP-3).

---

## Gaps

Severity is the attendee's (ruling 6). *Covered* means a board ticket already holds it.

```
GAP-1  | G-1        | Major | new | The arrangement is outside every slot lint and simulation | schedule.py:189-208, schedule.py:166 (own _rng); audit.py:357,501,537 read slot.py and slot_for_day calls only; M1c survived | Lint schedule.arrange like the slot path (no bank, used or ledger reads; no draw but slot_for_day) and run the attendee simulation over arrange's output positions
GAP-2  | G-1, G-12  | Major | new (not PQ-48 or PQ-45) | The standalone fallback builder bakes stored order, answer at E for every bank question, with no date, no arrange, no affirmation or used check | fallback.py:27-29,306-333; `python -m popquiz.fallback q3 --bake` exit 0, E = q3's answer; test_fallback.py:288 pins it; room/README.md:596 documents it; M12d | Require --date and arrange, apply schedule.refusal, or remove the CLI in favour of `popquiz schedule --no-push`
GAP-3  | G-1        | Minor | covered by PQ-46 (needs: also retire answer_slots) | build_deck.py balances answer positions in the multi-question deck | build_deck.py:96-109 | Retire answer_slots with slot_for_meetup's write
GAP-4  | G-2        | Major | new | The provenance check is not on the build path, so an uncommitted or local bank schedules without it | check_provenance (verify.py:577) runs in CI over committed files only (test_verify.py:425-428); schedule.refusal (schedule.py:127) and fallback.bake (fallback.py:105) skip it; the replay covers MIGRATED only (migrate_mvp.py:76); M2a survived | Call check_provenance in refusal and bake, and replay every bank question that has a recording
GAP-5  | G-2        | Minor | new | Scheduling does not refuse a stale pin | verify.is_stale (verify.py:507) has no caller; SPEC 7.2 | Call is_stale in schedule.refusal for non-legacy records
GAP-6  | G-2        | Minor | new (CONTRACT NOTE 1) | A self-consistent hand edit to a verified record is undetectable | verify.py:592-596; M2c accepted by check_provenance | Contract: a digest or recording id binding the record to its run
GAP-7  | G-2        | Minor | new | The MVP deck trusts verified.json's answer with no provenance and no test | mvp/tools/build_deck.py:241,266; mvp/tools/verify.py:166-216 | A provenance check in build_deck, or retire the MVP deck once the room runs
GAP-8  | G-3        | Major | new | Two live rooms on one question: releasing one publishes the answer at /last while the other is pre-reveal | rooms.rs:846-886 (create_for checks used only), rooms.rs:925 (global take_home), routes.rs:639; probe test | Refuse create_for while another unreleased room holds the question, with a test
GAP-9  | G-3        | Major | new | A RevealWitness can be forged without boundary.rs noticing | phase.rs:336; boundary.rs:98-113,148-156; M3e, M3e2 survived | Put RevealWitness in the no-derive, no-impl scan and count every constructor form in phase.rs
GAP-10 | G-10       | Minor | new | fit is writable after release; the guard is untested | rooms.rs:438; routes.rs:364 (no auth); M10d survived | A test that a released room ignores PUT fit and its revision does not move
GAP-11 | G-5        | Major | new | The Forbidden row is not enforced on question prose | copylint.py:183-194 warns only; test_copylint.py:191-199; M5d2 survived | Fail `just test` on a Forbidden match in explains, why_tempting, hint (tropes stay warnings, SPEC 11.1)
GAP-12 | G-5        | Major | covered by PQ-42 (sufficient) | trace.steps[].note is rendered and not linted | copylint.py:157-180 | PQ-42 A
GAP-13 | G-5        | Minor | covered by PQ-42 (sufficient) | Whitespace and Unicode variants bypass the prose patterns | copylint.js:95-97, copylint.py:126-129 | PQ-42 C
GAP-14 | G-5        | Minor | new | Room-authored reason strings sit outside the copy module and are unlinted | routes.rs:196,208; M5b3 survived | Move them into copy.rs/copy.js or lint room/src string literals
GAP-15 | Phase inv. | Minor | new | The host sheet's phase order is a third literal no test ties to G-6 | fallback.py:90; M6h survived | Assert fallback.PHASES equals phase.js's order
GAP-16 | G-7        | Major | new | A receipt can claim a Miri run the record does not hold | answers.rs:347, receipt.py:120-121; answers.rs:174 accepts runs without miri; M7a, M7b survived | A receipt fixture with runs and no miri, asserting no Miri line, in both twins
GAP-17 | G-9        | Minor | new | The route-table tests read routes.rs only | canary_scan/mod.rs:566-592; admin.rs:292; M9a2 survived | Walk the built Router, or scan every file that calls .route/.nest/.fallback
GAP-18 | G-9        | NIT   | covered by PQ-50 (sufficient) | HOST_DEV_TOKEN residue | .env.example:39-40; fly.toml | PQ-50 item 7
GAP-19 | G-9        | Minor | new | Deployed burst keeps an organizer session and the admin token as repo secrets | .github/workflows/deployed-burst.yml:7-11,58-67,100 | Name it in SPEC 8 as a credential path, or pass them as one-run inputs
GAP-20 | G-10       | Major | covered by PQ-46 (sufficient) | build_deck.py still writes the ledger at build | build_deck.py:220-234 | PQ-46
GAP-21 | G-10, G-12 | Major | covered by PQ-41 (sufficient) | smoke and burst default to the real q3 and release it on the deployed room | burst.rs:100, smoke.rs:96 | PQ-41 B
GAP-22 | G-11       | Minor | new (CONTRACT NOTE 2) | The distribution scan's file list misses docs/RUNBOOK.md, web/README.md, the page shells and room/src/copy.rs | audit.py:636-647; M11b survived | Scan by rule (every authored file outside bank/) and settle which organizer docs may carry tells
GAP-23 | G-11       | Minor | covered by PQ-49 (sufficient) | bank-audit is not its own CI step | .github/workflows/ci.yml | PQ-49
GAP-24 | G-12       | Major | covered by PQ-23 (needs: bind the affirmation to the text it affirmed; `edited` clears it) | No affirmation writer; the gate trusts hand-typed strings | schedule.py:147; audit.py:1085-1087 (in_reserve); bank.py:206 (affirmed()); M12b survived | PQ-23, with CONTRACT NOTE 3
GAP-25 | G-12       | Minor | new | The room accepts a pushed question with no affirmation | answers.rs:100 (review ignored); admin.rs | Decide whether the room refuses an unaffirmed bank question or the gate stays client-side and is said so
GAP-26 | G-1        | NIT   | covered by PQ-48 (needs: the option-position tell measures stored order, which arrange discards) | The AC-26 position tell reads the bank's order, not the wall's | audit.py tell pool, answer_index | PQ-48's ruling should say what position it measures
GAP-27 | Cross-cut. | Minor | new | Persisted fields with no writer or no production reader | see the one-writer table | Delete or wire each, per SPEC 3
```

## Closed since the interim report

The interim report audited `0a4b6fb`. Each item below was re-checked on `0c7d60a`.

- **"The answer is at E for every question; nothing arranges options by date."** Closed
  for `popquiz schedule` by PR #47: `schedule.arrange` (`schedule.py:189-208`) places
  the answer at `slot_for_day(date)`, tested by `test_ac23_*` (M1d caught). Still open
  for `python -m popquiz.fallback` (GAP-2). The stored order is still E for all four, which is
  correct: the bank's order means nothing.
- **"G-12 has no enforcement on this tree."** The schedule-side gate now exists
  (`schedule.refusal`, PR #47; M12a caught). The affirmation writer is still unbuilt
  (GAP-24).
- **"AC-62, AC-63 have no surface."** Closed by PR #48: `docs/RUNBOOK.md:273,275` carries
  both sentences.
- **Still open, re-verified:**
  - `build_deck.py`'s ledger write (GAP-20)
  - `bank-audit` not in CI (GAP-23)
  - the AC-26 small-bank rule (PQ-48; `bank-audit` still WARNs at 5.00x)
  - `HOST_DEV_TOKEN` residue (GAP-18)
- **Not re-verified here:** the in-memory used ledger and organizer store, and the
  smaller fallback and live-region items. None is a guardrail mechanism.

## What I could not verify

- **The verifier on the real toolchain** (AC-12 containment, Miri, the pin):
  `just test-full` needs Docker and the sandbox image. Every G-2 and G-7 mutation here
  ran against `StubRunner`'s recordings.
- **The deployed room** (Fly v11, `1263add`): out of scope by ruling 1. The canary's
  `--url` and `--full` halves were not run.
- **The wall's measured layout and the static fallback stepped offline** need a browser.
- **Real phones, a projector, and the human checkpoints** (HC-1…HC-4): the felt halves
  of AC-43, AC-98 and the rest.
- **Timing of the admin compare.** A non-constant-time compare that keeps a dead `ct_eq`
  is caught only because the test pins the exact count of `ct_eq(` calls (M9b2).
  Timing itself is not testable here.

## Mutation table

Every mutation was reverted with `git restore`, and the tree checked clean, before the
next. *Copied bank* means a copy of `bank/` in the session's scratch directory, never
in the tree.

| ID | Mutation | Where | Caught by, or SURVIVED |
|---|---|---|---|
| M1a | `slot_for_day` gains `history=None` | slot.py:55 | `bank-audit` slot lint (`FAIL slot path and ledger`); `test_audit.py` signature anchor |
| M1b | `slot_for_day` returns `day.toordinal() % 5` | slot.py:76 | `test_slot.py::test_slot_for_day_draws_exactly_what_the_mvp_drew`; `bank-audit` generator (`TOO EVEN`), attendee simulation, slot lint |
| M1b′ | a least-used, never-repeat balancer passed straight to the audit (`audit_generator`, `simulate_attendees`) | audit.py:163,261 | `audit_generator` raises `TOO EVEN (chi2=0.000)`; the simulation's least-used attendee hits 100%, outside the 18–22% band |
| M1c | `arrange` avoids last meetup's letter, read from the bank's `used` | schedule.py:198 | **SURVIVED** (all of `pipeline` pytest) |
| M1d | `arrange` puts the answer at E | schedule.py:198 | `test_schedule.py::test_ac23_the_correct_option_sits_at_the_dates_slot` (+3) |
| M2a | non-legacy record without `verifier_version`, `verified_at`; affirmed; copied bank | bank record | **SURVIVED** (`popquiz schedule --no-push` exit 0) |
| M2b | `"correct": 1` added; copied bank | bank record | `bank.question_from_dict` (`unknown field(s) ['correct']`), exit 1 |
| M2c | `verified.stdout` edited so the answer moves to A, and E given a `why_tempting` (without it `schedule` refused, exit 1); copied bank | bank record | **SURVIVED** (`check_provenance` accepts; `schedule` arranges A) |
| M2c-q3 | the same edit to `bank/questions/q3.json` | q3.json | `test_migration.py::test_the_correct_option_is_the_machines_output[q3]` (+5); `take_home::the_fixtures_are_what_the_room_builds` |
| M2c-q4 | the same edit to `bank/questions/q4.json` | q4.json | `test_migration.py::test_the_correct_option_is_the_machines_output[q4]` (+2) |
| M2d | room's `correct_index` returns 0 | answers.rs:268 | `canary::every_phase_every_surface_the_answer_stays_sealed_until_reveal` (+7) |
| M2f | `verified` deleted; copied bank | bank record | `schedule.refusal` (`has no verified record`), exit 1 |
| M3a | `stdout` rows kept in the walk-through | answers.rs:655 | `canary::every_phase_every_surface…`, `…does_not_compile` |
| M3c | `work` may reach step M-1 | phase.rs:258 | `canary::every_phase_every_surface…` (`work shows steps 0..=M-2`) |
| M3d | pre-reveal options reversed | view.rs:305 | `canary::every_phase_every_surface…` (`correct option's text is somewhere other than option E's slot`) |
| M3e | `RevealWitness::forge()` added | phase.rs:336 | **SURVIVED** (`boundary`, lib) |
| M3e2 | `#[derive(Default)]` on `RevealWitness` | phase.rs:336 | **SURVIVED** (`boundary`, lib) |
| M3e3 | a `peek` method added to the vault | answers.rs:496 | `boundary::the_vault_holds_only_the_allowed_functions_and_every_read_takes_the_witness` |
| M4b | `sessions.release()` removed at release | rooms.rs:915 | `lifecycle::ac56_nothing_per_person_survives_release` |
| M4a | `UsedRecord` gains `totals`, filled at release | used.rs:43, rooms.rs:922 | the compile-time field pin at `used.rs:56` (E0027, `pattern does not mention field totals`) |
| M5a | *Turn to your neighbour.* in `buzzer_idle` | copy.js:63 | `copylint.test.js` Forbidden row; AC-98 test |
| M5a2 | the same with two spaces | copy.js:63 | AC-98 test; copy freeze (not the Forbidden-row lint) |
| M5b | the same in `copy.rs` only | copy.rs:27 | `twins::the_copy_module_mirrors_copy_js_in_both_directions` |
| M5b3 | *Turn to your neighbour and volunteer one.* in the answer route's reason | routes.rs:196 | **SURVIVED** (room suite, web suite) |
| M5d | *Turn to your neighbour and argue it out.* in q3 `explains.what` | q3.json | `test_migration.py::test_the_committed_record_is_what_a_fresh_run_writes` (q3's migration pin, not a lint) |
| M5d2 | the same in q4 `explains.what` | q4.json | **SURVIVED** (pipeline, web, room suites) |
| M6a | `split → reveal` | phase.rs:117 | `phase_table::every_state_and_command_does_exactly_what_the_table_says`, `::no_transition_skips_a_phase_and_reveal_needs_the_walk_through`, `phase::tests::the_witness_exists_only_in_reveal`, canary |
| M6a2 | `reveal → work` | phase.rs:119 | the same four (the `phase_table` binary also loops forever, so a CI job would time out) |
| M6c | `closed` and `split` swapped | phase.js:34 | `phase.test.js` (`the seven phases are G-6's, in G-6's order`) |
| M6d | colour allowed in `work` | phase.js:37 | four AC-99 tests |
| M6h | `closed` and `split` swapped | fallback.py:90 | **SURVIVED** (pipeline, web suites) |
| M7a | room renders *Miri ran clean* with no `miri` | answers.rs:347 | **SURVIVED** (`twins`, `take_home`, `canary`, lib) |
| M7b | pipeline renders *Miri ran clean* with no `miri` | receipt.py:120-121 | **SURVIVED** (`test_receipt.py`) |
| M8a | source in the buzzer's `live` lines | view.rs:516 | `buzzer_page::a_phone_through_the_wired_room_shows_what_the_server_holds` |
| M8b | buzzer opens `/ws/wall` | buzzer.js:600 | `buzzer.test.js` (AC-48 shell test) |
| M9a | `/ops/used` route in `routes.rs` | routes.rs:364 | `canary::every_route_in_routes_rs_is_scanned_or_listed` |
| M9a2 | `/ops/used` route in `lib.rs` | lib.rs:101 | **SURVIVED** (`admin`, `canary`, `no_fallback_route`) |
| M9b | admin check accepts a token prefix | admin.rs:91 | `admin::ac101_a_missing_or_wrong_token_is_refused_saying_nothing`, `::g9_one_constant_time_compare_and_no_log` |
| M9b2 | admin check uses `==`, dead `ct_eq` left | admin.rs:91 | `admin::g9_one_constant_time_compare_and_no_log` |
| M9c | any guild member hosts | discord.rs:515 | `auth::ac64_a_member_without_the_role_is_denied` (+2) |
| M10b | `schedule` writes the bank | schedule.py:548 | `test_ac92_schedule_writes_nothing_under_the_bank`, `test_ac23_the_stored_order_is_never_rewritten` |
| M10c | `merge_used` accepts two rooms for one question | schedule.py:436 | `test_sync_refuses_one_question_released_by_two_rooms_in_one_ledger` |
| M10d | `fit` guard against `released` removed | rooms.rs:438 | **SURVIVED** (whole room suite) |
| M11a | distribution sentence in `README.md` | README.md | `test_audit.py::test_the_real_participant_facing_files_are_clean`, `::test_the_real_bank_passes_every_bank_level_check` |
| M11b | the same in `docs/RUNBOOK.md` | RUNBOOK.md | **SURVIVED** (`test_audit.py`) |
| M11c | the same in the buzzer page shell | web/buzzer/index.html | `copy-freeze.test.js` (`the page shell types no string of its own`), not the AC-25 lint |
| M12a | affirmation checks removed from `refusal` | schedule.py:148,156 | `test_ac72_*` (4) |
| M12b | `affirmed_by: "x"`, `affirmed_at: "x"` typed in; copied bank | bank record | **SURVIVED** (`popquiz schedule --no-push` exit 0) |
| M12d | `python -m popquiz.fallback q3` on the unaffirmed record | copied bank | **SURVIVED** (exit 0, answer at E) |

**Planned and not run.** M3g (the `/last` snapshot taken at `reveal`) and M4c (the
buzzer sending its saved answer after close) were in the plan and were not run. The
tests that would catch them were read (`used.rs::ac92_nothing_is_written_before_release`,
`canary_scan/mod.rs:1277-1295`; `canary_scan/mod.rs:1614-1637`), not proven. G-4's
post-close claim rests on M4b, M4a and that reading.

**Probe P1** (GAP-8, not counted as a mutation): a throwaway test in
`room/tests/take_home.rs` created two rooms on q3, put one live and released the other.
`/last` served `"correct": true` on E while the live room's buzzer showed its hint.
Reverted.
