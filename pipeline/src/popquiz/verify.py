"""Verify a candidate: pinned rustc, N native runs, Miri (AC-6 to AC-13, SPEC.md 7.2).

**The verifier writes only what a machine observed.** Every field of the record it
produces is read off a step the `Runner` executed, or off the pin the steps ran
under; nothing is typed, remembered or inferred (G-2, CLAUDE.md). It reaches the
toolchain only through `popquiz.runner.Runner.run(kind, program_id)` - never
`rustc`, a binary or Miri directly - which is what lets `just test` prove every
decision below against recorded outputs with no Docker at all.

The procedure, in order. Each step is asked only if the one before it passed:

1. **The pin** (D-17, AC-6). `pin.toml` through `sandbox.read_pin()`, then the
   runner's `toolchain` step: its `rustc -Vv` `release` and `commit-hash`, and its
   `cargo miri --version` line, must equal the pin's recorded values, and its
   `host` must be a triple the pin supports. Any mismatch **refuses** - raises
   `PinMismatch` - before a candidate step is asked. Miri's version is compared as
   the whole recorded string rather than by the nightly's date: the toolchain name
   `+nightly-<date>` already fixes the date, and Miri's own build date is the day
   before it, so a date comparison would refuse every run.
2. **Compile** under the pin's edition and flag set. A candidate declared
   *does not compile* must fail with at least one `error[Exxxx]` code, and every
   code is recorded in the order the compiler gave them (AC-11); one that compiles
   is rejected. Any other candidate that fails to compile is rejected.
3. **N native runs** (N=5, a configured hypothesis - AC-8). Every run's stdout and
   exit code must be identical; otherwise the candidate is rejected, and the
   verdict says how many distinct outputs there were. Exit 0 or a panic (101) is
   an answer; any other exit is rejected, because "it exited 3" is not an answer
   any option kind names.
4. **Miri, Stacked Borrows with strict provenance** (AC-9, AC-10). UB when UB was
   not declared rejects. Clean but with Miri's stdout or exit code different from
   the native run's rejects. Miri failing without reporting UB (an unsupported
   operation, a leak) rejects: it could not check the program.
5. **Tree Borrows**, only for a candidate declared UB, and accepted only if both
   models reported UB (AC-9). Stacked Borrows clean on a declared-UB candidate
   rejects outright.

Refusals (`VerifyError`) say nothing about the candidate: the verifier would not
run. Rejections are a `Verdict` with a code and a plain-words reason, and no record
is written. An accepted verdict carries the SPEC 3.2 record as a `bank.Verified`,
which **replaces** any record the candidate had, whole (D-16).

**Miri proves absence of UB on the paths it executed, and nothing wider** (AC-43).
Nothing here says otherwise.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import re
import sys
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Literal

from popquiz import bank, sandbox
from popquiz.bank import (
    BORROW_MODELS,
    BankError,
    Flags,
    Miri,
    Question,
    Runs,
    Verified,
)
from popquiz.runner import COMPILE, TOOLCHAIN, RealRunner, Runner, RunStep, miri_kind, run_kind

#: What a candidate declares its answer to be. Not stored: the answer derives from
#: the record (G-2). The generator (T-16) passes it; `expect_from_record` recovers
#: it for re-verification (T-20).
Expect = Literal["ran", "does_not_compile", "ub"]
EXPECTS: tuple[Expect, ...] = ("ran", "does_not_compile", "ub")

#: AC-8's N. A configured hypothesis, not a proof of determinism.
RUNS = 5

#: The Miri seeds. One, recorded, so a re-run asks for the same schedule.
MIRI_SEEDS: tuple[int, ...] = (0,)

#: The exit code of a Rust panic.
PANIC_EXIT = 101

#: The marker Miri prints when it finds undefined behaviour.
_UB_MARKER = "error: Undefined Behavior"
_ERROR_CODE = re.compile(r"^error\[(E\d{4})\]", re.MULTILINE)

# Rejection codes. Stable strings: T-16's run report tallies them (AC-8's count).
ACCEPTED = "accepted"
COMPILED = "compiled_but_declared_does_not_compile"
NO_ERROR_CODE = "failed_without_an_error_code"
DID_NOT_COMPILE = "did_not_compile"
STOPPED = "stopped_by_a_limit"
NONDETERMINISTIC = "output_varied"
UNEXPLAINED_EXIT = "unexplained_exit_code"
UB_NOT_DECLARED = "ub_not_declared"
MIRI_OUTPUT_DIFFERS = "miri_output_differs"
MIRI_COULD_NOT_RUN = "miri_could_not_run"
UB_NOT_FOUND = "declared_ub_but_miri_clean"
MODELS_DISAGREE = "borrow_models_disagree"


class VerifyError(Exception):
    """The verifier refused to run. Says nothing about the candidate."""


class PinUnreadable(VerifyError):
    """`pin.toml` is missing, malformed, or its recorded fields are empty."""


class ToolchainUnreadable(VerifyError):
    """The runner's toolchain report is missing a line the pin is compared on."""


class PinMismatch(VerifyError):
    """The runner's toolchain is not the pinned one (D-17, AC-6)."""


class ProvenanceError(Exception):
    """A candidate's answer does not derive from a machine-written record (AC-7)."""


@dataclass(frozen=True)
class Toolchain:
    """What the runner's `toolchain` step reported, parsed."""

    rustc: str
    release: str
    commit_hash: str
    host: str
    miri_version: str


@dataclass(frozen=True)
class Verdict:
    """The outcome for one candidate.

    `verified` is the record to write, present only when `accepted`. `steps` lists
    the kinds that were asked, in order. `distinct_outputs` is set for a
    nondeterministic rejection (AC-8 reports it). `miri_wall_clock_s` sums the
    Miri steps' wall-clock where the runner measured it (Q-E4).
    """

    code: str
    reason: str
    verified: Verified | None = None
    steps: tuple[str, ...] = ()
    distinct_outputs: int | None = None
    miri_wall_clock_s: float | None = None

    @property
    def accepted(self) -> bool:
        return self.code == ACCEPTED


# --------------------------------------------------------------------------- #
# The pin
# --------------------------------------------------------------------------- #


def parse_toolchain(stdout: str) -> Toolchain:
    """Split the `toolchain` step's output into `rustc -Vv` and the Miri line.

    Raises `ToolchainUnreadable` when a line the pin is compared on is missing.
    """
    lines = stdout.splitlines()
    miri_lines = [line for line in lines if line.startswith("miri ")]
    rustc_lines = [line for line in lines if line and not line.startswith("miri ")]
    fields = dict(
        line.split(": ", 1) for line in rustc_lines[1:] if ": " in line
    )
    missing = [
        name
        for name, present in (
            ("the rustc version line", bool(rustc_lines) and rustc_lines[0].startswith("rustc ")),
            ("release", "release" in fields),
            ("commit-hash", "commit-hash" in fields),
            ("host", "host" in fields),
            ("the Miri version line", len(miri_lines) == 1),
        )
        if not present
    ]
    if missing:
        raise ToolchainUnreadable(
            "the runner's toolchain report is missing "
            + ", ".join(missing)
            + ", so it cannot be compared with the pin. The verifier will not run."
        )
    return Toolchain(
        rustc="\n".join(rustc_lines),
        release=fields["release"],
        commit_hash=fields["commit-hash"],
        host=fields["host"],
        miri_version=miri_lines[0],
    )


def check_toolchain(toolchain: Toolchain, pin: sandbox.Pin) -> None:
    """Refuse a toolchain that is not the pinned one (D-17). Plain words."""
    problems = []
    if toolchain.release != pin.release:
        problems.append(f"rustc release is {toolchain.release}, the pin says {pin.release}")
    if toolchain.commit_hash != pin.commit_hash:
        problems.append(
            f"rustc commit-hash is {toolchain.commit_hash}, the pin says {pin.commit_hash}"
        )
    if toolchain.miri_version != pin.miri_version:
        problems.append(
            f"Miri reports {toolchain.miri_version!r}, the pin says {pin.miri_version!r}"
        )
    if toolchain.host not in pin.target_triples:
        problems.append(
            f"the host triple {toolchain.host} is not one the pin supports "
            f"({', '.join(pin.target_triples)})"
        )
    if problems:
        raise PinMismatch(
            "the toolchain does not match the pin, so nothing was verified: "
            + "; ".join(problems)
            + ". Rebuild the image with `just sandbox-build`, or bump the pin."
        )


def _load_pin(pin: sandbox.Pin | None) -> sandbox.Pin:
    if pin is not None:
        missing = [f for f in ("release", "commit_hash", "miri_version") if not getattr(pin, f)]
        if missing:
            raise PinUnreadable(
                f"the pin has no recorded {', '.join(missing)}, so there is nothing to "
                "compare the toolchain with. The verifier will not run."
            )
        return pin
    try:
        return sandbox.read_pin()
    except sandbox.PinError as exc:
        raise PinUnreadable(f"{exc} The verifier will not run.") from exc


# --------------------------------------------------------------------------- #
# The procedure
# --------------------------------------------------------------------------- #


def error_codes(stderr: str) -> tuple[str, ...]:
    """Every `error[Exxxx]` code, once each, in the order the compiler gave them."""
    return tuple(dict.fromkeys(_ERROR_CODE.findall(stderr)))


def verifier_version() -> str:
    """Derived from the code that decides, never typed: a digest of this module,
    the runner seam and the sandbox. A change to any of them changes it."""
    here = Path(__file__).resolve().parent
    digest = hashlib.sha256()
    for name in ("verify.py", "runner.py", "sandbox.py"):
        digest.update((here / name).read_bytes())
    return f"popquiz-verify {digest.hexdigest()[:12]}"


def _utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _flags(pin: sandbox.Pin) -> Flags:
    return Flags(
        opt_level=str(pin.flags["opt-level"]),
        overflow_checks=bool(pin.flags["overflow-checks"]),
        debug_assertions=bool(pin.flags["debug-assertions"]),
    )


def _stopped(step: RunStep) -> str | None:
    if step.stopped_by or step.exit_code is None:
        fired = ", ".join(step.stopped_by) or "no exit code"
        return f"the {step.kind} step was stopped by the sandbox ({fired})"
    return None


def verify(
    question: Question,
    runner: Runner,
    *,
    expect: Expect,
    pin: sandbox.Pin | None = None,
    runs: int = RUNS,
    now: Callable[[], str] = _utc_now,
) -> Verdict:
    """Run the procedure on one candidate. See the module docstring.

    Raises `VerifyError` when it refuses to run. Returns a `Verdict` otherwise.
    """
    if expect not in EXPECTS:
        raise ValueError(f"expect must be one of {', '.join(EXPECTS)}, not {expect!r}")
    pin = _load_pin(pin)
    asked: list[str] = []

    def ask(kind: str) -> RunStep:
        asked.append(kind)
        return runner.run(kind, question.id)

    def reject(code: str, reason: str, **extra: Any) -> Verdict:
        return Verdict(code=code, reason=reason, steps=tuple(asked), **extra)

    toolchain = parse_toolchain(ask(TOOLCHAIN).stdout)
    check_toolchain(toolchain, pin)

    def record(**fields: Any) -> Verified:
        return Verified(
            rustc=toolchain.rustc,
            edition=pin.edition,
            target_triple=toolchain.host,
            flags=_flags(pin),
            verified_at=now(),
            verifier_version=verifier_version(),
            **fields,
        )

    # --- compile ----------------------------------------------------------
    compiled = ask(COMPILE)
    if (stopped := _stopped(compiled)) is not None:
        return reject(STOPPED, stopped)
    if compiled.exit_code != 0:
        codes = error_codes(compiled.stderr)
        if expect != "does_not_compile":
            return reject(
                DID_NOT_COMPILE,
                f"it did not compile ({', '.join(codes) or 'no error code'}), "
                f"and it was declared to {'have UB' if expect == 'ub' else 'run'}",
            )
        if not codes:
            return reject(
                NO_ERROR_CODE,
                "it failed to compile without an error code, so there is no "
                "error to record (AC-11)",
            )
        return Verdict(
            code=ACCEPTED,
            reason=f"it does not compile, as declared: {', '.join(codes)}",
            verified=record(compile_error_code=codes),
            steps=tuple(asked),
        )
    if expect == "does_not_compile":
        return reject(COMPILED, "it compiled, and it was declared not to (AC-11)")

    # --- N native runs ----------------------------------------------------
    observed: list[tuple[str, int | None]] = []
    for i in range(1, runs + 1):
        step = ask(run_kind(i))
        if (stopped := _stopped(step)) is not None:
            return reject(STOPPED, stopped)
        observed.append((step.stdout, step.exit_code))
    distinct = len(set(observed))
    if distinct > 1:
        return reject(
            NONDETERMINISTIC,
            f"its output varied: {distinct} distinct outputs in {runs} runs (AC-8)",
            distinct_outputs=distinct,
        )
    stdout, exit_code = observed[0]
    if exit_code not in (0, PANIC_EXIT):
        return reject(
            UNEXPLAINED_EXIT,
            f"it exited {exit_code} on every run, which is neither success nor a panic",
        )

    # --- Miri -------------------------------------------------------------
    miri_steps: list[RunStep] = []

    def miri(model: str) -> tuple[str, RunStep | None, str]:
        """Run one model over every seed. Returns (outcome, step, reason) where
        outcome is 'ub', 'clean' or a rejection code."""
        last: RunStep | None = None
        saw_ub = False
        for seed in MIRI_SEEDS:
            step = ask(miri_kind(model, seed))
            miri_steps.append(step)
            last = step
            if (stopped := _stopped(step)) is not None:
                return STOPPED, step, stopped
            if _UB_MARKER in step.stderr:
                saw_ub = True
                continue
            if step.exit_code != exit_code:
                return (
                    MIRI_COULD_NOT_RUN,
                    step,
                    f"Miri ({model}) exited {step.exit_code} without reporting UB, "
                    f"where the native run exited {exit_code}: it could not check "
                    "the program",
                )
            if step.stdout != stdout:
                return (
                    MIRI_OUTPUT_DIFFERS,
                    step,
                    f"Miri's output ({model}) differs from the native output (AC-10)",
                )
        return ("ub" if saw_ub else "clean"), last, ""

    def miri_seconds() -> float | None:
        times = [s.wall_clock_s for s in miri_steps if s.wall_clock_s is not None]
        return round(sum(times), 3) if times else None

    outcome, sb_step, why = miri("stacked_borrows")
    if outcome not in ("ub", "clean"):
        return reject(outcome, why, miri_wall_clock_s=miri_seconds())

    if expect == "ran":
        if outcome == "ub":
            return reject(
                UB_NOT_DECLARED,
                "Miri (Stacked Borrows) found undefined behavior, and UB was not "
                "the declared answer (AC-9)",
                miri_wall_clock_s=miri_seconds(),
            )
        return Verdict(
            code=ACCEPTED,
            reason="it ran the same every time and Miri ran clean on the paths it executed",
            verified=record(
                runs=Runs(count=runs, byte_identical=True),
                stdout=stdout,
                exit_code=exit_code,
                miri=Miri(
                    clean=True,
                    output_matched=True,
                    version=toolchain.miri_version,
                    configs=("stacked_borrows",),
                    seeds=MIRI_SEEDS,
                ),
            ),
            steps=tuple(asked),
            miri_wall_clock_s=miri_seconds(),
        )

    # expect == "ub"
    if outcome == "clean":
        return reject(
            UB_NOT_FOUND,
            "UB was declared, but Miri (Stacked Borrows) ran clean on the paths it "
            "executed (AC-9)",
            miri_wall_clock_s=miri_seconds(),
        )
    tb_outcome, tb_step, why = miri("tree_borrows")
    if tb_outcome not in ("ub", "clean"):
        return reject(tb_outcome, why, miri_wall_clock_s=miri_seconds())
    if tb_outcome == "clean":
        return reject(
            MODELS_DISAGREE,
            "Stacked Borrows found UB and Tree Borrows did not; declared UB is "
            "accepted only when both agree (AC-9)",
            miri_wall_clock_s=miri_seconds(),
        )
    # Both reported UB. Their output up to the UB must be the native output,
    # exactly: `output_matched` is equality, never a prefix (AC-10).
    assert sb_step is not None and tb_step is not None
    for model, step in (("stacked_borrows", sb_step), ("tree_borrows", tb_step)):
        if step.stdout != stdout:
            return reject(
                MIRI_OUTPUT_DIFFERS,
                f"Miri's output before the UB ({model}) differs from the native "
                "output (AC-10); a UB question prints only before its UB",
                miri_wall_clock_s=miri_seconds(),
            )
    return Verdict(
        code=ACCEPTED,
        reason="Miri found undefined behavior under both borrow models, as declared",
        verified=record(
            runs=Runs(count=runs, byte_identical=True),
            stdout=stdout,
            exit_code=exit_code,
            miri=Miri(
                clean=False,
                output_matched=True,
                version=toolchain.miri_version,
                configs=BORROW_MODELS,
                seeds=MIRI_SEEDS,
            ),
        ),
        steps=tuple(asked),
        miri_wall_clock_s=miri_seconds(),
    )


def with_record(question: Question, verdict: Verdict) -> Question:
    """The candidate with the verdict's record in place of whatever it had -
    replaced whole, never merged (D-16)."""
    if verdict.verified is None:
        raise ValueError(f"{question.id}: a rejected verdict has no record to write")
    return dataclasses.replace(question, verified=verdict.verified)


# --------------------------------------------------------------------------- #
# Reads for scheduling (T-20) and the build (AC-7)
# --------------------------------------------------------------------------- #


def _vv_field(rustc: str, name: str) -> str | None:
    for line in rustc.splitlines():
        if line.startswith(f"{name}: "):
            return line.split(": ", 1)[1]
    return None


def is_stale(verified: Verified, pin: sandbox.Pin) -> bool:
    """Whether a record must re-verify before it is scheduled (SPEC 7.2, AC-6).

    A `legacy` record is exempt: it was never verified under any pin. Anything
    else is stale unless its `rustc` release and commit-hash, its edition, its
    flag set and - when Miri ran - Miri's version all equal the pin's: the whole
    `[pin]` table is what `pin.toml` says a record is compared against. A `rustc`
    that is not a `-Vv` report cannot be compared, so it is stale.
    """
    if verified.legacy:
        return False
    if (
        _vv_field(verified.rustc, "release") != pin.release
        or _vv_field(verified.rustc, "commit-hash") != pin.commit_hash
        or verified.edition != pin.edition
        or verified.flags != _flags(pin)
    ):
        return True
    return verified.miri is not None and verified.miri.version != pin.miri_version


def expect_from_record(verified: Verified) -> Expect | None:
    """What a record says the candidate was declared to be, for re-verification."""
    kind = bank.receipt_class(verified)
    if kind == "does_not_compile":
        return "does_not_compile"
    if kind == "ran":
        return "ub" if bank.ub_confirmed(verified) else "ran"
    return None


def _record_problems(v: Verified) -> list[str]:
    """Fields of one record that contradict each other."""
    problems = []
    if v.compile_error_code:
        extra = [
            name
            for name in ("runs", "stdout", "exit_code", "miri")
            if getattr(v, name) is not None
        ]
        if extra:
            problems.append(
                f"a does-not-compile record carries {', '.join(extra)}, but nothing ran"
            )
        return problems
    if v.runs is None:
        problems.append("the record has neither runs nor a compile error")
        return problems
    if not v.runs.byte_identical:
        problems.append("the record's runs were not byte-identical, so it was rejected")
    if not v.legacy:
        missing = [
            name
            for name in ("target_triple", "flags", "stdout", "exit_code", "miri")
            if getattr(v, name) is None
        ]
        if missing:
            problems.append(f"a complete record is missing {', '.join(missing)}")
    if v.miri is not None:
        if v.miri.configs is not None:
            unknown = sorted(set(v.miri.configs) - set(BORROW_MODELS))
            if unknown:
                problems.append(f"Miri configs {unknown} are not borrow models")
        if not v.miri.output_matched:
            problems.append("Miri's output did not match, so it was rejected (AC-10)")
        if not v.legacy and v.miri.clean is False and not bank.ub_confirmed(v):
            problems.append("Miri found UB under only one borrow model (AC-9)")
    return problems


def check_provenance(data: Mapping[str, Any]) -> Question:
    """The build's provenance check (AC-7, G-2). Returns the question or raises.

    Takes a candidate file's contents as parsed JSON, because a hand edit is an
    edit to the file. Refuses:

    * a `correct` field, or any other field the format does not have
      (`bank.question_from_dict` refuses unknown fields);
    * a record `verify` did not write: no `verified`, or a non-legacy record
      without `verified_at` and `verifier_version`, or fields that contradict
      each other (a does-not-compile record that ran, UB under one model);
    * options that no longer derive an answer from the record - an option's text
      or kind hand-edited away from what the machine observed, or two options now
      claiming it.

    **What it cannot catch, stated plainly:** a hand edit *inside* the record
    that leaves it self-consistent - `stdout` changed together with the matching
    option's text. Nothing in SPEC 3.2 binds the record to the run that produced
    it; closing that needs a field the contract does not have (a digest over the
    record), which is reported upstream rather than invented here.
    """
    if "correct" in data:
        raise ProvenanceError(
            f"{data.get('id', '<no id>')}: carries a 'correct' field. The correct "
            "answer is derived from the verified record and is never stored (G-2)."
        )
    try:
        question = bank.question_from_dict(dict(data))
    except BankError as exc:
        raise ProvenanceError(str(exc)) from exc

    v = question.verified
    if v is None:
        raise ProvenanceError(f"{question.id}: has no verified record; nothing derives")
    if not v.legacy and (not v.verified_at or not v.verifier_version):
        raise ProvenanceError(
            f"{question.id}: the record has no verified_at or verifier_version, so "
            "it was not written by the verifier"
        )
    problems = _record_problems(v)
    if problems:
        raise ProvenanceError(f"{question.id}: " + "; ".join(problems))
    try:
        index = bank.correct_index(question)
    except BankError as exc:
        raise ProvenanceError(str(exc)) from exc
    if index is None:
        raise ProvenanceError(
            f"{question.id}: no option derives from the verified record - the "
            "options and what the machine observed have drifted apart (AC-7)"
        )
    return question


# --------------------------------------------------------------------------- #
# `verify <program>`
# --------------------------------------------------------------------------- #


def main(argv: Sequence[str] | None = None, *, runner: Runner | None = None) -> int:
    """`python -m popquiz.verify <candidate.json> [--expect ...]`.

    Exit 0: accepted, and the record was written into the candidate file.
    Exit 1: rejected; the file is untouched. Exit 2: refused, or unusable input.
    """
    parser = argparse.ArgumentParser(
        prog="verify",
        description="Verify one candidate on the pinned toolchain and write its record.",
    )
    parser.add_argument("candidate", type=Path, help="a candidate question's JSON file")
    parser.add_argument(
        "--expect",
        choices=EXPECTS,
        help="the declared answer; required unless the file already has a record",
    )
    args = parser.parse_args(argv)

    try:
        data = json.loads(args.candidate.read_text(encoding="utf-8"))
        question = bank.question_from_dict(data)
    except (OSError, json.JSONDecodeError, BankError, KeyError) as exc:
        print(f"verify: cannot read {args.candidate}: {exc}", file=sys.stderr)
        return 2

    expect = args.expect or (
        expect_from_record(question.verified) if question.verified else None
    )
    if expect is None:
        print(
            "verify: say what the candidate is declared to do with --expect "
            f"({', '.join(EXPECTS)}); it has no record to recover it from.",
            file=sys.stderr,
        )
        return 2

    try:
        pin = _load_pin(None)
        verdict = verify(
            question,
            runner or RealRunner({question.id: question.source}, pin=pin),
            expect=expect,
            pin=pin,
        )
    except VerifyError as exc:
        print(f"verify: refused. {exc}", file=sys.stderr)
        return 2

    timing = (
        f" Miri took {verdict.miri_wall_clock_s:.1f} s."
        if verdict.miri_wall_clock_s is not None
        else ""
    )
    if not verdict.accepted:
        print(f"{question.id}: REJECTED ({verdict.code}): {verdict.reason}.{timing}")
        return 1

    bank._write_json(args.candidate, bank.question_to_dict(with_record(question, verdict)))
    print(f"{question.id}: ACCEPTED: {verdict.reason}.{timing} Record written to {args.candidate}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
