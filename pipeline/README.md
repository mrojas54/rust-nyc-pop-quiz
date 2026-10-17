# pipeline

The offline half. Generate candidate questions, verify them against a real
toolchain, drop duplicates, put them in front of an organizer, and keep a
growing private bank. Nothing here runs inside a room, and a room runs with this
package absent from the environment (AC-1).

There is no deadline on any of it. Freshness is a supply problem, not a runtime
problem.

## Running it

From the repository root:

    just setup     # once, and after any dependency change
    just test      # the pipeline's tests, along with the room's and the web's

Or just this project's tests:

    cd pipeline && uv run pytest

Python 3.12, managed by `uv`. `.python-version` and `uv.lock` are both committed,
so the interpreter and every dependency are the same everywhere.

## What is here

| Module | What it does, or will do | Ticket |
|---|---|---|
| `bank` | The record format (`SPEC.md` §3.1–3.3): the types, the derived answer, the store, the history shape | T-14 |
| `receipt` | `SPEC.md` §7.5's receipt as a pure function of a `verified` record | T-14 |
| `migrate_mvp` | One-shot: the August batch into the bank as `legacy` records | T-14 |
| `runner` | The seam the verifier reaches a toolchain through | T-01 |
| `generate` | Candidates in the bank's shape, with a run report, from the Message Batches API or from headless `claude -p` on a subscription (`generate_cli`); proven on fixtures and a fake `claude`, neither yet run for real | T-16 |
| `verify` | Pinned `rustc`, N native runs, Miri — the machine decides the answer | T-15b |
| `dedupe` | Exact, normalized, and near-duplicate detection | T-17 |
| `review` | The organizer's screen: accept, reject, edit, affirm | T-18 |
| `audit` | `bank-audit` — uniformity in both tails, the enumerated tells, fit | T-19 |
| `schedule` | `popquiz schedule`: the affirm gate, the date's arrangement, the push, and the static fallback with its host sheet; `popquiz sync`: the used ledger pulled back; the reserve count, trend and warning | T-20 |

## Reviewing

`python -m popquiz.review <command> <id> --bank DIR` records the organizer's
decisions on one question. It is the only writer of a review decision; the
organizer TUI's review screen (`v`, `prototypes/organizer-tui`) runs it as a
subprocess. `--bank` has no default.

| Command | What it records |
|---|---|
| `show` | Nothing; prints the review report as JSON |
| `accept [--checked-quote SPAN]…` | `status: accepted`. Refused while the answer does not derive, or while quoted text that is not a printed line is unchecked (AC-73) |
| `reject --reason TEXT` | `status: rejected` and the reason; clears any affirmation. `reasons` lists them (AC-19) |
| `difficulty LEVEL` | `difficulty_judged`, 1 to 5 (AC-88) |
| `export --out FILE` / `edit --from FILE` | An edit of the source, choices, hint, explanation or trace notes. Any edit clears the affirmation; an accepted record becomes `status: edited` (still accepted), and a rejected or unreviewed one keeps its status; a source or choice change re-verifies through `verify`, and the bank file is replaced only if the verifier accepts |
| `reverify` | Nothing itself; runs `verify` on the record as it stands |
| `affirm [--by NAME]` | `affirmed_by`, `affirmed_at` and `affirmed_digest`. The name comes from `--by` or `POPQUIZ_ORGANIZER`; there is no default (AC-72, G-12) |

Review never writes `verified` or the trace's `stdout` row; only the verifier
does (AC-7). The digest binds an affirmation to the text it affirmed, so
`schedule` and the reserve refuse a question edited after affirming (GAP-24). A
digest-less affirmation counts on no record, q3's hand affirmation included: it
is shown as unbound, and `affirm` binds it again (RULING PQ-23, 2026-10-12).
Trope warnings come from `copylint` and block nothing (D-23). Exit status: 0 done, 1 refused with one sentence and the
record unchanged, 2 unreadable input.

## Generating

**The generator has never called a model.** It has two backends, chosen with
`--backend`, and neither ever falls back to the other:

- `batches` (the default): the Message Batches API. It needs H-4, an Anthropic API
  key and a spend cap.
- `claude-cli`: headless Claude Code (`claude -p`) on a logged-in subscription. It
  runs one process per candidate and needs a candidate cap.

Everything below has run against recorded fixtures and a fake `claude` only. The
first real run of either is the client's (HC-2).

Against a fixture, from the repository root — no key, no network, no cost:

    just generate --replay pipeline/tests/fixtures/generate/clean.json \
      --count 3 --topic ownership --topic iterators --topic integers --difficulty 2 \
      --out /tmp/popquiz-gen

Each run writes a directory, `<out>/<run id>/`: one candidate per file in the bank's
shape (`SPEC.md` §3.1 without `verified` or `review`), and under `_run/` the run's
state and `report.json` — per candidate its tokens, cost (list price at the batch
discount, labelled an estimate) and wall-clock, every reason a candidate was reported
instead of emitted, and which of count, topics and difficulty the run could not
honour. A fixture run's numbers are labelled *FIXTURE NUMBERS*. The default `--out`
is `bank/candidates/`; nothing is ever written into `bank/questions/`. Candidates go
on to `just verify` (with the `expect` the report records) and dedupe.

Talk mode takes `--talk-title` and `--talk-abstract` in place of `--topic`; each
candidate's topic is the std or core-language concept the model names (AC-4).

The paid run, once H-4 is done, is the same command without `--replay`, with the key
in the environment and a cap:

    ANTHROPIC_API_KEY="$(op read 'op://<vault>/<item>/<field>')" \
      just generate --count 10 --topic ownership --difficulty 2 --cap-usd 5

There is no default cap. A run with none, or whose worst case (every request's
`max_tokens` at the output price, plus its size at the input price, at the batch
discount) is over it, refuses in one sentence before anything is built or sent. The
model is one configured value, `claude-opus-5-5` (the client's ruling of 2026-10-10).
A run that is killed resumes with `--run-id <id>`: it picks up the batch already
submitted and never pays for a second one.

### On the subscription (`--backend claude-cli`)

**This backend has never made a real call.** Its tests run a fake `claude` that
answers in the shape the Claude Code docs describe for `--output-format json`. Log
in once with `claude` (`/login`), then:

    just generate --backend claude-cli --max-candidates 10 \
      --count 10 --topic ownership --difficulty 2

**The billing trap.** If `ANTHROPIC_API_KEY` is set, `claude -p` uses it and bills the
API, not the plan. The generator starts each `claude` with a copy of its own
environment from which every `ANTHROPIC_*`, `CLAUDE_CODE_USE_*` and
`AWS_BEARER_TOKEN_BEDROCK` variable is removed. It also passes flags that load no
settings, CLAUDE.md, hooks, MCP servers, plugins or tools, and it never passes
`--bare`, which can only use the API key. Still, run it from a shell without the
key. At the first run (HC-2), check two things:

- `claude` accepts `--setting-sources ""`;
- the run shows up as subscription usage, not API usage.

`--max-candidates` has no default, and a run asking for more refuses in one
sentence. `--cap-usd` is the batches backend's cap; a `claude-cli` run refuses it,
so nobody believes a subscription run is dollar-capped.

A non-zero exit, an error result (a usage or rate limit, say) or a timeout stops
the run at that candidate. There is no retry. The candidates already finished are
written, and rerunning with the same `--run-id` picks up the rest. Killing the
generator itself (`kill -9`) cannot reach the `claude` it was waiting on. That one
candidate finishes on plan usage, and the rerun asks for it again.

The report gives each candidate's tokens, `duration_ms` and the CLI's
`total_cost_usd`. The dollars are labelled API-equivalent, not billed, and they are
never added up.

### The answer is never the model's

The model's own idea of what a program prints is never written as an answer: there
is no field for it, and option order means nothing (the night's order is drawn from
the date). The model drafts the trace's steps, which the organizer checks at review
like the beats; it never writes the last step's `stdout` row. A `stdout` value it
offers is discarded and the report says so, and `just verify` writes the row from
what the program printed. `pipeline/src/popquiz/generate.py`'s docstring has the
reasons.

## Scheduling a meetup

One question a meetup, from the reserve: accepted, affirmed, unused (`SPEC.md`
§3.3, §7.7). From the repository root, with the admin token read from 1Password
into the environment so it never lands in shell history or a file:

    POPQUIZ_ADMIN_TOKEN="$(op read 'op://<vault>/<item>/<field>')" \
      just schedule q3 --date 2026-10-14 --room https://rustnyc-popquiz.fly.dev --out ~/popquiz-night

On a machine without `just`, the same command from `pipeline/` is
`uv run --offline --no-sync python -m popquiz.schedule schedule …` (and `… sync …`
below). The night itself, from scheduling to sync, is `docs/RUNBOOK.md`.

What it does, in order:

1. **Prints the reserve** — how many questions are ready, the trend (accepted and
   waiting for affirmation, not yet reviewed, used in the last 90 days) and a
   warning when fewer than `--threshold` are ready (default: the `--lead-time`,
   two meetups at one question each). `sync` and `reserve` open the same way.
2. **Refuses** a question with no `verified` record, one not accepted at review,
   one missing `affirmed_by` or `affirmed_at` (AC-72, G-12), or one already used
   (G-10). A refusal is a hard error: nothing is sent and nothing is written.
3. **Arranges** the options for that night: the correct one at
   `slot_for_day(date)` — the date and nothing else (AC-23) — and the other four
   shuffled on the date and the question id. The file in `bank/` is not changed.
4. **Pushes** the arranged record to `PUT /admin/questions/<id>` (or
   `/admin/clubs/<club>/questions/<id>` with `--club`, D-26: the record is per
   club, because it is arranged for that club's date) and prints
   `scheduled: new` or `scheduled: replaced`. A refusal prints the room's reason.
   Connection failures, 5xx, 408 and 429 are retried three times with growing
   waits; a 4xx other than 408 or 429 is never retried; a redirect is never followed. The room must be `https`, or
   plain `http` to this machine.
5. **Writes** `<id>.html` (the static fallback) and `<id>.host-sheet.txt` beside
   it, from the same arranged record, so the room and the file agree about where
   the answer sits. Both hold the answer, so `--out` must be outside the
   repository. With `--no-push` (no `--room`, no token) it writes the two files
   alone, for a night the room cannot be reached.

Scheduling writes nothing under `bank/`, and a scheduled question still counts in
the reserve: it is *used* when its room is released, not before (G-10, AC-92).

**After the meetup, sync before anything restarts the room.** The room keeps its
used ledger in memory, so a restart before a sync loses the night's record:

    POPQUIZ_ADMIN_TOKEN="$(op read 'op://<vault>/<item>/<field>')" \
      just sync --room https://rustnyc-popquiz.fly.dev

`sync` writes each released question's `used` block into its bank record and
prints the ledger report: every record synced, and any room whose wall reported a
fit other than *fits* — or none at all (`fit` absent). Ids that are not bank
questions (`smoke-q3`, `burst-q3` from the deployed test runs) are listed and
skipped. A question the ledger names under a second room is refused for that
record, and the run exits non-zero; so is a record already synced from the same
room whose details differ from the ledger's. Running it twice changes nothing.

**Today one committed question can be scheduled.** q3 is accepted and affirmed
through `popquiz.review`, its affirmation bound to its text (PQ-23), and the
reserve reads 1 ready (`docs/RUNBOOK.md`, *The reserve*, has the snapshot). The
other three records in `bank/questions/` are unreviewed and unaffirmed, so
`schedule` refuses them until an organizer accepts and affirms one through
`popquiz.review` (*Reviewing*, above).

## The bank record, in one paragraph

One JSON file per question under `bank/questions/`, typed by `bank.py`'s frozen
dataclasses and published as `bank/schema/question.schema.json`. Two properties are
worth knowing before you touch it. **There is no `correct` field** — `SPEC.md` §3.1
marks the answer derived (G-2), so `bank.correct_index()` computes it from
`verified` and there is nowhere to hand-edit one in (AC-7). And **a field nobody
observed has no key at all**, rather than a key holding `null`: for the migrated
`legacy` records that absence is the whole proof that no target triple, flag set or
`-Vv` was back-filled (D-16). `bank/README.md` has the rest, including which of the
eight MVP questions are in the bank and why the other four are not.

## The receipt lives in two languages

`SPEC.md` §7.5 wants one function, and the wall and take-it-home need it in
JavaScript (T-05). So the cases are data — `bank/fixtures/receipts/*.json`, each
holding a `verified` record and the lines it must render — and both implementations
test against the same files. `receipt.py` imports only the standard library and
`popquiz.bank`, which is how AC-13 is proven structurally: the receipt renders with
no toolchain, no filesystem and no clock anywhere in its reach.

## The `Runner` seam

`just test` has sixty seconds and no toolchain, so the verifier never calls
`rustc` or Miri directly. It calls a `Runner`:

- **`StubRunner`** replays outputs recorded once from the real toolchain, which
  is how the verifier's decision logic gets proven in the inner loop.
- **`RealRunner`** executes inside the sandbox image. T-15a builds the image and
  defines the toolchain pin; T-15b writes the runner. Until then it raises.

`just test-full` re-runs the same cases against the real toolchain, so a
recording that has drifted from what the toolchain now does surfaces as the two
hooks disagreeing rather than as a quiet pass.

**A recorded output names the toolchain it came from, or it is refused.** That
field is not bookkeeping: nobody on this project writes down what a program
prints, and an output with no run behind it is indistinguishable from an
invented one. See `tests/fixtures/README.md`.
