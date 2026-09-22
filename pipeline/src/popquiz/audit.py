"""bank-audit: the slot generator, the enumerated tells, and fit (SPEC 7.6).

Criteria: AC-23, AC-23a, AC-23b, AC-24, AC-25, AC-26, AC-27, AC-88, AC-100; guardrails
G-1 and G-11. Run it as `just bank-audit`; every check also runs in `just test`.

**The one rule this module keeps: nothing in it can change tonight's answer
position.** The answer's letter is `slot_for_day(date)`, which lives in
`popquiz.slot`, takes the date and nothing else, and never imports this module.
What this module does with the slot is call it on *synthetic* nights and measure
what comes out. It never reads a real night, never reads the used-question record,
and never hands anything it computes to anything that places an answer. PHILOSOPHY.md
section 2 says why this is the rule above the others: *a fairness mechanism that
shapes output is an oracle*. If a check could change what tonight's answer is, an
attendee could run the same check. That guardrail has regressed four times, once
while somebody was fixing the previous regression.

So the audit tests the **generator** in both tails - too skewed is a broken
generator, too even is somebody balancing it - and it tests the **bank** for tells
in the questions' *content*, whose remedy is re-authoring a question. Neither kind
of finding has a position to adjust, because there is no position state anywhere.

What this module is, in order:

* the generator audit (AC-23b), ported from `mvp/tools/build_deck.py:175-217`;
* the attendee simulation (AC-23a), written new;
* the lints that keep the slot path pure and the ledger retired (AC-23, G-1, G-10);
* the bank checks: five options (AC-24), no published distribution (AC-25, G-11),
  the enumerated tells (AC-26), `unsafe` parity (AC-27), difficulty drift (AC-88);
* the wall's type model mirrored from `web/shared/typemodel.js` (AC-100, D-15);
* the report, written only under `bank/audit/`, and the command line.
"""

from __future__ import annotations

import argparse
import ast
import datetime
import hashlib
import json
import random
import re
import sys
from collections import Counter
from collections.abc import Callable, Iterable, Mapping, Sequence
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Literal

from popquiz import bank
from popquiz import slot as _slot_module
from popquiz.bank import Question
from popquiz.slot import slot_for_day

__all__ = [
    "AuditFailure",
    "audit_generator",
    "simulate_attendees",
    "slot_for_day",
    "slot_path_violations",
    "slot_call_violations",
    "ledger_violations",
    "check_five_options",
    "distribution_lint",
    "measure_tells",
    "check_unsafe_parity",
    "difficulty_drift",
    "Room",
    "floor_px",
    "source_metrics",
    "desired_font_px",
    "fit",
    "option_fits",
    "question_flags",
    "run_audit",
    "write_report",
    "main",
]

Verdict = Literal["pass", "warn", "fail", "n/a"]

LETTERS = "ABCDE"
OPTION_COUNT = bank.OPTION_COUNT
CHANCE = 1 / OPTION_COUNT

# The pipeline directory and the repository, for the default command line. The
# justfile passes --repo explicitly; this is only the fallback.
PIPELINE_SRC = Path(__file__).resolve().parent
DEFAULT_REPO = PIPELINE_SRC.parents[2]


class AuditFailure(Exception):
    """A check found what it exists to find. Raised rather than asserted: `assert`
    vanishes under `python -O`, and an audit that can be switched off by an
    interpreter flag is not one."""


@dataclass(frozen=True)
class Check:
    """One line of the report: which criteria, what was measured, and the verdict.

    `warn` is reported and printed and does not fail the run (see `run_audit`);
    `n/a` means the check had nothing to measure, which is said rather than
    passed silently.
    """

    name: str
    criteria: tuple[str, ...]
    verdict: Verdict
    summary: str
    detail: Mapping[str, Any] = field(default_factory=dict)


# --------------------------------------------------------------------------- #
# AC-23b - the generator, in both tails
# --------------------------------------------------------------------------- #

# Chi-square critical values, df = 4 (five options). Two-tailed on purpose: the
# upper tail catches a skewed generator; the LOWER tail catches someone
# reintroducing balancing - output too evenly spread is not evidence of fairness,
# it is evidence of a constraint, which is the exploitable case.
#
# The tails are not symmetric, and that is inherited, not invented. The upper
# value is the 0.1 % point (P(X >= 18.467) = 0.001). The lower value is the 1 %
# point (P(X <= 0.297) = 0.0100); `build_deck.py:180` labels it "p = 0.999", which
# is wrong - the 0.1 % lower point is 0.091. The stricter 1 % value is kept
# because it is the one that catches balancing, and the label is corrected here.
# `tests/test_audit.py` checks both against df=4's closed-form CDF.
CHI2_DF4_UPPER = 18.467
CHI2_DF4_LOWER = 0.297

GENERATOR_SAMPLE = 20_000
# The first meetup the segment was scheduled for. The synthetic nights are the
# 20,000 days from there, the dates the generator will actually be asked about.
GENERATOR_START = datetime.date(2026, 8, 12)
# A memoryless draw must repeat itself. Counted over the first 2,000 draws, as the
# MVP did.
REPEAT_WINDOW = 2_000

SlotGenerator = Callable[[datetime.date], int]


@dataclass(frozen=True)
class GeneratorAudit:
    sample: int
    start: str
    counts: tuple[int, ...]
    chi2: float
    repeats: int

    def summary(self) -> str:
        spread = ", ".join(f"{LETTERS[i]}={c}" for i, c in enumerate(self.counts))
        return (
            f"generator uniform: chi2={self.chi2:.2f} (df=4, both tails) over "
            f"{self.sample} synthetic nights from {self.start}; {spread}; "
            f"{self.repeats} repeats in the first {REPEAT_WINDOW}"
        )


def synthetic_nights(start: datetime.date, count: int) -> list[datetime.date]:
    return [start + datetime.timedelta(days=i) for i in range(count)]


def audit_generator(
    generator: SlotGenerator = slot_for_day,
    sample: int = GENERATOR_SAMPLE,
    start: datetime.date = GENERATOR_START,
) -> GeneratorAudit:
    """Fails if the slot generator is biased, in either direction. Raises `AuditFailure`.

    Tested against a large synthetic run of dates, never against a real night or
    the used-question record. That gives the test enough power to catch a broken
    generator while constraining nothing about what tonight's position turns out
    to be. An audit that can change tonight's output is not an audit, it is a
    rule - and rules leak.

    `generator` is injectable so the failure-mode tests can hand it a skewed or a
    re-balanced one. The real run always audits `slot_for_day` itself.
    """
    nights = synthetic_nights(start, sample)
    draws = [generator(night) for night in nights]
    counts = [0] * OPTION_COUNT
    for n, slot in enumerate(draws):
        if not (isinstance(slot, int) and 0 <= slot < OPTION_COUNT):
            raise AuditFailure(f"generator drew {slot!r} for {nights[n]}, not a slot 0..4")
        counts[slot] += 1

    expected = sample / OPTION_COUNT
    chi2 = sum((c - expected) ** 2 / expected for c in counts)
    spread = ", ".join(f"{LETTERS[i]}={c}" for i, c in enumerate(counts))
    if chi2 >= CHI2_DF4_UPPER:
        raise AuditFailure(
            f"slot generator is skewed (chi2={chi2:.1f} over {sample} draws): {spread}"
        )
    if chi2 <= CHI2_DF4_LOWER:
        raise AuditFailure(
            f"slot generator is TOO EVEN (chi2={chi2:.3f} over {sample} draws): {spread}. "
            "Something is balancing the output. Balanced answer positions are "
            "predictable answer positions; see PHILOSOPHY.md section 2."
        )

    window = draws[:REPEAT_WINDOW]
    repeats = sum(1 for a, b in zip(window, window[1:]) if a == b)
    if repeats == 0:
        raise AuditFailure(
            "slot generator never repeats a position on consecutive draws - an "
            "attendee could rule out last meetup's letter for free"
        )
    return GeneratorAudit(sample, start.isoformat(), tuple(counts), chi2, repeats)


# --------------------------------------------------------------------------- #
# AC-23a - three attendees with perfect memory
# --------------------------------------------------------------------------- #

ATTENDEE_NIGHTS = 10_000
ATTENDEE_START = GENERATOR_START
# The simulation's own randomness: only the not-last-night's attendee needs any.
ATTENDEE_SEED = 20260918
# Chance is 20 %; over 10,000 nights the standard error is 0.4 points, so the band
# is five standard errors wide on each side and cannot flake (EVALUATION AC-23a).
ATTENDEE_BAND = (0.18, 0.22)


class _MostUsed:
    """Always guesses the letter used most so far; ties go to the lowest letter."""

    def __init__(self) -> None:
        self.counts = [0] * OPTION_COUNT

    def guess(self) -> int:
        return max(range(OPTION_COUNT), key=lambda k: (self.counts[k], -k))

    def observe(self, slot: int) -> None:
        self.counts[slot] += 1


class _LeastUsed(_MostUsed):
    """Always guesses the letter used least so far - the balancer's own logic,
    turned into a guess. Ties go to the lowest letter."""

    def guess(self) -> int:
        return min(range(OPTION_COUNT), key=lambda k: (self.counts[k], k))


class _NotLastNight:
    """Guesses uniformly among the letters that were not last night's. On the first
    night there is no last night, so it guesses uniformly among all five."""

    def __init__(self, rng: random.Random) -> None:
        self.rng = rng
        self.previous: int | None = None

    def guess(self) -> int:
        choices = [k for k in range(OPTION_COUNT) if k != self.previous]
        return self.rng.choice(choices)

    def observe(self, slot: int) -> None:
        self.previous = slot


def simulate_attendees(
    generator: SlotGenerator = slot_for_day,
    start: datetime.date = ATTENDEE_START,
    nights: int = ATTENDEE_NIGHTS,
    seed: int = ATTENDEE_SEED,
) -> dict[str, float]:
    """Each attendee's hit rate over `nights` consecutive nights (AC-23a).

    Every guess is collected **before** that night's slot is drawn, and each
    attendee is told the slot only afterwards, so "sees only the past slots" holds
    by the structure of the loop rather than by care. The slots it draws are
    synthetic nights' and go nowhere but into these counts.
    """
    attendees = {
        "most used so far": _MostUsed(),
        "least used so far": _LeastUsed(),
        "not last night's": _NotLastNight(random.Random(seed)),
    }
    hits = dict.fromkeys(attendees, 0)
    for night in synthetic_nights(start, nights):
        guesses = {name: a.guess() for name, a in attendees.items()}
        slot = generator(night)
        for name, attendee in attendees.items():
            hits[name] += guesses[name] == slot
            attendee.observe(slot)
    return {name: hits[name] / nights for name in attendees}


def attendees_outside_band(rates: Mapping[str, float]) -> dict[str, float]:
    low, high = ATTENDEE_BAND
    return {name: r for name, r in rates.items() if not (low <= r <= high)}


# --------------------------------------------------------------------------- #
# AC-23, G-1, G-10 - the slot path stays pure and the ledger stays retired
# --------------------------------------------------------------------------- #
#
# These are static: source text handed to `ast.parse`, never executed. They catch
# the ordinary ways the slot path could acquire an input - a parameter, an import,
# a module-level variable, a file read - and the ordinary shape of the MVP's
# ledger-writing wrapper. They do NOT survive an author working around them: a
# name computed at runtime, an attribute reached through `getattr` on an allowed
# object, or a write split across two functions would all pass. They exist so that
# anyone widening the slot path has to do it in a way a reader sees - the same
# honesty `tests/test_runner.py` keeps about its process guard.

SLOT_IMPORTS = frozenset({"__future__", "hashlib", "datetime"})
SLOT_BUILTINS = frozenset({"int", "str", "map", "type", "isinstance", "TypeError", "ValueError"})
SLOT_ATTRIBUTES = frozenset({"blake2b", "hexdigest", "encode", "join", "isoformat", "date"})
FORBIDDEN_CALLS = frozenset(
    {"open", "getattr", "setattr", "globals", "locals", "vars", "eval", "exec", "compile", "__import__"}
)
# A name on the slot path that says any of these is reaching for history.
FORBIDDEN_WORDS = re.compile(r"history|ledger|previous|prior|last|used|count|balance|quota|bank", re.I)
LEDGER_NAME = re.compile(r"answer[-_ ]?history", re.I)
FILE_WRITES = frozenset({"open", "write_text", "write_bytes", "dump", "writelines"})


def _docstring_nodes(tree: ast.AST) -> set[int]:
    """ids of the Constant nodes that are docstrings, so prose is not linted as code."""
    ids = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            body = node.body
            if (
                body
                and isinstance(body[0], ast.Expr)
                and isinstance(body[0].value, ast.Constant)
                and isinstance(body[0].value.value, str)
            ):
                ids.add(id(body[0].value))
    return ids


def _bound_in_functions(tree: ast.AST) -> set[str]:
    """Every name a function in the module binds: parameters, assignments, nested
    defs. Module-level bindings are checked separately and more strictly."""
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
            a = node.args
            for arg in (*a.posonlyargs, *a.args, *a.kwonlyargs):
                names.add(arg.arg)
            if a.vararg:
                names.add(a.vararg.arg)
            if a.kwarg:
                names.add(a.kwarg.arg)
            if not isinstance(node, ast.Lambda):
                for inner in ast.walk(node):
                    if isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Store):
                        names.add(inner.id)
                    elif isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef)) and inner is not node:
                        names.add(inner.name)
    return names


def slot_path_violations(source: str) -> list[str]:
    """What is wrong with this source as the slot path (AC-23 static, G-1). Empty is clean.

    The slot module may hold a docstring, imports of `hashlib`/`datetime`,
    functions, and module constants bound once to an int or str literal - nothing
    else at the top level, so there is nowhere for state to live. `slot_for_day`
    must take exactly `(day, n_options=5)`. Inside, every name read must be a
    parameter, a local, an import, one of the module's own functions or literal
    constants, or an allow-listed builtin; attributes are allow-listed; `global`,
    `with`, nested imports and the calls that reach outside (`open`, `getattr`,
    `eval`, ...) are refused; and no identifier or non-docstring string may say
    history, ledger, previous, last, count, balance, quota or bank.

    `nonlocal` is allowed: Python only lets it bind a name in an enclosing
    *function*, never at module level, and the ported `_rng` needs it for the
    generator's own state.
    """
    try:
        tree = ast.parse(source)
    except SyntaxError as exc:
        return [f"does not parse: {exc}"]

    problems: list[str] = []
    docstrings = _docstring_nodes(tree)
    imported: set[str] = set()
    functions: set[str] = set()
    constants: Counter[str] = Counter()

    for n, node in enumerate(tree.body):
        where = f"line {node.lineno}"
        if n == 0 and isinstance(node, ast.Expr) and id(node.value) in docstrings:
            continue
        if isinstance(node, ast.Import):
            for alias in node.names:
                root = alias.name.split(".")[0]
                if root not in SLOT_IMPORTS:
                    problems.append(f"{where}: imports {alias.name!r}; the slot path imports hashlib and datetime only")
                imported.add(alias.asname or root)
        elif isinstance(node, ast.ImportFrom):
            root = (node.module or "").split(".")[0]
            if node.level or root not in SLOT_IMPORTS:
                problems.append(f"{where}: imports from {node.module!r}; the slot path imports hashlib and datetime only")
            for alias in node.names:
                imported.add(alias.asname or alias.name)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            functions.add(node.name)
        elif (
            isinstance(node, (ast.Assign, ast.AnnAssign))
            and isinstance(node.value, ast.Constant)
            and isinstance(node.value.value, (int, str))
            and not isinstance(node.value.value, bool)
        ):
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            for target in targets:
                if isinstance(target, ast.Name):
                    constants[target.id] += 1
                else:
                    problems.append(f"{where}: module-level assignment to something that is not a plain name")
        else:
            problems.append(
                f"{where}: top-level {type(node).__name__} - the slot module holds imports, "
                "functions and literal constants only, so there is nowhere for state to live"
            )

    for name, times in constants.items():
        if times > 1:
            problems.append(f"module constant {name!r} is bound {times} times; a rebound value is state")

    # The signature is the security property.
    slot_fn = next(
        (n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "slot_for_day"), None
    )
    if slot_fn is None:
        problems.append("no top-level function slot_for_day")
    else:
        a = slot_fn.args
        params = [arg.arg for arg in a.args]
        if a.posonlyargs or a.vararg or a.kwarg or a.kwonlyargs or params != ["day", "n_options"]:
            problems.append(
                f"slot_for_day takes {ast.unparse(a)!r}; it must take exactly (day, n_options=5) "
                "- the date and nothing else"
            )
        default = a.defaults[-1] if a.defaults else None
        if len(a.defaults) != 1 or not (
            isinstance(default, ast.Constant) and default.value == OPTION_COUNT and not isinstance(default.value, bool)
        ):
            problems.append("slot_for_day's n_options must default to the literal 5 (SPEC G-1)")

    allowed_names = SLOT_BUILTINS | imported | functions | set(constants) | _bound_in_functions(tree)
    for node in ast.walk(tree):
        line = f"line {getattr(node, 'lineno', '?')}"
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if node.decorator_list:
                problems.append(f"{line}: {node.name} is decorated; a decorator can cache or wrap the draw")
            if FORBIDDEN_WORDS.search(node.name):
                problems.append(f"{line}: function name {node.name!r} reaches for history")
            for arg in (*node.args.posonlyargs, *node.args.args, *node.args.kwonlyargs):
                if FORBIDDEN_WORDS.search(arg.arg):
                    problems.append(f"{line}: parameter {arg.arg!r} reaches for history")
        elif isinstance(node, ast.Global):
            problems.append(f"{line}: `global {', '.join(node.names)}` - module state on the slot path")
        elif isinstance(node, (ast.With, ast.AsyncWith)):
            problems.append(f"{line}: a `with` block on the slot path")
        elif isinstance(node, (ast.Import, ast.ImportFrom)) and node not in tree.body:
            problems.append(f"{line}: an import inside a function")
        elif isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in FORBIDDEN_CALLS:
            problems.append(f"{line}: calls {node.func.id}()")
        elif isinstance(node, ast.Attribute) and node.attr not in SLOT_ATTRIBUTES:
            problems.append(f"{line}: reads attribute {node.attr!r}, which the slot path has no use for")
        elif isinstance(node, ast.Name):
            if FORBIDDEN_WORDS.search(node.id):
                problems.append(f"{line}: name {node.id!r} reaches for history")
            elif isinstance(node.ctx, ast.Load) and node.id not in allowed_names:
                problems.append(f"{line}: reads {node.id!r}, which is not a parameter, local, import or literal constant")
        elif isinstance(node, ast.keyword) and node.arg and FORBIDDEN_WORDS.search(node.arg):
            problems.append(f"{line}: keyword {node.arg!r} reaches for history")
        elif (
            isinstance(node, ast.Constant)
            and isinstance(node.value, str)
            and id(node) not in docstrings
            and FORBIDDEN_WORDS.search(node.value)
        ):
            problems.append(f"{line}: string {node.value!r} names history")
    return problems


def _slot_aliases(tree: ast.AST) -> set[str]:
    """The names this module calls `slot_for_day` by, including `import ... as`."""
    names = {"slot_for_day"}
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom):
            for alias in node.names:
                if alias.name == "slot_for_day" and alias.asname:
                    names.add(alias.asname)
    return names


def _calls_slot(node: ast.Call, aliases: set[str]) -> bool:
    func = node.func
    return (isinstance(func, ast.Name) and func.id in aliases) or (
        isinstance(func, ast.Attribute) and func.attr == "slot_for_day"
    )


def slot_call_violations(sources: Mapping[str, str]) -> list[str]:
    """Every call of `slot_for_day` passes the date and at most the literal 5 (G-1).

    `n_options` is the constant 5 and never derived from data - so
    `slot_for_day(day, len(question.options))` is refused even though it would
    return the same thing today, because the day it does not is the day a
    four-option question draws from a different distribution.
    """
    problems = []
    for label, source in sources.items():
        tree = ast.parse(source)
        aliases = _slot_aliases(tree)
        for node in ast.walk(tree):
            if not (isinstance(node, ast.Call) and _calls_slot(node, aliases)):
                continue
            where = f"{label}:{node.lineno}"
            extra_args = node.args[1:]
            bad = any(isinstance(a, ast.Starred) for a in node.args) or len(node.args) > 2
            for a in extra_args:
                if not (isinstance(a, ast.Constant) and a.value == OPTION_COUNT and not isinstance(a.value, bool)):
                    bad = True
            for kw in node.keywords:
                if kw.arg == "day":
                    continue
                if kw.arg != "n_options" or not (
                    isinstance(kw.value, ast.Constant) and kw.value.value == OPTION_COUNT
                ):
                    bad = True
            if bad:
                problems.append(
                    f"{where}: {ast.unparse(node)} - slot_for_day takes the date and, at most, "
                    "the literal 5; n_options is never derived from data (SPEC G-1)"
                )
    return problems


def ledger_violations(sources: Mapping[str, str]) -> list[str]:
    """Nothing in the pipeline writes, reads or imports the MVP's ledger (G-1, G-10).

    Three things are refused anywhere in `pipeline/src`: a string constant outside
    a docstring naming `answer-history`; an import of `build_deck` or anything under
    `mvp`; and a function that both calls `slot_for_day` and does file I/O. That
    last is the shape of `build_deck.py:220`'s `slot_for_meetup` - the ledger-writing
    wrapper one function below the pure draw - caught under any name. The
    used-question record's only writer is the release transition (T-11).
    """
    problems = []
    for label, source in sources.items():
        tree = ast.parse(source)
        docstrings = _docstring_nodes(tree)
        aliases = _slot_aliases(tree)
        for node in ast.walk(tree):
            line = f"{label}:{getattr(node, 'lineno', '?')}"
            if (
                isinstance(node, ast.Constant)
                and isinstance(node.value, str)
                and id(node) not in docstrings
                and LEDGER_NAME.search(node.value)
            ):
                problems.append(f"{line}: names the MVP ledger in code: {node.value!r}")
            elif isinstance(node, (ast.Import, ast.ImportFrom)):
                modules = (
                    [a.name for a in node.names] if isinstance(node, ast.Import) else [node.module or ""]
                )
                for module in modules:
                    if module.split(".")[0] in {"mvp", "build_deck"} or module.endswith("build_deck"):
                        problems.append(f"{line}: imports {module!r}; the pipeline inherits no code from mvp/ by import")
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                calls = [c for c in ast.walk(node) if isinstance(c, ast.Call)]
                draws = any(_calls_slot(c, aliases) for c in calls)
                writes = any(
                    (isinstance(c.func, ast.Name) and c.func.id in FILE_WRITES)
                    or (isinstance(c.func, ast.Attribute) and c.func.attr in FILE_WRITES)
                    for c in calls
                )
                if draws and writes:
                    problems.append(
                        f"{line}: {node.name}() draws a slot and writes a file - the shape of "
                        "slot_for_meetup, which is not ported under any name (G-10)"
                    )
    return problems


def pipeline_sources(src_dir: Path = PIPELINE_SRC) -> dict[str, str]:
    return {
        path.name: path.read_text(encoding="utf-8") for path in sorted(Path(src_dir).glob("*.py"))
    }


def slot_source() -> str:
    return Path(_slot_module.__file__).read_text(encoding="utf-8")


# --------------------------------------------------------------------------- #
# AC-24 - five options, exactly one "does not compile"
# --------------------------------------------------------------------------- #


def check_five_options(record: Mapping[str, Any], where: str) -> list[str]:
    """AC-24 on one raw record. Reads the JSON rather than a loaded `Question`, so a
    malformed record is reported by name instead of crashing the whole load."""
    options = record.get("options")
    if not isinstance(options, list):
        return [f"{where}: no options list"]
    problems = []
    if len(options) != OPTION_COUNT:
        problems.append(f"{where}: {len(options)} options, expected {OPTION_COUNT}")
    dnc = sum(1 for o in options if isinstance(o, dict) and o.get("kind") == "does_not_compile")
    if dnc != 1:
        problems.append(f"{where}: {dnc} options of kind does_not_compile, expected exactly 1")
    return problems


def _raw_records(bank_dir: Path) -> list[tuple[Path, Any]]:
    directory = Path(bank_dir) / bank.QUESTIONS_DIR
    if not directory.is_dir():
        return []
    records = []
    for path in sorted(directory.glob("*.json")):
        try:
            records.append((path, json.loads(path.read_text(encoding="utf-8"))))
        except json.JSONDecodeError as exc:
            records.append((path, exc))
    return records


# --------------------------------------------------------------------------- #
# AC-25, G-11 - no answer-category distribution where an attendee can read it
# --------------------------------------------------------------------------- #

# Every authored participant-facing file (EVALUATION AC-25): the copy table, the
# public README, take-it-home's authored text, the organizer docs. Rendered wall
# text is excluded - a room's split legitimately prints a share beside an option's
# own text. `bank/` is excluded because it is private and is where the tells are
# allowed to live (SPEC G-11: the organizer-only report carries them).
PARTICIPANT_FACING = (
    # The copy table, SPEC section 11's port. It arrives with PR #9 (branch
    # ai-c11-cc/shared-web-layer, BUILDPLAN T-02) and is linted from the moment
    # it is in the tree.
    "web/shared/copy.js",
    "README.md",
    "mvp/README.md",
    "mvp/PRACTICE-RUN.md",
    "mvp/FIELD-NOTES-TEMPLATE.md",
)
# Take-it-home's authored text (T-12), whatever files it lands as.
PARTICIPANT_FACING_DIRS = ("web/home",)

# A percentage or a ratio. `N:M` is deliberately absent: times and 16:9 would trip it.
_NUMBER = r"(?:\d+(?:\.\d+)?|one|two|three|four|five|six|seven|eight|nine|ten|twenty)"
RATIO = re.compile(
    rf"""(?ix)
      \d+(?:\.\d+)?\s*%
    | \b\d+(?:\.\d+)?\s*per\s*cent\b
    | \b\d+\s*/\s*\d+\b
    | \b{_NUMBER}\s+(?:in|of|out\s+of)\s+(?:every\s+)?{_NUMBER}\b
    | \b(?:half|a\s+third|a\s+quarter|a\s+fifth|two\s+thirds|most)\s+(?:of\s+)?(?:the\s+)?(?:answers|questions)\b
    """
)
# The option-kind words (EVALUATION AC-25). `UB` is case-sensitive: "ub" inside a
# word or a lower-case "ub" is not the category.
KIND_WORD = re.compile(r"(?i:\b(?:compil\w*|undefined|panic\w*|output\w*)\b)|\bUB\b")

# Reviewed exemptions, keyed on the exact line. Any edit to the line re-arms the
# check, so an exemption can never cover a sentence nobody reviewed.
LINT_EXEMPTIONS = {
    (
        "mvp/README.md",
        "with perfect memory of every previous answer is still guessing 1 in 5.",
    ): (
        "the odds of guessing an answer POSITION, which the slot draw keeps at chance; "
        "it sits beside the 'does not compile' bullet but says nothing about how often "
        "any category is the answer. mvp/ is outside this ticket's reach to reword."
    ),
}


@dataclass(frozen=True)
class DistributionHit:
    path: str
    line: int
    text: str

    def __str__(self) -> str:
        return f"{self.path}:{self.line}: {self.text}"


def distribution_hits(text: str, label: str) -> list[DistributionHit]:
    """Lines of `text` holding a percentage or ratio within one line of an option-kind word."""
    lines = text.splitlines()
    hits = []
    for i, line in enumerate(lines):
        if not RATIO.search(line):
            continue
        window = lines[max(0, i - 1) : i + 2]
        if not any(KIND_WORD.search(w) for w in window):
            continue
        if (label, line.strip()) in LINT_EXEMPTIONS:
            continue
        hits.append(DistributionHit(label, i + 1, line.strip()))
    return hits


def participant_facing_files(repo: Path) -> tuple[list[Path], list[str]]:
    """The files the AC-25 lint reads, and the named ones not in the tree yet."""
    repo = Path(repo)
    present: list[Path] = []
    absent: list[str] = []
    for rel in PARTICIPANT_FACING:
        if (repo / rel).is_file():
            present.append(repo / rel)
        else:
            absent.append(rel)
    for rel in PARTICIPANT_FACING_DIRS:
        found = [p for p in sorted((repo / rel).rglob("*")) if p.is_file() and p.name != ".gitkeep"]
        if found:
            present.extend(found)
        else:
            absent.append(f"{rel}/")
    return present, absent


def distribution_lint(repo: Path) -> tuple[list[DistributionHit], list[str], list[str]]:
    """AC-25 over the tree: (hits, files scanned, named files not present yet)."""
    repo = Path(repo)
    present, absent = participant_facing_files(repo)
    hits, scanned = [], []
    for path in present:
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue  # a font or an image: nothing authored to read
        label = path.relative_to(repo).as_posix()
        scanned.append(label)
        hits.extend(distribution_hits(text, label))
    return hits, scanned, absent


def bank_audit_references(repo: Path) -> list[str]:
    """G-11: nothing participant-facing includes the tell report. Any file under
    `web/` or `room/` that mentions `bank/audit` is a leak path."""
    repo = Path(repo)
    found = []
    for top in ("web", "room"):
        root = repo / top
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*")):
            parts = set(path.relative_to(repo).parts)
            if not path.is_file() or parts & {"target", "node_modules", ".git"}:
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, OSError):
                continue
            if "bank/audit" in text:
                found.append(path.relative_to(repo).as_posix())
    return found


# --------------------------------------------------------------------------- #
# AC-26 - the enumerated tells
# --------------------------------------------------------------------------- #
#
# One statistic for every tell. A tell is a small, fixed family of rules an
# attendee might learn - "when the source says unsafe, pick the UB option", "pick
# the longest option". For each question a rule either names a set S of options or
# abstains. Over the questions where it fires:
#
#     H = the questions where the correct option is in S
#     E = sum(|S| / 5)   what the rule would score if the answer ignored the feature
#
# and the rule's ratio is H / E - how many times chance it does. Chance is the
# blind guess, 1 in 5 per option named, the same baseline AC-23a uses; this is the
# measure USER_STORIES.md records for option length in August ("2/8 vs. a 1.6/8
# chance baseline"). Rules that learn from the bank do it leave-one-out, so no
# question predicts itself.
#
# SPEC 7.6's margin is 1.5 x chance. Read bare, it fails 2 hits out of 4 on luck
# alone: on banks with no planted tell it fails 20-30 % of the time at the bank's
# present size. So a rule FAILs only when it is above the margin AND the exact
# probability of doing that well by chance - the Poisson-binomial tail under
# p_i = |S_i| / 5 - is at most 0.01 / m, m being the rules in that tell's family.
# Above the margin without that is a WARN, reported and printed. It cannot flake -
# there is no randomness and the bank is fixed - and it is not vacuous: 4 hits in 4
# on a one-rule tell has a tail of 0.0016 and fails.
#
# Why none of this can become an input to the slot: it reads the bank and returns
# a verdict and a report under bank/audit/. `popquiz.slot` cannot import this
# module or read that directory (the slot-path lint), and a failing tell is fixed
# by re-authoring a question - there is no position state anywhere to adjust.

TELL_MARGIN = 1.5
TELL_ALPHA = 0.01
UNSAFE = re.compile(r"\bunsafe\b")
DOES_NOT_COMPILE = "does_not_compile"


@dataclass(frozen=True)
class Face:
    """What an attendee can see of one question, plus its answer for scoring.

    `kinds` and `lengths` are in bank order. Bank order is not meant to be the
    wall's order, but EVALUATION's canary row sends pre-reveal options "in bank
    order", so until the room's arrangement exists and is proven to re-draw it the
    position tell treats bank order as visible.
    """

    id: str
    unsafe: bool
    lines: int
    topic: str
    kinds: tuple[str, ...]
    lengths: tuple[int, ...]
    correct: int


Rule = Callable[[Face, Sequence[Face]], "frozenset[int] | None"]


@dataclass(frozen=True)
class RuleScore:
    rule: str
    fired: int
    hits: int
    expected: float
    ratio: float | None
    tail: float | None


@dataclass(frozen=True)
class TellResult:
    tell: str
    statistic: str
    verdict: Verdict
    rules: tuple[RuleScore, ...]
    worst: RuleScore | None


def answer_index(question: Question) -> int | None:
    """The correct option, as far as it derives from the verified record.

    `bank.correct_index` covers a record that ran (the option equal to `stdout`)
    and a does-not-compile record. A question whose declared answer is undefined
    behaviour is recorded as a Miri run that was not clean (AC-9); its answer is the
    single `ub` option. Anything else that does not derive returns None and is left
    out of the tells by name, never guessed.
    """
    try:
        index = bank.correct_index(question)
    except bank.BankError:
        return None
    if index is not None:
        return index
    v = question.verified
    if v is not None and v.miri is not None and v.miri.clean is False:
        ub = [i for i, o in enumerate(question.options) if o.kind == "ub"]
        if len(ub) == 1:
            return ub[0]
    return None


def face_of(question: Question) -> Face | None:
    correct = answer_index(question)
    if correct is None:
        return None
    lines, _ = source_metrics(question.source)
    return Face(
        id=question.id,
        unsafe=bool(UNSAFE.search(question.source)),
        lines=lines,
        topic=question.topic,
        kinds=tuple(o.kind for o in question.options),
        lengths=tuple(js_length(o.text) for o in question.options),
        correct=correct,
    )


def tell_pool(questions: Iterable[Question]) -> tuple[list[Face], list[str]]:
    """The faces the tells measure - every question not rejected whose answer
    derives - and the ids left out, so the report can name them."""
    faces, left_out = [], []
    for q in questions:
        if q.review is not None and q.review.status == "rejected":
            continue
        face = face_of(q)
        if face is None:
            left_out.append(q.id)
        else:
            faces.append(face)
    return faces, left_out


def poisson_binomial_tail(ps: Sequence[float], hits: int) -> float:
    """P(at least `hits` successes) over independent trials with probabilities `ps`."""
    dist = [1.0]
    for p in ps:
        nxt = [0.0] * (len(dist) + 1)
        for k, mass in enumerate(dist):
            nxt[k] += mass * (1 - p)
            nxt[k + 1] += mass * p
        dist = nxt
    return min(1.0, sum(dist[hits:]))


def _majority(kinds: Iterable[str]) -> frozenset[str]:
    counts = Counter(kinds)
    if not counts:
        return frozenset()
    top = max(counts.values())
    return frozenset(k for k, c in counts.items() if c == top)


def _options_of(face: Face, kinds: frozenset[str]) -> frozenset[int] | None:
    chosen = frozenset(i for i, k in enumerate(face.kinds) if k in kinds)
    # Naming nothing, or all five, is not a guess about this question.
    return chosen if 0 < len(chosen) < len(face.kinds) else None


def _answer_kind(face: Face) -> str:
    return face.kinds[face.correct]


def _unsafe_rule(face: Face, others: Sequence[Face]) -> frozenset[int] | None:
    if not face.unsafe:
        return None
    peers = [o for o in others if o.unsafe]
    # With no other unsafe question to learn from, the attendee brings the prior
    # PHILOSOPHY section 2 names: unsafe means undefined behaviour.
    kinds = _majority(_answer_kind(o) for o in peers) if peers else frozenset({"ub"})
    return _options_of(face, kinds)


def _bucketed_rule(key: Callable[[Face], object]) -> Rule:
    """Learn the answer kind from the other questions sharing this bucket; fire
    only where the bucket's majority differs from the whole bank's, so the rule
    measures the feature and not the base rate."""

    def rule(face: Face, others: Sequence[Face]) -> frozenset[int] | None:
        peers = [o for o in others if key(o) == key(face)]
        if not peers:
            return None
        bucket = _majority(_answer_kind(o) for o in peers)
        if bucket == _majority(_answer_kind(o) for o in others):
            return None
        return _options_of(face, bucket)

    return rule


def _length_bucket(face: Face) -> str:
    if face.lines <= 3:
        return "<=3"
    if face.lines <= 5:
        return "4-5"
    if face.lines <= 7:
        return "6-7"
    return "8+"


def _extreme_length(pick: Callable[[Sequence[int]], int]) -> Rule:
    def rule(face: Face, others: Sequence[Face]) -> frozenset[int] | None:
        target = pick(face.lengths)
        chosen = frozenset(i for i, n in enumerate(face.lengths) if n == target)
        return chosen if len(chosen) < len(face.lengths) else None

    return rule


def _at_index(j: int) -> Rule:
    def rule(face: Face, others: Sequence[Face]) -> frozenset[int] | None:
        return frozenset({j}) if j < len(face.kinds) else None

    return rule


def _dnc_at_index(j: int) -> Rule:
    def rule(face: Face, others: Sequence[Face]) -> frozenset[int] | None:
        return frozenset({j}) if j < len(face.kinds) and face.kinds[j] == DOES_NOT_COMPILE else None

    return rule


def _kind_rule(kind: str) -> Rule:
    def rule(face: Face, others: Sequence[Face]) -> frozenset[int] | None:
        return _options_of(face, frozenset({kind}))

    return rule


TELLS: tuple[tuple[str, str, tuple[tuple[str, Rule], ...]], ...] = (
    (
        "unsafe",
        "for a source containing `unsafe`, pick the options of the answer kind most "
        "common among the other unsafe questions (with none to learn from, `ub`)",
        (("unsafe -> learned kind", _unsafe_rule),),
    ),
    (
        "source length",
        "line-count bucket (<=3, 4-5, 6-7, 8+); pick the bucket's leave-one-out "
        "majority answer kind where it differs from the bank's",
        (("line bucket -> learned kind", _bucketed_rule(_length_bucket)),),
    ),
    (
        "option text length",
        "pick the longest option; pick the shortest (ties take every tied option)",
        (("longest", _extreme_length(max)), ("shortest", _extreme_length(min))),
    ),
    (
        "option position",
        "pick bank index j; pick 'does not compile' when it sits at bank index j. Bank "
        "order is measured as if visible (EVALUATION's canary row). The wall's own "
        "answer position is slot_for_day(date), audited by AC-23a/AC-23b",
        tuple((f"index {LETTERS[j]}", _at_index(j)) for j in range(OPTION_COUNT))
        + tuple((f"does-not-compile at {LETTERS[j]}", _dnc_at_index(j)) for j in range(OPTION_COUNT)),
    ),
    (
        "topic",
        "pick the topic's leave-one-out majority answer kind where it differs from the bank's",
        (("topic -> learned kind", _bucketed_rule(lambda f: f.topic)),),
    ),
    (
        "answer category",
        "not one of AC-26's five: pick the options of one kind, for each kind - "
        "'always / never pick does not compile', the shortcut PHILOSOPHY section 2 names",
        tuple((f"pick {k}", _kind_rule(k)) for k in ("does_not_compile", "ub", "panic", "output")),
    ),
)


def score_rule(name: str, rule: Rule, faces: Sequence[Face]) -> RuleScore:
    hits, ps = 0, []
    for n, face in enumerate(faces):
        others = [o for m, o in enumerate(faces) if m != n]
        chosen = rule(face, others)
        if not chosen:
            continue
        p = len(chosen) / len(face.kinds)
        ps.append(p)
        hits += face.correct in chosen
    if not ps:
        return RuleScore(name, 0, 0, 0.0, None, None)
    expected = sum(ps)
    return RuleScore(name, len(ps), hits, expected, hits / expected, poisson_binomial_tail(ps, hits))


def measure_tell(name: str, statistic: str, rules: Sequence[tuple[str, Rule]], faces: Sequence[Face]) -> TellResult:
    scores = tuple(score_rule(rule_name, rule, faces) for rule_name, rule in rules)
    fired = [s for s in scores if s.fired]
    if not fired:
        return TellResult(name, statistic, "n/a", scores, None)
    worst = max(fired, key=lambda s: (s.ratio, -(s.tail or 0.0)))
    alpha = TELL_ALPHA / len(rules)
    over = [s for s in fired if s.ratio is not None and s.ratio > TELL_MARGIN]
    if any(s.tail is not None and s.tail <= alpha for s in over):
        verdict: Verdict = "fail"
        worst = min((s for s in over if s.tail <= alpha), key=lambda s: s.tail)
    elif over:
        verdict = "warn"
    else:
        verdict = "pass"
    return TellResult(name, statistic, verdict, scores, worst)


def measure_tells(faces: Sequence[Face]) -> list[TellResult]:
    """AC-26: every enumerated tell, plus the answer-category base rate."""
    return [measure_tell(name, statistic, rules, faces) for name, statistic, rules in TELLS]


# --------------------------------------------------------------------------- #
# AC-27 - unsafe parity, and what "accepted" means here
# --------------------------------------------------------------------------- #

# SPEC 7.4's edit action re-verifies a question and keeps it; it is not a
# rejection, and SPEC 3.1's status enum has no value for "edited, then accepted".
# Wherever the contract says accepted - AC-27, AC-88, the reserve - this reads both,
# because leaving `edited` out would let an edited question slip past the check.
ACCEPTED = frozenset({"accepted", "edited"})


def is_accepted(question: Question) -> bool:
    return question.review is not None and question.review.status in ACCEPTED


def in_reserve(question: Question) -> bool:
    """SPEC 3.3: accepted, affirmed and unused - what scheduling can pick tonight."""
    return is_accepted(question) and question.review.affirmed() and question.used is None


def answer_is_ub(question: Question) -> bool:
    v = question.verified
    if v is not None and v.miri is not None and v.miri.clean is False:
        return True
    index = answer_index(question)
    return index is not None and question.options[index].kind == "ub"


def check_unsafe_parity(questions: Iterable[Question]) -> Check:
    """AC-27: if any accepted question's answer is UB, at least one accepted non-UB
    question contains `unsafe` - otherwise `unsafe` in the source is the answer."""
    accepted = [q for q in questions if is_accepted(q)]
    ub = [q.id for q in accepted if answer_is_ub(q)]
    decoys = [q.id for q in accepted if not answer_is_ub(q) and UNSAFE.search(q.source)]
    detail = {"accepted": len(accepted), "ub_answers": ub, "non_ub_with_unsafe": decoys}
    if not ub:
        return Check("unsafe parity", ("AC-27",), "pass", "no accepted question has a UB answer", detail)
    if decoys:
        return Check(
            "unsafe parity",
            ("AC-27",),
            "pass",
            f"{len(ub)} UB answer(s), and `unsafe` also appears in {len(decoys)} non-UB question(s)",
            detail,
        )
    return Check(
        "unsafe parity",
        ("AC-27",),
        "fail",
        f"{len(ub)} accepted UB answer(s) ({', '.join(ub)}) and no accepted non-UB question "
        "contains `unsafe` - its presence names the answer",
        detail,
    )


# --------------------------------------------------------------------------- #
# AC-88 - difficulty drift fails the run, not the question
# --------------------------------------------------------------------------- #

DRIFT_LIMIT = 1.0


def difficulty_drift(questions: Iterable[Question]) -> Check:
    """Across a run's accepted questions, mean |judged - requested| above one level
    fails the **run** (AC-88, SPEC 7.6). No question is failed or named as the
    cause: drift is a generation defect, not a property of any one question.

    The record has no run id (SPEC 3.1), so the caller passes the run. `bank-audit`
    passes the accepted bank as one sample and says so in the report.
    """
    judged = [
        q
        for q in questions
        if is_accepted(q) and q.review.difficulty_judged is not None
    ]
    if not judged:
        return Check(
            "difficulty drift",
            ("AC-88",),
            "n/a",
            "no accepted question has an organizer-judged difficulty yet",
            {"judged": 0},
        )
    drift = sum(abs(q.review.difficulty_judged - q.difficulty_requested) for q in judged) / len(judged)
    verdict: Verdict = "fail" if drift > DRIFT_LIMIT else "pass"
    return Check(
        "difficulty drift",
        ("AC-88",),
        verdict,
        f"mean |judged - requested| = {drift:.2f} over {len(judged)} accepted question(s); "
        f"the run fails above {DRIFT_LIMIT:g}",
        {"judged": len(judged), "mean_drift": drift},
    )


# --------------------------------------------------------------------------- #
# AC-100, D-15 - the wall's type model, mirrored from web/shared/typemodel.js
# --------------------------------------------------------------------------- #
#
# Stage 1 of typemodel.js only - floorPx, sourceMetrics, desiredFontPx - because
# stage 2 measures a rendered well and there is no browser here. Same constants,
# same arithmetic, NO ROUNDING anywhere inside: two implementations that round at
# different moments disagree in the third decimal and then, after the wall's refit
# passes, in the first. Round only where a number is displayed or asserted.
#
# typemodel.js lives in PR #9 (branch ai-c11-cc/shared-web-layer, BUILDPLAN T-02).
# `tests/test_fit.py` compares these constants with it whenever it is in the tree.

CANVAS_H = 630
CAP_RATIO = 0.7
READ_RATIO = 150
MAX_FONT = 46
LINE_HEIGHT = 1.6
GUTTER_CH = 3.5
CHAR_ADVANCE = 0.6
READING_AREA = (994, 177)  # live, closed, split - under the 190 px options block
TRACE_AREA = (994, 190)  # work, reveal - under the 203 px beat block
OPTION_MAX_CHARS = bank.MAX_OPTION_LINE_CHARS


@dataclass(frozen=True)
class Room:
    """Organizer-set (SPEC 5.2); these defaults are a hypothesis until HC-1."""

    screen_width_ft: float = 15.0
    screen_height_ft: float = 8.44  # width x 9/16, as typemodel.js states it
    back_row_ft: float = 20.0


def js_length(text: str) -> int:
    """A string's length as JavaScript counts it - UTF-16 code units - so the audit
    and the wall agree on what a character is. Never less than Python's `len`."""
    return len(text.encode("utf-16-le")) // 2


def floor_px(room: Room = Room()) -> float:
    """The smallest legible size for the room, in canvas px (14.2 at the defaults)."""
    screen_h_in = room.screen_height_ft * 12
    cap_in = room.back_row_ft * 12 / READ_RATIO
    font_in = cap_in / CAP_RATIO
    return font_in / screen_h_in * CANVAS_H


def source_metrics(source: str) -> tuple[int, int]:
    """(lines, widest line), splitting exactly as well.js's sourceLines does: one
    trailing newline stripped, then split on newlines."""
    text = source[:-1] if source.endswith("\n") else source
    lines = text.split("\n")
    return len(lines), max(js_length(line) for line in lines)


def desired_font_px(metrics: tuple[int, int], area: tuple[int, int]) -> float:
    """What the source wants before the floor applies: min(by height, by width, 46).
    This is the number SPEC 5.2's worked examples report, including the ones below
    the floor."""
    lines, widest = metrics
    width, height = area
    by_height = height / (lines * LINE_HEIGHT)
    by_width = width / ((widest + GUTTER_CH) * CHAR_ADVANCE)
    return min(by_height, by_width, MAX_FONT)


@dataclass(frozen=True)
class Fit:
    wants: float
    font_px: float
    floor_px: float
    fits: bool
    area: tuple[int, int]


def fit(source: str, area: tuple[int, int] = READING_AREA, room: Room = Room()) -> Fit:
    """The derived size and verdict. The floor wins over the 46 cap; a source fits
    when what it wants is at or above the floor."""
    floor = floor_px(room)
    wants = desired_font_px(source_metrics(source), area)
    return Fit(wants, max(floor, wants), floor, wants >= floor, area)


def option_fits(text: str) -> bool:
    """D-15: one line of at most 29 characters (typemodel.js optionFits)."""
    return "\n" not in text and js_length(text) <= OPTION_MAX_CHARS


FLAG_PROGRAM = "program_fit"
FLAG_OPTION = "option_length"


def question_flags(question: Question, room: Room = Room()) -> tuple[str, ...]:
    """AC-100's two per-question flags: the program does not fit the reading
    layout at the floor, and an option breaks D-15."""
    flags = []
    if not fit(question.source, READING_AREA, room).fits:
        flags.append(FLAG_PROGRAM)
    if not all(option_fits(o.text) for o in question.options):
        flags.append(FLAG_OPTION)
    return tuple(flags)


# --------------------------------------------------------------------------- #
# The run, the report, the command line
# --------------------------------------------------------------------------- #

AUDIT_DIR = Path("bank") / "audit"


def _generator_check() -> tuple[Check, Check]:
    try:
        result = audit_generator()
        generator = Check("generator", ("AC-23b", "G-1"), "pass", result.summary(), asdict(result))
    except AuditFailure as exc:
        generator = Check("generator", ("AC-23b", "G-1"), "fail", str(exc))
    rates = simulate_attendees()
    outside = attendees_outside_band(rates)
    rendered = ", ".join(f"{name} {rate:.1%}" for name, rate in rates.items())
    attendees = Check(
        "attendees with perfect memory",
        ("AC-23a", "AC-23", "G-1"),
        "fail" if outside else "pass",
        f"over {ATTENDEE_NIGHTS} synthetic nights from {ATTENDEE_START}: {rendered} "
        f"(band {ATTENDEE_BAND[0]:.0%}-{ATTENDEE_BAND[1]:.0%})",
        {"rates": rates, "seed": ATTENDEE_SEED},
    )
    return generator, attendees


def _lint_check() -> Check:
    sources = pipeline_sources()
    problems = (
        slot_path_violations(slot_source())
        + slot_call_violations(sources)
        + ledger_violations(sources)
    )
    return Check(
        "slot path and ledger",
        ("AC-23", "AC-23a", "G-1", "G-10"),
        "fail" if problems else "pass",
        "; ".join(problems)
        or "slot.py takes the date and nothing else; nothing in the pipeline reads, "
        "writes or imports the MVP ledger",
        {"problems": problems},
    )


def run_audit(repo: Path, room: Room = Room()) -> dict[str, Any]:
    """Every check over the repository's bank, as the report's dict. Writes nothing."""
    repo = Path(repo)
    bank_dir = repo / "bank"
    checks: list[Check] = []

    checks.extend(_generator_check())
    checks.append(_lint_check())

    # AC-24 on the raw records, then load the rest.
    raw = _raw_records(bank_dir)
    shape_problems, questions, unreadable = [], [], []
    for path, record in raw:
        where = path.name
        if isinstance(record, Exception):
            unreadable.append(f"{where}: {record}")
            continue
        shape_problems.extend(check_five_options(record, where))
        try:
            questions.append(bank.question_from_dict(record))
        except (bank.BankError, KeyError, TypeError) as exc:
            unreadable.append(f"{where}: {exc}")
    checks.append(
        Check(
            "five options, one does not compile",
            ("AC-24",),
            "fail" if shape_problems else "pass",
            "; ".join(shape_problems) or f"all {len(raw)} question(s)",
            {"problems": shape_problems},
        )
    )
    if unreadable:
        checks.append(
            Check("records load", ("SPEC 3.1",), "fail", "; ".join(unreadable), {"problems": unreadable})
        )

    hits, scanned, absent = distribution_lint(repo)
    leaks = bank_audit_references(repo)
    checks.append(
        Check(
            "no published distribution",
            ("AC-25", "G-11"),
            "fail" if hits or leaks else "pass",
            "; ".join([str(h) for h in hits] + [f"{p} mentions bank/audit" for p in leaks])
            or f"{len(scanned)} participant-facing file(s) clean",
            {"scanned": scanned, "not_present_yet": absent, "hits": [str(h) for h in hits], "report_references": leaks},
        )
    )

    faces, left_out = tell_pool(questions)
    for tell in measure_tells(faces):
        worst = tell.worst
        if worst is None:
            summary = "no rule fired on this bank"
        else:
            summary = (
                f"worst rule '{worst.rule}': {worst.hits} hit(s) vs {worst.expected:.2f} by chance "
                f"= {worst.ratio:.2f}x over {worst.fired} question(s), tail p={worst.tail:.4f}"
            )
        checks.append(
            Check(
                f"tell: {tell.tell}",
                ("AC-26",),
                tell.verdict,
                summary,
                {
                    "statistic": tell.statistic,
                    "pool": len(faces),
                    "left_out": left_out,
                    "rules": [asdict(s) for s in tell.rules],
                },
            )
        )

    checks.append(check_unsafe_parity(questions))
    drift = difficulty_drift(questions)
    checks.append(
        Check(
            drift.name,
            drift.criteria,
            drift.verdict,
            drift.summary + " (the record has no run id, so the accepted bank is audited as one run)",
            drift.detail,
        )
    )

    per_question: dict[str, Any] = {}
    reserve_flagged = []
    for q in questions:
        flags = question_flags(q, room)
        reading = fit(q.source, READING_AREA, room)
        trace = fit(q.source, TRACE_AREA, room)
        reserve = in_reserve(q)
        if flags and reserve:
            reserve_flagged.append(q.id)
        per_question[q.id] = {
            "flags": list(flags),
            "in_reserve": reserve,
            "lines_widest": list(source_metrics(q.source)),
            "reading": {"wants_px": reading.wants, "fits": reading.fits},
            "trace": {"wants_px": trace.wants, "fits": trace.fits},
            "options_over": [i for i, o in enumerate(q.options) if not option_fits(o.text)],
        }
    flagged = sorted(qid for qid, entry in per_question.items() if entry["flags"])
    if flagged:
        named = [f"{qid} ({', '.join(per_question[qid]['flags'])})" for qid in flagged]
        reserve_note = (
            f" IN THE RESERVE, schedulable tonight: {', '.join(reserve_flagged)}."
            if reserve_flagged
            else " None is in the reserve."
        )
        summary = "flagged: " + "; ".join(named) + "." + reserve_note
    else:
        summary = "every question fits at the floor and every option is one line of <= 29"
    checks.append(
        Check(
            "fits the configured room",
            ("AC-100", "D-15"),
            # A flag is a failure the moment the question could be scheduled; before
            # that it is reported, because every too-long candidate would otherwise
            # hold the audit red until review (AC-100: "flagged before it can be
            # scheduled"). --strict fails on any flag.
            "fail" if reserve_flagged else ("warn" if flagged else "pass"),
            summary,
            {"reserve_flagged": reserve_flagged, "flagged": flagged, "floor_px": floor_px(room)},
        )
    )

    failed = [c.name for c in checks if c.verdict == "fail"]
    return {
        "_note": (
            "bank-audit's report. ORGANIZER ONLY: it names answer categories and how "
            "often rules hit, which AC-25 keeps away from attendees. Nothing "
            "participant-facing reads bank/audit/. Regenerate with `just bank-audit`."
        ),
        "room": {**asdict(room), "floor_px": floor_px(room), "status": "hypothesis until HC-1"},
        "bank": {
            path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path, _ in raw
        },
        "checks": [asdict(c) for c in checks],
        "questions": per_question,
        "failed": failed,
        "warned": [c.name for c in checks if c.verdict == "warn"],
    }


def write_report(repo: Path, report: Mapping[str, Any], day: str) -> Path:
    """Write `bank/audit/<day>.json` and nowhere else (AC-25: the tell report lives
    only under bank/audit/, which nothing participant-facing includes)."""
    datetime.date.fromisoformat(day)  # a date and nothing else: no path can ride in on it
    audit_dir = (Path(repo) / AUDIT_DIR).resolve()
    path = (audit_dir / f"{day}.json").resolve()
    if path.parent != audit_dir:
        raise AuditFailure(f"refusing to write the tell report outside {audit_dir}: {path}")
    audit_dir.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"date": day, **report}, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return path


def exit_status(report: Mapping[str, Any], strict: bool = False) -> int:
    """1 when a bank-level check fails or a flagged question is in the reserve.
    Under --strict, any warning or any flag fails too."""
    if report["failed"]:
        return 1
    if strict and (report["warned"] or any(q["flags"] for q in report["questions"].values())):
        return 1
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="bank-audit", description=__doc__.splitlines()[0])
    parser.add_argument("--repo", type=Path, default=DEFAULT_REPO, help="repository root")
    defaults = Room()
    parser.add_argument("--screen-width-ft", type=float, default=defaults.screen_width_ft)
    parser.add_argument("--screen-height-ft", type=float, default=None, help="default: width x 9/16")
    parser.add_argument("--back-row-ft", type=float, default=defaults.back_row_ft)
    parser.add_argument("--date", default=datetime.date.today().isoformat(), help="the report's date")
    parser.add_argument("--strict", action="store_true", help="fail on any warning or flag")
    args = parser.parse_args(argv)

    if args.screen_height_ft is not None:
        height = args.screen_height_ft
    elif args.screen_width_ft == defaults.screen_width_ft:
        height = defaults.screen_height_ft
    else:
        height = args.screen_width_ft * 9 / 16
    room = Room(args.screen_width_ft, height, args.back_row_ft)

    report = run_audit(args.repo, room)
    path = write_report(args.repo, report, args.date)
    marks = {"pass": "ok  ", "warn": "WARN", "fail": "FAIL", "n/a": "n/a "}
    for check in report["checks"]:
        print(f"{marks[check['verdict']]} {check['name']} [{', '.join(check['criteria'])}]: {check['summary']}")
    print(f"report: {path}")
    status = exit_status(report, args.strict)
    print("bank-audit: " + ("FAILED" if status else "passed"))
    return status


if __name__ == "__main__":
    sys.exit(main())
