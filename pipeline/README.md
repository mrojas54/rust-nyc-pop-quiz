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
| `generate` | Candidates from an LLM, in the bank's shape | T-16 |
| `verify` | Pinned `rustc`, N native runs, Miri — the machine decides the answer | T-15b |
| `dedupe` | Exact, normalized, and near-duplicate detection | T-17 |
| `review` | The organizer's screen: accept, reject, edit, affirm | T-18 |
| `audit` | `bank-audit` — uniformity in both tails, the enumerated tells, fit | T-19 |
| `schedule` | `popquiz schedule`: the affirm gate, the date's arrangement, the push, and the static fallback with its host sheet; `popquiz sync`: the used ledger pulled back; the reserve count, trend and warning | T-20 |

`generate` and `review` are empty stubs.

## Scheduling a meetup

One question a meetup, from the reserve: accepted, affirmed, unused (`SPEC.md`
§3.3, §7.7). From `pipeline/`, with the admin token read from 1Password into the
environment so it never lands in shell history or a file:

    POPQUIZ_ADMIN_TOKEN="$(op read 'op://<vault>/<item>/<field>')" \
      uv run --offline --no-sync python -m popquiz.schedule schedule q3 \
        --date 2026-10-14 --room https://rustnyc-popquiz.fly.dev --out ~/popquiz-night

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
4. **Pushes** the arranged record to `PUT /admin/questions/<id>` and prints
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
      uv run --offline --no-sync python -m popquiz.schedule sync --room https://rustnyc-popquiz.fly.dev

`sync` writes each released question's `used` block into its bank record and
prints the ledger report: every record synced, and any room whose wall reported a
fit other than *fits* — or none at all (`fit` absent). Ids that are not bank
questions (`smoke-q3`, `burst-q3` from the deployed test runs) are listed and
skipped. A question the ledger names under a second room is refused for that
record, and the run exits non-zero; so is a record already synced from the same
room whose details differ from the ledger's. Running it twice changes nothing.

**Today no committed question can be scheduled.** Every record in `bank/questions/`
is unreviewed and unaffirmed, so `schedule` refuses all four until an organizer
affirms one (T-18 builds that step).

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
