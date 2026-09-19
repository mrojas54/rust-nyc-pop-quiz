# Build plan — Rust NYC Pop Quiz

**Status: Stage 3 `tone-architect`, Phase 3. Drafted 2026-09-03; T-21 answered
the same day — *"rust on fly, i'll pay, D-H is fine."* The decisions in §1
are decided.** Reads with `SPEC.md` (what) and
`EVALUATION.md` (how judged). Consumed by `lattice-orchestrator`.

---

## 1. Decisions — options weighed, recommendation, and who decides

### D-A · Where the room runs, and in what language — **the taste fork**

| Option | For | Against | Yearly cost |
|---|---|---|---|
| **1. Rust — `axum` + `tokio` on Fly.io** (one small machine, in-memory room state, SQLite for organizer tokens and the used-question ledger) | It is Rust NYC's tool, in Rust, readable and hackable by the room. The whole live problem under D-7 is *one room, 200 sockets, one burst* — a single process holds that in a `Vec` and the "highest-risk moment" becomes a loop over an in-memory map. No platform-specific unknowns (Dimension 5). Fly does TLS, domains, and scale-to-zero. | A server to run: a Dockerfile, a deploy, a machine that can be down. Cold start on scale-to-zero (seconds, at room creation — acceptable). Cost is not zero and one organizer absorbs it. | **~$0–60** (scale-to-zero to one shared-cpu machine always on) |
| 2. TypeScript — Cloudflare Workers + Durable Objects | The research's lean: one DO per room is a textbook fit, WebSocket hibernation, SQLite-backed, no server, free tier covers it many times over. | TypeScript for a Rust meetup's segment. DO-specific APIs and local dev (`wrangler`); the client's project becomes one she cannot read as easily. | **~$0** |
| 3. Val Town | — | Rejected by the research: undocumented concurrency ceiling, single-writer SQLite under the burst, $252/yr. | ~$252 |
| 4. PartyKit | DO with a friendlier SDK | Same language objection; smaller project, less certain lifespan | ~$0 |

**Decided 2026-09-03 (touchpoint T-21): option 1, Rust on Fly.io.** *"rust on fly."* This reverses Dimension 5's lean
and the reason is D-7: the research sized a three-question, 15-minute room
with ~327k requests; the segment is now one 30-second burst and five minutes
of life, and for that the smallest mechanism (§8) is one process. Durable
Objects buys resilience the problem no longer needs at the price of a language
the room does not speak. The burst is proven either way by the spike (T-03)
before anything depends on it. It was the client's call — a taste call about
whose project this is — and she made it.

### D-B · The front-end — wall, buzzer, host phone

| Option | For | Against |
|---|---|---|
| **1. Plain HTML/CSS/JS, ported from the prototype**, design-system tokens and fonts vendored | The prototype *is* the design and it is already plain JS: `proto.js` is the component library (source well, colour, trace, type model). One-to-one reproduction is a port, not a rewrite. No build step, three pages, testable with Playwright. | No component framework; the design system's React components (`SourceCode.jsx`, `ChoiceButton`) are not used as code, only as reference — the tokens are. |
| 2. React + Vite with the design system's components | The house component set as shipped | A build for three pages; `SourceCode` must be forked anyway (AC-99 diverges from it); nothing in the loved prototype is React. |

**Recommendation: option 1.** Decide-and-log unless the client objects.

### D-C · The pipeline runtime and the sandbox

| Decision | Recommendation | Alternatives |
|---|---|---|
| Language | **Python 3.12, `uv`-managed**, inheriting `mvp/tools/verify.py` and `build_deck.py` after audit. The pipeline is glue around `rustc`, Miri, and an LLM. | A Rust CLI — more work, no benefit. |
| Sandbox (AC-12) | **Docker**: an image built once with the pinned toolchain and nightly Miri; run with `--network none`, memory/CPU/pids limits, read-only root, tmpfs work dir, a hard timeout. Works on the client's Mac. | Modal Sandbox (sponsored, isolation undisclosed); bubblewrap (Linux only). |
| Bank storage | **JSON files in `bank/` in this private repo** — reviewable in PRs, diffable, no server; `bank-audit` is a script over them. | SQLite — nothing here needs a query engine. |
| Review surface | **A local page served by the CLI** (`popquiz review`) that writes back to the bank | A static page can render but not affirm. |

Decide-and-log.

### D-D · The generator's model

**Claude Opus 5 (`claude-opus-5`) through the Anthropic Python SDK, via the
Message Batches API** — the pipeline is offline, so batch's 50% discount is
free money, and structured outputs give the §3.1 record shape directly.
Adaptive thinking on. List price is $5 in / $25 out per million tokens; a
candidate at ~2k in / ~2k out is roughly **five cents at batch rates**, so a
run of 40 candidates is a couple of dollars. **These are estimates; AC-5's
per-candidate report replaces them at the first run.** Spend cap in config.
Decide-and-log; the client supplies the key and the cap.

### D-E · How a question gets from the bank to the room, and back

The laptop holds the bank; the server holds one question at a time.
`popquiz schedule <qid> --for <date>` pushes the question record (answer
included — the server needs it to reveal, behind AC-61's boundary) to the
room server and writes the static fallback file. At **release** the server
writes `used` and rebuilds take-it-home; `popquiz sync` pulls the `used`
record into the bank. Every field in `SPEC.md` §3 keeps one writer.
Decide-and-log.

### D-F · Domain and the short link (AC-28)

**`popquiz.rustnyc.org`**, with `join @ popquiz.rustnyc.org/‹code›` on the wall.
Needs a DNS record on `rustnyc.org` — **the client supplies it**. Fallback: the
Fly-provided hostname and her placeholder short link.

### D-G · Budget (Q-E1) and who pays

Under D-A option 1 the unsponsored cost is single-digit dollars a month at
most. **Decided 2026-09-03 (touchpoint T-21): the client carries it** — *"i'll pay."*
`ECONOMICS.md` §Who pays is amended.

### D-H · AC-95 — decided as D-9, no longer a client call

The spec judge found the *predicted versus actual* construction incoherent
with the read-aloud beat. `SPEC.md` §4.5 now has **no prediction**: every
incorrect option carries its own short *why you'd pick it* text, review
refuses a question missing one, and at reveal the wall and the host phone name
the room's actual most-chosen incorrect option and read that option's text.
Cheap to reverse; decided and logged. **Client, touchpoint T-21: *"D-H is fine."***

---

## 2. Architecture, as recommended

```
laptop                                   Fly.io (one machine)                   phones / projector
──────                                   ────────────────────                   ──────────────────
pipeline/  (Python, uv)                  room/  (Rust, axum)
  generate  ─► verify (Docker) ─► dedupe   ├─ auth        Discord OAuth, role-ID check at create
  ─► review (local page) ─► bank/*.json    ├─ rooms       phase machine, sessions, totals (in memory)
  ─► bank-audit                            ├─ answers     the sealed module: correct option, receipt,
  ─► schedule ──push question + fallback──►│               explanation — unreachable from public queries
  ◄─ sync ────pull `used` ledger ─────────  ├─ ws          one broadcast per room: wall + buzzers + host
                                           ├─ static      web/  (wall, buzzer, host, take-it-home)
                                           └─ sqlite      organizer tokens, used-question ledger
                                                                                      ▲
web/  (plain JS, ported from prototypes/)                                            │ WebSocket
  shared/  source well · colour · trace · type model · tokens.css · fonts             │
  wall/  buzzer/  host/  home/                                              wall ◄────┴────► buzzers, host phone
```

Merge-friendly boundaries: `web/shared/` and `room/src/phase.rs` are consumed
by three tickets each and are built **first** and **alone** (T-02, T-04a).
Three hand-edited aggregators exist — `justfile`, `pyproject.toml`, the CI
config — and every ticket that touches one says so in its Notes column and is
serialized on it by the orchestrator. Names checked: Rust modules `auth`, `rooms`,
`answers`, `ws`, `phase` (no `type`, `match`, `ref`); Python package
`popquiz` with modules `generate`, `verify`, `dedupe`, `review`, `audit`,
`schedule` (no `import/`, no `test` as a package).

**Brownfield reconcile** (`mvp/tools/`, audited by the inventory):

| Item | Status | Where |
|---|---|---|
| Pinned rustc, N=5 byte-identical runs, compile-failure with error code, `verified.json` written by code | DONE | `verify.py:166-216` |
| Miri in the verifier | **MISSING** — never called in code; the on-disk `miri` fields came from an out-of-repo pass | `verify.py` (no `miri` invocation); `README.md:193-195` |
| Sandbox | MISSING | — |
| `slot_for_day` pure, both-tails audit, repeat check | DONE — inherit | `build_deck.py:162-217` |
| Used-question record at build time | **DEFECT** — retire (G-10) | `build_deck.py:246-259` |
| Participant-mode leak assertions | PARTIAL — precedent for `canary` | `build_deck.py:347-352` |
| Three-beat explanation, trace, structured receipt | MISSING in the bank (present only in `prototypes/_shared/data.js`) | `content.json`, `data.js:62-129` |
| Python pin, tests, CI | MISSING | — |

---

## 3. The path — milestones, checkpoints, tickets

Ticket size is half a day to a day. Each carries its criteria IDs and its
harness hook (`EVALUATION.md`). `HC-n` are the human-use checkpoints; nothing
user-facing is done on green tests alone.

### M0 · Foundations and the spike

| # | Ticket | Criteria | Depends on | Notes |
|---|---|---|---|---|
| **T-01** | Repo scaffold: `room/` in whichever stack **H-1 decided** (Rust crate, or a Workers project), `pipeline/` (uv project, Python 3.12), `web/`, `bank/`, `justfile` with `test` (≤60 s, hermetic) and `test-full`, CI running both. States what `canary` runs as in `test`: an **in-process** harness (the router driven without a socket), with the deployed scan in `test-full` | — | **H-1** | Shared files: `justfile`, `pyproject.toml`, CI config |
| **T-02** | `web/shared/`: tokens and fonts vendored from the design system; port `proto.js` — source well, syntax colour, trace renderer, the type model with measured refit and the clipped-edge, the phase strings from `SPEC.md` §11 | AC-99, AC-100, AC-40 | T-01 | Built alone; three tickets consume it |
| **T-03** | **Spike — the burst.** Minimal `axum` WebSocket room with 200 synthetic clients; the deadline burst in isolation; p95 write and reveal-fan-out measured on a deployed Fly machine. Go/no-go on D-A. | AC-52, AC-53, AC-54, AC-41 | T-01 | **Blocks M1.** If p95 fails, D-A option 2 is re-opened before any room code is written. |

### M1 · The walking skeleton → **HC-0**

| # | Ticket | Criteria | Depends on | Notes |
|---|---|---|---|---|
| **T-04a** | The phase machine: `idle→…→released`, host-only transitions, no skipping, `trace_step`; **the sealed `answers` module** unreachable from the public state query, proven by a type/module boundary test | AC-45, AC-47, AC-61, AC-81, AC-93, AC-97, G-3, G-6 | T-03 | Built alone and first; `room/src/phase.rs` is consumed by everything in M1 |
| **T-04b** | Sessions and the answer store: join, capacity, answer upsert while live, refusal after close, totals frozen at close, sessions dropped at release | AC-30, AC-34–37, AC-46, AC-56 | T-04a | |
| **T-04c** | Transport: one broadcast per room over WebSockets, reconnect with the same session token, host and wall subscriptions | AC-37, AC-41, AC-81 | T-04a | |
| **T-05** | The wall: seven phases on the 1120×630 canvas, full-width source with options beneath, type model, split bars, reveal with **the receipt rendered by the one approved-wording function** (§7.5), released with link + QR, no interaction | AC-33, AC-39, AC-40, AC-71, AC-74, AC-78, AC-79, AC-99, AC-100, G-7 | T-02, T-04a–c | |
| **T-06** | The buzzer: join by link or code, six failure states, capacity refusal, letters, saving/saved/failed, reconnect, the hint hidden in the live payload, the count on split/work/reveal, the released line | AC-28–32, AC-34–38, AC-48, AC-58, AC-83, AC-85, AC-94 | T-02, T-04a–c | |
| **T-07** | The host phone: one screen per phase, one primary action, counts, trace stepping, the read-aloud script with the receipt's provenance line (G-7), resume on refresh/device change | AC-45–47, AC-49, AC-50, AC-71, AC-74 | T-02, T-04a–c | |
| **T-08** | `canary`: plant secrets in answer, explanation, receipt, hint-before-live; scan every payload, frame, and page in every phase; post-close traffic carries no answer | AC-32, AC-47, AC-48, AC-58, AC-60, AC-79, G-3, G-4, G-8 | T-04a–07 | In-process scan in `test`; the deployed-room scan in `test-full`. Serialized on `justfile` |
| **T-09** | Deploy: Fly app, `popquiz.rustnyc.org` (or fallback host), the short link carrying the code, `smoke` | AC-28 | T-05–07 | Human track: DNS. Serialized on `justfile` (`smoke`) |

**HC-0:** the client drives the deployed skeleton beside the prototype through
all seven phases with mock participants. Drift from the prototype is a defect.

### M2 · Real rooms

| # | Ticket | Criteria | Depends on | Notes |
|---|---|---|---|---|
| **T-10** | Discord OAuth: scopes, member lookup, role-**ID** check at room creation, the Administrator test, refresh-token rotation persisted, bounded retries, denial wording | AC-64–70, G-9 | T-04a–b | Human track: app id, secret, guild id, role id |
| **T-11** | Lifecycle: 4 h expiry, room deleted at release, totals with expiry, **`used` written at release**, take-it-home rebuild at release, *Run it again* refuses a used question | AC-56, AC-57, AC-69, AC-92, G-4, G-10 | T-04b | **The only writer of the used-question ledger.** |
| **T-12** | Take it home: the last question with colour, the free-stepping trace, three beats, receipt, options with ✓, container-scroll on a phone | AC-33, AC-95 (count from release), `SPEC.md` §13 | T-02, T-11 | |
| **T-13** | `a11y`: keyboard, live regions per phase and the private hint, AA contrast, 44 px, reduced motion, over every surface and phase | AC-82–86 | T-05–07 | Serialized on `justfile` (`a11y`) |

### M3 · The pipeline → **HC-2**

| # | Ticket | Criteria | Depends on | Notes |
|---|---|---|---|---|
| **T-14** | Bank format (`SPEC.md` §3.1–3.3) and migration of the eight MVP questions: verified facts carried over, `explains` three beats and `trace` drafted for organizer affirmation, `explanation` kept as `legacy` | AC-13, AC-17, AC-73 | T-01 | **Only q3 migrates as authored** (touchpoint T-22, D-15). q4, q7 and q8 fit as programs but their options must be re-authored to one line of at most 29 characters and re-verified (the machine decides the new answer). q1, q2, q5 and q6 exceed the wall's 7-line capacity at the guessed room and stay in the MVP bank only, until re-authored or until the rehearsal's measurements raise the capacity. |
| **T-15a** | The sandbox: a Docker image with the pinned toolchain and nightly Miri, run with no network, memory/CPU/pids limits, read-only root, tmpfs work dir, hard timeout; the AC-12 fixture suite proven on it | AC-12 | T-01 | Serialized on `justfile` |
| **T-15b** | The verifier in code: inherit `verify.py`'s procedure after audit; **add Miri** (strict provenance, Tree Borrows for UB-intended), the flag set, target triple, the §3.2 record; `verify <program>` | AC-6–11, AC-13, AC-87, G-2 | T-14, T-15a | Audit ticket for `verify.py`. Serialized on `pyproject.toml` with T-16, T-17 |
| **T-16** | The generator: Claude Opus 5 via Message Batches, structured outputs into the §3.1 shape, count/topics/difficulty honoured or reported, talk mode, per-candidate cost and wall-clock, re-runnable | AC-1–5 | T-14 | Human track: API key, spend cap. Serialized on `pyproject.toml` The generator's brief carries the room's capacity — source lines at the floor, each option a single line, at most 29 characters (`SPEC.md` §7.1, D-15). |
| **T-17** | Dedupe: exact hash, normalized-AST fingerprint, near-duplicates by token-bigram Jaccard at the configured threshold (D-13) to the review queue, persistent history with visible size | AC-14–18 | T-14 | Serialized on `pyproject.toml` |
| **T-18** | Review surface (local, served by the CLI): one screen per candidate, accept/reject-with-reason/edit-and-reverify, difficulty judged, **affirm** as a blocking gate with who/when, middle-beat and quoted-output checks | AC-19–22, AC-72–74, AC-88, AC-95, G-12 | T-14, T-15b | |
| **T-19** | `bank-audit`: inherit `slot_for_day` and the both-tails audit after audit; **retire** `build_deck.py`'s ledger write; the five enumerated tells at 1.5× chance; `unsafe` parity; five options with one *does not compile*; no published distribution; fit against the configured room | AC-23–27, AC-100, G-1, G-11 | T-14, T-02 (type model) | Audit ticket for `build_deck.py` — G-1 has regressed four times. **Port `slot_for_day` only.** `slot_for_meetup`, the wrapper one function below it that writes the ledger, is **not** ported under any name; the ledger's only writer is T-11's release transition. Serialized on `justfile` (`bank-audit`) Adds the **option-length** rule: every option one line of at most 29 characters (`SPEC.md` §5.2, D-15). |
| **T-20** | `popquiz schedule` / `sync`: push the affirmed question and the **static fallback** to the server; pull `used`; reserve count, trend, and the low-reserve warning; a meetup from reserve with no generation | AC-75–77, AC-89, AC-92, `SPEC.md` §12, G-12 | T-11, T-18, T-19 | |

**HC-2:** the client reviews the first generated batch on the review surface,
stopwatch running, reads each explanation aloud, affirms what passes.

### M4 · Hardening → **HC-1, HC-3, HC-4**

| # | Ticket | Criteria | Depends on | Notes |
|---|---|---|---|---|
| **T-21** | `burst` and `smoke` in `test-full`, run against the deployed room in CI; failed-request logging the client can read after a meetup | AC-41, AC-52–55 | T-09 | Serialized on `justfile` (`burst`, `test-full`) |
| **T-22** | The copy freeze: every participant-facing string from `SPEC.md` §11 in one module; the forbidden-copy lint in `test` | AC-98, AC-59, G-5 | T-05–07, T-12 | |
| **T-23** | **Guardrail audit** — an adversarial read of all merged code against G-1…G-12 and the phase invariants; files gap tickets | G-1…G-12 | everything above | The Phase-4 pass, re-run on the built tree |
| **T-24** | The organizer runbook: successor to `mvp/README.md`'s run-of-show for the built room; host script per phase; the statements that option text is public and that a Rust-expert host can infer the answer; the rehearsal and field-notes sheets pointed at the build | AC-51, AC-62, AC-63, AC-90 | T-09, T-20 | |

**Settled at a checkpoint, no ticket, by design:** AC-20, AC-21, AC-42, AC-43, AC-44, AC-80, AC-91, AC-96 are `felt` or operator-measured and land at HC-1, HC-2, and HC-4 (`EVALUATION.md`). Their absence from the ticket tables is a decision, not an omission.

**HC-1** (the rehearsal, touchpoint T-18, on the client's calendar) runs on the built room
if M1 has passed HC-0, else on the prototype; it sets AC-100's room
configuration. **HC-3** the client opens take-it-home on the train. **HC-4**
is October.

### Minimum viable cut

If October arrives before M3: **M0 + M1 + M2, plus T-14 (the MVP questions that
fit the wall: q3 as authored, and q4, q7 and q8 after re-authoring), T-18 (affirm — G-12), T-19 (the slot mechanism and
`bank-audit` — G-1, G-11), T-22 (the copy lint — G-5), and T-20's push half.**
That is the smallest set that meets every non-negotiable; generation (T-16),
dedupe (T-17), the sandbox (T-15a) and the in-code verifier (T-15b) wait, and
those questions carry their August verification with the Miri caveat
stated on the receipt. **Supply is short until the generator refills it:** one
question a night from at most four questions is four meetups, not eight months
(D-7), which is exactly the job the generator, now told the wall's limits, exists to do. If October arrives before M2: the prototype's
static fallback (§12) built from T-02 and T-05, driven from a laptop — the
built wall with no phones, which is the MVP deck with the new phases.

---

## 4. The human track

The client's work plan, in dependency order. None of it goes to the fleet.

| # | Do | Unblocks |
|---|---|---|
| ~~H-1~~ | Decided 2026-09-03: Rust on Fly; the client pays | — |
| H-2 | DNS: `popquiz.rustnyc.org` → the app | T-09 |
| H-3 | Discord: create the application, record app id, secret, guild id, organizer role id | T-10 |
| H-4 | An Anthropic API key and a monthly cap | T-16 |
| H-5 | Set a rehearsal date and venue (touchpoint T-18); take the three measurements | HC-1, AC-100 config |
| H-6 | Drive HC-0 against the prototype | M2 |
| H-7 | Review and affirm the first batch (HC-2); time it | AC-21, AC-72, October's question |
| H-8 | ~~Confirm D-H~~ decided as D-9; nothing to do | — |
| H-9 | Cole, on the colour — not blocking | — |
| H-10 | Host in October with a co-organizer holding `FIELD-NOTES-TEMPLATE.md` | HC-4 |

---

## 5. Risks that the sequence retires early

| Risk | Retired at |
|---|---|
| The burst (AC-54) — the one real engineering risk | T-03, before any room code |
| Drift from the loved prototype | HC-0, before auth or pipeline work |
| The room measurements are wrong and the bank does not fit | HC-1 sets them; T-19's fit flag reports the bound |
| Miri wall-clock and cost unknown | T-15 measures on the first migrated question |
| Discord specifics unverified (member shape, approval for the scope) | T-10 against the client's real account, first thing |
| G-1 regresses a fifth time | T-19's audit ticket; the simulation in `test` on every PR touching the slot path |
| October arrives first | The minimum viable cut above |
