# Rust NYC Pop Quiz — the harness.
#
# The eight recipe names below are reserved by EVALUATION.md's harness table and
# mean the same thing everywhere in the contract. Five of them name suites that
# do not exist yet; those print which ticket delivers them and fail, so that a
# suite can never look green by not being there.
#
#   just setup       fetch dependencies (the one recipe allowed to use the network)
#   just test        the inner loop: hermetic, parallel, offline, <= 60 s
#   just test-full   everything that exists, plus what is still pending
#
# Why `setup` exists: `test` has to be hermetic, and a fresh .venv has to come
# from somewhere. Splitting the one network step out is what lets `test` be run
# with --offline and held to it, rather than merely asked not to reach out.

# Suites not built yet, and the ticket that delivers each. Read by both `_pending`
# and `test-full`, so the two can never disagree about what is missing.
PENDING := "verify:T-15b bank-audit:T-19 burst:T-21 a11y:T-13 smoke:T-09"

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
test-pipeline:
    cd pipeline && uv run --offline --no-sync pytest

# Node's built-in runner. No package.json, no node_modules, nothing to install.
# The glob is quoted so node expands it, not the shell: with no package.json
# anywhere, node resolves a bare directory argument as a module specifier and
# fails looking for an index, rather than scanning it for tests.
test-web:
    node --test 'web/test/*.test.js'

# Everything that exists today, then an honest list of what does not.
# Green by contract: the pending suites are named here, never invoked here.
test-full: test
    #!/usr/bin/env bash
    set -euo pipefail
    echo ""
    echo "=== test-full ==="
    echo "Ran: test (room, pipeline, web) and the in-process canary seam."
    echo ""
    echo "PENDING — these suites are not built yet:"
    for entry in {{ PENDING }}; do
        printf '  %-12s delivered by %s\n' "${entry%%:*}" "${entry##*:}"
    done
    echo ""
    echo "T-15a adds the sandbox image and T-21 the deployed runs; both extend"
    echo "this hook. Docker is installed for test-full only, and nothing uses it yet."

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

verify *ARGS:
    @just _pending verify

bank-audit *ARGS:
    @just _pending bank-audit

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
