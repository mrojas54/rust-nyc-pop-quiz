# PQ-19: Verification sandbox image

BUILDPLAN.md T-15a (M3).

The sandbox: a Docker image with the pinned toolchain and nightly Miri — **the pin (`rustc -Vv` release and commit-hash, the nightly's date) is defined here, once** (`SPEC.md` §7.2, D-17) — run with no network, memory/CPU/pids limits, read-only root, tmpfs work dir, hard timeout; the AC-12 fixture suite proven on it

Criteria: AC-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-01
BUILDPLAN notes: Serialized on `justfile`

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-20)

## What this ticket is, in one line

Define the pin once, build the image that embodies it, give the pipeline one
module that can run a program inside it, and prove containment on the real
thing — five programs that each try to escape a different way.

## The shape of the thing

`run_in_sandbox` is not a verifier. It executes one step and reports what the
sandbox did about it. T-15b decides what that means for a candidate. So the
module's whole job is: build an argv, run it with a deadline, and turn
`(exit_code, timed_out, stderr)` into named verdicts.

The one structural constraint that shapes every file below: **`just test` must
be able to import this module without acquiring the ability to run Docker.**
That is solved by dependency injection, not by keeping the module out of `test`
— the decision logic is the part worth testing in the inner loop, and it is
testable only if it is reachable. The real `subprocess` call sits behind a
default argument; the tests pass a fake.

## Files to create

| Path | What it holds |
|---|---|
| `pipeline/sandbox/pin.toml` | **The pin, and the only place it is defined.** `rustc` release + commit-hash, the nightly date for Miri, the target triple, the flag set (`opt-level`, `overflow-checks`, `debug-assertions`), the docker platform, the image tag, and the resource limits. Header comment names D-17 and says T-15b reads this file and nothing re-declares it. |
| `pipeline/sandbox/Dockerfile` | `FROM rust:<pinned>-slim-bookworm`, `--platform` pinned; `rustup toolchain install nightly-<date> --component miri --profile minimal`; `cargo +nightly-<date> miri setup` at **build** time so run time needs no network; a non-root `sandbox` user; `WORKDIR /work`. Nothing else. |
| `pipeline/sandbox/README.md` | What the image is, how to build it, how to bump the pin, **which of the docker flags the five fixtures actually exercise and which are set but unexercised**, and the SPEC sentence: Miri proves absence of UB on executed paths only. |
| `pipeline/src/popquiz/sandbox.py` | `read_pin()`, `docker_argv()`, `verdicts_for()`, `run_in_sandbox(program_source, mode, *, runner=…)` → `SandboxResult(exit_code, stdout, stderr, timed_out, verdicts, wall_clock_s)`. The one module in the package allowed to start a process. |
| `pipeline/tests/test_sandbox.py` | Decision logic in `just test`, no Docker: argv shape (every AC-12 flag present, no bind mount, no env passthrough), pin loading and its refusals, and `verdicts_for` over every case. Uses a fake `runner`. |
| `pipeline/tests/sandbox/conftest.py` | Refuses collection unless `POPQUIZ_SANDBOX_SUITE=1`, and errors rather than skipping, so the suite has no quiet-green path. |
| `pipeline/tests/sandbox/test_containment.py` | The AC-12 suite. Five programs, five verdicts, on the real image. `test-full` only. |
| `pipeline/tests/sandbox/programs/*.rs` | `opens_a_socket.rs`, `reads_etc_passwd.rs`, `reads_an_env_var.rs`, `allocates_past_the_limit.rs`, `loops_forever.rs`. Programs, not recordings. |

## Files to change (both cleared, nobody else edits them this wave)

- **`justfile`** — add `sandbox-build` (reads `pin.toml`, builds and tags);
  `test-pipeline` gains `--ignore=tests/sandbox`; `test-full` builds the image
  and runs the suite, and its "T-15a adds the sandbox image … nothing uses it
  yet" paragraph is replaced by what now happens. The header comment saying
  `setup` is the only recipe that touches the network is amended: `sandbox-build`
  does too. `PENDING` is untouched — AC-12 is not on it.
- **`.github/workflows/ci.yml`** — the `test-full` job builds the image with a
  buildx layer cache keyed on `hashFiles('pipeline/sandbox/Dockerfile',
  'pipeline/sandbox/pin.toml')`.
- **`pipeline/tests/test_runner.py`** — narrow the shell-out guard as its own
  docstring asks (below). Not delete.

Not touched: `pipeline/pyproject.toml` (not needed — `tomllib` is stdlib in
3.12, and the suite is excluded by `--ignore`, not by a registered marker),
`runner.py`, anything under the contract paths.

## Narrowing the shell-out guard, precisely

`test_nothing_just_test_imports_can_start_a_process` currently allows no module
to import `subprocess`. `sandbox.py` must. The docstring says: *"Narrow it to
exempt the one module that owns execution rather than deleting it, so the set of
modules that can start a process stays visible at a glance."* So:

1. Add `MAY_START_A_PROCESS = {"popquiz/sandbox.py"}` — an exact path set, with
   the reason in a comment.
2. The assertion becomes **two-directional**: an offender not on the list fails
   (as now), *and* a list entry with no offence fails as a stale exemption. A
   one-way allowlist rots; this one cannot.
3. Add `test_the_sandbox_module_executes_nothing_at_import_time` — AST-assert
   that every `subprocess` call in `sandbox.py` is inside a function body, so
   importing it is inert. This is the test that carries the ticket's real
   requirement (`test` never imports a path that *executes* Docker) rather than
   the weaker one (`test` imports nothing that *could*).
4. Extend `test_the_guard_catches_the_ways_it_claims_to` with a case proving the
   exemption is scoped to the one path, not to the whole tree.

## The five fixtures, and what each one's verdict is

| Program | Sandbox mechanism | Verdict asserted |
|---|---|---|
| `opens_a_socket.rs` | `--network none` | `NETWORK_UNREACHABLE` |
| `reads_etc_passwd.rs` | no bind mount; `--read-only` | `HOST_FS_ISOLATED` — it reads the *image's* `/etc/passwd` (contains the `sandbox` user the Dockerfile made), and a write to it fails `EROFS` |
| `reads_an_env_var.rs` | no `--env`, cleared environment | `ENV_NOT_PASSED` |
| `allocates_past_the_limit.rs` | `--memory` | `MEMORY_LIMIT` |
| `loops_forever.rs` | hard timeout, kill on expiry | `TIMEOUT` |

The assertion is on the verdict, never on what a program prints (house rule).
`reads_etc_passwd.rs` is the one that needs care: `/etc/passwd` exists inside
any container and is world-readable, so "it was denied" would be a false
claim. What AC-12 asks is *no host filesystem access*, and the honest proof is
that the file it reads is the image's, plus a structural assertion that the argv
carries no `-v`/`--mount`/`--volume`.

## Open choices, and the side I take

**1. Pin the platform to `linux/amd64`.** Both linux arches ship miri for
`nightly-2026-09-19`, so the image would build natively on the client's Mac.
I am pinning amd64 anyway, because AC-8 (N=5 byte-identical), AC-9 and AC-10
(Miri output compared to native) are all **output comparisons**, and an
unpinned architecture makes "passes on the laptop, fails in CI" a live class of
bug in exactly the criteria the verifier exists to prove. One pin means the
client's Mac and CI write byte-identical records for the same question, which is
what "defined here, once" is for. The cost is emulated Miri on Apple Silicon; I
will measure it and put the number in the README and the completion comment, and
`platform` is one line in `pin.toml` so T-15b or the client can flip it with a
documented consequence (every non-legacy record re-verifies).

**2. The pin values are observed, not typed.** The channel manifest says stable
is `1.98.1 (48a229cea 2026-09-01)` and `nightly-2026-09-19` has miri on both
linux arches — that is what I pin. But the `release`/`commit-hash`/`miri_version`
fields get their values from `rustc -Vv` and `cargo miri --version` **run inside
the built image**, and `read_pin()` refuses a pin whose fields are empty with a
message saying to record them from the image. If Docker cannot be started, the
file ships unfilled and failing rather than plausibly wrong — the same rule the
fixtures README applies to recorded outputs.

**3. CPU and pids are set but not proven by these five.** AC-12's text says
"enforced CPU/memory/time limits". `--cpus` throttles rather than kills, so no
fixture can show it firing; what bounds a runaway is the hard timeout, which
`loops_forever.rs` does prove. `--pids-limit` is in the ticket's flag list but
not in AC-12's text at all. The README says which flags the suite exercises and
which are set but unexercised, rather than letting "AC-12 green" imply more than
it does. This is PHILOSOPHY §4 applied to the sandbox instead of to Miri.

**4. A bare `pytest` in `pipeline/` will error, not skip.** `testpaths = ["tests"]`
means bare `pytest` collects `tests/sandbox`; the conftest refuses it. I chose a
loud error over a skip because a skipped Docker suite that `test-full` does not
notice is precisely how AC-12 would come to look green without having run. It
matches the scaffold's own taste — `_pending` fails rather than passing quietly.

## Contract tension found, and the side I take

**The ticket body says `PR base: origin/main`; the boot prompt says base
`ai-c11-cc/repo-scaffold`, stacked on #8.** The boot prompt is the later and
more specific instruction, it explains why (this ticket needs the harness #8
adds), and it says the Orchestrator retargets after #8 merges. I follow the
boot prompt: PR against `ai-c11-cc/repo-scaffold`. Flagged for the Orchestrator;
no contract file edited.

**`EVALUATION.md`'s harness row for `test-full` says CI "installs Docker and the
image for this hook only"; the ubuntu runner already has Docker and the scaffold
asserts it rather than installing it.** No conflict in substance — I extend the
existing assertion with the image build. Noting it so nobody reads "installs"
as a missing step.

## Tests by criterion

| Criterion | Where it is proven | Hook |
|---|---|---|
| **AC-12** | `pipeline/tests/sandbox/test_containment.py` — five programs, each contained, each reporting which mechanism held it, on the real image | `test-full` |
| AC-12 (argv) | `test_sandbox.py` — every required flag present, no bind mount, no env passthrough, timeout set | `test` |
| AC-12 (verdicts) | `test_sandbox.py::test_verdicts_for_*` — the decision logic over every case, fake runner | `test` |
| AC-6 (supply) | `test_sandbox.py` — `read_pin()` returns release, commit-hash, nightly date, target triple and the flag set; refuses an unfilled pin. T-15b enforces the comparison; this ticket only defines the value | `test` |
| D-18 / harness | `test_runner.py` — narrowed guard, two-directional; `sandbox.py` inert at import | `test` |

`just test` stays under 60 s: the additions are one fast test file and an
`--ignore`. I will report the warm number.

## Order of work

1. `pin.toml` (fields unfilled), `Dockerfile`, `sandbox.py`, `test_sandbox.py`,
   the narrowed guard. `just test` green.
2. Ask the client to start Docker Desktop (the daemon is not running; the CLI is
   29.7.2). Build the image, read the real `rustc -Vv` and miri version out of
   it, fill `pin.toml`.
3. The five programs and `test_containment.py`. Run the suite on the image.
   Record the wall-clock.
4. `justfile`, `ci.yml`, `README.md`.
5. Code review, validation evidence, PR against `ai-c11-cc/repo-scaffold`.

If Docker cannot be started: steps 1, 3 (written, not run), 4 and 5 still ship;
`pin.toml` ships unfilled and failing; AC-12 is marked pending the client's run
in the completion comment. No faked pass.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: fresh-eyes subagent, given the contract paths and the scaffold only —
deliberately **not** the delegator boot prompt. Two findings turn on text it
could not see; both are recorded as such rather than waved away.

### 1. Critical — the tmpfs work dir is never delivered. ACCEPTED.

**Concern.** `BUILDPLAN.md:47` (D-C) and the T-15a row both require a **tmpfs
work dir**; the plan's text names `--network none`, `--read-only` and `--memory`
and never once says `tmpfs`. With a read-only root and no writable mount, the
compiler has nowhere to put a binary — this is not an unexercised flag, it is
the sandbox being unable to run anything.

**Resolution.** `--tmpfs /work` is in the argv, in `docker_argv()`'s required
set, and asserted by `test_sandbox.py`. Two details the finding does not reach
but that follow from it:

- The mount is **`exec`**, deliberately. `noexec` is the reflex hardening choice
  and it would break the sandbox outright: `rustc` writes a binary into `/work`
  and the next step runs it. `noexec,nosuid,nodev` minus the `noexec` — so
  `nosuid,nodev` — plus an explicit `size=` so a fixture cannot fill the host's
  memory through the tmpfs instead of through the heap.
- `size=` is set from `pin.toml`'s limits and is **≤ the memory limit**, because
  a tmpfs is accounted against the container's memory cgroup. A tmpfs larger
  than `--memory` would let a program reach the memory limit by writing files,
  which would make `allocates_past_the_limit.rs`'s verdict ambiguous about which
  mechanism fired.

### 2. Major — no stated mechanism for getting the source in with no bind mount. ACCEPTED.

**Concern.** `run_in_sandbox(program_source, …)` takes source as data, the plan
asserts no bind mount, and never says how the text crosses the boundary.

**Resolution.** **stdin.** `docker run -i` with no TTY; the container's command
reads stdin into `/work/main.rs` and then compiles or runs it. Written into the
plan as a named mechanism, asserted in `test_sandbox.py` (argv carries `-i`, the
source is passed as the process's stdin and never as an argument or an env var),
and it is the reason `docker_argv()` and the fake `runner` both take the source
separately from the argv. Passing source as an argv element would also put
candidate text in the host's process table, which is a second reason not to.

### 3. Major — `PR base: origin/main` overridden on outside authority. KEPT, with the gap named.

**Concern.** The ticket metadata says `origin/main`; the plan targets
`ai-c11-cc/repo-scaffold` citing a document not in the reviewed set.

**Resolution.** Kept. The delegator boot prompt is this ticket's operative
instruction and the reviewer was not given it: it designates PQ-19 a
**press-ahead ticket**, sets the base to `ai-c11-cc/repo-scaffold` @ `2ab95e3`
(PR #8) *because this ticket needs the harness #8 adds*, and says the
Orchestrator retargets to `main` once #8 merges. `origin/main` in the ticket body
is generated metadata that predates that decision. The finding is correct that
nothing in the **contract** licenses the override — which is why it goes to the
Orchestrator in the completion comment rather than being treated as settled.

### 4. Major — `pin.toml` absorbs fields BUILDPLAN assigns to T-15b. PARTLY ACCEPTED.

**Concern.** D-17 and `SPEC.md` §7.2 define the pin narrowly as release +
commit-hash + nightly date. `BUILDPLAN.md:191` lists "the flag set, target
triple" among **T-15b's** deliverables. The plan puts both in `pin.toml` and also
adds the image tag and the resource limits, without flagging the expansion.

**Resolution, three parts.**

- **The flag set and the target triple stay.** The boot prompt asks for them by
  name: *"the target triple, and the flag set the verifier will use
  (`opt-level`, `overflow-checks`, `debug-assertions`) — values SPEC §3.2
  records. T-15b reads this file; nothing else re-declares it."* T-15b's row
  adds them **to the record**; this ticket defines the values it reads. No
  conflict, and the reviewer could not see the instruction.
- **The limits and the image tag move out of the pin, inside the same file.**
  The finding is right that calling the whole file "the pin" muddies what
  staleness means. `pin.toml` gets two tables: `[pin]` — release, commit-hash,
  nightly date, edition, target triple, flags: the values a record is compared
  against — and `[limits]` / `[image]`, with a comment stating in words that
  **nothing outside `[pin]` affects the stale check**, so bumping a memory limit
  never invalidates a verified question.
- **The AC-6 row is reworded.** T-15a's criterion is AC-12 and nothing else. The
  table row now reads *supplies the value AC-6 is later checked against; proves
  nothing about AC-6* — the earlier wording invited the reading the finding took.

### 5. Major — nothing keeps the Dockerfile and `pin.toml` from drifting. ACCEPTED, and it is the best finding of the seven.

**Concern.** A Dockerfile with `FROM rust:<version>` and
`rustup toolchain install nightly-<date>` written into it, plus a `pin.toml` that
records the same two facts, is the pin defined **twice** — exactly what D-17
exists to prevent, in a repo whose own house rules say this class of thing has
regressed four times.

**Resolution.** The Dockerfile holds **no version literal at all**. It takes
`ARG RUST_VERSION`, `ARG NIGHTLY` (and the platform) with no defaults, so a
build that does not pass them fails rather than silently picking something;
`sandbox-build` reads `pin.toml` and passes them as `--build-arg`. Plus a test in
`test_sandbox.py` that greps the Dockerfile for a version-shaped literal
(`rust:1.`, `nightly-20`) and **fails if one is present** — so the single-source
property is enforced, not merely intended. `ARG` with no default is what turns
"remember to pass it" into a build error.

### 6. Major — the platform-pin rationale reaches into another ticket's criteria. ACCEPTED; **the decision is reversed.**

**Concern.** The plan pinned `linux/amd64` for every host, accepting emulated
Miri on the client's Mac, and justified it with AC-8, AC-9 and AC-10 — T-15b's
criteria. No contract text says the platform must be pinned for those
comparisons to hold, while `BUILDPLAN.md:47` does claim the Docker sandbox
"works on the client's Mac."

**Resolution — Open choice 1 is reversed. The image builds natively; the
platform is not pinned.** On re-examination my argument was the weaker half of
the trade:

- D-17 defines the pin as release + commit-hash + nightly date. The
  **commit-hash is identical across architectures** for a given release, so the
  pin is already arch-independent. The triple was never part of the stale check.
- `SPEC.md` §3.2 makes `target_triple` a **recorded** field. Recording the
  triple that actually ran is more correct than asserting a constant one — and
  it is the house rule: write what was observed.
- The divergence I was guarding against is close to non-existent for these
  programs. aarch64 and x86_64 linux-gnu are both 64-bit and little-endian, and
  Miri is an interpreter with an explicit target model — more arch-independent
  than native execution, not less.
- The cost was concrete and fell on the one person who runs the pipeline. Q-E4
  measures the Miri wall-clock; QEMU would inflate the number the project
  intends to measure, against a decision record that promises the Mac works.

`nightly-2026-09-19` ships miri for **both** `x86_64-unknown-linux-gnu` and
`aarch64-unknown-linux-gnu` (checked against the channel manifest), so native on
both is available. `pin.toml` records `target_triple` as the two supported
triples; the runner records which one ran. `platform` stays a value in
`pin.toml`, now **unset by default** — setting it forces one arch, which is
still a one-line change if a real divergence ever appears. The escape hatch now
runs in the safe direction: a constraint can be added later, whereas a slow
laptop cannot be undone.

### 7. Minor — CI "installs Docker" wording. ACKNOWLEDGED, no change.

Self-flagged in the plan already; the finding agrees with the resolution
(extend the existing `docker --version` assertion with the image build).

### Consequent edits to the sections above

- Dockerfile row: no version literals; `ARG RUST_VERSION` / `ARG NIGHTLY`, no
  defaults; `--tmpfs /work` documented as `exec`.
- `pin.toml` row: `[pin]` vs `[limits]`/`[image]`; `platform` unset.
- `sandbox.py` row: source crosses on **stdin**; `docker_argv()` returns argv
  only and the source is handed to the runner separately.
- `test_sandbox.py` row: adds the tmpfs assertion, the stdin assertion, and the
  no-version-literal check on the Dockerfile.
- Tests-by-criterion: the AC-6 row no longer reads as coverage.

### Update to resolution 3 — the tension dissolved, 2026-09-20 ~22:50

PR #8 **merged at 2026-09-20T22:07:26Z** and its branch was deleted, which is why
`origin/ai-c11-cc/repo-scaffold` stopped resolving mid-ticket. `origin/main` is
now `0742d35`, whose tree is **byte-identical** to the scaffold tip `2ab95e3`.
The press-ahead condition is gone, so this branch rebases onto `origin/main` and
the PR is opened against `main` — which is what the ticket metadata said and what
the boot prompt said would happen once #8 merged. **No deviation to report on the
base.** Finding 3 is resolved by events rather than by argument.

### Design detail settled while implementing, reported here because it changes an assertion

The five verdicts split into two kinds, and conflating them would have produced a
false claim:

- **Configuration verdicts** — `NETWORK_UNREACHABLE`, `ENV_NOT_PASSED`,
  `HOST_FS_ISOLATED`, `ROOT_READ_ONLY`, `WORKDIR_IS_TMPFS` — are facts about the
  argv, true independent of any program. Derived from the argv, asserted in `test`.
- **Outcome verdicts** — `TIMEOUT`, `MEMORY_LIMIT` — are facts about how the run
  ended, and can only be observed at run time.

This matters for the house rule. Asserting `"Network is unreachable" in stderr`
would be **writing down what a program prints**. So no fixture assertion quotes
program output: the isolation fixtures signal through an **exit code the program
itself chooses** (`process::exit(0)` when the escape failed as expected), and the
containment claim rests on the configuration verdict plus that exit code. The
suite asserts verdicts and deliberate exit codes, never printed text.

Two more consequences worth recording:

- `--memory-swap` is set equal to `--memory`. Without it Docker allows swap at
  twice the memory limit, so `allocates_past_the_limit.rs` could complete instead
  of being killed and `MEMORY_LIMIT` would never fire.
- "Kill on expiry" needs a second call: `subprocess`'s timeout kills the `docker`
  CLI, not the container. On expiry the module runs `docker kill <name>` against
  a per-run container name. Asserted with the fake runner.
