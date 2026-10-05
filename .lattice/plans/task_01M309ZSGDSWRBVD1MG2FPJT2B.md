# PQ-28: Guardrail audit

BUILDPLAN.md T-23 (M4).

**Guardrail audit** — an adversarial read of all merged code against G-1…G-12 and the phase invariants; files gap tickets

Criteria: G-1…G-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): everything above
BUILDPLAN notes: The Phase-4 pass, re-run on the built tree

Orchestrator notes: Runs last, on the assembled tree. Files gap tickets; does not fix.

Workflow mode: sub-agent-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-10-05)

Audit tree: `origin/main` @ `0c7d60a`, read from `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/guardrail-audit`. Deliverables: `docs/audit/2026-10-guardrail-audit.md` and `docs/audit/README.md`, nothing else.

Baseline (sandbox bypassed: uv cache and the Discord mock's local bind are denied inside it): `just test` EXIT 0, room 236 passed / 4 ignored, pipeline 710 passed, web 286 passed, 17.7 s warm.

## Method

1. Four Sonnet auditors (A answer integrity, B secrecy, C phase+receipt, D copy+auth) read only and return mechanisms (file:line), enforcing tests, candidate gaps and reachability lists. Capped at 1,200 words each.
2. I run every mutation, one at a time: edit, run the narrowest test that should catch it, record caught (test name) or SURVIVED, `git restore`, `git diff --quiet && git status --short` clean. Data experiments (a copied bank under $TMPDIR, never in-tree) count as mutations when they probe a gate.
3. Each candidate gap is re-checked adversarially (is it caught elsewhere, or does SPEC permit it?) before it goes in.
4. Severity per ruling 6. A Critical stops the ticket at once (comment, needs_human, flag).

## Per guardrail: expected mechanisms → mutations

**G-1** (slot). `pipeline/src/popquiz/slot.py` (pure, date only); `audit.slot_path_violations` (AST lint over slot.py and its call sites); `audit_generator` chi² both tails; AC-23a attendee simulation in `tests/test_audit.py`; `schedule.arrange` (the one arrangement, `schedule.py:189`); MVP `build_deck.py:162` + `slot_for_meetup:220` (inheritance).
- M1a `history=None` parameter on `slot_for_day` → test_audit / test_slot.
- M1b `slot_for_day` returns `day.toordinal() % 5` (round-robin: balanced and predictable) → chi² too-even tail + simulation.
- M1c `arrange` "never last time's letter": reads `mvp/answer-history.json` (or bank `used`) and bumps the slot when it repeats → anything? (the lint covers slot.py; does it cover schedule.py?)
- M1d `arrange` ignores the slot and keeps stored order (answer at E) → test_schedule.
- M1e room sorts options before the wall sees them (`question.rs`) → twins/canary bank-order assertion.
- Audit demand: is the simulation in `just test` (CI runs `just test`)? Is `build_deck.py` audited?

**G-2** (provenance). `verify.check_provenance` (`verify.py:577`); `bank.question_from_dict` refuses unknown fields (`correct`); `bank.correct_index` and room `answers.rs:224 Record::correct_index` (twins); `verifier_version` digest.
- M2a data: a non-legacy record with `verifier_version` and `verified_at` deleted, affirmed in a temp bank copy → `popquiz schedule --no-push` (does the build path run check_provenance?).
- M2b data: `"correct": 2` added to a temp record → load / schedule refuses?
- M2c data: `stdout` and its matching option text hand-edited together → expected survive (documented in `check_provenance`); record it.
- M2d room `Record::correct_index` returns a fixed index → twins.rs.
- M2e `build_deck.py` takes the answer from `content.json` rather than `verified.json` → no tests for mvp/ (expect survive; inheritance).

**G-3 / G-4 / G-8** (secrecy). The sealed `answers::vault` (`answers.rs:456`, opened only with a `RevealWitness` from `phase.rs:302`); `room/tests/boundary.rs`; canary (`room/tests/canary.rs`, `canary_scan/mod.rs`); `view.rs` payload builders; `used.rs` ledger; sessions dropped at release.
- M3a correct letter added to a pre-reveal wall payload in `view.rs` → canary.
- M3b explains in the host's `work` payload → canary.
- M3c `work` bound lets `trace_step` reach M-1 → phase_table / canary.
- M3d pre-reveal options reordered so the answer is first → canary's `{letter,text}` bank-order assertion.
- M3e a route calls the vault without a witness → compile error (structural proof).
- M4a `UsedRecord` gains `totals` → canary / used tests.
- M4b room kept after release → lifecycle.
- M4c buzzer sends its saved answer after close → canary AC-58.
- M8a `source` added to the buzzer payload → canary.
- Reachability: enumerate every route (`routes.rs:227-672`, `admin.rs`), every WS message type (`ws.rs`), every static asset (`/shared/{file}`, `/wall/*`, `/join/*`, `/host/*`, `/home/*`, `/last`, `/{code}`), per phase, against the canary's scanned set.

**G-5** (copy). `web/shared/copy.js` + `web/test/copylint.js` + `copy-freeze.test.js`; `pipeline/src/popquiz/copylint.py`; `room/src/copy.rs` (Rust twin).
- M5a forbidden phrase in a `copy.js` participant string → copylint test.
- M5b the same only in `room/src/copy.rs` → twin test?
- M5c a literal in `buzzer.js` outside copy.js → copy-freeze.
- M5d a forbidden phrase in a bank `trace.steps[].note` → expect survive (PQ-42 A).
- M5e a UI element that counts/waits for contributions — reading only.

**G-6 + phase invariants**. `phase.rs` (`Phase`, `next`, `apply`), `room/tests/phase_table.rs`, `web/shared/phase.js`, wall/buzzer/host render from `phase`.
- M6a `Split => Some(Reveal)` → phase_table.
- M6b a back transition `Reveal → Work` in `apply` → phase_table.
- M6c `web/shared/phase.js` order swapped → phase.test.js.
- M6d syntax colour on in `work` in the wall → wall.test (AC-99).
- M6e a ✓ in the wall's `work` view → wall.test / canary.

**G-7** (receipt). `answers.rs:321 receipt_lines`, `pipeline/src/popquiz/receipt.py`, fixtures `bank/fixtures/receipts/**`, twins.
- M7a "Miri ran clean" when `miri` absent (room) → twins / receipt tests.
- M7b same in `receipt.py` → test_receipt.
- M7c a claim word ("verified") in the line text → test.

**G-9** (credentials). `routes.rs:486-489` admin nest, `admin.rs:85 AdminToken::check` (subtle CT), `discord.rs` role-ID check, `auth.rs`, stand-in removed (no `dev-host-token` feature in `room/Cargo.toml`); `room/tests/admin.rs`, `auth.rs`.
- M9a an admin route registered outside the check → route-table test.
- M9b `check` accepts a prefix of the token → admin tests.
- M9c role check reads `permissions` / matches role name → auth tests.
- M9d an organizer-session route that also accepts the admin token → admin test.
- Also: burst/smoke's credential use (`POPQUIZ_ORGANIZER_SESSION`), harness `HostAuth` in tests only.

**G-10** (used). `used.rs` ledger written at release (`rooms.rs`), `schedule.merge_used` (sync), `build_deck.py:220-234`.
- M10a write `used` at `reveal` → used tests.
- M10b `schedule` writes `used` on push → test_schedule.
- M10c `merge_used` accepts a second room for a used question → test_schedule.
- Audit demand: `build_deck.py:233` write still present? (PQ-46.) Smoke/burst default `--question q3` retiring a real question (PQ-41 B).

**G-11** (distribution). `audit.distribution_lint` over `PARTICIPANT_FACING` (`audit.py:636-647`).
- M11a a share-of-answers sentence in `README.md` → bank-audit / test_audit.
- M11b the same in `docs/RUNBOOK.md` → expect survive (not in the list).
- M11c the same in `room/src/copy.rs` / `web/buzzer/buzzer.js` → expect survive.

**G-12** (affirmation). `schedule.refusal` (`schedule.py:127`); room `answers.rs:100 review: Option<Value>` (ignored); T-18 unbuilt (UNENFORCED, ruling 3).
- M12a affirmation check removed from `refusal` → test_schedule.
- M12b data: `affirmed_by: "x"`, `affirmed_at: "x"` typed by hand → schedule accepts (no writer exists; UNENFORCED evidence).
- M12c the room's admin PUT with no `review` → accepted? (smoke/burst push this way.)

## Cross-cutting
- One writer / one reader per SPEC §3 field (bank, room, organizer); list orphans.
- New credential and data paths since the guardrail tests: T-25 admin channel, PR #47 `schedule`/`sync`, against G-9, G-10, G-12.
- Any rule that reads history, the ledger or past letters and could change tonight's output (PHILOSOPHY §2) — grep every reader of `answer-history`, `used`, `history.json`, ledger.

## Existing tickets cross-referenced (not re-filed)
PQ-21 (T-16), PQ-23 (T-18, G-12), PQ-30, PQ-41 (B: q3 default retire), PQ-42 (A: trace notes unlinted; C: lint bypasses), PQ-43, PQ-46 (G-10 build_deck write), PQ-47 (AC-1 static dep check), PQ-48 (AC-26 position tell), PQ-49 (bank-audit in CI), PQ-50.

## Report outline
Header (tree sha, date, commands + exit codes + counts) → summary table (G-1…G-12 + Phase invariants: verdict, gaps by severity) → one section per guardrail (quote, mechanisms checked, mutations, audit demand, reachability where it applies, verdict, gaps) → Phase invariants → Cross-cutting checks → Gap list (GAP-n) → Closed since the interim report → What I could not verify → Mutation table.

## Plan-Review Cycle 1 Resolutions (AUTHORITATIVE)

Reviewer: Sonnet, read-only, 2026-10-05. Every finding accepted unless marked otherwise. These override the plan above where they differ.

1. **G-1 mechanisms added:** `audit.ledger_violations` (`audit.py:537-581`; non-recursive glob `:586`), `test_slot.py:54-131` (MVP parity, no file read, signature), `FORBIDDEN_WORDS` in `slot_path_violations` (`audit.py:314`).
2. **M1a/M1b split:** a mutated `slot.py` proves only the lint and parity tests. The audit's own power is proven separately: M1b' passes a round-robin generator and a least-used balancer to `audit_generator` / `simulate_attendees` (as `test_audit.py:122-140` does) and confirms both fail.
3. **M1c rewritten:** `arrange` gains `if question.used: slot = (slot + 1) % 5` style history reaction with no ledger name and no file I/O. Second variant: avoid the letter of the most recently used bank question via `load_bank` (a read, not a write). Expect both to slip past `ledger_violations` and `slot_path_violations`; record what catches them.
4. **M1e rewritten:** reverse the option iterator in the room's options builder (`question.rs:94` / `view.rs`), since the room has no sort to mutate.
5. **M2a:** `check_provenance` has no production caller (only `test_verify.py:419-445`, `test_migration.py:145`); M2a is run anyway as a data experiment so the gap carries evidence. Added M2f: delete `verified` entirely → `refusal` (`schedule.py:139`).
6. **M3e rewritten:** construct a `RevealWitness` in a second place (expect `boundary.rs:98`) and add a vault method (expect `boundary.rs:116`). A bare compile error is not a test result.
7. **Reachability oracle:** `canary_scan/mod.rs:518-562` (route table + exemption list). Mutation M3f: add a route to `routes.rs` and confirm the table test fails. Also `no_fallback_route.rs:54,70`, `take_home.rs:109,148`, `web/test/wall-static.test.js`.
8. **G-8 widened:** `/rooms/{id}/wall` and `/ws/wall` carry `source` and `trace` with no auth (`view.rs:458,476,581`). SPEC G-8 binds participant *payloads*; a phone can fetch the wall projection, which shows the same source on the projector. Assessed against SPEC (wall is public by design) before any gap is filed. M8b: `buzzer.js` opens `/ws/wall` → anything catch?
9. **G-3 widened:** M3g builds the `/last` snapshot at `reveal` instead of release → `canary_scan/mod.rs:683-693`.
10. **G-4/G-10 widened:** unauthenticated `PUT /rooms/{id}/fit` writes `fit` into `UsedRecord.fit`; guard `rooms.rs:438`. M10d drops the guard.
11. **G-1/G-12 widened: `popquiz fallback <id>`** (`fallback.py:306-333`) bakes stored order with no `arrange`, no date and no `refusal`. M12d: run it on an unaffirmed temp-bank record. Also confirm `generate.py` (stub) and `migrate_mvp.py` leave order alone.
12. **G-5 widened:** bank prose (`explains.*`, `why_tempting`, `hint`, trace notes) reaches take-it-home; `copylint.check_prose` only warns by design (SPEC §11.1 for tropes, but the *Forbidden* row is not a warning). M5d covers `explains.what` as well as a trace note.
13. **G-9 widened:** host checks on `/rooms/{id}/host`, `/ws/host` attach bearer, `POST /rooms`; `deployed-burst.yml:58-67,124` stores `POPQUIZ_ORGANIZER_SESSION` as a repo secret (a credential path §8 does not enumerate; assess). M9a places the extra admin route outside `/admin`.
14. **CI:** `ci.yml` runs `just test` (secret-scan included via `cargo test --test admin`) and `test-full`; `bank-audit` not in CI (PQ-49).
15. **Untested vs failed:** `mvp/tools/build_deck.py` and `mvp/tools/verify.py` have no tests beyond `test_slot`'s load; mutations there are reported as *no test exists*, not as a test that failed to catch.
16. **M11b/M11c:** kept, but reported as a coverage statement: scanned set is `copy.js`, `README.md`, three mvp docs, `web/home/*`; `docs/RUNBOOK.md`, `web/buzzer/*`, `web/wall/*`, `room/src/copy.rs`, the fallback template are not.

Validation hooks run before the audit phase (sandbox bypassed): `just canary` EXIT 0 (12 passed); `just secret-scan` EXIT 0 (2 passed); `just bank-audit` EXIT 0, `bank-audit: passed`, with the known `WARN tell: option position [AC-26]: worst rule 'index E': 4 hit(s) vs 0.80 by chance = 5.00x over 4 question(s), tail p=0.0016`.
