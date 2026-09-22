# The verification sandbox

Candidate programs come from a language model and are executed by this project on
purpose. This is the box they run in: a Docker image holding the pinned stable
toolchain and the pinned nightly with Miri, run with no network, no host
filesystem, no inherited environment, and hard limits on memory, CPU, processes
and time.

BUILDPLAN T-15a. Criterion AC-12. The pin is D-17 and `SPEC.md` §7.2.

## The three files

| File | What it is |
|---|---|
| `pin.toml` | **The pin, defined once.** Nothing else declares the toolchain. T-15b's verifier reads it; the Dockerfile takes its versions from it as build arguments. |
| `Dockerfile` | The image. Holds **no version literal** — `ARG RUST_VERSION` and `ARG NIGHTLY`, neither with a default, so a build that does not pass the pin fails instead of quietly choosing. |
| `../src/popquiz/sandbox.py` | `run_in_sandbox(source, mode)`. Builds the `docker run` argv, feeds the source in on stdin, kills on expiry, and reports what the sandbox did. |

## Building it

```sh
just sandbox-build
```

Reads `pin.toml`, builds, tags the image
`popquiz-sandbox:<rust>-<nightly>-<dockerfile sha256, 12 hex>`, then prints the
toolchain the image reports.

The tag carries **every input that decides the image** — the pin and the
Dockerfile — deliberately. An image under that tag *is* the image those two files
describe, so the build is skipped when it already exists (`SANDBOX_REBUILD=1`
forces it), and changing either file changes the tag and rebuilds on its own. A
stale image quietly answering to the expected name is exactly how a verification
would run on the wrong image without anyone noticing. (The first version of this
tag carried only the versions, and a Dockerfile fix under an unchanged pin was
skipped as already built. `test_sandbox.py` now holds that shut.)

## Bumping the pin

1. Edit `[pin].rust_version` and/or `[pin].nightly` in `pin.toml`.
2. Blank `release`, `commit_hash` and `miri_version`.
3. `just sandbox-build`, and paste back the three values it prints.
4. **Re-verify the bank.** Every non-legacy record is now stale: its recorded
   `rustc` no longer matches the pin, so T-15b's stale check refuses to schedule
   its question until it has been verified again (AC-6). Legacy records are exempt
   (D-16) and are replaced whole, never merged, when re-verified.

Step 2 is not bookkeeping. `read_pin()` refuses a pin whose recorded fields are
empty, so a toolchain nobody has looked at cannot be verified against — and
nobody types a recorded field. **This project does not write down what a program
prints** (`CLAUDE.md`, house rules), and a compiler version is not an exception
just because it is short.

## What the containment suite proves, and what it does not

`just test-full` builds the image and runs `pipeline/tests/sandbox/` — five Rust
programs, each trying to get out a different way. They are programs, not
recordings: the assertion is on the containment verdict and on an exit code each
program chooses for itself, never on what one printed.

| Flag | Exercised by | What fires |
|---|---|---|
| `--network none` | `opens_a_socket.rs` | Nothing connects, by address or by name |
| no mount, `--read-only` | `reads_etc_passwd.rs` | The filesystem it sees is the image's, and writes to it are refused |
| no `-e` of any kind | `reads_an_env_var.rs` | Planted host variables are absent inside |
| `--memory` (+ `--memory-swap`) | `allocates_past_the_limit.rs` | The process is killed |
| hard timeout | `loops_forever.rs` | The container is killed on expiry, not merely orphaned |

**Set but not exercised by these five, and worth saying plainly:**

- **`--cpus`** throttles a container's share of a core. It never terminates
  anything, so no program can be written that shows it *firing*. What bounds a
  runaway is the timeout, which `loops_forever.rs` does prove. AC-12's text says
  "enforced CPU/memory/time limits"; the CPU limit is real and it is a rate limit,
  not a kill.
- **`--pids-limit`** is set and no fixture forks. It is in the sandbox's flag list
  (D-C) but not in AC-12's own text, so no fixture was written for it.
- **`--cap-drop ALL`** and `--security-opt no-new-privileges` are asserted to be
  in the argv and are not separately exercised.

`MEMORY_LIMIT` is also an **inference**, and the code says so where it is drawn:
a container the OOM killer takes exits 137, and one whose allocator refused an
allocation aborts with 134. Neither carries a label reading "memory", so some
other external kill would look the same. It is told apart from the timeout kill —
also 137 — because the module knows whether it did the killing, and it is
corroborated by the fixture being a program whose only action is to allocate.
That is the whole of the evidence.

## Miri

The image has Miri installed on the pinned nightly, with `cargo miri setup`
already run at **build** time — which is the only reason the container can run
with `--network none`. There is nothing left to download.

**Miri proves absence of undefined behaviour on the paths actually executed.** It
is evidence, not a proof (`SPEC.md` §7.2, AC-43, PHILOSOPHY §4). The sandbox's job
is to make Miri runnable and contained; what a Miri result means about a
candidate — strict provenance, Stacked Borrows and Tree Borrows having to agree,
Miri's output matching native — is T-15b's, and the receipt shown to the room
never claims more than the record holds.

## Why the platform is not pinned

`[image].platform` is empty, so the image builds for the host's own architecture,
and `[pin].target_triples` lists both linux-gnu targets that Miri ships for. The
runner records which one actually ran.

Pinning `linux/amd64` was considered and rejected. It would give one recorded
`target_triple` everywhere, which is tidy, at the cost of running Miri under
emulation on Apple Silicon — where the pipeline is actually used, and where Q-E4
measures the Miri wall-clock the project plans around. D-17's pin is the release,
the commit-hash and the nightly date; the **commit-hash is identical across
architectures** for a given release, so the pin is already
architecture-independent, and `SPEC.md` §3.2 makes `target_triple` a *recorded*
field rather than a configured one. Recording what ran is the house rule.

If a real cross-architecture divergence ever appears, setting `platform` is one
line — and that is the direction the escape hatch should run in. A constraint can
be added later; a slow laptop cannot be undone.

## Running it without Docker

You cannot, and nothing pretends otherwise. `sandbox.py` is **importable** with no
Docker present, and its argv, pin handling and verdict logic are all proven in
`just test` against an injected fake — that is what lets the inner loop stay
hermetic, offline and under 60 s. But containment itself is proven on the sandbox
actually used, not on a stand-in (`EVALUATION.md`, AC-12), so
`pipeline/tests/sandbox/` refuses to run rather than skipping when it is reached
without a container. A skipped Docker suite and a passing one look identical in a
summary line, and AC-12 is not a criterion that may ever look green by absence.
