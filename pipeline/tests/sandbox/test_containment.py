"""AC-12, on the real image: five programs, each contained, each reporting which
mechanism held it.

> `test-full`: a fixture program that opens a socket, reads `/etc/passwd`, reads an
> env var, allocates past the limit, and loops forever — each is contained and
> reported. **Proven on the sandbox actually used, not a stand-in.**
> — `EVALUATION.md`, AC-12

Two rules this file keeps, and they are the reason it looks the way it does.

**No assertion reads what a program printed.** The three isolation fixtures signal
through an exit code they choose themselves, documented at the top of each `.rs`
file; the two limit fixtures are stopped by the sandbox and the verdict comes from
how. Nothing here quotes program output, because writing down what a program prints
is the one thing this project does not do (`CLAUDE.md`, house rules).

**The environment fixture plants its own canary first.** Asserting that
`POPQUIZ_ADMIN_TOKEN` is absent inside the container proves nothing unless it is
present outside it. The suite sets it on the host before running, exactly as the
`canary` hook does for payloads.
"""

from __future__ import annotations

import os
import pathlib
import subprocess

import pytest

from popquiz.sandbox import (
    ENV_NOT_PASSED,
    HOST_FS_ISOLATED,
    MEMORY_LIMIT,
    NETWORK_UNREACHABLE,
    ROOT_READ_ONLY,
    TIMEOUT,
    WORKDIR_IS_TMPFS,
    read_pin,
    run_in_sandbox,
)

PROGRAMS = pathlib.Path(__file__).parent / "programs"

#: Long enough that a slow machine compiling under emulation is not mistaken for a
#: runaway, short enough that the suite stays usable.
COMPILE_AND_RUN_TIMEOUT = 300

#: The deadline for the runaway fixture. It has to outlast compiling a four-line
#: program, since `run` compiles in the same container, and nothing else — the
#: mechanism is under test here, not the configured number.
LOOP_TIMEOUT = 30


def program(name: str) -> str:
    return (PROGRAMS / f"{name}.rs").read_text(encoding="utf-8")


@pytest.fixture(scope="session")
def pin():
    """The recorded pin.

    `read_pin()` requires the recorded fields, so this fails with a message naming
    `just sandbox-build` if the image has never been built and the pin never
    confirmed against it. That ordering is deliberate: the suite cannot run against
    a toolchain nobody has looked at.
    """
    return read_pin()


@pytest.fixture(scope="session", autouse=True)
def the_image_exists(pin) -> None:
    """Errors rather than skips. A missing image must not read as a pass."""
    found = subprocess.run(
        ["docker", "image", "inspect", pin.image],
        capture_output=True,
        text=True,
        check=False,
    )
    if found.returncode != 0:
        pytest.fail(
            f"the sandbox image {pin.image} is not built. Run `just sandbox-build`.\n"
            "AC-12 is proven on the sandbox actually used, so there is nothing to "
            "fall back to."
        )


@pytest.fixture
def patient(pin):
    """The pin with a timeout long enough to compile in, for the fixtures whose
    containment is not the timeout."""
    return _with_timeout(pin, COMPILE_AND_RUN_TIMEOUT)


def _with_timeout(pin, seconds: int):
    import dataclasses

    limits = dataclasses.replace(pin.limits, timeout_seconds=seconds)
    return dataclasses.replace(pin, limits=limits)


# --- The three isolation fixtures --------------------------------------------


def test_a_program_that_opens_a_socket_cannot_reach_the_network(patient) -> None:
    result = run_in_sandbox(program("opens_a_socket"), "run", pin=patient)

    assert NETWORK_UNREACHABLE in result.verdicts
    assert result.exit_code == 0, (
        "the program reports that it reached the network. Its exit code is its own "
        f"and is documented in opens_a_socket.rs; stderr: {result.stderr[-400:]}"
    )
    assert not result.timed_out


def test_a_program_that_reads_etc_passwd_sees_only_the_images_own_filesystem(patient) -> None:
    """The claim is *no host filesystem*, not *no `/etc/passwd`* — see the comment
    at the top of the fixture for why the difference matters. Containment here is
    the argv carrying no mount, plus a root filesystem that refuses writes."""
    result = run_in_sandbox(program("reads_etc_passwd"), "run", pin=patient)

    assert HOST_FS_ISOLATED in result.verdicts
    assert ROOT_READ_ONLY in result.verdicts
    assert WORKDIR_IS_TMPFS in result.verdicts
    assert result.exit_code == 0, (
        "exit 1 means a write to the root filesystem succeeded; exit 2 means "
        f"/etc/passwd could not be read at all. stderr: {result.stderr[-400:]}"
    )


def test_no_host_environment_variable_reaches_a_program(patient, monkeypatch) -> None:
    """Plant the canaries on the host first, or the observation is vacuous."""
    for name in (
        "POPQUIZ_ADMIN_TOKEN",
        "POPQUIZ_SANDBOX_CANARY",
        "ANTHROPIC_API_KEY",
        "GH_TOKEN",
        "AWS_SECRET_ACCESS_KEY",
    ):
        monkeypatch.setenv(name, f"canary-{name.lower()}-must-not-cross")

    # Proof the plant took: the host running this test can see it.
    assert os.environ["POPQUIZ_ADMIN_TOKEN"].startswith("canary-")

    result = run_in_sandbox(program("reads_an_env_var"), "run", pin=patient)

    assert ENV_NOT_PASSED in result.verdicts
    assert result.exit_code == 0, (
        "a planted host variable crossed into the sandbox. The fixture names it on "
        f"stderr and never prints its value: {result.stderr[-400:]}"
    )
    # And the planted value appears in neither stream, which is the canary rule.
    assert "must-not-cross" not in result.stdout
    assert "must-not-cross" not in result.stderr


# --- The two limit fixtures --------------------------------------------------


def test_a_program_that_allocates_past_the_limit_is_killed(patient) -> None:
    result = run_in_sandbox(program("allocates_past_the_limit"), "run", pin=patient)

    assert MEMORY_LIMIT in result.verdicts, (
        f"exit code {result.exit_code}, timed out: {result.timed_out}. The fixture "
        "exits 1 by itself if it allocated everything without being stopped, which "
        f"would mean the limit did not fire. stderr: {result.stderr[-400:]}"
    )
    assert not result.timed_out, "this should hit the memory limit, not the clock"
    assert result.contained


def test_a_program_that_loops_forever_is_killed_on_expiry(pin) -> None:
    """A short timeout on purpose: the mechanism is under test, not the number. The
    configured number is asserted from `pin.toml` in `test_sandbox.py`.

    The timeout has to outlast compilation, since `run` compiles in the same
    container — so a failure here is the clock, not a slow `rustc`.
    """
    result = run_in_sandbox(program("loops_forever"), "run", pin=_with_timeout(pin, LOOP_TIMEOUT))

    assert TIMEOUT in result.verdicts
    assert result.timed_out
    assert result.contained
    assert result.wall_clock_s < LOOP_TIMEOUT * 2, (
        f"killed, but only after {result.wall_clock_s:.1f}s — the kill should follow "
        "the deadline closely"
    )


def test_the_container_does_not_survive_the_timeout(pin) -> None:
    """"Hard timeout" means the container is gone, not merely orphaned. Killing the
    `docker` CLI would leave it running and still holding its limits."""
    name = "popquiz-sandbox-timeout-probe"
    subprocess.run(["docker", "rm", "-f", name], capture_output=True, check=False)

    run_in_sandbox(program("loops_forever"), "run", pin=_with_timeout(pin, LOOP_TIMEOUT), container_name=name)

    still_there = subprocess.run(
        ["docker", "ps", "-a", "--filter", f"name=^{name}$", "--format", "{{.Names}}"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert name not in still_there.stdout, f"{name} outlived its deadline"


# --- The image is the one the pin describes ----------------------------------


def test_the_image_reports_the_pinned_toolchain(pin) -> None:
    """The pin's recorded fields are what this image actually says, not what
    someone typed into `pin.toml` (D-17, and the house rule about writing down what
    a program prints).

    Asked directly rather than through a mode: `compile`, `run` and `miri` each mean
    something about a candidate, and none of them means "what version are you".
    """
    out = subprocess.run(
        ["docker", "run", "--rm", "--network", "none", pin.image, "rustc", "-Vv"],
        capture_output=True,
        text=True,
        check=True,
    )
    assert f"release: {pin.release}" in out.stdout
    assert f"commit-hash: {pin.commit_hash}" in out.stdout
    triple = [ln.split(": ", 1)[1] for ln in out.stdout.splitlines() if ln.startswith("host: ")]
    assert triple and triple[0] in pin.target_triples, (
        f"the image reports host {triple} which pin.toml does not list"
    )


def test_miri_is_installed_and_needs_no_network(pin) -> None:
    """The image's whole reason for existing is that Miri is ready inside it with
    `--network none`. This does not prove anything about UB — Miri proves absence of
    UB on executed paths only (`SPEC.md` §7.2), and what it proves about a
    *candidate* is T-15b's. It proves the toolchain is here and invokable."""
    out = subprocess.run(
        [
            "docker", "run", "--rm", "--network", "none", pin.image,
            "cargo", f"+{pin.nightly_toolchain}", "miri", "--version",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    assert out.stdout.strip(), "miri reported no version"
    assert pin.miri_version.split()[-1] in out.stdout


def test_a_trivial_program_runs_under_miri_in_the_sandbox(pin) -> None:
    """Miri end to end through `run_in_sandbox`, so the `miri` mode is known to work
    before T-15b builds a verifier on top of it."""
    result = run_in_sandbox(
        "fn main() { let v = vec![1, 2, 3]; println!(\"{}\", v.len()); }",
        "miri",
        pin=_with_timeout(pin, COMPILE_AND_RUN_TIMEOUT),
    )

    assert NETWORK_UNREACHABLE in result.verdicts
    assert result.exit_code == 0, f"miri failed: {result.stderr[-800:]}"
    assert not result.timed_out
