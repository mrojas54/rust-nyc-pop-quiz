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
