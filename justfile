# Rust NYC Pop Quiz — the harness.
#
# The eight recipe names below are reserved by EVALUATION.md's harness table and
# mean the same thing everywhere in the contract. Five of them name suites that
# do not exist yet; those print which ticket delivers them and fail, so that a
# suite can never look green by not being there.
#
#   just setup          fetch dependencies
#   just test           the inner loop: hermetic, parallel, offline, <= 60 s
#   just test-full      everything that exists, plus what is still pending
#   just sandbox-build  build the verification image from pin.toml (T-15a)
#
# Why `setup` exists: `test` has to be hermetic, and a fresh .venv has to come
# from somewhere. Splitting the network step out is what lets `test` be run
# with --offline and held to it, rather than merely asked not to reach out.
#
# Two recipes use the network, and neither is reachable from `test`: `setup`, and
# `sandbox-build`, which downloads a toolchain into an image. `sandbox-build` is a
# prerequisite of `test-full` only. The image it produces runs with no network at
# all, which is why the toolchain has to be baked in at build time.

# Suites not built yet, and the ticket that delivers each. Read by both `_pending`
# and `test-full`, so the two can never disagree about what is missing.
PENDING := "burst:T-21 a11y:T-13 smoke:T-09"

_default:
    @just --list --unsorted

# Fetch every dependency. The only recipe that touches the network.
setup:
    cd pipeline && uv sync --frozen
    cd room && cargo fetch --locked

# The inner loop. No network, no Docker, no pinned toolchain, no Miri.
# EVALUATION.md budgets this at 60 s wall-clock; a slower default suite is a defect.
[parallel]
test: test-room test-pipeline test-web

# Builds and tests the room crate with whatever cargo this machine has.
test-room:
    cd room && cargo test --offline --locked

# The pipeline's decision logic, against StubRunner's recorded outputs.
#
# tests/sandbox is ignored here and nowhere else: it is the AC-12 containment suite
# and it needs a real container, which this hook is forbidden from having. Ignoring
# the directory means its conftest is never even imported — and that conftest
# refuses to run unasked, so the suite has no way to look green without a container.
test-pipeline:
    cd pipeline && uv run --offline --no-sync pytest --ignore=tests/sandbox

# Node's built-in runner. No package.json, no node_modules, nothing to install.
# The glob is quoted so node expands it, not the shell: with no package.json
# anywhere, node resolves a bare directory argument as a module specifier and
# fails looking for an index, rather than scanning it for tests.
test-web:
    node --test 'web/test/*.test.js'

# Everything that exists today, then an honest list of what does not.
# Green by contract: the pending suites are named here, never invoked here.
test-full: test sandbox-build test-sandbox test-verify-full
    #!/usr/bin/env bash
    set -euo pipefail
    echo ""
    echo "=== test-full ==="
    echo "Ran: test (room, pipeline, web), the in-process canary seam, the AC-12"
    echo "     containment suite, and the verifier's cases on the sandbox image."
    echo ""
    echo "PENDING — these suites are not built yet:"
    for entry in {{ PENDING }}; do
        printf '  %-12s delivered by %s\n' "${entry%%:*}" "${entry##*:}"
    done
    echo ""
    echo "T-21 adds the deployed runs and extends this hook further."

# The AC-12 containment suite, on the image `sandbox-build` just produced.
#
# POPQUIZ_SANDBOX_SUITE is what tells tests/sandbox/conftest.py it was asked for.
# Without it the suite refuses to collect, so a recipe that forgets to set it fails
# instead of reporting a suite that never ran. Not --offline: the fixtures run
# containers, and the container is where the network is denied.
#
# -vv, not -v: pyproject's addopts carries -q, and one -v only cancels it. The
# suite names each fixture on purpose — "9 passed" says nothing about which ways
# out were tried.
test-sandbox:
    cd pipeline && POPQUIZ_SANDBOX_SUITE=1 uv run --no-sync pytest tests/sandbox -vv

# The verifier's cases (tests/fixtures/verify/cases.toml) on the real toolchain,
# the twin of what `test` replays on StubRunner (D-18). A recording that no longer
# describes what the toolchain does fails here. POPQUIZ_VERIFY_FULL is this
# directory's own switch, so `test-sandbox` stays the containment suite alone.
test-verify-full: sandbox-build
    cd pipeline && POPQUIZ_SANDBOX_SUITE=1 POPQUIZ_VERIFY_FULL=1 uv run --no-sync pytest tests/sandbox/verify -vv

# Build the verification image from pin.toml, then report what it actually says.
#
# Every version comes out of pin.toml and goes in as a build argument, because the
# pin is defined in one place (SPEC.md 7.2, D-17). The Dockerfile holds no version
# of its own and test_sandbox.py fails if it grows one.
#
# The three recorded fields are printed, not written. They go into pin.toml by hand,
# once, after a build — read_pin() refuses a pin whose recorded fields are empty, so
# a toolchain nobody has looked at cannot be verified against. Nobody types a
# recorded field from memory; this is where the values come from.
sandbox-build:
    #!/usr/bin/env bash
    set -euo pipefail
    cd pipeline
    read -r RUST NIGHTLY PLATFORM IMAGE <<<"$(uv run --no-sync python -c 'from popquiz.sandbox import read_pin; p = read_pin(require_recorded=False); print(p.rust_version, p.nightly, p.platform or "-", p.image)')"

    # The tag carries the pin and a digest of the Dockerfile, so a tag that exists
    # is by construction the image those two files describe — and changing either
    # changes the tag, which rebuilds on its own. That is what makes skipping safe
    # rather than merely fast, and what lets CI restore a cached image (keyed on the
    # same two files) and have this step become a no-op.
    if [ -z "${SANDBOX_REBUILD:-}" ] && docker image inspect "$IMAGE" >/dev/null 2>&1; then
        echo "=== $IMAGE is already built — set SANDBOX_REBUILD=1 to force ==="
    else
        echo "=== building $IMAGE (rust $RUST, nightly-$NIGHTLY, platform ${PLATFORM/-/native}) ==="
        args=(--build-arg "RUST_VERSION=$RUST" --build-arg "NIGHTLY=$NIGHTLY")
        [ "$PLATFORM" = "-" ] || args+=(--platform "$PLATFORM")
        docker build "${args[@]}" -t "$IMAGE" -f sandbox/Dockerfile sandbox
    fi

    echo ""
    echo "=== what the image reports — paste these into sandbox/pin.toml [pin] ==="
    vv=$(docker run --rm --network none "$IMAGE" rustc -Vv)
    miri=$(docker run --rm --network none "$IMAGE" cargo "+nightly-$NIGHTLY" miri --version)
    printf 'release = "%s"\n' "$(printf '%s\n' "$vv" | awk '/^release: /{print $2}')"
    printf 'commit_hash = "%s"\n' "$(printf '%s\n' "$vv" | awk '/^commit-hash: /{print $2}')"
    printf 'miri_version = "%s"\n' "$miri"
    printf '# host triple this build produced: %s\n' "$(printf '%s\n' "$vv" | awk '/^host: /{print $2}')"

# The secrecy suite, in-process: the router driven without a socket.
# EVALUATION.md splits this hook — the in-process scan runs inside `test`, the
# scan of the deployed room's real frames and pages runs inside `test-full`.
canary:
    cd room && cargo test --offline --locked --test canary
    @echo "canary: in-process scan only. T-08 plants the phase-scoped secrets;"
    @echo "the deployed-room scan joins test-full once T-09 has something to deploy."

# --- Reserved names whose suites do not exist yet ------------------------------
# Each fails loudly rather than passing quietly. `test` and `test-full` do not
# depend on them, which is why those two are green while these are not.

# One candidate on the pinned toolchain: writes its verified record and prints the
# verdict and the Miri wall-clock (Q-E4). Needs the image (`just sandbox-build`).
#   just verify path/to/candidate.json    (relative to the repo root) --expect ran|does_not_compile|ub
verify *ARGS:
    uv run --offline --no-sync --project pipeline python -m popquiz.verify {{ ARGS }}

# The whole bank against SPEC 7.6; writes bank/audit/<date>.json (T-19). Every
# check also runs inside `test`. Room flags: --screen-width-ft --screen-height-ft
# --back-row-ft (SPEC 5.2 defaults, a hypothesis until HC-1); --strict fails on any flag.
bank-audit *ARGS:
    cd pipeline && uv run --offline --no-sync python -m popquiz.audit --repo .. {{ ARGS }}

burst *ARGS:
    @just _pending burst

a11y *ARGS:
    @just _pending a11y

smoke *ARGS:
    @just _pending smoke

# Prints which ticket delivers a suite, then fails.
_pending SUITE:
    #!/usr/bin/env bash
    set -euo pipefail
    for entry in {{ PENDING }}; do
        name="${entry%%:*}"
        ticket="${entry##*:}"
        if [ "$name" = "{{ SUITE }}" ]; then
            echo "just {{ SUITE }}: not built yet — BUILDPLAN $ticket delivers it." >&2
            exit 1
        fi
    done
    echo "just {{ SUITE }}: no such suite, and it is not on the pending list." >&2
    exit 1
