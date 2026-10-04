# Recorded outputs

`StubRunner` replays the files under `verify/recordings/` so that the verifier's
decision logic can be proven in `just test` without a toolchain, without Docker
and without the network. `just test-full` re-runs the same cases against the real
toolchain on T-15a's image (`just test-verify-full`), so a recording that has
drifted shows up as the two hooks disagreeing rather than as a quiet pass.

## How a recording is made

By running it. Never by typing it.

    just sandbox-build
    cd pipeline && uv run --no-sync python tests/fixtures/verify/record.py

`record.py` runs every case in `verify/cases.toml` through `verify` on the pinned
image, keeping each step the first time a case asks for it and replaying it for
every later case of the same program, so all of one program's cases see one set of
runs. **It writes nothing unless every case returns the verdict `cases.toml` names**
— a recording that does not show the property its case is about is not a fixture
for that case. It rewrites every file from scratch.

Each recording is one JSON file per program: one key per step (`toolchain`,
`compile`, `run:1` … `run:5`, `miri:<model>:<seed>`), each carrying `exit_code`,
`stdout`, `stderr`, `recorded_from` (the image tag) and, where they apply,
`stopped_by` and `wall_clock_s`. Two bookkeeping keys describe the file itself and
are never replayed: `_recorded` (the recorder, the image, the host, the pin's
recorded values and when) and `_source_sha256` (the program it is a recording of).

`StubRunner` refuses a step whose `recorded_from` is missing or empty, and
`test_verify.py` fails with *re-record* when a program's source or the pin has moved
since its recording was made.

## What is here

| Program | Source | What it shows |
|---|---|---|
| `q3` | `bank/questions/q3.json` | deterministic, Miri clean: accepted as *ran*; rejected declared *does not compile* or *UB* |
| `q4` | `bank/questions/q4.json` | deterministic, Miri clean: accepted as *ran*; `test_migration.py` derives its correct option from this replay |
| `q7` | `bank/questions/q7.json` | deterministic, Miri clean: accepted as *ran*; `test_migration.py` derives its correct option from this replay |
| `q8` | `bank/questions/q8.json` | E0502: accepted declared *does not compile*, code recorded (AC-11); rejected declared *ran* |
| `syntax-error` | `verify/programs/` | fails to compile with no error code: rejected (AC-11) |
| `loops-forever` | `sandbox/programs/loops_forever.rs` | stopped by the deadline, shortened to 5 s for this case: rejected |
| `hashmap-order` | `verify/programs/` | `HashMap` iteration order differs between runs: rejected (AC-8) |
| `panics` | `verify/programs/` | identical panic every run: accepted, exit code recorded |
| `ub-both` | `verify/programs/` | UB under both borrow models: rejected undeclared, accepted declared (AC-9) |
| `ub-sb-only` | `verify/programs/` | UB under Stacked Borrows only: rejected declared, the models disagree (AC-9) |
| `miri-differs` | `verify/programs/` | `cfg!(miri)` changes the output: rejected (AC-10) |
| `miri-unsupported` | `verify/programs/` | Miri's isolation refuses `getcwd`: rejected, Miri could not run it |

All twelve were recorded on `popquiz-sandbox:1.98.1-2026-09-19-25a914853547`
(aarch64-unknown-linux-gnu) on 2026-10-03, when q4 and q7 were added (`record.py`
rewrites every recording at once); each file's `_recorded` says so exactly.
`hashmap-order`'s five `stdout` strings are new random iteration orders every time
`record.py` runs, because that case exists to show output varying, so a re-record
changes them while its verdict stays the same; no test pins them
(`test_verify.py` counts the distinct outputs in the recording itself).

The bank programs are read from the bank, not copied, so a recording cannot stand
for a question whose source has since changed without `_source_sha256` noticing.

## What is not here

No step's output appears anywhere but its recording. The tests compare the
verifier's record against the recording, never against a value written into a
test; `test_verify.py` scans the verifier and its tests for any structured
recorded output and any pin value, and fails if it finds one (CLAUDE.md; AC-7;
SPEC.md G-2).
