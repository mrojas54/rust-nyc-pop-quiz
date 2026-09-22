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
| `schedule` | Push one affirmed question to the room; pull the used ledger back | T-20 |

`generate`, `verify`, `dedupe`, `review`, `audit` and `schedule` are empty stubs.

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
