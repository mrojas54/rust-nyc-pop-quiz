# PQ-4: Phase machine and sealed answers module

BUILDPLAN.md T-04a (M1).

The phase machine: `idle→…→released`, host-only transitions, no skipping, `trace_step`; **the sealed `answers` module** unreachable from the public state query, proven by a type/module boundary test

Criteria: AC-45, AC-47, AC-61, AC-81, AC-93, AC-97, G-3, G-6
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-03
BUILDPLAN notes: Built alone and first; `room/src/phase.rs` is consumed by everything in M1

Orchestrator notes: `room/src/phase.rs` is consumed by everything in M1: build it alone and first. AC-61 is a type or module boundary proof, not a review.

Workflow mode: sub-agent-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-26)

Planner subagent (opus) output merged; the delegator owns this text.

## Files
- `room/src/lib.rs` — `#![forbid(unsafe_code)]`; `pub mod phase, rooms, answers, question, view, copy, auth; mod routes;` `router()` (DenyAll auth, no question → creation refused, keeps the scaffold test) and `router_with(AppState)`.
- `room/src/phase.rs` — pure machine. `Phase` (7, §4 order), `HostAction` (8, AC-45), `Step {Back, Forward}` (not transitions), `Command {Host, Step}`, `TraceLen` (M ≥ 2), `Machine` (private fields: phase, trace_step, trace_len), `State {Unmade(TraceLen), Room(Machine)}`, `Applied {Room(Machine), NewRoom}`, `Refused {from, command, reason}`, `apply(State, Command) -> Result<Applied, Refused>` (signature pinned: no clock, no context), `RevealWitness<'m>` (private field, minted only by `Machine::revealed()` iff phase == Reveal), `next_action(Phase)`. Match without `_` arm over (phase, command).
- `room/src/question.rs` — `Letter` (A–E), `PublicQuestion` (id, source, option texts in arrival order, hint, public trace steps 0..M-2 with any `values` entry named `stdout` stripped, M).
- `room/src/answers.rs` — SEALED. Deserializes the bank record (serde, deny_unknown_fields, mirrors question.schema.json), splits it into `PublicQuestion` + `Sealed` at load (`Scheduled`). `Sealed` (private fields): option kinds, why_tempting, explains, verified, derived correct letter (twin of `bank.correct_index`), receipt lines (twin of `receipt.receipt_lines`), full trace incl. resolving step. Every accessor demands `&RevealWitness`. `Sealed::judge(&Totals) -> Verdict` at close (most-chosen incorrect, ties → lower letter, all-zero → Nobody); `Verdict` readable only with the witness.
- `room/src/rooms.rs` — room record §3.4 (id, code from `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`, join_url, question_id, machine, host_session (no setter, AC-50), host_resume_url, created_at, expires_at = +4h, organizer, present, answered_live, frozen {answered, totals, verdict}, released_at, fit). `Room::act(cmd, now, close_snapshot)` is the only assignment to `machine`, from `apply`'s Ok. `Room::open() -> Option<Opened<'_>>` is the only exit to sealed data (borrows Room, so it cannot outlive a phase change). `Room::public() -> PublicView<'_>` has no path to Sealed. Seam for T-04b/T-04c: `LiveCounts {present, answered_live}`, `CloseSnapshot::from_totals(totals)` (answered = sum), trait `Sessions {live_counts, close_snapshot, release}`, `NoSessions` default, `Room::accepts_answers()`, `Room::set_live_counts()`, `Room::revision()` (bumped on every change, for T-04c's broadcast). `AppState` (std Mutex; rooms map, auth, scheduled question store, released-question set, urls).
- `room/src/view.rs` — THE public state query: `wall(&Room)`, `buzzer(&Room)`, `host(&Room)` dispatch on `room.open()`: `None` → pre-reveal builders that take only `PublicView`; `Some` → reveal builders. Typed payload structs; pre-reveal option type is exactly `{letter, text}`.
- `room/src/copy.rs` — every `web/shared/copy.js` key as a const with the same name upper-cased, plus `fill()`.
- `room/src/auth.rs` — `HostAuth` trait: `authorize_create(bearer) -> Result<OrganizerId, Denied>`, `authorize_host(room, bearer)` (default: constant-time compare with host_session via `subtle`). Ships `DenyAll` only; `TestAuth` lives in `tests/common`. No `HOST_DEV_TOKEN`, no feature.
- `room/src/routes.rs` — `POST /rooms`; `POST /rooms/{id}/{put-on-screen|close-answers|show-split|walk-it|reveal|release|run-it-again|step-back|step-forward}` built from one `HOST_ROUTES` table; `GET /rooms/{id}/{wall|buzzer|host}` (host bearer-checked). Refusal 409 `{reason}`; bad bearer 401 empty body.
- `room/tests/common/mod.rs`, `room/tests/phase_table.rs`, `room/tests/room_record.rs`, extended `room/tests/canary.rs`, fixtures under `room/tests/fixtures/` (planted canary questions derived in-test from q3 — no program output typed by hand).
- `room/Cargo.toml` / `Cargo.lock` — serde/serde_json non-optional (drop `dep:` entries from `spike`); add `getrandom = "0.3"`, `subtle = "2.6"`; both are already in the lock and the local cache → offline lock refresh, no network.
- `room/README.md` — machine, seam, sealed module and its proof.

## Payloads (§4) — every payload carries exactly one `phase` field and `code`
- Wall: idle title + join; live/closed/split source (`colour: true`) + options {letter,text} + phase lines (+ totals, answered/present at split); work source (`colour: false`) + options + three beat lines + trace {at ≤ M-2, m, step}; reveal options with correct ✓, totals, middle (`‹n› of us said ‹X›` | Nobody), receipt {heading, lines}, trace from full trace entering M-1; released title, link, line.
- Buzzer: never source, trace or option texts (G-8). idle line + foot; live letters + hint (only here) + foot; closed; split/work totals + counts (the "including you" count is client-computed, AC-58); reveal correct letter; released line.
- Host (bearer): phase label, primary action = `next_action(phase)`, counts, code; idle resume url + §8.1 sentences; work/reveal step {at, m, note}; reveal Read-it-aloud beats {what, middle, takeaway}. No options/answer before reveal.

## Sealing proof (AC-61, G-3)
compile_fail doctests (no error codes — ignored on stable, which would silently not run them), each with a compiling twin that differs only in the forbidden line: (A) forge `RevealWitness`; (B) read `room.sealed`; (C) reach `open()` from `PublicView`; (D) hold `Opened` across `act` (borrow); (E) forge a `Machine` in Reveal. Plus an in-crate source-scan unit test (`RevealWitness {` only in phase.rs; `.sealed` only in rooms.rs; `machine =` assigned once in rooms.rs), since `pub(crate)` holes are invisible to doctests. Mutation check during implementation: open each boundary, watch the doctest go red; recorded in the PR.

## Tests by criterion
AC-45 exhaustive 8×10 table + route table + host primary action; AC-93 reveal only from work, work only from split, every Ok moves +1; AC-97/D-10 bounds on M=5 and q3 M=6, reveal enters M-1; G-6 doctest E + single-assignment scan; AC-61/G-3 doctests + scan + payload shape; AC-47 host pre-reveal canary; AC-81 one phase field, all three projections equal; §4.4 frozen answered/totals while present moves, close snapshot taken once; §4.5/AC-95 tie → lower letter, correct excluded, Nobody variant, wall == host; G-7 receipt twin over all bank/fixtures/receipts (assert ≥ 9 files); G-2 correct_index twin; copy.rs vs copy.js both directions; §3.4 code alphabet, 4h expiry, host_session stable, released_at only at release, run-again → new room; canary through every phase with positive controls at reveal.

## Choices made / tensions (side taken)
1. Option order: the room never arranges; the pushed record's order is the wall order (ticket, EVALUATION). bank schema says file order ≠ wall order → arrangement is T-20's push. Flag to Orchestrator.
2. correct index for `panic`/`ub`: the ticket's parenthetical says "by option kind for does-not-compile, panic and ub", but `bank.correct_index` matches by kind only for does-not-compile and by text otherwise. The twin must not drift, so the room mirrors bank.py exactly; flag to Orchestrator (a panic question whose option text is not stdout would be refused at load by both twins).
3. Hint: buzzer `live` only (§4.2, AC-48).
4. Run it again requires `authorize_create`, same organizer, refuses the same question id (in-memory released set; T-11's ledger later).
5. Refusal reasons are API diagnostics, not §11 copy; kept clear of Forbidden patterns; flag to T-22.
6. ←/→ at a bound: refused.
7. Room stays `Released` in memory; deletion is T-11's.
8. Receipt canary planted via a does-not-compile fixture with code `ECANARY0`.
9. Load refuses: M < 2, underivable correct index, a trace whose last step lacks a stdout value for a record that ran.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: general-purpose subagent (sonnet), 2026-09-26.

1. **Critical — a later witness-free method added inside `answers.rs` (or a `Debug`/`Serialize`/`Clone` derive on `Sealed`) would leak and pass A–E and the three greps.** *Resolution: accepted.* The secret fields move into a nested module `answers::vault`, whose struct fields are private to `vault` itself. So `answers.rs` code outside `vault` cannot read them either; field privacy is per module. `vault` exposes exactly: `Vault::seal(..)` (constructor from the parsed record), `Vault::judge(&self, &Totals) -> Verdict` (returns an opaque sealed type), and `Vault::open(&self, &RevealWitness) -> &Revealed`. Every read goes through that one chokepoint. Added to the source-scan test:
   - (a) every `fn` declared inside `mod vault` is on an allowlist (`seal`, `judge`, `open`, plus the `Verdict` read that takes the witness), and every non-constructor takes `&RevealWitness`;
   - (b) no `derive(` naming `Debug`, `Clone`, `Serialize` or `Copy` on `Vault`, `Verdict`, `Scheduled` or `Room`; `Debug` is hand-written to redact;
   - (c) `mod vault` is declared exactly once, in `answers.rs`.

   Doctest F is added: `sealed.vault` or `vault.correct` is unreachable (private module).
2. **Major — scope exceeds T-04a.** *Resolution: rejected, with a reason.* The ticket's deliverables list (boot prompt §2, verbatim from the Orchestrator) explicitly orders all of these:
   - `rooms.rs` with the §3.4 shape;
   - routes for every host action plus the three viewer queries;
   - the `HostAuth` seam, as a trait with a test implementation only;
   - a Rust `copy` module mirroring `copy.js`;
   - payloads for all three viewers, which the extended canary needs.

   What stays out, per the ticket: sessions, joins, capacity and the answer upsert (T-04b); sockets (T-04c); pages (T-05/06/07); canary plants (T-08); the stand-in and deploy (T-09); Discord (T-10); timers and the ledger (T-11). The payloads are data; T-05, T-06 and T-07 render them.
3. **Major — the panic/ub correct-index rule is inconsistent.** *Resolution: decided and flagged.* The room mirrors `bank.correct_index` exactly (by kind for does-not-compile, by text otherwise), because a Rust twin that disagrees with the Python twin is the drift both are meant to prevent. Raised to the Orchestrator by `lattice comment` now. If the ruling changes the rule, both twins change together in one ticket.
4. **Minor — how M1 seeds a question.** *Resolution:* `AppState::new(auth, Vec<Scheduled>, urls)`. Tests seed from `bank/questions/q3.json` and derived fixtures. `main.rs` seeds none: creation is refused with *No question is scheduled*, and auth is `DenyAll`. T-09 wires the HC-0 mock question and its stand-in `HostAuth`; T-25 replaces seeding with `PUT /admin/questions/{id}`. The README says so.
5. **Minor — G-6 canary half.** *Resolution:* the README and the completion comment state that this ticket proves G-6's machine half and the in-process `work` scan (no ✓, receipt or `stdout` values entry, `colour:false`). Full canary coverage with real plants and page and frame scans is T-08's.
