"""Running a candidate program in the verification sandbox (T-15a, AC-12).

This module is not a verifier. It executes one step — compile, run, or Miri —
inside a container that has no network, no host filesystem, no inherited
environment and hard resource limits, and it reports what the sandbox did about
it. What any of that *means* for a candidate is T-15b's decision.

**It is the one module in this package allowed to start a process.** The guard in
`tests/test_runner.py` names it by path and fails if any other module acquires
the same ability, and fails again if this one stops needing the exemption. That
is the whole reason the exemption is a list of one rather than a deleted test.

**`just test` imports this module and must not be able to run Docker through it.**
Those are both true because the only call that starts anything is the injected
`runner`, whose default is bound inside `run_in_sandbox` rather than at import.
Importing this module executes nothing; `test_sandbox.py` asserts that
structurally, by walking the AST, rather than trusting the arrangement to stay
that way. Everything worth testing in the inner loop — the argv, the pin, the
verdicts — is reachable without a container.

**Two kinds of verdict, and conflating them would be a false claim.**

* *Configuration* verdicts (`NETWORK_UNREACHABLE`, `ENV_NOT_PASSED`,
  `HOST_FS_ISOLATED`, `ROOT_READ_ONLY`, `WORKDIR_IS_TMPFS`,
  `CAPABILITIES_DROPPED`) are facts about the argv. They hold whatever the
  program does, and they are derived from the argv itself so that the claim and
  the thing it describes cannot drift apart.
* *Outcome* verdicts (`TIMEOUT`, `MEMORY_LIMIT`) are facts about how the run
  ended, and nothing but a real run can establish them.

Notably absent: any verdict inferred from what a program printed. The AC-12
fixtures signal through exit codes they set themselves, and no assertion anywhere
quotes program output — writing down what a program prints is the one thing this
project does not do (`CLAUDE.md`, house rules).
"""

from __future__ import annotations

import hashlib
import subprocess  # noqa: S404 — see the module docstring; this is the exempt module
import time
import tomllib
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping, Protocol, Sequence

# pipeline/src/popquiz/sandbox.py -> pipeline/sandbox/pin.toml
PIN_PATH = Path(__file__).resolve().parents[2] / "sandbox" / "pin.toml"

#: The container's working directory, and the only writable path in it.
WORK_DIR = "/work"

#: What a caller may ask for. Every command template is keyed on them.
#:
#: `toolchain` runs no candidate: it prints the image's `rustc -Vv` and
#: `cargo miri --version`, which the verifier compares against the pin before it
#: runs anything (D-17, AC-6). It is the only mode that reads no source.
MODES = ("compile", "run", "miri", "toolchain")

#: The modes that take a candidate's source on stdin.
SOURCE_MODES = ("compile", "run", "miri")

#: `MIRIFLAGS` per borrow model (AC-9). Strict provenance under both: a
#: candidate whose answer leans on an integer-to-pointer cast is one Miri should
#: refuse to call clean. Tree Borrows is the second opinion the verifier asks
#: for when a candidate declares UB. The keys are `bank.BORROW_MODELS`, which
#: `test_sandbox.py` holds this dict to.
MIRI_CONFIGS: Mapping[str, tuple[str, ...]] = {
    "stacked_borrows": ("-Zmiri-strict-provenance",),
    "tree_borrows": ("-Zmiri-strict-provenance", "-Zmiri-tree-borrows"),
}
DEFAULT_MIRI_CONFIG = "stacked_borrows"

# Configuration verdicts — true because of how the container was asked for.
NETWORK_UNREACHABLE = "NETWORK_UNREACHABLE"
ENV_NOT_PASSED = "ENV_NOT_PASSED"
HOST_FS_ISOLATED = "HOST_FS_ISOLATED"
ROOT_READ_ONLY = "ROOT_READ_ONLY"
WORKDIR_IS_TMPFS = "WORKDIR_IS_TMPFS"
CAPABILITIES_DROPPED = "CAPABILITIES_DROPPED"

# Outcome verdicts — true because of how the run ended.
TIMEOUT = "TIMEOUT"
MEMORY_LIMIT = "MEMORY_LIMIT"

#: 128 + SIGKILL. The kernel's OOM killer uses it, and so does `docker kill`,
#: which is why `MEMORY_LIMIT` is only read off it when we did not do the killing.
_SIGKILL_EXIT = 137
#: 128 + SIGABRT. Rust's allocator aborts here when an allocation is refused
#: outright rather than the cgroup killing the process after it touches the pages.
_SIGABRT_EXIT = 134

#: What a runner raises when a step outlives its deadline.
#:
#: Aliased here so that nothing else — callers or tests — has to import
#: `subprocess` in order to talk about a timeout. That keeps the shell-out guard's
#: exemption at exactly one module, the one that owns execution, and it is better
#: encapsulation besides: that the transport happens to be a subprocess is this
#: module's business and nobody else's.
RunnerTimeout = subprocess.TimeoutExpired

#: Flags that would hand the container something it must not have. Used to derive
#: the configuration verdicts, so the verdict is a reading of the argv rather than
#: a second claim about it.
_MOUNT_FLAGS = ("-v", "--volume", "--mount")
_ENV_FLAGS = ("-e", "--env", "--env-file")


class SandboxError(Exception):
    """The sandbox could not be used as configured."""


class PinError(SandboxError):
    """`pin.toml` is missing, malformed, or has not had its recorded fields filled."""


@dataclass(frozen=True)
class Limits:
    """AC-12's enforced limits, as `pin.toml` gives them."""

    memory: str
    memory_swap: str
    cpus: str
    pids: int
    timeout_seconds: int
    tmpfs_size: str


@dataclass(frozen=True)
class Pin:
    """The toolchain pin, plus the image and limit values that sit beside it.

    `release`, `commit_hash` and `miri_version` are recorded from the built image.
    `rust_version` and `nightly` are what the build was asked for. `read_pin`
    checks that `release` agrees with `rust_version`, which is what catches an
    image built from a tag that does not match what `pin.toml` requested.
    """

    rust_version: str
    release: str
    commit_hash: str
    edition: str
    nightly: str
    miri_version: str
    target_triples: tuple[str, ...]
    flags: Mapping[str, Any]
    image_tag: str
    platform: str
    limits: Limits
    #: The first 12 hex digits of the Dockerfile's sha256, or "" when no Dockerfile
    #: sits beside the pin (a test's temporary copy). Part of the tag; see `image`.
    dockerfile_digest: str = ""

    @property
    def nightly_toolchain(self) -> str:
        """What `cargo +…` and `rustup` call the pinned nightly."""
        return f"nightly-{self.nightly}"

    @property
    def image(self) -> str:
        """The image reference, tagged with everything that decides its contents.

        That is the pin *and* the Dockerfile. Two images built from different
        inputs must never occupy the same name, because `sandbox-build` skips a tag
        that already exists — and a stale image still answering to the expected
        tag is exactly how a verification would silently run on the wrong image.
        With the versions alone in the tag, editing the Dockerfile under an
        unchanged pin would have been skipped as "already built".
        """
        tag = f"{self.image_tag}:{self.rust_version}-{self.nightly}"
        return f"{tag}-{self.dockerfile_digest}" if self.dockerfile_digest else tag


def read_pin(path: Path | str = PIN_PATH, *, require_recorded: bool = True) -> Pin:
    """Load the pin. The only reader of `pin.toml` in the package.

    `require_recorded=False` is for `sandbox-build`, which has to read the
    requested versions in order to produce the recorded ones. Everything else
    leaves it alone: a pin whose recorded fields are blank has never been
    confirmed against an image, and silently accepting one would mean verifying
    against a toolchain nobody has looked at.
    """
    path = Path(path)
    try:
        raw = tomllib.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise PinError(f"no pin at {path}. The sandbox has no toolchain without it.") from exc
    except tomllib.TOMLDecodeError as exc:
        raise PinError(f"{path} is not valid TOML: {exc}") from exc

    pin = raw.get("pin")
    if not isinstance(pin, dict):
        raise PinError(f"{path} has no [pin] table.")

    for field in ("rust_version", "nightly", "edition"):
        if not pin.get(field):
            raise PinError(f"{path}: [pin].{field} is empty, and it is the build's input.")

    if require_recorded:
        missing = [f for f in ("release", "commit_hash", "miri_version") if not pin.get(f)]
        if missing:
            raise PinError(
                f"{path}: [pin].{', [pin].'.join(missing)} "
                "is empty. Recorded fields come out of the built image — run "
                "`just sandbox-build` and paste back what it prints. Do not type "
                "them; this project does not write down what a program prints."
            )
        if pin["release"] != pin["rust_version"]:
            raise PinError(
                f"{path}: [pin].release is {pin['release']!r} but "
                f"[pin].rust_version asked for {pin['rust_version']!r}. The image "
                "was built from a different toolchain than this file requests."
            )

    triples = pin.get("target_triples") or []
    if not isinstance(triples, list) or not all(isinstance(t, str) for t in triples):
        raise PinError(f"{path}: [pin].target_triples must be a list of strings.")

    flags = pin.get("flags")
    if not isinstance(flags, dict):
        raise PinError(f"{path}: [pin.flags] is missing, and AC-6 records the flag set.")
    for field in ("opt-level", "overflow-checks", "debug-assertions"):
        if field not in flags:
            raise PinError(f"{path}: [pin.flags] has no {field!r}. SPEC.md §3.2 records it.")

    image = raw.get("image") or {}
    limits_raw = raw.get("limits")
    if not isinstance(limits_raw, dict):
        raise PinError(f"{path} has no [limits] table, and AC-12 is about limits.")
    try:
        limits = Limits(
            memory=str(limits_raw["memory"]),
            memory_swap=str(limits_raw["memory_swap"]),
            cpus=str(limits_raw["cpus"]),
            pids=int(limits_raw["pids"]),
            timeout_seconds=int(limits_raw["timeout_seconds"]),
            tmpfs_size=str(limits_raw["tmpfs_size"]),
        )
    except KeyError as exc:
        raise PinError(f"{path}: [limits] has no {exc.args[0]!r}.") from exc

    if not image.get("tag"):
        raise PinError(f"{path}: [image].tag is empty.")

    dockerfile = path.parent / "Dockerfile"
    digest = hashlib.sha256(dockerfile.read_bytes()).hexdigest()[:12] if dockerfile.is_file() else ""

    return Pin(
        rust_version=str(pin["rust_version"]),
        release=str(pin.get("release", "")),
        commit_hash=str(pin.get("commit_hash", "")),
        edition=str(pin["edition"]),
        nightly=str(pin["nightly"]),
        miri_version=str(pin.get("miri_version", "")),
        target_triples=tuple(triples),
        flags=dict(flags),
        image_tag=str(image["tag"]),
        platform=str(image.get("platform", "")),
        limits=limits,
        dockerfile_digest=digest,
    )


def _rustc_flags(pin: Pin) -> list[str]:
    """The pinned flag set as `rustc` arguments.

    Booleans become `on`/`off` because that is what `-C` takes; the record keeps
    them as booleans (SPEC.md §3.2), so this is the only place the two spellings
    meet.
    """

    def onoff(value: Any) -> str:
        if isinstance(value, bool):
            return "on" if value else "off"
        return str(value)

    return [
        "--edition",
        pin.edition,
        "-C",
        f"opt-level={pin.flags['opt-level']}",
        "-C",
        f"overflow-checks={onoff(pin.flags['overflow-checks'])}",
        "-C",
        f"debug-assertions={onoff(pin.flags['debug-assertions'])}",
    ]


def _profile_value(value: Any) -> str:
    """A `[pin.flags]` value as Cargo.toml spells it: booleans and integers bare,
    anything else (`"s"`, `"z"`) quoted."""
    if isinstance(value, bool):
        return "true" if value else "false"
    text = str(value)
    return text if text.isdigit() else f'"{text}"'


def cargo_manifest(pin: Pin) -> str:
    """The Cargo.toml of the Miri project, with the pinned flag set as `[profile.dev]`.

    Miri builds through cargo, not through `rustc` with `-C` flags, so without the
    profile table it would run under cargo's defaults and the record's `flags`
    would describe a build Miri never did. Written from the pin, like
    `_rustc_flags`, so the two cannot disagree.
    """
    return (
        "[package]\n"
        'name = "candidate"\n'
        'version = "0.0.0"\n'
        f'edition = "{pin.edition}"\n'
        "\n"
        "[profile.dev]\n"
        f"opt-level = {_profile_value(pin.flags['opt-level'])}\n"
        f"overflow-checks = {_profile_value(pin.flags['overflow-checks'])}\n"
        f"debug-assertions = {_profile_value(pin.flags['debug-assertions'])}\n"
    )


def miri_flags(config: str, seed: int) -> str:
    """The `MIRIFLAGS` value for one borrow model and one seed."""
    if config not in MIRI_CONFIGS:
        raise SandboxError(
            f"unknown Miri config {config!r}; the configs are {', '.join(MIRI_CONFIGS)}."
        )
    return " ".join((*MIRI_CONFIGS[config], f"-Zmiri-seed={int(seed)}"))


def container_command(
    mode: str,
    pin: Pin,
    *,
    miri_config: str = DEFAULT_MIRI_CONFIG,
    miri_seed: int = 0,
) -> list[str]:
    """The command the container runs, as `["sh", "-c", script]`.

    The source arrives on **stdin** and is written to a file by the script's first
    line. It is never an argument and never an environment variable: with no bind
    mount there is nowhere else for it to come from, and an argument would also
    put candidate source in the host's process table.

    Each mode is self-contained, because each call gets a fresh tmpfs and nothing
    survives between them. `run` therefore compiles again rather than reusing a
    binary from an earlier `compile`. That costs a compile per run and buys
    hermeticity: AC-8's N runs are N independent containers, so a determinism
    result cannot depend on state left behind by the run before it.
    """
    if mode not in MODES:
        raise SandboxError(f"unknown mode {mode!r}; the modes are {', '.join(MODES)}.")

    flags = " ".join(_rustc_flags(pin))

    if mode == "toolchain":
        # No candidate: the toolchain reporting itself, from inside the image the
        # candidate will run in. Cargo's home moves onto the tmpfs for the same
        # reason as in the Miri mode.
        script = (
            "set -e\n"
            "export CARGO_HOME=/work/.cargo\n"
            "rustc -Vv\n"
            f"exec cargo '+{pin.nightly_toolchain}' miri --version\n"
        )
    elif mode == "compile":
        # Diagnostics only. A candidate declared non-compiling (AC-11) is this
        # call's exit code and stderr; nothing is executed.
        script = f"set -e\ncat > main.rs\nrustc {flags} -o main main.rs\n"
    elif mode == "run":
        # `exec` so the program's own exit code and stdout are the container's.
        # When compilation fails this reports the compiler's exit code instead —
        # T-15b separates the two by calling `compile` first, which is why both
        # modes exist.
        script = f"set -e\ncat > main.rs\nrustc {flags} -o main main.rs\nexec ./main\n"
    else:
        # Miri needs a cargo project, so one is generated on the tmpfs. CARGO_HOME
        # and the target dir move onto the tmpfs because the root filesystem is
        # read-only; --offline makes a reach for the network a loud failure rather
        # than a hang, on top of --network none already making it impossible.
        #
        # MIRIFLAGS is exported inside the script, never passed with `-e`: the
        # container's environment stays the image's (ENV_NOT_PASSED). The manifest
        # goes through a quoted heredoc so no character in it is the shell's.
        script = (
            "set -e\n"
            "export CARGO_HOME=/work/.cargo CARGO_TARGET_DIR=/work/target\n"
            f"export MIRIFLAGS='{miri_flags(miri_config, miri_seed)}'\n"
            "mkdir -p candidate/src\n"
            "cat > candidate/src/main.rs\n"
            "cat > candidate/Cargo.toml <<'POPQUIZ_MANIFEST'\n"
            f"{cargo_manifest(pin)}"
            "POPQUIZ_MANIFEST\n"
            "cd candidate\n"
            f"exec cargo '+{pin.nightly_toolchain}' miri run --offline -q\n"
        )

    return ["sh", "-c", script]


def docker_argv(
    mode: str,
    pin: Pin,
    *,
    container_name: str,
    miri_config: str = DEFAULT_MIRI_CONFIG,
    miri_seed: int = 0,
) -> list[str]:
    """The full `docker run` argv for one step.

    Every flag here is load-bearing for AC-12, and
    `configuration_verdicts` reads this list back to say so:

    * ``--network none`` — no network.
    * ``--memory`` with ``--memory-swap`` equal to it. Without the second flag
      Docker allows swap at twice the limit, and a program that swaps instead of
      dying is not a contained program.
    * ``--cpus`` — a share of one core. This throttles; it does not kill, so no
      fixture can show it "firing". What bounds a runaway is the timeout.
    * ``--pids-limit`` — no fork bombs.
    * ``--read-only`` with ``--tmpfs /work`` — nothing writable but the work dir.
      The tmpfs is ``exec`` deliberately: a binary is written there and then run,
      so ``noexec`` would break the sandbox rather than harden it.
    * ``--cap-drop ALL`` and ``--security-opt no-new-privileges``.
    * **No** ``-e`` of any kind, and **no** mount. The container's environment is
      the image's and nothing else, and no host path is reachable.
    * ``-i`` — the source crosses on stdin.
    * ``--name`` — so a timeout can kill the container rather than only the CLI.
    """
    argv = ["docker", "run", "--rm", "-i", "--name", container_name]

    if pin.platform:
        argv += ["--platform", pin.platform]

    argv += [
        "--network",
        "none",
        "--memory",
        pin.limits.memory,
        "--memory-swap",
        pin.limits.memory_swap,
        "--cpus",
        pin.limits.cpus,
        "--pids-limit",
        str(pin.limits.pids),
        "--read-only",
        "--tmpfs",
        f"{WORK_DIR}:rw,exec,nosuid,nodev,size={pin.limits.tmpfs_size},mode=1777",
        "--cap-drop",
        "ALL",
        "--security-opt",
        "no-new-privileges",
        "--workdir",
        WORK_DIR,
        pin.image,
    ]
    return argv + container_command(
        mode, pin, miri_config=miri_config, miri_seed=miri_seed
    )


def configuration_verdicts(argv: Sequence[str]) -> frozenset[str]:
    """What this argv guarantees, whatever the program does.

    Read off the argv rather than asserted alongside it, so that a flag dropped
    from `docker_argv` takes its verdict with it instead of leaving a claim
    behind that nothing backs. That is the only reason this is a function of the
    argv and not a constant.
    """
    argv = list(argv)
    verdicts: set[str] = set()

    def flag_value(name: str) -> str | None:
        for i, item in enumerate(argv):
            if item == name and i + 1 < len(argv):
                return argv[i + 1]
            if item.startswith(f"{name}="):
                return item.split("=", 1)[1]
        return None

    if flag_value("--network") == "none":
        verdicts.add(NETWORK_UNREACHABLE)
    if not any(item.split("=", 1)[0] in _ENV_FLAGS for item in argv):
        verdicts.add(ENV_NOT_PASSED)
    if not any(item.split("=", 1)[0] in _MOUNT_FLAGS for item in argv):
        verdicts.add(HOST_FS_ISOLATED)
    if "--read-only" in argv:
        verdicts.add(ROOT_READ_ONLY)
    tmpfs = flag_value("--tmpfs")
    if tmpfs and tmpfs.split(":", 1)[0] == WORK_DIR:
        verdicts.add(WORKDIR_IS_TMPFS)
    if flag_value("--cap-drop") == "ALL":
        verdicts.add(CAPABILITIES_DROPPED)

    return frozenset(verdicts)


def outcome_verdicts(exit_code: int | None, *, timed_out: bool) -> frozenset[str]:
    """Which limit the run's ending says fired.

    `MEMORY_LIMIT` is *inferred* from the kill signal, and the inference is worth
    stating plainly: a container killed by the OOM killer exits 137, and one whose
    allocator refused an allocation outright aborts with 134. Neither carries a
    label saying "memory", so a 137 from some other external kill would read the
    same. It is disambiguated from our own timeout kill — which is also 137 —
    because we know whether we did the killing, and it is corroborated by the
    fixture being a program whose only action is to allocate. That is as far as
    the evidence goes and the README says so.
    """
    if timed_out:
        # Our own kill. Nothing about a memory limit can be read from this exit
        # code, so TIMEOUT is the only thing claimed.
        return frozenset({TIMEOUT})
    if exit_code in (_SIGKILL_EXIT, _SIGABRT_EXIT):
        return frozenset({MEMORY_LIMIT})
    return frozenset()


@dataclass(frozen=True)
class SandboxResult:
    """What one step did, and what the sandbox did about it."""

    mode: str
    exit_code: int | None
    stdout: str
    stderr: str
    timed_out: bool
    verdicts: frozenset[str]
    wall_clock_s: float
    container_name: str

    @property
    def contained(self) -> bool:
        """Whether a limit fired. Not "whether the candidate is good" — that is
        T-15b's question, and a perfectly good program fires no limit at all."""
        return bool(self.verdicts & {TIMEOUT, MEMORY_LIMIT})


class _Completed(Protocol):
    """The shape `run_in_sandbox` needs back. `subprocess.CompletedProcess` has
    it, and so does whatever a test hands in."""

    returncode: int
    stdout: str
    stderr: str


class Runner(Protocol):
    """How this module starts a process. The seam that keeps `just test` honest.

    The real one is `subprocess.run`. A test passes a callable with the same
    shape and never goes near Docker, which is what makes every decision in this
    module provable in the inner loop.
    """

    def __call__(
        self,
        argv: Sequence[str],
        *,
        input: str | None = None,
        timeout: float | None = None,
    ) -> _Completed: ...


def _subprocess_runner(
    argv: Sequence[str],
    *,
    input: str | None = None,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[str]:
    """The real runner. Bound inside `run_in_sandbox`, never at import time."""
    return subprocess.run(  # noqa: S603 — argv is built by docker_argv, never a shell string
        list(argv),
        input=input,
        capture_output=True,
        text=True,
        timeout=timeout,
        check=False,
    )


def run_in_sandbox(
    program_source: str,
    mode: str = "run",
    *,
    pin: Pin | None = None,
    runner: Runner | None = None,
    container_name: str | None = None,
    miri_config: str = DEFAULT_MIRI_CONFIG,
    miri_seed: int = 0,
) -> SandboxResult:
    """Run one step on `program_source` inside the sandbox.

    `runner` is the injection point: leave it out and the real `subprocess.run`
    is used, pass one and nothing is executed. That default is resolved *here*
    rather than in the signature, so importing this module binds nothing that can
    start a process.

    On expiry the container is killed, not merely abandoned. `subprocess`'s own
    timeout kills the `docker` CLI and leaves the container running, so the
    second call is what makes "hard timeout" true rather than approximately true.

    `miri_config` and `miri_seed` shape the `miri` mode only. The `toolchain` mode
    reads no source; pass `""`.
    """
    pin = pin or read_pin()
    run = runner or _subprocess_runner
    name = container_name or f"popquiz-sandbox-{uuid.uuid4().hex[:12]}"

    argv = docker_argv(
        mode, pin, container_name=name, miri_config=miri_config, miri_seed=miri_seed
    )
    timeout = pin.limits.timeout_seconds

    started = time.monotonic()
    timed_out = False
    try:
        completed = run(argv, input=program_source, timeout=timeout)
        exit_code: int | None = completed.returncode
        stdout, stderr = completed.stdout or "", completed.stderr or ""
    except RunnerTimeout as expired:
        timed_out = True
        exit_code = None
        stdout = _as_text(expired.stdout)
        stderr = _as_text(expired.stderr)
        # The CLI is gone; the container is not. Kill it by name, and do not let a
        # failure here mask the timeout we are already reporting.
        try:
            run(["docker", "kill", name], timeout=timeout)
        except Exception:  # noqa: BLE001 — a failed cleanup must not replace the result
            pass
    wall_clock_s = time.monotonic() - started

    return SandboxResult(
        mode=mode,
        exit_code=exit_code,
        stdout=stdout,
        stderr=stderr,
        timed_out=timed_out,
        verdicts=configuration_verdicts(argv)
        | outcome_verdicts(exit_code, timed_out=timed_out),
        wall_clock_s=wall_clock_s,
        container_name=name,
    )


def _as_text(value: object) -> str:
    """`TimeoutExpired` carries whatever the child produced before the kill, as
    bytes or str depending on how it was started."""
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return str(value)
