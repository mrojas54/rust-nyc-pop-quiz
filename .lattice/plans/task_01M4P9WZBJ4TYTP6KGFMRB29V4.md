# PQ-70: Generator follow-ups from the PR #62 review

Follow-ups from the PQ-69 exact-head review of PR #62 (merged at 296262b), plus F6 carried from PQ-68. Held until the client releases it.

Minor:
1. The real-claude guard (autouse PATH=/usr/bin:/bin, which claude is None, sockets blocked) lives only in pipeline tests/test_generate_cli.py:115. Move it to a conftest so no test anywhere can build ClaudeCliClient(None, ...) or pass an absolute --claude-bin that resolves to the real binary.
2. SIGTERM/SIGHUP to the generator orphans the claude child (default SIGTERM skips the except BaseException kill, generate_cli.py:218; child is in its own session) and it keeps consuming plan usage for up to 900 s. Install a handler that kills the child process group, test it under test-generate-full, and update the README (it mentions only kill -9). Do this before the client's first real claude-cli run (HC-2).
3. OSError from Popen (binary removed, E2BIG) and UnicodeDecodeError from communicate(text=True) escape main (generate.py:1388) as tracebacks; the decode case calls communicate() a second time inside _kill. Map both to a clean GenerateError stop; finished candidates stay cached.
4. DEFAULT_TIMEOUT_S=900 is not settable from the CLI. Add a flag.

NIT:
5. generate.py:1187: a claude-cli run killed between the submitting state and submit is told to "check the Console" about a batch and cannot resume under its run id. Give claude-cli its own message and let it resume.
6. F6 (from PQ-68): add ANTHROPIC_API_KEY= to .env.example. The sandbox denies .env.* in worktrees; the client adds the line or allows the edit.

Upstream, not in this ticket: BUILDPLAN D-D's model line keeps "$5 in / $25 out" beside the amendment's $4/$20, and the T-16 row reads "Claude Opus 5". Also confirm --safe-mode, --setting-sources "" and --effort against the real binary at HC-2.

Guardrails carried from PQ-69: no real claude -p or API call from tests, CI or a delegator; the child env never sees ANTHROPIC_API_KEY; Batches stays the default, no fallback; never write down what a program prints (AC-7); never overstate verification (AC-43).

# Plan (delegator, 2026-11-05)

Base `7dda474`. Files: only the cleared set.

## Item 1: one real-binary guard (new `pipeline/tests/conftest.py`)

Seam: the two places a test can reach `claude`. (a) `ClaudeCliClient.__init__`, wrapped by an autouse fixture: after the constructor resolves `self.binary`, the test fails (`pytest.fail`, a BaseException, so no `except Exception` swallows it) if its realpath is the real `claude`. (b) `subprocess.Popen`, replaced for every test by a subclass that refuses, before anything is spawned, an argv[0] that is the real `claude`, and a `python -m popquiz.generate --backend claude-cli` whose `--claude-bin` (or, without it, `claude` on the child env's PATH) resolves to the real binary. "The real binary" = realpath of `shutil.which("claude")` on the ambient PATH at conftest import, plus any existing standard install path (`~/.local/bin`, `~/.claude/local`, `/opt/homebrew/bin`, `/usr/local/bin`). No root PATH scrub, no root socket refusal: the Docker suites keep `docker`, `cargo`, `uv` and the daemon socket.
The PATH scrub, billing-variable removal and socket refusal move into one non-autouse `hermetic` fixture in the same conftest; `test_generate.py` and `test_generate_cli.py` take it with `pytestmark = pytest.mark.usefixtures("hermetic")`, and their own copies go. `test_kill_rerun.py` keeps `PATH=/usr/bin:/bin` only as its child's environment; its guard is (b).
RED/tests: a `claude_decoy` fixture registers a tmp executable as "the real claude" (it writes a marker if ever run). Tests in `test_generate_cli.py`: constructor with the decoy's path, constructor with `None` and the decoy first on PATH, `generate.main --claude-bin decoy`, and a subprocess generator with `--claude-bin decoy`, each fails on the guard and the marker is absent. One in `tests/sandbox/generate/` proves the root guard reaches that directory.
Proof for the sandbox suites: `pytest tests/sandbox --collect-only -q` with `POPQUIZ_SANDBOX_SUITE=1 POPQUIZ_GENERATE_FULL=1`; `just test-generate-full` run; Docker suites left to CI's `test-full` unless the client approves a local run. Throwaway demos (not committed): one test under `tests/` and one under `tests/sandbox/generate/` that only construct `ClaudeCliClient(<the real path>)`, shown failing on the guard, then deleted.
Mutation: drop the conftest guard -> the decoy tests fail (the decoy constructs/runs).

## Item 2: SIGTERM/SIGHUP kill the child

Seam: `ClaudeCliClient._run_one`. A context manager installed *before* `Popen` and removed after the child is reaped, main thread only, only for a signal whose current handler is `SIG_DFL` (an ignored SIGHUP under `nohup` stays ignored). Handler: if a spawn is in progress (between the call and the assignment), record the signal and act right after `Popen` returns; if a live child is held (`returncode is None`), `killpg(SIGKILL)` and reap it; then restore the previous handler and `os.kill(getpid(), signum)`, so the parent exits with the signal's status. Previous handlers restored on the way out in every case.
Tests (`tests/sandbox/generate/test_kill_rerun.py`, `just test-generate-full`): parametrized SIGTERM and SIGHUP sent while c02 is in flight; the generator's return code is `-signum` and the fake's process group is gone within a bounded wait. Unit test in `test_generate_cli.py`: handlers are the same before and after a run. README :134-138 updated.
Mutation: remove the handler -> the SIGTERM test finds the group alive.

## Item 3: `OSError` from Popen, non-UTF-8 stdout

Seam: `_run_one`. `Popen` raising `OSError` -> `GenerateError("Stopped at cXX: could not start <binary> (<strerror>). Candidates finished before it are kept; rerun with the same --run-id to pick up the rest.")`. The child runs in bytes mode (prompt encoded UTF-8); stdout is decoded strictly after the child is reaped, stderr with replacement; a decode failure -> `GenerateError("Stopped at cXX: <binary> printed output that is not UTF-8. ...same tail")`. No second `communicate` on a finished child; `_kill` only signals a child whose `returncode is None`.
RED tests first (`test_generate_cli.py`, through `generate.main`): a fake wrapper that deletes itself after answering c01 (ENOENT on c02), and a fake that answers c02 with invalid UTF-8. Each: return 2, stderr `generate: Stopped at c02: ...`, no exception, `cli/c01.json` kept, a rerun with a good fake exits 0 and starts only c02 and c03.
Mutation: remove the OSError mapping -> FileNotFoundError escapes the test.

## Item 4: the timeout flag

Name: `--claude-timeout-s` (the style of `--poll-interval-s`, `--cap-usd`; `claude-` like `--claude-bin`). Default `None` -> `DEFAULT_TIMEOUT_S`; help "claude-cli: seconds one candidate may run before it is killed and the run stops; default 900". A batches run given it refuses like `--claude-bin`. A value that is zero, negative or not finite refuses in `main` before the client is built, and again in `ClaudeCliClient.__init__`.
Tests: `--help` lists it; the value reaches `ClaudeCliClient(timeout_s=...)` (spy on the class); 0, -1, nan refuse with exit 2 and the fake never called; batches refuses it.
Mutation: let the flag bypass the positivity check -> the refusal test sees a run.

## Item 5: a claude-cli run stopped while submitting resumes

Seam: `generate.run`, state without `batch_id`. Batches: the message unchanged, character for character. claude-cli: there is no batch to leak, so it finishes the submit (its `submit` only rewrites `requests.json`) and goes on; the report says `resumed: true`.
Tests (`test_generate.py`/`test_generate_cli.py`): a submit interrupted after the submitting state was written; the claude-cli rerun under the same id emits all three; the Batches rerun raises the exact pinned text.
Mutation: route claude-cli back to the Batches message -> the resume test fails.

## Reset 2026-11-05 by agent:delegator-pq70
