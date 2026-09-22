"""The sandbox's decision logic, proven without Docker (T-15a, AC-12).

These run in `just test`: the argv is built, the pin is read, the verdicts are
derived, and nothing is executed. Containment itself cannot be proven here — it
needs a real container, and it lives in `tests/sandbox/`, which `just test`
does not collect.

The division is the point. What `just test` can prove is that the sandbox *asks*
for the right thing; what only `test-full` can prove is that asking works.
"""

from __future__ import annotations

import ast
import pathlib
import re

import pytest

from popquiz import sandbox
from popquiz.sandbox import (
    CAPABILITIES_DROPPED,
    ENV_NOT_PASSED,
    HOST_FS_ISOLATED,
    MEMORY_LIMIT,
    MODES,
    NETWORK_UNREACHABLE,
    ROOT_READ_ONLY,
    TIMEOUT,
    WORKDIR_IS_TMPFS,
    PinError,
    RunnerTimeout,
    configuration_verdicts,
    container_command,
    docker_argv,
    outcome_verdicts,
    read_pin,
    run_in_sandbox,
)

PIPELINE = pathlib.Path(__file__).resolve().parents[1]
SANDBOX_DIR = PIPELINE / "sandbox"
SANDBOX_MODULE = PIPELINE / "src" / "popquiz" / "sandbox.py"


# --- The pin -----------------------------------------------------------------
# Read with require_recorded=False throughout, because the recorded fields are
# filled by `just sandbox-build` from a real image and are legitimately empty in a
# fresh checkout. The tests that care about the recorded fields say so.


def _pin():
    return read_pin(require_recorded=False)


def _pin_file(tmp_path, **overrides: str):
    """A copy of the real `pin.toml` with `[pin]` fields rewritten.

    Derived from the shipped file rather than written from scratch, so a test
    fixture cannot drift into describing a pin the project does not have — and so
    that a new required field makes these tests fail rather than quietly pass
    against a stale hand-built copy.
    """
    text = (SANDBOX_DIR / "pin.toml").read_text(encoding="utf-8")
    for key, value in overrides.items():
        text, count = re.subn(
            rf'^{re.escape(key)} = ".*"$', f'{key} = "{value}"', text, count=1, flags=re.M
        )
        assert count == 1, f"pin.toml has no `{key} = \"…\"` line to override"
    path = tmp_path / "pin.toml"
    path.write_text(text, encoding="utf-8")
    return path


def test_the_pin_supplies_what_a_verified_record_needs() -> None:
    """Supplies the values AC-6 is later checked against. Proves nothing about
    AC-6 itself — the comparison and the stale rejection are T-15b's."""
    pin = _pin()

    assert pin.rust_version
    assert pin.nightly
    assert pin.edition
    assert pin.target_triples
    assert set(pin.flags) >= {"opt-level", "overflow-checks", "debug-assertions"}
    assert pin.nightly_toolchain == f"nightly-{pin.nightly}"
    # The image tag carries the pin, so two pins cannot share one image name.
    assert pin.rust_version in pin.image and pin.nightly in pin.image


def test_the_image_tag_changes_when_the_dockerfile_does(tmp_path) -> None:
    """`sandbox-build` skips a tag that already exists, so the tag must carry every
    input that decides the image. With only the versions in it, a Dockerfile edit
    under an unchanged pin would be skipped as already built — which is what happened
    to this ticket's own first image before the digest was added."""
    pin_text = (SANDBOX_DIR / "pin.toml").read_text(encoding="utf-8")
    dockerfile = (SANDBOX_DIR / "Dockerfile").read_text(encoding="utf-8")

    (tmp_path / "pin.toml").write_text(pin_text, encoding="utf-8")
    (tmp_path / "Dockerfile").write_text(dockerfile, encoding="utf-8")
    before = read_pin(tmp_path / "pin.toml").image

    (tmp_path / "Dockerfile").write_text(dockerfile + "\n# a change\n", encoding="utf-8")
    after = read_pin(tmp_path / "pin.toml").image

    assert before == read_pin().image, "a byte-identical Dockerfile gives the same tag"
    assert before != after


def test_the_shipped_pin_has_been_recorded_against_a_real_image() -> None:
    """The recorded fields are filled, which means someone built the image and read
    them out of it. An unrecorded pin in the repository would mean the bank could be
    verified against a toolchain nobody has looked at."""
    pin = read_pin()

    assert pin.release == pin.rust_version
    assert len(pin.commit_hash) == 40, "a full commit-hash, as `rustc -Vv` reports it"
    assert pin.miri_version


@pytest.mark.parametrize(
    "blanked", ["release", "commit_hash", "miri_version"]
)
def test_the_pin_refuses_to_be_used_with_any_recorded_field_missing(tmp_path, blanked) -> None:
    """Each recorded field on its own, because a guard that only fires when all
    three are empty would pass a half-filled pin."""
    path = _pin_file(tmp_path, **{blanked: ""})

    with pytest.raises(PinError, match="Recorded fields come out of the built image"):
        read_pin(path)


def test_the_pin_refuses_a_release_that_disagrees_with_what_it_asked_for(tmp_path) -> None:
    """The check that catches an image built from a tag other than the one this file
    asks for — which is how verification would silently run under the wrong
    compiler."""
    path = _pin_file(tmp_path, release="1.0.0")

    with pytest.raises(PinError, match="asked for"):
        read_pin(path)


def test_the_pin_reports_a_missing_file_rather_than_defaulting(tmp_path) -> None:
    with pytest.raises(PinError, match="no pin at"):
        read_pin(tmp_path / "absent.toml")


@pytest.mark.parametrize(
    "table",
    ["[limits]", "[pin.flags]"],
    ids=["limits", "flags"],
)
def test_the_pin_refuses_a_file_missing_a_required_table(tmp_path, table: str) -> None:
    source = (SANDBOX_DIR / "pin.toml").read_text(encoding="utf-8")
    path = tmp_path / "pin.toml"
    path.write_text(source.replace(table, "[unused]", 1), encoding="utf-8")

    with pytest.raises(PinError):
        read_pin(path, require_recorded=False)


# --- The argv ----------------------------------------------------------------


def test_the_argv_carries_every_limit_ac12_names() -> None:
    pin = _pin()
    argv = docker_argv("run", pin, container_name="c")

    assert argv[:2] == ["docker", "run"]
    for flag in ("--network", "--memory", "--memory-swap", "--cpus", "--pids-limit", "--tmpfs", "--cap-drop"):
        assert flag in argv, f"{flag} is missing from the argv"
    assert "--read-only" in argv
    assert argv[argv.index("--network") + 1] == "none"
    assert argv[argv.index("--cap-drop") + 1] == "ALL"


def test_swap_is_capped_at_the_memory_limit() -> None:
    """Without this Docker allows swap at twice the limit, and a program that
    swaps instead of dying is not a contained program — the memory fixture would
    simply complete."""
    pin = _pin()
    argv = docker_argv("run", pin, container_name="c")

    assert argv[argv.index("--memory-swap") + 1] == argv[argv.index("--memory") + 1]


def test_the_work_dir_is_a_sized_tmpfs_that_allows_exec() -> None:
    """`exec` is not an oversight. rustc writes a binary into /work and the next
    step runs it, so `noexec` would break the sandbox rather than harden it."""
    pin = _pin()
    argv = docker_argv("run", pin, container_name="c")
    spec = argv[argv.index("--tmpfs") + 1]

    mount, _, options = spec.partition(":")
    assert mount == sandbox.WORK_DIR
    assert "exec" in options.split(",")
    assert "noexec" not in options.split(",")
    assert f"size={pin.limits.tmpfs_size}" in options
    assert "nosuid" in options and "nodev" in options


def test_no_host_path_and_no_environment_variable_crosses_the_boundary() -> None:
    pin = _pin()
    argv = docker_argv("run", pin, container_name="c")

    assert not [a for a in argv if a.split("=", 1)[0] in ("-v", "--volume", "--mount")]
    assert not [a for a in argv if a.split("=", 1)[0] in ("-e", "--env", "--env-file")]


def test_the_source_crosses_on_stdin_and_never_as_an_argument() -> None:
    """With no bind mount there is nowhere else for it to come from, and an
    argument would put candidate source in the host's process table."""
    pin = _pin()
    source = "fn main() { println!(\"{}\", 1); }"
    seen: dict[str, object] = {}

    def fake(argv, *, input=None, timeout=None):
        seen["argv"], seen["input"] = list(argv), input
        return _Done(0, "", "")

    run_in_sandbox(source, "run", pin=pin, runner=fake, container_name="c")

    assert seen["input"] == source
    assert "-i" in seen["argv"]
    assert source not in seen["argv"]
    # The command reads stdin into a file rather than receiving the text inline.
    assert any("cat >" in part for part in seen["argv"])


def test_the_container_is_named_so_a_timeout_has_something_to_kill() -> None:
    pin = _pin()
    argv = docker_argv("run", pin, container_name="a-specific-name")

    assert argv[argv.index("--name") + 1] == "a-specific-name"


def test_the_platform_flag_appears_only_when_the_pin_sets_one() -> None:
    """Empty means native, which is the shipped choice: pinning one architecture
    would make Miri run under emulation on Apple Silicon."""
    pin = _pin()
    assert pin.platform == "", "the shipped pin builds natively; see README"
    assert "--platform" not in docker_argv("run", pin, container_name="c")

    forced = dataclass_replace(pin, platform="linux/amd64")
    argv = docker_argv("run", forced, container_name="c")
    assert argv[argv.index("--platform") + 1] == "linux/amd64"


@pytest.mark.parametrize("mode", MODES)
def test_every_mode_builds_a_command_that_reads_stdin(mode: str) -> None:
    command = container_command(mode, _pin())

    assert command[0] == "sh" and command[1] == "-c"
    assert "cat >" in command[2]


def test_an_unknown_mode_is_refused() -> None:
    with pytest.raises(sandbox.SandboxError, match="unknown mode"):
        container_command("interpret", _pin())


def test_the_compile_and_run_modes_pass_the_pinned_flag_set() -> None:
    pin = _pin()
    for mode in ("compile", "run"):
        script = container_command(mode, pin)[2]
        assert f"--edition {pin.edition}" in script
        assert f"opt-level={pin.flags['opt-level']}" in script
        assert "overflow-checks=on" in script
        assert "debug-assertions=on" in script


def test_only_the_run_mode_executes_the_compiled_binary() -> None:
    pin = _pin()
    assert "./main" not in container_command("compile", pin)[2]
    assert "exec ./main" in container_command("run", pin)[2]


def test_the_miri_mode_uses_the_pinned_nightly_and_stays_offline() -> None:
    script = container_command("miri", _pin())[2]

    assert f"+{_pin().nightly_toolchain}" in script
    assert "miri run" in script
    assert "--offline" in script
    # The root filesystem is read-only, so cargo's home has to be on the tmpfs.
    assert f"CARGO_HOME={sandbox.WORK_DIR}" in script


# --- The verdicts ------------------------------------------------------------


def test_the_configuration_verdicts_are_read_off_the_argv() -> None:
    argv = docker_argv("run", _pin(), container_name="c")

    assert configuration_verdicts(argv) == {
        NETWORK_UNREACHABLE,
        ENV_NOT_PASSED,
        HOST_FS_ISOLATED,
        ROOT_READ_ONLY,
        WORKDIR_IS_TMPFS,
        CAPABILITIES_DROPPED,
    }


@pytest.mark.parametrize(
    ("removed", "lost"),
    [
        (["--network", "none"], NETWORK_UNREACHABLE),
        (["--read-only"], ROOT_READ_ONLY),
        (["--cap-drop", "ALL"], CAPABILITIES_DROPPED),
    ],
    ids=["network", "read-only", "cap-drop"],
)
def test_a_dropped_flag_takes_its_verdict_with_it(removed: list[str], lost: str) -> None:
    """Why the verdicts are derived from the argv instead of asserted beside it: a
    flag that goes missing must not leave a claim standing that nothing backs."""
    argv = docker_argv("run", _pin(), container_name="c")
    for item in removed:
        argv.remove(item)

    assert lost not in configuration_verdicts(argv)


@pytest.mark.parametrize(
    ("flag", "value", "lost"),
    [
        ("-v", "/etc:/host-etc", HOST_FS_ISOLATED),
        ("--mount", "type=bind,src=/etc,dst=/host-etc", HOST_FS_ISOLATED),
        ("-e", "POPQUIZ_ADMIN_TOKEN=shh", ENV_NOT_PASSED),
        ("--env-file", "/tmp/secrets", ENV_NOT_PASSED),
    ],
    ids=["volume", "mount", "env", "env-file"],
)
def test_adding_a_way_in_withdraws_the_verdict_that_denied_it(flag, value, lost) -> None:
    argv = docker_argv("run", _pin(), container_name="c")
    argv[2:2] = [flag, value]

    assert lost not in configuration_verdicts(argv)


def test_an_expired_run_reports_the_timeout_and_claims_nothing_else() -> None:
    """Our own kill also exits 137, so nothing about memory may be read from it."""
    assert outcome_verdicts(None, timed_out=True) == {TIMEOUT}
    assert outcome_verdicts(137, timed_out=True) == {TIMEOUT}


@pytest.mark.parametrize("exit_code", [137, 134], ids=["oom-killed", "alloc-aborted"])
def test_a_killed_run_that_we_did_not_kill_reports_the_memory_limit(exit_code: int) -> None:
    assert outcome_verdicts(exit_code, timed_out=False) == {MEMORY_LIMIT}


@pytest.mark.parametrize("exit_code", [0, 1, 101])
def test_an_ordinary_ending_fires_no_limit(exit_code: int) -> None:
    """A good candidate trips nothing. `contained` is not a synonym for `passed`."""
    assert outcome_verdicts(exit_code, timed_out=False) == frozenset()


# --- Timeout handling --------------------------------------------------------


def test_a_timeout_kills_the_container_not_merely_the_cli() -> None:
    """`subprocess`'s timeout reaps the `docker` CLI and leaves the container
    running. The second call is what makes the hard timeout hard."""
    calls: list[list[str]] = []

    def fake(argv, *, input=None, timeout=None):
        calls.append(list(argv))
        if calls[0] == list(argv):
            raise RunnerTimeout(cmd=list(argv), timeout=timeout or 0)
        return _Done(0, "", "")

    result = run_in_sandbox("fn main() {}", "run", pin=_pin(), runner=fake, container_name="doomed")

    assert result.timed_out
    assert result.exit_code is None
    assert TIMEOUT in result.verdicts
    assert result.contained
    assert ["docker", "kill", "doomed"] in calls


def test_a_failed_cleanup_does_not_replace_the_timeout_being_reported() -> None:
    def fake(argv, *, input=None, timeout=None):
        if "kill" in argv:
            raise OSError("docker went away")
        raise RunnerTimeout(cmd=list(argv), timeout=timeout or 0)

    result = run_in_sandbox("fn main() {}", "run", pin=_pin(), runner=fake, container_name="c")

    assert result.timed_out and TIMEOUT in result.verdicts


def test_partial_output_produced_before_the_kill_survives() -> None:
    def fake(argv, *, input=None, timeout=None):
        if "kill" in argv:
            return _Done(0, "", "")
        raise RunnerTimeout(cmd=list(argv), timeout=1, output=b"half a line")

    result = run_in_sandbox("fn main() {}", "run", pin=_pin(), runner=fake, container_name="c")

    assert result.stdout == "half a line"


def test_the_configured_timeout_is_the_deadline_the_runner_is_given() -> None:
    """The containment suite shortens the deadline to keep `test-full` quick, which
    proves the mechanism but not the number. This proves the number: the value in
    `pin.toml` is the one handed to the runner."""
    pin = _pin()
    seen: list[float | None] = []

    def fake(argv, *, input=None, timeout=None):
        seen.append(timeout)
        return _Done(0, "", "")

    run_in_sandbox("fn main() {}", "run", pin=pin, runner=fake, container_name="c")

    assert pin.limits.timeout_seconds > 0
    assert seen == [pin.limits.timeout_seconds]


def test_a_clean_run_reports_the_child_and_fires_no_limit() -> None:
    def fake(argv, *, input=None, timeout=None):
        return _Done(0, "whatever the program wrote", "")

    result = run_in_sandbox("fn main() {}", "run", pin=_pin(), runner=fake, container_name="c")

    assert result.exit_code == 0
    assert not result.timed_out
    assert not result.contained
    assert result.wall_clock_s >= 0
    assert result.container_name == "c"


def test_each_run_gets_its_own_container_name_when_none_is_given() -> None:
    names = set()

    def fake(argv, *, input=None, timeout=None):
        names.add(argv[argv.index("--name") + 1])
        return _Done(0, "", "")

    for _ in range(3):
        run_in_sandbox("fn main() {}", "run", pin=_pin(), runner=fake)

    assert len(names) == 3


# --- The structural guarantees `just test` depends on ------------------------


def test_importing_the_sandbox_module_executes_nothing() -> None:
    """The property `just test` actually needs.

    The shell-out guard in `test_runner.py` exempts this module because it has to
    import `subprocess`. That exemption is only safe if importing it *runs*
    nothing — so this walks the AST and asserts every `subprocess` call sits
    inside a function body, where it cannot fire until someone calls it. The
    default runner is bound inside `run_in_sandbox` for exactly this reason.
    """
    tree = ast.parse(SANDBOX_MODULE.read_text(encoding="utf-8"), filename=str(SANDBOX_MODULE))

    inside: set[ast.AST] = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            for child in ast.walk(node):
                inside.add(child)

    at_module_level = [
        node.lineno
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id == "subprocess"
        and node not in inside
    ]

    assert not at_module_level, (
        "sandbox.py calls subprocess at module level, so importing it would start "
        f"a process (line {at_module_level}). `just test` imports this module."
    )


def test_the_dockerfile_holds_no_version_of_its_own() -> None:
    """The pin is defined once (D-17). A `FROM rust:1.98.1` or a
    `nightly-2026-09-19` written into the Dockerfile would be a second place that
    has to agree with `pin.toml`, and this project's own history says that is how
    this class of thing regresses.
    """
    text = (SANDBOX_DIR / "Dockerfile").read_text(encoding="utf-8")
    code = "\n".join(
        line for line in text.splitlines() if not line.lstrip().startswith("#")
    )

    assert "rust:1." not in code, "the Dockerfile pins a rustc version of its own"
    assert "nightly-20" not in code, "the Dockerfile pins a nightly date of its own"
    # And it must still demand both, with no default, so a bare build fails loudly.
    assert "ARG RUST_VERSION\n" in code and "ARG NIGHTLY\n" in code


def test_the_containment_suite_is_not_collected_by_the_inner_loop() -> None:
    """`just test` is Docker-free by contract (EVALUATION.md, the harness table).
    The suite that needs a container lives in its own directory and its conftest
    refuses to run without being asked, so there is no path by which `just test`
    reaches Docker and no path by which the suite looks green without running.
    """
    conftest = (PIPELINE / "tests" / "sandbox" / "conftest.py").read_text(encoding="utf-8")

    assert "POPQUIZ_SANDBOX_SUITE" in conftest
    justfile = (PIPELINE.parent / "justfile").read_text(encoding="utf-8")
    assert "--ignore=tests/sandbox" in justfile


class _Done:
    """The three attributes `run_in_sandbox` reads back. Standing in for
    `subprocess.CompletedProcess` without importing it into the test surface."""

    def __init__(self, returncode: int, stdout: str, stderr: str) -> None:
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


def dataclass_replace(pin, **changes):
    """`dataclasses.replace` under a name that says what it is at the call site."""
    import dataclasses

    return dataclasses.replace(pin, **changes)
