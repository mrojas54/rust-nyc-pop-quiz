# Recorded outputs

`StubRunner` replays the files in this directory so that the verifier's decision
logic can be proven in `just test` without a toolchain, without Docker and
without the network. `just test-full` re-runs the same cases against the real
toolchain on T-15a's image, so a fixture that has drifted shows up as the two
hooks disagreeing rather than as a quiet pass.

## How a real one is made

By running it. Never by typing it.

A fixture is recorded once, on the T-15a image, from the pinned toolchain, and
its `recorded_from` field names that toolchain. `StubRunner` refuses to replay a
step whose `recorded_from` is missing or empty — a recorded output that cannot
say where it came from is indistinguishable from an invented one, and this
project does not write down what a program prints (`CLAUDE.md`, house rules;
AC-7; SPEC.md G-2).

That also means **this directory is empty of real recordings until T-15b**,
because T-15a's image is what produces them and it does not exist yet.

## What `scaffold-placeholder.json` is

Not a recording, and not a Rust program's output.

It exists so that T-01 can prove the replay mechanism works — that a fixture is
found, parsed, checked for provenance and returned as a `RunStep`, and that a
missing one is an error rather than a silent empty result. Its `recorded_from`
says in words that nothing was run, and every one of its fields is visibly
synthetic so that it cannot be mistaken for, or quietly promoted into, a real
case.

T-15b deletes it and records the real cases: the N=5 determinism cases (AC-8),
the E0502 compile failure (AC-11), and the Miri cases under both borrow models
(AC-9, AC-10).
