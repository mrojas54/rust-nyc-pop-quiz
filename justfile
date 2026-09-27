# Rust NYC Pop Quiz — the harness.
#
# The eight recipe names below are reserved by EVALUATION.md's harness table and
# mean the same thing everywhere in the contract. Those whose suites do not exist
# yet (PENDING) print which ticket delivers them and fail, so that a suite can
# never look green by not being there.
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
PENDING := "burst:T-21 a11y:T-13"

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
test-full: test sandbox-build test-sandbox test-verify-full test-transport-full (canary "--full-only")
    #!/usr/bin/env bash
    set -euo pipefail
    echo ""
    echo "=== test-full ==="
    echo "Ran: test (room, pipeline, web), the canary over real sockets, the AC-12"
    echo "     containment suite, the verifier's cases on the sandbox image, and"
    echo "     the transport at 200 buzzers."
    echo ""
    echo "PENDING — these suites are not built yet:"
    for entry in {{ PENDING }}; do
        printf '  %-12s delivered by %s\n' "${entry%%:*}" "${entry##*:}"
    done
    echo ""
    echo "Run in a browser, not here — no headless browser on this machine or in CI:"
    echo "  wall-layout  the wall's measured layout, AC-100 / AC-33 (just wall-layout)"
    echo "  fallback     the static fallback stepped offline, AC-102 (room/README.md, Pages)"
    echo ""
    echo "T-21 adds the deployed runs and extends this hook further."

# AC-81 and AC-37 at 200 buzzers over loopback sockets (T-04c). The test is
# #[ignore]d so `test` never runs it. 201 sockets are ~400 descriptors in one
# process, so the soft limit is raised first, and a limit still under 1024 (the
# `burst` client's own floor) fails here rather than as a flaky socket error.
test-transport-full:
    #!/usr/bin/env bash
    set -euo pipefail
    ulimit -n 4096 2>/dev/null || true
    if [ "$(ulimit -n)" != unlimited ] && [ "$(ulimit -n)" -lt 1024 ]; then
        echo "test-transport-full: ulimit -n is $(ulimit -n); 1024 or more is needed" >&2
        exit 1
    fi
    cd room && cargo test --offline --locked --test transport_full -- --ignored

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

# The wall's measured layout (AC-100, AC-33): every bank question, every phase
# that renders source, measured by the real wall code in a real layout engine.
# Serves the repo on loopback and names the page; open it in a browser (the c11
# browser, or any other) and read the JSON report it writes. room/README.md,
# Pages, records the last run.
wall-layout PORT="8765":
    @echo "open http://127.0.0.1:{{ PORT }}/web/wall/measure.html?run=1"
    python3 -m http.server {{ PORT }} --bind 127.0.0.1

# The secrecy suite (T-08). Plants a canary in every secret field, walks a room
# through every phase and scans every payload, frame, page and served file, the
# rendered wall and the real buzzer page (room/README.md, Canary). The scan runs
# inside `test` too (cargo runs every tests/*.rs); this recipe is the hook.
#
#   just canary            the scan: HTTP in-process, sockets over loopback
#   just canary --full     also the test-full half: every request over TCP, two
#                          rooms, the reconnect path
#   just canary --full-only  that half alone (test-full: `test` already ran the first)
#   just canary --url U    the deployed room at U: not yet — it needs T-09's
#                          stand-in token and T-25's admin push; fails saying so
#
# The secrecy suite: every phase, every surface (--full: over TCP; --url: T-09).
canary *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    full=0; url=""
    set -- {{ ARGS }}
    while [ $# -gt 0 ]; do
        case "$1" in
            --full) full=1 ;;
            --full-only) full=2 ;;
            --url) url="${2:?--url needs a URL}"; shift ;;
            *) echo "just canary: unknown argument $1 (--full | --full-only | --url U)" >&2; exit 2 ;;
        esac
        shift
    done
    cd room
    if [ -n "$url" ]; then
        CANARY_URL="$url" cargo test --offline --locked --test canary_full -- --ignored
        exit 0
    fi
    [ "$full" = 2 ] || cargo test --offline --locked --test canary
    if [ "$full" != 0 ]; then
        cargo test --offline --locked --test canary_full -- --ignored
    fi

# The deployed room end to end (T-09): all seven phases over real sockets with
# mock participants, the join link and the stand-in's refusals checked, q3's
# secrets scanned for in every pre-reveal frame (room/README.md, Deploying).
# The host credential is read from HOST_DEV_TOKEN in the environment, never
# from the command line. A by-hand gate until T-21 puts it in test-full.
#
#   just smoke https://rustnyc-popquiz.fly.dev [--participants N]   (1-200, default 20)
#
# One run releases q3, and a machine refuses a question it has already run
# until it restarts: `fly apps restart rustnyc-popquiz` before the next run.
#
# The deployed room, all seven phases (HOST_DEV_TOKEN in the environment).
smoke URL *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    ulimit -n 4096 2>/dev/null || true
    cd room
    cargo run --release --offline --locked --features smoke --bin smoke -- --url "{{ URL }}" {{ ARGS }}

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
