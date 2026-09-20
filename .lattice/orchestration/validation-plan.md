# Validation Plan
Source spec: [SPEC.md](../../SPEC.md) · Source evaluation: [EVALUATION.md](../../EVALUATION.md) · Date: 2026-09-20 · Status: **approved by the client 2026-09-20** (Phase 0 draft accepted as-is)


`runnable_at` values: **pre-merge-static** (the PR diff and source), **pre-merge-runtime**
(the exact PR head run locally: `just test`, `just canary`, `just bank-audit`, a local
server, or the c11 browser), **post-merge-smoke** (the assembled tree, Docker or the
deployed room, a real third party, or a human). The Result Validator runs the first two;
the client runs the third. Rows tagged `test-full` that need Docker or the deployed room
are smoke rows, because the validator's session is sandboxed too. Artifact = the ticket
whose PR carries the proof.

| # | Criterion (ID) | Verification method | Artifact | Pass condition | runnable_at |
|---|---|---|---|---|---|
| 1 | AC-1 room never calls the generator | Read `room/Cargo.toml` and the `use` graph; `just test` runs the static dependency check | T-16 | No generator, verifier or LLM-client crate or module reachable from `room/`; the check exists and passes | pre-merge-static |
| 2 | AC-1 runtime | Run `smoke` with the pipeline binary absent | T-09 | Room created and run to release | post-merge-smoke |
| 3 | AC-2 generator re-runnable | `just test-full` generation-kill fixture | T-16 | Killed mid-candidate, re-run completes, no partial record in `bank/` | post-merge-smoke |
| 4 | AC-3 count/topics/difficulty honoured | `just test` fixture run | T-16 | n candidates tagged T and d, or a report naming the unmet request | pre-merge-runtime |
| 5 | AC-4 talk mode tags a concept | `just test` fixture title + abstract | T-16 | Every candidate carries a named std/core concept | pre-merge-runtime |
| 6 | AC-4 the connection is real | HC-2 | — | Organizer judges the first talk-mode batch | post-merge-smoke |
| 7 | AC-5 cost and wall-clock in the report | `just test` fixture run report | T-16 | Per-candidate cost and wall-clock fields present | pre-merge-runtime |
| 8 | AC-5 real figures | HC-2 first paid run | — | Figures replace ECONOMICS guess bands | post-merge-smoke |
| 9 | AC-6 record fields and the pin | `just test` with the stub Runner | T-15b | Record carries `-Vv`, edition, target triple, flags; wrong-pin candidate rejected stale; legacy exempt; verify refuses a mismatched compiler | pre-merge-runtime |
| 10 | AC-6 real toolchain | `just test-full` on the T-15a image | T-15b | Same cases pass on the real toolchain | post-merge-smoke |
| 11 | AC-7 answer never hand-written | Read the writer of `correct`; `just test` provenance fixture | T-15b | Only the verifier's output reader writes `correct`; a hand-edited answer fails the build | pre-merge-static |
| 12 | AC-8 varying output rejected | `just test` stub fixture with differing runs | T-15b | Rejected, not flagged; deterministic passes at N=5; rejection count reported | pre-merge-runtime |
| 13 | AC-8 real HashMap program | `just test-full` | T-15b | Rejected on the image | post-merge-smoke |
| 14 | AC-9 Miri UB handling | `just test` stub Miri output | T-15b | UB program rejected; UB-declared accepted only when both borrow models agree | pre-merge-runtime |
| 15 | AC-9 real Miri | `just test-full` | T-15b | Same on real Miri, both configs | post-merge-smoke |
| 16 | AC-10 Miri stdout ≠ native | `just test` stub | T-15b | Mismatch rejects | pre-merge-runtime |
| 17 | AC-10 real | `just test-full` | T-15b | Same on real Miri | post-merge-smoke |
| 18 | AC-11 declared non-compile | `just test` E0502 stub | T-15b | Fails with code recorded; a compiling candidate rejected; does-not-compile receipt renders | pre-merge-runtime |
| 19 | AC-11 real rustc | `just test-full` | T-15b | Same on the image | post-merge-smoke |
| 20 | AC-12 sandbox containment | `just test-full` fixture suite on the T-15a image | T-15a | Socket, `/etc/passwd`, env var, over-allocation, infinite loop each contained and reported | post-merge-smoke |
| 21 | AC-13 receipt from the record alone | `just test` with toolchain absent | T-15b | Receipt renders from `verified` only | pre-merge-runtime |
| 22 | AC-87 receipt lists and take-it-home detail | `just test` receipt fixtures; take-it-home fixture | T-15b, T-12 | Ran-record → four lines; does-not-compile → three; neither → none; take-it-home shows `-Vv`, edition, triple, Miri config for complete, *not recorded* and *run separately* for legacy | pre-merge-runtime |
| 23 | AC-14 exact duplicate | `just test` | T-17 | Byte-identical resubmission rejected | pre-merge-runtime |
| 24 | AC-15 normalized duplicate | `just test` | T-17 | Renamed/reformatted fixtures rejected | pre-merge-runtime |
| 25 | AC-16 near-duplicate to review | `just test` | T-17 | Lands in the queue marked *near-duplicate of q*, neither accepted nor dropped | pre-merge-runtime |
| 26 | AC-17 history persists | `just test` two runs, two working dirs | T-17, T-14 | Count and growth in the run report and on the review surface | pre-merge-runtime |
| 27 | AC-18 uniqueness wording | `just test` corpus scan | T-17 | Every string is *no exact or normalized duplicate found*; none says *original* | pre-merge-runtime |
| 28 | AC-19 review round-trips | `just test` | T-18 | Accept / reject-with-reason / edit each round-trip; reason readable by the generator | pre-merge-runtime |
| 29 | AC-20 one-screen review | HC-2 | — | Organizer reviews a real batch on one screen | post-merge-smoke |
| 30 | AC-21 review time | HC-2 stopwatch | — | A measured number replaces the 15-minute hypothesis | post-merge-smoke |
| 31 | AC-22 reviewer-has-seen statement | `just test` | T-18 | Statement present on the review surface | pre-merge-runtime |
| 32 | AC-88 difficulty drift fails the run | `just test` / `just bank-audit` fixture | T-18, T-19 | `difficulty_judged` recorded; mean drift > 1 fails the run, not the question | pre-merge-runtime |
| 33 | AC-88 first sample | HC-2 | — | First real judged difficulties recorded | post-merge-smoke |
| 34 | AC-23 slot is pure | Read `slot_for_day`; `just test` | T-19 | Signature `(date, n_options=5)`; no ledger, history or bank import on the slot path (static lint) | pre-merge-static |
| 35 | AC-23a attendee simulation | `just test` 10,000-night simulation | T-19 | Three strategies each between 18% and 22% | pre-merge-runtime |
| 36 | AC-23b generator both tails | `just bank-audit`; `just test` skewed and re-balanced generators | T-19 | χ² df=4 both tails; skewed fails; re-balanced fails | pre-merge-runtime |
| 37 | AC-24 five options, one does-not-compile | `just bank-audit` | T-19 | Every bank question passes | pre-merge-runtime |
| 38 | AC-25 no published distribution | `just bank-audit` lint over authored participant-facing files | T-19 | No percentage/ratio within one line of an option-kind word | pre-merge-runtime |
| 39 | AC-26 enumerated tells | `just bank-audit` | T-19 | Each of five tells measured; fails above 1.5× chance | pre-merge-runtime |
| 40 | AC-27 unsafe parity | `just bank-audit` fixture | T-19 | UB answer present ⇒ a non-UB accepted question contains `unsafe` | pre-merge-runtime |
| 41 | AC-71 take-it-home sentence, wall claims none | `just test` | T-12, T-05 | Take-it-home contains the two sentences; wall receipt contains no sentence about the explanation | pre-merge-runtime |
| 42 | AC-72 affirmation gate | `just test` | T-18, T-20 | Unaffirmed schedule refused as a hard error; who/when recorded; < 2 trace steps cannot be affirmed | pre-merge-runtime |
| 43 | AC-73 quoted output checked | `just test` wrong-quote fixture | T-18 | Mismatch blocks acceptance | pre-merge-runtime |
| 44 | AC-74 provenance markup | `just test` | T-05, T-07, T-12 | Machine and human markers distinct on wall, host phone, take-it-home | pre-merge-runtime |
| 45 | AC-74 reads as two things | HC-1 | — | Client confirms | post-merge-smoke |
| 46 | AC-75 reserve count and trend | `just test` | T-20 | On the organizer's first screen | pre-merge-runtime |
| 47 | AC-76 low-reserve warning | `just test` | T-20 | Warning at the configured lead time, default two meetups | pre-merge-runtime |
| 48 | AC-77 runs from reserve offline | `just test-full` network-blocked segment | T-20 | Segment completes with outbound blocked except the room's host | post-merge-smoke |
| 49 | AC-102 static fallback file | c11 browser on the built file from the PR head with network blocked | T-26 | Zero network requests; `Space`/`←`/`→`/`Esc` behave; `work` clean; `reveal` enters at M-1; static and room renders identical for the fixture | pre-merge-runtime |
| 50 | AC-102 host sheet | `just test` | T-26, T-20 | Plain text, every beat and every note verbatim in phase order, no other prose, produced beside the file by the same call | pre-merge-runtime |
| 51 | AC-102 felt | HC-0 | — | Client opens the file offline, steps it, reads the sheet against it | post-merge-smoke |
| 52 | AC-28 join by link or code | `just test` | T-06 | Both paths join; the form has one field | pre-merge-runtime |
| 53 | AC-29 six failure states | `just test` | T-06 | Six distinct messages, each with its next step, matching §11 | pre-merge-runtime |
| 54 | AC-30 capacity before session | `just test` | T-04b | 201st join refused; nothing reserved | pre-merge-runtime |
| 55 | AC-31 join time on venue wifi | HC-1, HC-4 | — | Client-side join-to-lobby logged and reported | post-merge-smoke |
| 56 | AC-32 no source on a phone | `just canary` | T-08 | Source and trace absent from every participant payload, page and state in every phase | pre-merge-runtime |
| 57 | AC-33 wall fits, phone no h-scroll | c11 browser at 375 px on review and take-it-home from the PR head | T-12, T-18 | No horizontal document scroll with the widest bank question | pre-merge-runtime |
| 58 | AC-33 wall overflow measured | `just test-full` layout measurement | T-05 | Zero overflow wherever the state says *fits* | post-merge-smoke |
| 59 | AC-34 last answer wins | `just test` | T-04b | Five changes record the last; a change after close refused | pre-merge-runtime |
| 60 | AC-35 exactly one of saving/saved/failed | `just test` | T-06 | Invariant holds while live | pre-merge-runtime |
| 61 | AC-36 failed write is safe | `just test` | T-06, T-04b | Last saved shown as safe with retry; server keeps the previous answer | pre-merge-runtime |
| 62 | AC-37 reconnect | `just test-full` socket drop | T-04c, T-06 | Saved answer survives; *paused* until fresh state | post-merge-smoke |
| 63 | AC-38 legible from the back row | HC-1, HC-4 | — | Felt | post-merge-smoke |
| 64 | AC-39 reveal surfaces | `just test` | T-05, T-06, T-07 | Wall: ✓, totals, named incorrect with count, receipt, no explanation; host: three beats; buzzer: letter and count | pre-merge-runtime |
| 65 | AC-40 ✓ as well as colour | `just test` | T-02, T-05, T-06, T-12, T-18 | ✓ glyph present on wall, buzzer, take-it-home, review | pre-merge-runtime |
| 66 | AC-41 reveal fan-out ≤ 2 s | `just burst` against the deployed room | T-03, T-21 | p95 ≤ 2 s to 200 clients | post-merge-smoke |
| 67 | AC-42 trope check | `just test` copy module and retired-strings fixture | T-22 | Any copy-module match fails; every pattern group fires on the fixture; question prose yields warnings only | pre-merge-runtime |
| 68 | AC-42 read aloud | HC-1, HC-2 | — | Felt | post-merge-smoke |
| 69 | AC-43 receipt exact lines | `just test` fixtures: q8 E0502, UB-declared, panic | T-05 | Exactly §7.5's lines in precedence order; UB → *Miri flagged undefined behavior*; panic → four lines; no wall line ends in punctuation or contains *verified/established/proves/always/guaranteed* | pre-merge-runtime |
| 70 | AC-43 not overstated | HC-2 | — | Felt | post-merge-smoke |
| 71 | AC-44 beginner explains it | HC-4 | — | Field notes | post-merge-smoke |
| 72 | AC-45 host actions, exactly these | `just test` | T-04a, T-07 | The eight actions plus ←/→; nothing else transitions; no skipping | pre-merge-runtime |
| 73 | AC-46 live counts | `just test` | T-04b, T-07 | Present and answered update while live | pre-merge-runtime |
| 74 | AC-47 host has no answer pre-reveal | `just canary` | T-08 | Host pre-reveal payloads carry no answer | pre-merge-runtime |
| 75 | AC-48 hint is free and private | `just test` + `just canary` | T-06, T-08 | Hint in the live payload; no server-side hint record; no host/wall payload changes | pre-merge-runtime |
| 76 | AC-49 host screen shape | `just test` | T-07 | Phase label, one primary action, code where returning is possible, every screen | pre-merge-runtime |
| 77 | AC-50 host resume | `just test-full` two devices | T-07 | Refresh survives; resume link attaches a second device; session never rotates | post-merge-smoke |
| 78 | AC-51 host never apologises | HC-1, HC-4 | — | Logged | post-merge-smoke |
| 79 | AC-52 200 sessions counted once | `just burst` deployed | T-03, T-21 | Each final answer counted exactly once | post-merge-smoke |
| 80 | AC-53 write p95 < 500 ms | `just burst` deployed | T-03, T-21 | p95 under 500 ms including the burst | post-merge-smoke |
| 81 | AC-54 the deadline burst | `just burst` deployed, separate report | T-03 | 200 writes inside 2 s reported separately and passing; measured before HC-0 | post-merge-smoke |
| 82 | AC-55 failed-request rate | HC-4 logs | — | Client reports from the room's logs | post-merge-smoke |
| 83 | AC-56 nothing survives release | `just test`; read the schema | T-11 | Room and sessions gone; only `used` and take-it-home survive, neither with a count; no per-participant table | pre-merge-runtime |
| 84 | AC-57 no identity anywhere | Read schema, API types, client state; `just test` | T-11, T-06 | No identity, nickname, score or cross-room key | pre-merge-static |
| 85 | AC-58 recap computed on the phone | `just canary` | T-08 | No post-close request carries the answer | pre-merge-runtime |
| 86 | AC-59 *nothing about you was recorded* | HC-1, HC-3 | — | Felt | post-merge-smoke |
| 87 | AC-60 canary on every pre-reveal path | `just canary` | T-08 | Plants absent from HTML, JS state, API, frames | pre-merge-runtime |
| 88 | AC-61 sealed answers module | Read the module boundary; `cargo test` boundary test | T-04a | Public state query cannot reach answer storage; proof is a type or module boundary, not review | pre-merge-static |
| 89 | AC-62 option text is public | `just test` docs assertion | T-24 | Sentence present verbatim in the runbook and on the host's first screen | pre-merge-runtime |
| 90 | AC-63 host can infer, not a guarantee | Same | T-24, T-07 | Sentence present verbatim | pre-merge-runtime |
| 91 | AC-78 legible at the real projector | HC-1 | — | Felt | post-merge-smoke |
| 92 | AC-99 colour scoped | `just test` | T-02, T-05 | Colour spans present in live/closed/split, zero in work/reveal, no source in idle/released | pre-merge-runtime |
| 93 | AC-79 wall is inert and clean | `just canary`; `just test` | T-05, T-08 | No interactive control; plants absent from wall payloads | pre-merge-runtime |
| 94 | AC-80 no dark mode | Read CSS; `just test` | T-02 | No dark-mode media query or theme ships | pre-merge-static |
| 95 | AC-81 one phase everywhere | `just test-full` wall + 200 buzzers | T-04c | Same phase within one broadcast of each transition | post-merge-smoke |
| 96 | AC-100 fit at the floor | `just bank-audit` fixture set (too-long program, too-long option, passing) | T-19, T-02 | Flag fires on each failing fixture and only those; option > 29 chars flagged | pre-merge-runtime |
| 97 | AC-100 measured overflow | `just test-full` every bank question | T-05 | Zero overflow where *fits*; clipped edge where not | post-merge-smoke |
| 98 | AC-82 keyboard | `just a11y` locally via the c11 browser from the PR head | T-13 | Every control reachable, visible focus | pre-merge-runtime |
| 99 | AC-83 live regions | Same | T-13, T-06 | Each listed state change announces politely, strings per §11 | pre-merge-runtime |
| 100 | AC-84 AA contrast | Same | T-13 | AA on every surface and phase | pre-merge-runtime |
| 101 | AC-85 44 px targets | Same | T-13, T-06 | Buzzer and host targets ≥ 44 px | pre-merge-runtime |
| 102 | AC-86 reduced motion | Same | T-13 | No animation under the preference; every state legible | pre-merge-runtime |
| 103 | AC-89 one question, no next | `just test` | T-04a | No *next question* path exists | pre-merge-runtime |
| 104 | AC-89 timing | HC-1, HC-4 | — | Clocked | post-merge-smoke |
| 105 | AC-90 scheduled last | Runbook + client | T-24 | Host script says *last*; client confirms the run-of-show | post-merge-smoke |
| 106 | AC-91 first-look time | HC-4 | — | Watched | post-merge-smoke |
| 107 | AC-92 `used` written at release only | `just test` | T-11, T-19 | Release writes it; building writes nothing; an unreleased room writes nothing; `build_deck.py`'s ledger write is gone | pre-merge-runtime |
| 108 | AC-93 reveal only via the walk | `just test` | T-04a | No transition to reveal except from work, reachable only from split | pre-merge-runtime |
| 109 | AC-94 no ✗ | `just test` | T-06, T-12 | No ✗, red mark or *wrong* against a participant's choice in any phase | pre-merge-runtime |
| 110 | AC-95 most-chosen incorrect, same everywhere | `just test` fixture where the popular option is not the obvious one | T-04b, T-05, T-07, T-18 | Affirm refuses a missing `why_tempting`; wall and host name the same option and count; the nobody-else variant; take-it-home has no count | pre-merge-runtime |
| 111 | AC-95 affirmation | HC-2 | — | Organizer affirms each text | post-merge-smoke |
| 112 | AC-96 second beginner | HC-4 | — | Field notes | post-merge-smoke |
| 113 | AC-97 the walk-through phase | `just test` + `just canary` | T-04a, T-05, T-08 | Phase exists between split and reveal; work carries no ✓/receipt/colour/`stdout`/step > M-2; host-stepped; reveal enters at M-1 | pre-merge-runtime |
| 114 | AC-98 forbidden-copy lint | `just test` | T-22 | Exactly §11's patterns over every participant-facing string; no UI counts or waits on a contribution | pre-merge-runtime |
| 115 | AC-98 host behaviour | HC-1 | — | Watched | post-merge-smoke |
| 116 | AC-64 role check, stand-in, live | `just test` Discord mock; feature-off build test | T-10, T-09 | With role hosts, without denied; no-feature build accepts no stand-in token; stand-in build refuses without it | pre-merge-runtime |
| 117 | AC-64 live Discord | Client's account against the real guild | T-10 | Room created; T-10's exit criterion, before HC-1 | post-merge-smoke |
| 118 | AC-65 roles not permissions | `just test` | T-10 | Administrator without the role denied; code reads `roles` only | pre-merge-runtime |
| 119 | AC-66 refresh rotation persisted | `just test` | T-10 | Each rotation persisted; replayed old token detected without lockout | pre-merge-runtime |
| 120 | AC-67 participants never authenticate | Read routes; `just test` | T-04c, T-06 | No participant route touches the auth module; no participant request carries a credential | pre-merge-static |
| 121 | AC-68 only the creator controls | `just test` | T-10 | Organizer B cannot read or control A's room | pre-merge-runtime |
| 122 | AC-69 runs with Discord down | `just test-full` | T-11, T-10 | Open room runs to release; no new room; dies at 4 h | post-merge-smoke |
| 123 | AC-70 denial wording | `just test` | T-10 | *wrong server* / *wrong role*, never membership | pre-merge-runtime |
| 124 | AC-101 admin channel | `just test` route-table test; repo scan | T-25 | Both routes accept the right token, refuse missing/wrong with no information; admin prefix served to the token check alone; no secret-shaped literal or planted value in the repo | pre-merge-runtime |
| 125 | AC-101 canary plant | `just canary` | T-25 | Token absent from every payload, page and log line | pre-merge-runtime |
| 126 | AC-101 live push | Client's laptop to the deployed server | T-20 | One push succeeds before the first real batch is scheduled | post-merge-smoke |

126 rows over 104 criteria: 8 static, 74 runtime, 44 smoke. Every `felt`, `external-oracle`
and `operator-assisted` row is smoke-side; none is converted to a unit test. Rows 6, 8,
29, 30, 33, 45, 51, 55, 63, 68, 70, 71, 78, 82, 86, 91, 104, 106, 111, 112, 115 and 117
form the client's smoke checklist as written.

**Two honesty notes on the tagging.** Rows 49, 57 and 98–102 assume the Result Validator
may drive the c11 browser against a local server started from the PR head; if that
approval is not given, they become smoke rows. Rows 66, 79–81 for T-03 are smoke rows
because the spike measures on Fly; T-03's PR carries the numbers as an artifact and the
validator reads them.
