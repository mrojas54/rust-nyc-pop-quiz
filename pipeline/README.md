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

| Module | What it will do | Ticket |
|---|---|---|
| `generate` | Candidates from an LLM, in the bank's shape | T-16 |
| `verify` | Pinned `rustc`, N native runs, Miri — the machine decides the answer | T-15b |
| `dedupe` | Exact, normalized, and near-duplicate detection | T-17 |
| `review` | The organizer's screen: accept, reject, edit, affirm | T-18 |
| `audit` | `bank-audit` — uniformity in both tails, the enumerated tells, fit | T-19 |
| `schedule` | Push one affirmed question to the room; pull the used ledger back | T-20 |
| `runner` | The seam the verifier reaches a toolchain through | T-01 (this) |

Every module but `runner` is an empty stub today.

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
