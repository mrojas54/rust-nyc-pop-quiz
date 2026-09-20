# PQ-1: Repo scaffold, justfile, CI

BUILDPLAN.md T-01 (M0).

Repo scaffold: `room/` in whichever stack **H-1 decided** (Rust crate, or a Workers project), `pipeline/` (uv project, Python 3.12), `web/`, `bank/`, `justfile` with `test` (≤60 s, hermetic) and `test-full`, CI running both. States what `canary` runs as in `test`: an **in-process** harness (the router driven without a socket), with the deployed scan in `test-full`. States that `test` never runs the pinned `rustc` or Miri as a verification step and never Docker (building the room crate uses the machine's own `cargo`) — the verifier is reached through a stub `Runner` with recorded fixtures (`SPEC.md` §7.2, D-18) — and that `test-full` re-runs those cases on the T-15a image, so CI installs Docker for `test-full` only

Criteria: —
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): **H-1**
BUILDPLAN notes: Shared files: `justfile`, `pyproject.toml`, CI config

Orchestrator notes: Stack is decided: Rust `axum` crate in `room/`, not a Workers project (the row text is stale on that point, finding F-6). CI defines the `test-full` job and runs what exists; T-15a and T-21 extend it (F-12). Measure `just test` cold and warm and report both numbers in the completion comment; if the warm run exceeds 60 s that is a contract defect, stop and escalate (F-3).

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

---

# Plan (delegator, 2026-09-20)

Base: `origin/main` @ `7abfce0`. Worktree `…/rust-nyc-pop-quiz-worktrees/repo-scaffold`,
branch `ai-c11-cc/repo-scaffold`.

## Install facts on this machine

| Tool | Found | Version |
|---|---|---|
| `cargo` / `rustc` | `~/.cargo/bin` | 1.96.1 (31fca3adb 2026-06-26) |
| `just` | `/opt/homebrew/bin/just` | 1.57.0 |
| `uv` | `/opt/homebrew/bin/uv` | 0.12.1 |
| `node` | nvm v22.22.3 | 22.22.3 (has `node --test`) |
| `python3` | pyenv shim | 3.13.3 — **not** 3.12; `uv` supplies 3.12 |
| `docker` | `/usr/local/bin/docker` | present, not used by `test` |
| `gh` | `/opt/homebrew/bin/gh` | present |

Sandbox notes (recorded so the Orchestrator is not surprised): `.lattice/` writes,
the `c11` socket, `git fetch`/`push`, `~/.cargo/registry` and `~/.cache/uv` all sit
outside the Bash sandbox's write allowlist, so those commands run with the bypass.
No sandbox relaxation is requested for anything the ticket does not need.

## The two contract tensions, and how this plan resolves them

**T-1 · Hermetic vs. cold.** The ticket asks for `just test` to be hermetic with
*no network*, and also to be measured **cold, after `cargo clean` and a fresh
`.venv`*. A fresh `.venv` means downloading `pytest`; a cold `cargo` means
populating the registry. Those cannot both hold in one recipe.

Resolution: one extra, non-reserved recipe **`setup`** owns the single network
step (`uv sync --frozen`, `cargo fetch --locked`). `test` is then provably
offline — `uv run --offline --no-sync`, `cargo test --offline`, `node --test`
(no deps at all) — and the 60 s budget measures the inner loop it is actually
about: compile + run. Cold is measured as `cargo clean` + `rm -rf .venv` +
`setup` (timed, reported separately, network) then `test` (timed, offline).
Warm is `test` on a hot target dir and venv. **All three numbers** go in the
completion comment and the PR body. This is a deviate-with-flag: it is the only
reading under which "hermetic" is a property the recipe can be held to.

**T-2 · Recorded fixtures vs. house rule 8.** The ticket wants a `StubRunner`
that "replays recorded fixtures"; `SPEC.md` §7.2 says each fixture is "recorded
once from the real toolchain on the T-15a image". T-15a does not exist, and
house rule 8 forbids writing down what a program prints. So T-01 ships the
*replay mechanism* with a fixture that is **visibly synthetic** — a
`scaffold-placeholder` program whose recorded stdout says so in words — and a
`pipeline/tests/fixtures/README.md` stating that real recordings land with
T-15b, from T-15a's image, by running them. No fixture in this PR resembles a
Rust program's output, so nothing here can be mistaken for a recording.

**T-3 · `test-full` green vs. stubs exiting non-zero.** The ticket says a stub
recipe "prints which ticket delivers it and exits non-zero", *and* that
`test-full` must be green. So `test-full` must not invoke the stubs as
dependencies. It runs `test`, then prints a loud `PENDING` block naming each
unbuilt suite and its ticket, and exits 0. Ticket mapping lives in **one**
justfile variable that both the stub recipe and `test-full` read, so the two
cannot drift.

## Files to create

```
justfile                                 the eight reserved recipes + setup, fmt-check
.github/workflows/ci.yml                 job `test` (every PR/push to main), job `test-full`
README.md                                (edit) a "Building and testing" section

room/Cargo.toml                          package `room`, axum + tokio; NOT a workspace
room/Cargo.lock                          committed
room/src/lib.rs                          `pub fn router() -> axum::Router` — the seam
room/src/main.rs                         binds and serves `router()`; no room behaviour
room/tests/canary.rs                     in-process: drives `router()` with no socket
room/README.md                           what the crate is, how to run its tests

pipeline/pyproject.toml                  package `popquiz`, requires-python == 3.12
pipeline/uv.lock                         committed
pipeline/.python-version                 3.12
pipeline/src/popquiz/__init__.py
pipeline/src/popquiz/{generate,verify,dedupe,review,audit,schedule}.py   empty modules w/ docstring
pipeline/src/popquiz/runner.py           Runner protocol, StubRunner, RealRunner
pipeline/tests/test_runner.py            replay works; RealRunner refuses; no shell-out path
pipeline/tests/fixtures/README.md        why the fixture here is synthetic
pipeline/tests/fixtures/scaffold-placeholder.json
pipeline/README.md

web/{shared,wall,buzzer,host,home}/.gitkeep
web/test/layout.test.js                  node --test; asserts the five dirs exist
web/README.md                            plain classic scripts (see "web runner" below)

bank/README.md                           what lands here (T-14), and that it is private
```

`.gitignore` already carries `target/`, `.venv/`, `__pycache__/`, `.pytest_cache/`,
`.ruff_cache/`, `dist/`, `node_modules/`. **Adds:** `.lattice/`, `HANDOFF.md`,
`*.orig`/`*.rej`, and `uv.lock`'s sibling noise is already covered. Does **not**
touch `.env.example`, `mvp/`, `prototypes/`, or any contract file.

## The justfile

Single source of truth for pending suites:

```
PENDING := "verify:T-15b bank-audit:T-19 burst:T-21 a11y:T-13 smoke:T-09"
```

| Recipe | Runs | Green today |
|---|---|---|
| `setup` | `uv sync --frozen` (in `pipeline/`), `cargo fetch --locked` (in `room/`). The only recipe allowed network. | yes |
| `test` | Three suites **in parallel** (background jobs, per-suite output buffered to `$TMPDIR` and replayed in order, exit codes collected): `cargo test --offline` in `room/` (includes `tests/canary.rs`), `uv run --offline --no-sync pytest` in `pipeline/`, `node --test web/test/` | yes |
| `test-full` | `test`, then the `PENDING` block, exit 0. Docker is *available* to this hook (CI installs it) but nothing today uses it. | yes |
| `canary` | `cargo test --offline --test canary` — the in-process seam. Prints one line that the deployed-room scan arrives with T-08 and runs in `test-full` once T-09 deploys. | yes |
| `verify` | `_pending verify` → "`verify` is not built yet — BUILDPLAN T-15b delivers it." exit 1 | no, by design |
| `bank-audit` | `_pending bank-audit` → T-19, exit 1 | no, by design |
| `burst` | `_pending burst` → T-21, exit 1 | no, by design |
| `a11y` | `_pending a11y` → T-13, exit 1 | no, by design |
| `smoke` | `_pending smoke` → T-09, exit 1 | no, by design |

`canary` is not a stub: its suite *does* exist today as a seam, and `test`
includes it (`EVALUATION.md` harness row: the in-process scan is part of `test`).

## The `canary` seam (`room/tests/canary.rs`)

Drives the router with no socket, via `tower::ServiceExt::oneshot` on
`axum::Router` (which is a `tower::Service`), reading the body with
`axum::body::to_bytes`. One trivial assertion — a `GET /healthz` returns 200 —
so T-08 has a place to land its phase-scoped plants. Dev-deps: `tower` (`util`)
only; `axum`/`tokio` are already runtime deps.

## The `Runner` seam (`pipeline/src/popquiz/runner.py`)

```
@dataclass(frozen=True) RunStep: kind, exit_code, stdout, stderr
class Runner(Protocol): def run(self, kind: str, program_id: str) -> RunStep
class StubRunner:  replays a recorded JSON fixture from a directory; KeyError-equivalent
                   with a clear message when a case was never recorded
class RealRunner:  every method raises NotImplementedError("not until T-15a/T-15b")
```

`test` must never reach a path that shells out. Enforced by a test that walks
every module under `popquiz/` and asserts none imports `subprocess` — cheap, and
it makes T-15b's first shell-out a *deliberate* edit to this guard rather than a
silent one. Flagged for the plan reviewer as possibly-too-clever.

## CI (`.github/workflows/ci.yml`)

Triggers: `pull_request`, and `push` to `main`. Default runners (`ubuntu-latest`).

- **job `test`** — `actions/checkout`, `actions/cache` for `~/.cargo` +
  `room/target` (keyed on `Cargo.lock`) and `~/.cache/uv` (keyed on `uv.lock`),
  `actions/setup-node` (22), `uv` + `just` installed by pinned release download
  with a verified `sha256`, then `just setup` and `just test`.
- **job `test-full`** — same, plus Docker (ubuntu runners ship it; the step
  asserts `docker --version` so the dependency is visible), then `just test-full`.

First-party actions only. `just` and `uv` come from pinned GitHub release
tarballs with checksums rather than third-party setup actions — no third-party
action trust, explicit pin. **Fallback if the checksum fetch is blocked at
implementation time:** `cargo install just --locked` behind the cargo cache, and
`uv` from its official installer; whichever lands is stated in the PR body.
CI cannot be run locally; the validation note will say so plainly.

## Open choices (made here, stated for review)

1. **Web test runner: `node --test`** (Node 22's built-in), no `package.json`,
   no `node_modules`, no network. The prototype's JS is *classic scripts* with
   globals (`prototypes/C-projector-first.html` loads `_shared/data.js` and
   `_shared/proto.js` via plain `<script src>`), so `web/` stays classic and the
   scaffold does **not** declare `"type": "module"`. `web/README.md` tells T-02
   that loading a classic script under `node --test` wants `node:vm`, not
   `import`.
2. **No `rust-toolchain.toml`.** It is optional, and adding one (a) invites the
   exact confusion the ticket warns against — it is not the verification pin,
   which is T-15a's — and (b) makes `just test` fetch a toolchain over the
   network on any machine that lacks it, breaking hermeticity. Build with the
   machine's own `cargo`, as the ticket says. Recorded here so T-15a knows the
   slot is free.
3. **`.python-version` lives at `pipeline/.python-version`**, not the repo root,
   because `pipeline/` is the uv project root and the repo has no other Python.
4. **`room/` is a standalone crate, not a workspace**; `Cargo.lock` at
   `room/Cargo.lock`. BUILDPLAN names only three shared aggregators (`justfile`,
   `pyproject.toml`, CI config) — adding a root workspace would make a fourth.
5. **Crate/package names:** Cargo package `room`, Python package `popquiz`
   under `pipeline/src/` (src layout, so tests import the installed package and
   not the working tree).

## Out of scope, and not touched

Room behaviour; wall/buzzer/host markup; bank content; generator or verifier
logic; fonts; tokens; `.env.example`; anything under `mvp/` or `prototypes/`;
every contract file.

## Measurement method

```
# cold
cargo clean --manifest-path room/Cargo.toml && rm -rf pipeline/.venv
/usr/bin/time -p just setup      # network, reported separately
/usr/bin/time -p just test       # the cold number
# warm
/usr/bin/time -p just test       # the warm number, 3 runs, median reported
```

Warm > 60 s ⇒ contract defect ⇒ stop and escalate. Cold > 60 s alone ⇒ note it
and rely on the CI caches, which this plan already configures.

---

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: `general-purpose`, Sonnet, fresh context, given only the plan path, the
boot prompt and the contract section paths. Verdict: no CRITICAL, three MAJOR,
two MINOR, one NIT. All six triaged below; **five accepted, one accepted with an
amendment**. Nothing was dropped.

### R-1 · MAJOR — `requires-python == 3.12` is an exact-match pin (accepted)

**Concern.** The plan's file table specifies `requires-python == 3.12`. Under
PEP 440 a bare `==3.12` matches `3.12` and `3.12.0` and **excludes** `3.12.13`,
which is the interpreter `uv` actually has on this machine. `uv sync` would
refuse it and `just setup` / `just test` would fail — the one outcome this
ticket forbids.

**Resolution.** Accepted; this was a real defect in the plan. `pyproject.toml`
carries **`requires-python = ">=3.12,<3.13"`**, which is a minor-series pin and
the form that matches `.python-version`'s tolerant `3.12`. Verified against the
installed interpreter before the pipeline suite is declared green.

### R-2 · MAJOR — fixture provenance, not just fixture content (accepted, amended)

**Concern.** The plan makes the placeholder fixture "visibly synthetic", which
addresses whether it can be *mistaken* for a recording, but not whether it was
*run or authored*. Ground rule 8 restates house rule 8 broadly: "a test fixture
that hard-codes a program's output is a defect."

**Resolution.** Accepted, and the reviewer's distinction is the useful half. The
rule forbids asserting what a program prints without running it; the scaffold
fixture describes **no program**, so there is nothing whose output is being
written down — but the plan relied on a reader inferring that. Amended so the
file says it itself: every fixture record carries a required
**`recorded_from`** provenance field, and `StubRunner` **refuses to replay a
record whose `recorded_from` is absent**. The scaffold placeholder's value reads
`"not recorded — scaffold placeholder, no program was run"`. T-15b then cannot
add a real fixture without stating the toolchain it was recorded from, and
cannot quietly hand-write one. This is strictly stronger than what the ticket
asked for and costs one field and one guard clause.

### R-3 · MAJOR — hand-rolled parallelism duplicates a `just` feature (accepted)

**Concern.** The plan's `test` recipe backgrounds three shell jobs, buffers
output to `$TMPDIR` and collects exit codes by hand, while `just` 1.57 ships a
`[parallel]` recipe attribute and `--jobs`. Heavier mechanism than the tool
already provides; cuts against PHILOSOPHY §8 on a scaffold ticket.

**Resolution.** Accepted. Independently confirmed by this delegator before the
review returned: three 1 s recipes complete in 1.04 s, and when one sibling
exits 3 the other two still **run to completion** and `just` exits 3. That is
exactly the semantics `test` needs — every suite reports, and one failure fails
the whole recipe. `test` becomes:

```
[parallel]
test: test-room test-pipeline test-web
```

with `test-room` / `test-pipeline` / `test-web` as ordinary recipes. The
`$TMPDIR` buffering is deleted. Cost accepted: per-suite output interleaves
line-wise instead of arriving in blocks.

### R-4 · MINOR — `GET /healthz` is product code (accepted)

**Concern.** Giving `router()` a real health route so `tests/canary.rs` has
something to assert on writes a handler, and the ticket's out-of-scope list ends
"If you find yourself writing product code, stop." Asserting a 404 from a router
with no routes proves the same wiring with no handler at all.

**Resolution.** Accepted. `router()` is built with **no routes**; the canary
seam asserts that driving it in-process with no socket yields `404` for an
unrouted path. That proves everything the seam exists to prove — the router
builds, it is drivable without a socket, the response is readable — and asserts
nothing about room behaviour. It also stops T-01 from pre-deciding T-09's
health-check shape. `main.rs` serves the empty router; T-04a adds the first
route.

### R-5 · MINOR — CI tool installation is more machinery than the ticket asks (accepted in part)

**Concern.** Pinned release tarballs with checksums for both `just` and `uv`,
plus a stated implementation-time fallback, is complexity and implementation
risk the contract does not demand — it asks only for default runners and for
Cargo and uv to be cached.

**Resolution.** Accepted in part. The **conditional fallback is deleted** — one
path, decided now, since a branch that only fires on a blocked fetch is exactly
the untested machinery the concern names. `uv` comes from **`astral-sh/setup-uv`**
pinned to a tag: it is the vendor's own action and it supplies the uv cache the
ticket explicitly requires, so hand-rolling it bought nothing. `just` stays a
**pinned release tarball with a verified `sha256`** — about five lines, no
third-party action trust, and there is no vendor action for `just`. Declined the
implied alternative of a third-party `setup-just` action; recorded so the
Orchestrator can overrule.

### R-6 · NIT — `fmt-check` is undefined and outside the ask (accepted)

**Concern.** Listed in the justfile contents with no statement of what it checks.

**Resolution.** Accepted — **dropped entirely**. Formatting and lint recipes are
not among the eight reserved names and no ticket asks for them; whichever ticket
first needs `cargo fmt` or `ruff` adds one then.

### R-7 · Orchestrator amendment — `.gitignore` gets no board or handoff entries (accepted)

**Concern (Orchestrator, mid-implementation).** The plan's `.gitignore` section
proposed adding `.lattice/` and `HANDOFF.md`. Both are wrong: the Lattice board
is a **tracked** directory on the orchestration branch, so ignoring it on `main`
would hide board drift and force `git add -f` on every later add; and
`HANDOFF.md` is the client's own untracked file, so whether it is ignored is
their call, not this ticket's. Add nothing beyond build noise.

**Resolution.** Accepted in full. `.lattice/` and `HANDOFF.md` are **not** added
to `.gitignore`. Ground rule 1 is unchanged and is already enforced the right
way — by staging every path by name, never `git add -A`, so nothing under
`.lattice/` and no `HANDOFF.md` can be committed by accident.

**Consequence, and it is worth stating plainly: `.gitignore` needs no edit at
all.** The ticket says it "gains `target/`, `.venv/`, `__pycache__/`, and the
worktree-side noise". Checked against the file on `origin/main`: `target/`
(line 25), `.venv/` (11), `__pycache__/` (9) are already there, and `target/`
with no leading slash already matches `room/target/`, as `.venv/` does
`pipeline/.venv/`. `.pytest_cache/`, `.ruff_cache/`, `*.egg-info/` and
`.DS_Store` are there too. With `.lattice/` and `HANDOFF.md` withdrawn, the
`*.orig`/`*.rej` entries the plan also listed are merge noise rather than build
noise and fall under "add nothing else". So this ticket touches `.gitignore`
zero times and says so in the PR body, rather than inventing an entry to make
the bullet look satisfied.

### Reviewer checks that came back clean

All eight reserved recipe names present and mapped to the right delivering
tickets against BUILDPLAN's rows and its "Serialized on" notes; no collision
with BUILDPLAN §2's checked Rust/Python module names; `room/` correctly left a
standalone crate rather than a fourth shared aggregator; `.env.example` and every
contract file untouched; `web/` and `bank/` leave T-02, T-14 and T-26 room to
land without preempting them; every tool flag the plan uses verified to exist;
tensions T-1 and T-3 resolved consistently with the contract text; no fourth
tension found.

