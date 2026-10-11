"""Generate candidate questions (AC-1 to AC-5, SPEC.md 7.1).

A CLI run asks for `count` questions on `topics` at `difficulty` - or, in talk mode,
for questions drawn from a public talk's title and abstract - and gets back candidate
records in the bank's SPEC 3.1 shape, minus `verified` and `review`, plus a run
report. The model is reached through the Message Batches API (BUILDPLAN D-D: the
pipeline is offline, so the batch discount is free money) with structured outputs.
Nothing here runs inside a room (AC-1), and nothing here is on a deadline.

**The seam.** The generator reaches the API through `BatchClient` and nothing else -
submit a batch, poll it, read its results - the way the verifier reaches a toolchain
through `runner.Runner` (D-18). Two implementations:

* `ReplayBatchClient` replays a batch recorded as a fixture in the SDK's result shape.
  `just test` uses it, with no key and no network.
* `AnthropicBatchClient` wraps the Anthropic SDK. It is written in full and has never
  run: it is unexercised until the first paid run, HC-2.

**What the model is never asked for.** There is no `correct` field, no "expected
output", no letter and no position in what the model returns. The answer is derived
from `verified`, which only the verifier writes (G-2, AC-7); a model's idea of what a
program prints is never an answer. Three consequences, each a deliberate choice:

* `trace` is drafted by the model and has **no `stdout` row**. The steps - lines,
  focus, note, the values that are not output, an optional pivot - are authored
  content, like the beats: the organizer checks them at review (HC-2), and nothing
  here verifies them. The last step's `stdout` row is the program's output, so the
  model never writes it: any value the model names `stdout` is discarded from every
  step and the discard is reported, and `verify.with_record` writes the row from
  `verified.stdout` once the program has run. A record that ran and still lacks the
  row fails `bank.validate_question`, so a draft never passes as finished. A value
  under another name whose `now` is an output option's exact text is discarded the
  same way (PQ-68 F1). Anything else - a paraphrase, a note that states an output in
  prose - is authored content for review: the report lists every quoted span and
  every trace value for AC-73.
* `why_tempting` is asked for on all five options, so a missing one cannot mark the
  option the model believes is correct. The review step drops the correct option's
  beat once the verifier has said which that is.
* `expect` - ran, does not compile, or UB - is the class `verify` checks a candidate
  against (`verify.Expect`). It is a category, not an output, and it goes in the run
  report only, never into the candidate file.

The options are written in the order the model returned them. Bank order is canonical
and means nothing: the night's order is drawn from the date alone, by `popquiz
schedule`, and the generator never chooses, balances or records a position
(PHILOSOPHY 2).

**The room's capacity** comes from `popquiz.audit`, the Python form of the wall's type
model and of the room configuration (`audit.Room`, the same flags `bank-audit` takes):
the most source lines the wall holds at the floor, that each option is one line, and
that no option is longer than 29 characters (SPEC 7.1, D-15). The brief also states the
widest source line that fits, because the check a candidate is held to is
`audit.fit`, and a model should be told the rule it is measured against. A candidate
that breaks any of it is reported, never trimmed, wrapped or joined.

**Spend.** A paid run needs a cap, and there is no default. The worst case is
estimated from the requests actually built - every request's `max_tokens` at the
output price plus its UTF-8 size at the input price (bytes bound tokens from above),
at the batch discount - and a run that has no cap, or whose worst case is over it,
refuses before the client is built and before anything is submitted. Prices are list
prices from a table, labelled as estimates; AC-5's report is what replaces them.

**Re-runnable (AC-2).** Every file is written to a `.partial` beside it and renamed
into place, so a run killed at any point leaves whole files or a `.partial` that the
next run overwrites. The run's state is written before the batch is submitted and
again with the batch id after, so re-running with the same `--run-id` resumes the
batch already paid for and never submits a second one.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import sys
import time
from collections.abc import Callable, Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Protocol

from popquiz import audit, bank, copylint
from popquiz.bank import BankError, Explains, Option, Question, Trace, TraceStep, ValueDelta

# --------------------------------------------------------------------------- #
# Configuration - one place
# --------------------------------------------------------------------------- #

#: The model. One configured value. BUILDPLAN D-D named `claude-opus-5`; the client
#: ruled on 2026-10-10 ("use opus5.5", Lattice PQ-21) that the default is Opus 5.5.
DEFAULT_MODEL = "claude-opus-5-5"

#: List prices in dollars per million tokens (input, output). ESTIMATES, from the
#: claude-api skill's model table (Opus 5.5 at $4 / $20) and D-D (Opus 5 at $5 / $25).
#: A model not in this table cannot be estimated, so a paid run with it refuses.
#: There is no override on the command line: a price is not something to type at a
#: prompt that also decides whether money is spent.
LIST_PRICES_PER_MTOK: dict[str, tuple[float, float]] = {
    "claude-opus-5-5": (4.0, 20.0),
    "claude-opus-5": (5.0, 25.0),
}

#: The Message Batches API bills every token at half the list price.
BATCH_DISCOUNT = 0.5

#: The skill's multipliers on the input price for cache writes and reads. No request
#: sets `cache_control`, so neither should appear; if a result reports them anyway,
#: they are costed rather than ignored.
CACHE_WRITE_MULTIPLIER = 1.25
CACHE_READ_MULTIPLIER = 0.1

#: Output ceiling per request. Thinking counts toward it, so it bounds everything a
#: request can bill as output.
DEFAULT_MAX_TOKENS = 16000

#: Opus 5.5 defaults to `medium` effort (claude-api skill); writing a program whose
#: output is the whole question is work worth `high`.
EFFORT = "high"

#: No scale is written down anywhere; the bank's existing questions sit at 2 and 3.
DIFFICULTIES = range(1, 6)

DEFAULT_POLL_INTERVAL_S = 60.0

#: Where candidates land by default: a directory per run under `bank/candidates/`, the
#: home of candidates waiting for `verify`. Never `bank/questions/`.
DEFAULT_OUT = audit.DEFAULT_REPO / "bank" / "candidates"
QUESTIONS_DIR = audit.DEFAULT_REPO / "bank" / "questions"

RUN_SUBDIR = "_run"
STATE_FILE = "state.json"
REPORT_FILE = "report.json"

OPTION_KINDS = ("output", "does_not_compile", "ub", "panic")
EXPECTS = ("ran", "does_not_compile", "ub")
CONCEPT_KINDS = ("std", "core_language")
#: The trace value that is the program's output: written by `verify`, never the model.
STDOUT = "stdout"
#: SPEC 7.4: a question needs this many trace steps before it can be affirmed.
MIN_TRACE_STEPS = 2
_STEP_KEYS = frozenset({"lines", "focus", "note", "values", "pivot"})
_STD_PATH = re.compile(r"^(std|core|alloc)::\S+")
_RUN_ID = re.compile(r"^[a-z0-9][a-z0-9-]{0,47}$")

# --------------------------------------------------------------------------- #
# Refusals - one plain sentence each
# --------------------------------------------------------------------------- #

NO_CAP = "Refusing to submit: no spend cap is configured; a paid run needs --cap-usd."
NO_KEY = "Refusing to build the API client: ANTHROPIC_API_KEY is not set."
NO_CANDIDATE_CAP = (
    "Refusing to start: no candidate cap is configured; a claude-cli run needs --max-candidates."
)
CLI_NO_USD = (
    "Refusing to start: a claude-cli run is capped by --max-candidates, not dollars; drop --cap-usd."
)
BATCHES_NO_MAX = "Refusing to submit: a batches run is capped by --cap-usd; --max-candidates is claude-cli's."
CLI_NO_REPLAY = "Refusing to start: --replay replays a batch; a claude-cli run takes no --replay."
BATCHES_NO_CLAUDE_BIN = "Refusing to submit: --claude-bin is claude-cli's; a batches run starts no claude."
NOT_RUN = "not run: the run stopped before this candidate; rerun with the same --run-id to pick it up"


def over_candidate_cap(count: int, most: int) -> str:
    return f"Refusing to start: the run asks for {count} candidates, over the {most}-candidate cap."


def over_cap(requests: int, worst: float, cap: float) -> str:
    return (
        f"Refusing to submit: the worst case for {requests} requests is "
        f"${worst:.2f}, over the ${cap:.2f} cap."
    )


def no_price(model: str) -> str:
    return (
        f"Refusing to submit: there is no list price for {model}, "
        "so the worst case cannot be estimated."
    )


class GenerateError(Exception):
    """The run refused, or could not go on. The message is the sentence to show."""


# --------------------------------------------------------------------------- #
# The run request
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class Talk:
    """A public talk's title and abstract (AC-4)."""

    title: str
    abstract: str


@dataclass(frozen=True)
class RunRequest:
    """What the organizer asked for (SPEC 7.1, AC-3).

    In talk mode the topic of each question is the std/core concept the model names
    from the talk, so `topics` is empty there: SPEC 3.1 has no other field to carry
    the concept, and the concept is what the question is about.
    """

    count: int
    difficulty: int
    topics: tuple[str, ...] = ()
    talk: Talk | None = None

    def validate(self) -> None:
        if self.count < 1:
            raise GenerateError(f"A run asks for at least one question, not {self.count}.")
        if self.difficulty not in DIFFICULTIES:
            raise GenerateError(
                f"Difficulty is {DIFFICULTIES.start} to {DIFFICULTIES.stop - 1}, not {self.difficulty}."
            )
        if self.talk is not None:
            if self.topics:
                raise GenerateError(
                    "Talk mode takes its topics from the talk; give --topic or a talk, not both."
                )
            if not self.talk.title.strip() or not self.talk.abstract.strip():
                raise GenerateError("Talk mode needs both a title and an abstract.")
        elif not self.topics or any(not t.strip() for t in self.topics):
            raise GenerateError("A run needs at least one --topic, or a talk.")

    def topic_for(self, index: int) -> str | None:
        """The topic request `index` asks for; topics cycle. None in talk mode."""
        return None if self.talk is not None else self.topics[index % len(self.topics)]

    def to_dict(self) -> dict[str, Any]:
        out: dict[str, Any] = {"count": self.count, "difficulty": self.difficulty, "topics": list(self.topics)}
        if self.talk is not None:
            out["talk"] = {"title": self.talk.title, "abstract": self.talk.abstract}
        return out


# --------------------------------------------------------------------------- #
# The room's capacity (SPEC 5.2, 7.1, D-15)
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class Capacity:
    """The three facts SPEC 7.1 names, plus the source width from the same model."""

    source_lines: int
    source_width: int
    option_lines: int
    option_chars: int
    floor_px: float


def capacity(room: audit.Room = audit.Room()) -> Capacity:
    """What the configured room holds, derived from the wall's type model.

    A source fits when `audit.fit` says so: what it wants is at or above the floor,
    by height and by width. Solving those two inequalities for lines and characters
    gives the largest whole numbers that fit, so a candidate within both is one
    `audit.fit` passes.
    """
    floor = audit.floor_px(room)
    width, height = audit.READING_AREA
    lines = math.floor(height / (floor * audit.LINE_HEIGHT))
    chars = math.floor(width / (floor * audit.CHAR_ADVANCE) - audit.GUTTER_CH)
    if floor > audit.MAX_FONT or lines < 1 or chars < 1:
        raise GenerateError(
            f"This room's floor is {floor:.1f} px, and the wall holds no source at it."
        )
    return Capacity(
        source_lines=lines,
        source_width=chars,
        option_lines=bank.MAX_OPTION_LINES,
        option_chars=audit.OPTION_MAX_CHARS,
        floor_px=floor,
    )


def capacity_breaks(source: str | None, options: Sequence[str], cap: Capacity, room: audit.Room) -> list[str]:
    """Every capacity fact this candidate breaks, in words. Empty when it fits."""
    reasons: list[str] = []
    if isinstance(source, str) and source.strip():
        if not audit.fit(source, audit.READING_AREA, room).fits:
            lines, widest = audit.source_metrics(source)
            if lines > cap.source_lines:
                reasons.append(
                    f"capacity: the source is {lines} lines; the room holds {cap.source_lines} at the floor"
                )
            if widest > cap.source_width:
                reasons.append(
                    f"capacity: the widest source line is {widest} characters; the room holds {cap.source_width}"
                )
            if not reasons:  # the two facts and audit.fit disagree: report the fit itself
                reasons.append("capacity: the source does not fit the room at the floor")
    for i, text in enumerate(options):
        if not isinstance(text, str):
            continue
        lines = text.count("\n") + 1
        if lines > cap.option_lines:
            most = "each option is one line" if cap.option_lines == 1 else f"the most is {cap.option_lines}"
            reasons.append(f"capacity: option {i + 1} spans {lines} lines; {most}")
        if audit.js_length(text) > cap.option_chars:
            reasons.append(
                f"capacity: option {i + 1} is {audit.js_length(text)} characters; the most is {cap.option_chars}"
            )
    return reasons


# --------------------------------------------------------------------------- #
# The brief (SPEC 7.1, D-15, D-23)
# --------------------------------------------------------------------------- #

#: SPEC 7.1's style brief for the prose the model writes.
STYLE_BRIEF = (
    "Write `explains.what`, `explains.takeaway`, every `why_tempting` and the `hint` as "
    "short sentences meant to be read aloud: use contractions, state each claim once, "
    "and use no reassurance, no flattery and no signposting."
)


def brief_patterns() -> list[str]:
    """Every pattern the prose must avoid, from `popquiz.copylint` itself: SPEC 11.1's
    trope check and SPEC 11's Forbidden row. Rendered from the module, never copied."""
    return [f"- {rule.id}: {rule.source}" for rule in copylint.RULES]


def option_lines_rule(cap: Capacity) -> str:
    """The brief's line rule for options, read from the room's capacity."""
    if cap.option_lines == 1:
        return (
            "- Each option is a single line. A program whose output would span lines is "
            "rewritten to print on one line; never join or wrap an option."
        )
    return (
        f"- Each option is at most {cap.option_lines} lines. A program whose output would "
        "span more is rewritten; never join or wrap an option."
    )


def system_prompt(cap: Capacity, talk_mode: bool) -> str:
    """The brief every request carries. Identical across a run."""
    lines = [
        "You write one original multiple-choice question for a live Rust meetup quiz.",
        "The question is always: what does this Rust program print?",
        "It goes on a projector at the front of a room, read from the back row.",
        "",
        "The room's capacity. A program that breaks any of these is rejected, not trimmed:",
        f"- The source is at most {cap.source_lines} lines.",
        f"- No source line is wider than {cap.source_width} characters.",
        option_lines_rule(cap),
        f"- No option is longer than {cap.option_chars} characters.",
        "",
        "The shape:",
        "- Exactly five options, each with a `kind`: `output` (text the program might print), "
        "`does_not_compile`, `ub` (undefined behaviour) or `panic`.",
        "- Exactly one option is `does_not_compile`, with the text `does not compile`.",
        "- At most one option is `ub` and at most one is `panic`. No two options share text.",
        "- Give every one of the five options a `why_tempting`: one short sentence on why a "
        "reader would pick it.",
        "- `expect` is what the program is built to demonstrate: `ran`, `does_not_compile` or "
        "`ub`. If it is `ub`, exactly one option is `ub`.",
        "- Do not say which option is correct, anywhere. A machine runs the program and decides.",
        "- The `hint` is read before anyone answers, so it never states or quotes an output.",
        "",
        "The trace, a walk-through the host reads after the reveal:",
        "- At least two steps, in the order the program runs. The last step is where the "
        "program finishes.",
        "- `lines` are the 1-based source lines executing in that step; `focus` is the "
        "inclusive 1-based range `[first, last]` kept at full contrast.",
        "- `note` is what the host reads aloud. `values` are the variables that change, each "
        "with `was` and `now` (use `—` for a value that did not exist yet).",
        "- Set `pivot` true on at most one step: the one to pause on.",
        f"- Never name `{STDOUT}` in `values`, and never state or quote what the program prints "
        "in a note. A machine runs the program and adds its output to the last step.",
        "",
        "Style:",
        STYLE_BRIEF,
        "",
        "Never write prose that matches any of these case-insensitive patterns:",
        *brief_patterns(),
    ]
    if talk_mode:
        lines += [
            "",
            "Talk mode: the question comes from a public talk. Name one concept from the talk "
            "as `concept`: a standard-library item (kind `std`, written as a path such as "
            "`std::mem::take`) or a core-language concept (kind `core_language`, such as "
            "`drop order`). The program is about that concept.",
        ]
    return "\n".join(lines)


def user_message(request: RunRequest, index: int) -> str:
    if request.talk is not None:
        return (
            f"Difficulty {request.difficulty} of {DIFFICULTIES.stop - 1}. "
            f"This is question {index + 1} of {request.count} from this talk; pick a "
            "different concept from the others if the talk allows.\n\n"
            f"Talk title: {request.talk.title}\n\nAbstract:\n{request.talk.abstract}"
        )
    return f"Topic: {request.topic_for(index)}\nDifficulty {request.difficulty} of {DIFFICULTIES.stop - 1}."


def output_schema(talk_mode: bool) -> dict[str, Any]:
    """The structured output. Every object closes with `additionalProperties: false`;
    counts and lengths are unsupported in the schema and checked after."""

    def obj(props: dict[str, Any]) -> dict[str, Any]:
        return {"type": "object", "properties": props, "required": list(props), "additionalProperties": False}

    text = {"type": "string"}
    props: dict[str, Any] = {
        "source": text,
        "options": {
            "type": "array",
            "items": obj({"text": text, "kind": {"type": "string", "enum": list(OPTION_KINDS)}, "why_tempting": text}),
        },
        "hint": text,
        "explains": obj({"what": text, "takeaway": text}),
        "expect": {"type": "string", "enum": list(EXPECTS)},
        "trace": obj(
            {
                "steps": {
                    "type": "array",
                    "items": obj(
                        {
                            "lines": {"type": "array", "items": {"type": "integer"}},
                            "focus": {"type": "array", "items": {"type": "integer"}},
                            "note": text,
                            "values": {"type": "array", "items": obj({"name": text, "was": text, "now": text})},
                            "pivot": {"type": "boolean"},
                        }
                    ),
                }
            }
        ),
    }
    if talk_mode:
        props["concept"] = obj({"name": text, "kind": {"type": "string", "enum": list(CONCEPT_KINDS)}})
    return obj(props)


def custom_id(index: int, count: int) -> str:
    return f"c{index + 1:0{max(2, len(str(count)))}d}"


def build_requests(request: RunRequest, cap: Capacity, model: str, max_tokens: int) -> list[dict[str, Any]]:
    """One batch request per candidate, in the Batches API's `{custom_id, params}` shape."""
    talk_mode = request.talk is not None
    system = system_prompt(cap, talk_mode)
    schema = output_schema(talk_mode)
    return [
        {
            "custom_id": custom_id(i, request.count),
            "params": {
                "model": model,
                "max_tokens": max_tokens,
                "system": system,
                "messages": [{"role": "user", "content": user_message(request, i)}],
                "thinking": {"type": "adaptive"},
                "output_config": {"effort": EFFORT, "format": {"type": "json_schema", "schema": schema}},
            },
        }
        for i in range(request.count)
    ]


# --------------------------------------------------------------------------- #
# Spend (ruling: refuse first, spend never)
# --------------------------------------------------------------------------- #


def list_prices(model: str) -> tuple[float, float]:
    try:
        return LIST_PRICES_PER_MTOK[model]
    except KeyError:
        raise GenerateError(no_price(model)) from None


def worst_case_usd(requests: Sequence[Mapping[str, Any]]) -> float:
    """The most these requests can bill, at the batch discount.

    Output: each request's own `max_tokens`, which thinking counts toward. Input: the
    request's parameters serialized as UTF-8 bytes, which is never fewer than its
    tokens. Read from the requests actually built, so no second number can drift.
    """
    total = 0.0
    for r in requests:
        params = r["params"]
        price_in, price_out = list_prices(params["model"])
        in_bytes = len(json.dumps(params, ensure_ascii=False).encode("utf-8"))
        total += params["max_tokens"] * price_out + in_bytes * price_in
    return total / 1_000_000 * BATCH_DISCOUNT


def check_spend(requests: Sequence[Mapping[str, Any]], cap_usd: float | None, fixture: bool) -> float:
    """Refuse a paid run with no cap or a worst case over it. Returns the worst case.

    A fixture run spends nothing, so it needs no cap; one given is still enforced.
    """
    worst = worst_case_usd(requests)
    if cap_usd is not None and not (math.isfinite(cap_usd) and cap_usd > 0):
        # `nan` compares false with everything, so `worst > nan` would let it through.
        raise GenerateError(f"Refusing to submit: a spend cap is a positive amount, not {cap_usd}.")
    if cap_usd is None:
        if not fixture:
            raise GenerateError(NO_CAP)
    elif worst > cap_usd:
        raise GenerateError(over_cap(len(requests), worst, cap_usd))
    return worst


# --------------------------------------------------------------------------- #
# The seam (D-18's pattern)
# --------------------------------------------------------------------------- #


#: What a client's numbers are, by client type (F4). A label is never passed in
#: beside the client: it is read off the class that builds it.
FIXTURE = "fixture"  # recorded by hand; not a measurement
MEASURED = "measured"  # a real batch, costed at list price and the batch discount
API_EQUIVALENT = "api-equivalent"  # the claude CLI's own figures; subscription usage, not billed

BATCHES = "batches"
CLAUDE_CLI = "claude-cli"
BACKENDS = (BATCHES, CLAUDE_CLI)


class BatchClient(Protocol):
    """How the generator reaches a model. Three calls, nothing else.

    Every implementation says, as class attributes, which backend it is and what
    kind of numbers it produces, so the report labels them by the client's type.
    A client may also set `stopped` to a sentence once it has stopped early (the
    claude-cli backend at a usage limit); the run reports the rest as not run.
    """

    backend: str
    numbers: str

    def submit(self, requests: Sequence[Mapping[str, Any]]) -> str: ...

    def poll(self, batch_id: str) -> str: ...

    def results(self, batch_id: str) -> Iterable[Mapping[str, Any]]: ...


@dataclass(frozen=True)
class Factory:
    """Builds one client of type `cls`, and says what kind it is before it exists.

    The run checks its caps before anything is built (a refusal leaves no client),
    so the backend and the kind of numbers have to be known first; they are read
    from `cls`, never passed beside it, and the client built must be of that type.
    """

    cls: type
    build: Callable[[], Any]

    @property
    def backend(self) -> str:
        return self.cls.backend

    @property
    def numbers(self) -> str:
        return self.cls.numbers

    def __call__(self) -> BatchClient:
        client = self.build()
        if not isinstance(client, self.cls):
            raise TypeError(f"a {self.cls.__name__} factory built a {type(client).__name__}")
        return client


class ReplayBatchClient:
    """Replays one batch recorded as a fixture (`tests/fixtures/generate/`).

    A fixture is `{"batch_id": ..., "results": [...]}`, each result in the SDK's
    shape: `{custom_id, result: {type: succeeded|errored|canceled|expired, ...}}`. It
    keeps the requests it was handed, so a test can read the brief that was built, and
    counts `submit` calls, so a test can prove a resumed run never submitted again.
    `delay_s` makes the first poll wait, for a run that has to be interrupted while
    the batch is in flight.
    """

    backend = BATCHES
    numbers = FIXTURE

    def __init__(self, fixture: Path, delay_s: float = 0.0) -> None:
        data = json.loads(Path(fixture).read_text(encoding="utf-8"))
        self.batch_id: str = data["batch_id"]
        self._results: list[dict[str, Any]] = data["results"]
        self.delay_s = delay_s
        self.submitted: list[Mapping[str, Any]] = []
        self.submit_calls = 0

    def submit(self, requests: Sequence[Mapping[str, Any]]) -> str:
        self.submit_calls += 1
        self.submitted = list(requests)
        return self.batch_id

    def poll(self, batch_id: str) -> str:
        if self.delay_s:
            time.sleep(self.delay_s)
            self.delay_s = 0.0
        return "ended"

    def results(self, batch_id: str) -> Iterable[Mapping[str, Any]]:
        yield from (json.loads(json.dumps(r)) for r in self._results)


class AnthropicBatchClient:
    """The Message Batches API through the Anthropic SDK.

    **Unexercised until the first paid run, HC-2.** No test, script or manual check in
    this repository has run it; the calls follow the SDK's documented Batches
    surface (`messages.batches.create`, `.retrieve(id).processing_status`,
    `.results(id)`), and results are dumped to plain JSON so the same classifier reads
    them as reads a fixture.

    It refuses without a key before importing the SDK and before any I/O. The key is
    passed explicitly, so a credential the SDK would otherwise find on its own (an
    `ant auth` profile, say) is never used without anyone saying so.
    """

    backend = BATCHES
    numbers = MEASURED

    def __init__(self, api_key: str | None) -> None:
        if not api_key:
            raise GenerateError(NO_KEY)
        import anthropic

        self._client = anthropic.Anthropic(api_key=api_key)

    def submit(self, requests: Sequence[Mapping[str, Any]]) -> str:
        from anthropic.types.message_create_params import MessageCreateParamsNonStreaming
        from anthropic.types.messages.batch_create_params import Request

        batch = self._client.messages.batches.create(
            requests=[
                Request(custom_id=r["custom_id"], params=MessageCreateParamsNonStreaming(**r["params"]))
                for r in requests
            ]
        )
        return batch.id

    def poll(self, batch_id: str) -> str:
        return self._client.messages.batches.retrieve(batch_id).processing_status

    def results(self, batch_id: str) -> Iterable[Mapping[str, Any]]:
        for result in self._client.messages.batches.results(batch_id):
            yield result.model_dump(mode="json")


# --------------------------------------------------------------------------- #
# Reading one result: emitted, or reported with every reason
# --------------------------------------------------------------------------- #


@dataclass
class Outcome:
    custom_id: str
    id: str
    topic: str | None
    difficulty: int
    reasons: list[str] = field(default_factory=list)
    question: Question | None = None
    expect: str | None = None
    concept: dict[str, str] | None = None
    usage: dict[str, int] = field(default_factory=dict)
    discarded: list[str] = field(default_factory=list)
    #: Per-candidate figures a client measured itself (the claude-cli backend's
    #: `wall_clock_s` and `api_equivalent_usd`). Empty for a batch, which has one
    #: wall-clock for every request and is costed here from tokens.
    measured: dict[str, Any] = field(default_factory=dict)

    @property
    def emitted(self) -> bool:
        return self.question is not None and not self.reasons


def _usage(message: Mapping[str, Any]) -> dict[str, int]:
    usage = message.get("usage") or {}
    keys = ("input_tokens", "output_tokens", "cache_creation_input_tokens", "cache_read_input_tokens")
    return {k: int(usage[k]) for k in keys if isinstance(usage.get(k), int)}


def _text(value: Any) -> str | None:
    return value if isinstance(value, str) and value.strip() else None


def _is_line(value: Any, n_lines: int) -> bool:
    return type(value) is int and 1 <= value <= n_lines


def _read_trace(data: Any, source: str | None, outcome: Outcome, outputs: frozenset[str] = frozenset()) -> Trace | None:
    """The model's drafted steps as a `bank.Trace`, or `None` with every reason.

    A value named `stdout` (any case or padding) is dropped from every step and the
    drop is recorded in `outcome.discarded`, never silently: the output is the
    verifier's to write (`verify.with_record`). The value itself is not echoed into
    the report, which is read before anything has run.

    A value under any other name whose `now` is the text of one of the `output`
    options (`outputs`) is dropped the same way (PQ-68 F1): it says which option
    the program prints, under a name the stdout filter does not catch. The match is
    exact, so a paraphrase still reaches review as authored content; the report
    lists every remaining value for AC-73's comparison there.
    """
    reasons = outcome.reasons
    raw = data if isinstance(data, dict) else {}
    raw_steps = raw.get("steps") if isinstance(raw.get("steps"), list) else []
    n_lines = len(source.splitlines()) if source is not None else None
    steps: list[TraceStep] = []
    for i, step in enumerate(raw_steps, 1):
        step = step if isinstance(step, dict) else {}
        note, lines, focus = _text(step.get("note")), step.get("lines"), step.get("focus")
        problems = []
        if note is None:
            problems.append("no note")
        if not (isinstance(lines, list) and lines and all(type(n) is int for n in lines)):
            problems.append("no lines")
            lines = []
        if not (isinstance(focus, list) and len(focus) == 2 and all(type(n) is int for n in focus) and focus[0] <= focus[1]):
            problems.append("focus is not a range [first, last]")
            focus = []
        unknown = sorted(set(step) - _STEP_KEYS)
        if unknown:
            problems.append(f"unknown field(s) {unknown}")
        if not isinstance(step.get("pivot", False), bool):
            problems.append("pivot is not true or false")
        if n_lines is not None and not all(_is_line(n, n_lines) for n in (*lines, *focus)):
            problems.append(f"points at a line outside the source's {n_lines}")
        values: list[ValueDelta] = []
        for value in step.get("values") if isinstance(step.get("values"), list) else []:
            value = value if isinstance(value, dict) else {}
            name, was, now = _text(value.get("name")), value.get("was"), value.get("now")
            if name is not None and name.strip().lower() == STDOUT:
                outcome.discarded.append(f"step {i}: a {STDOUT} value, which only the verifier writes")
                continue
            if isinstance(now, str) and now in outputs:
                outcome.discarded.append(
                    f"step {i}: a value whose now is an output option's text, which only the verifier decides"
                )
                continue
            if name is None or not isinstance(was, str) or not isinstance(now, str):
                problems.append("a value needs a name, was and now")
                continue
            values.append(ValueDelta(name=name, was=was, now=now))
        if problems:
            reasons.append(f"trace: step {i}: " + "; ".join(problems))
            continue
        steps.append(
            TraceStep(lines=tuple(lines), focus=(focus[0], focus[1]), note=note or "", values=tuple(values), pivot=step.get("pivot") is True)
        )
    if len(raw_steps) < MIN_TRACE_STEPS:
        reasons.append(f"trace: {len(raw_steps)} step(s); a question needs at least {MIN_TRACE_STEPS} (SPEC 7.4)")
    pivots = sum(s.pivot for s in steps)
    if pivots > 1:
        reasons.append(f"trace: {pivots} pivot steps; at most one")
    if any(r.startswith("trace:") for r in reasons):
        return None
    return Trace(steps=tuple(steps))


def _read_candidate(data: Any, outcome: Outcome, talk_mode: bool, cap: Capacity, room: audit.Room) -> None:
    """Turn the model's JSON into a §3.1 record, or say everything wrong with it."""
    reasons = outcome.reasons
    if not isinstance(data, dict):
        reasons.append("shape: the output is not a JSON object")
        return

    source = _text(data.get("source"))
    if source is None:
        reasons.append("shape: no source")
    hint = _text(data.get("hint"))
    if hint is None:
        reasons.append("shape: no hint")
    explains = data.get("explains") if isinstance(data.get("explains"), dict) else {}
    what, takeaway = _text(explains.get("what")), _text(explains.get("takeaway"))
    if what is None or takeaway is None:
        reasons.append("shape: explains needs both what and takeaway")

    expect = data.get("expect")
    if expect not in EXPECTS:
        reasons.append(f"shape: expect is {expect!r}, not one of {', '.join(EXPECTS)}")
    else:
        outcome.expect = expect

    raw_options = data.get("options") if isinstance(data.get("options"), list) else []
    options: list[Option] = []
    for i, raw in enumerate(raw_options):
        raw = raw if isinstance(raw, dict) else {}
        text, kind, why = _text(raw.get("text")), raw.get("kind"), _text(raw.get("why_tempting"))
        if text is None or kind not in OPTION_KINDS:
            reasons.append(f"shape: option {i + 1} needs text and a kind")
            continue
        if why is None:
            reasons.append(f"shape: option {i + 1} has no why_tempting")
        options.append(Option(text=text, kind=kind, why_tempting=why))
    for kind in ("ub", "panic"):
        n = sum(o.kind == kind for o in options)
        if n > 1:
            reasons.append(f"shape: {n} options of kind {kind!r}; at most one, or the answer is ambiguous")
    if expect == "ub" and sum(o.kind == "ub" for o in options) != 1:
        reasons.append("shape: expect is ub, so exactly one option must be of kind 'ub'")

    if talk_mode:
        concept = data.get("concept") if isinstance(data.get("concept"), dict) else {}
        name, kind = _text(concept.get("name")), concept.get("kind")
        if name is None or kind not in CONCEPT_KINDS:
            reasons.append("talk: no named std or core-language concept")
        elif kind == "std" and not _STD_PATH.match(name):
            reasons.append(f"talk: {name!r} is marked std but is not a std/core/alloc path")
        else:
            outcome.concept = {"name": name, "kind": kind}
            outcome.topic = name

    outputs = frozenset(o.text for o in options if o.kind == "output")
    trace = _read_trace(data.get("trace"), source, outcome, outputs)
    reasons.extend(capacity_breaks(source, [o.text for o in options], cap, room))

    if source is None or hint is None or what is None or takeaway is None or outcome.topic is None:
        return
    question = Question(
        id=outcome.id,
        source=source,
        topic=outcome.topic,
        difficulty_requested=outcome.difficulty,
        options=tuple(options),
        hint=hint,
        explains=Explains(what=what, takeaway=takeaway),
        trace=trace if trace is not None else Trace(),  # a broken trace is already a reason
    )
    try:
        bank.validate_question(question)
    except BankError as exc:
        reasons.append(f"shape: {exc}")
        return
    outcome.question = question


def classify(entry: Mapping[str, Any] | None, outcome: Outcome, talk_mode: bool, cap: Capacity, room: audit.Room) -> Outcome:
    """One batch result into an `Outcome`. Never raises on what the batch returned."""
    if entry is None:
        outcome.reasons.append("no result came back for this request")
        return outcome
    if isinstance(entry.get("measured"), Mapping):
        outcome.measured = dict(entry["measured"])
    result = entry.get("result") or {}
    kind = result.get("type")
    if kind != "succeeded":
        detail = ""
        if kind == "errored":
            error = (result.get("error") or {}).get("error") or {}
            detail = f": {error.get('type', 'error')}: {error.get('message', '')}".rstrip(": ")
        outcome.reasons.append(f"{kind or 'unknown result'}{detail}")
        return outcome

    message = result.get("message") or {}
    outcome.usage = _usage(message)
    stop = message.get("stop_reason")
    if stop == "refusal":
        outcome.reasons.append("refused (stop_reason refusal)")
        return outcome
    if stop == "max_tokens":
        outcome.reasons.append("cut off at max_tokens; the output is incomplete")
    text = next((b.get("text") for b in message.get("content") or [] if b.get("type") == "text"), None)
    if not isinstance(text, str):
        outcome.reasons.append("shape: no text block in the response")
        return outcome
    try:
        data = json.loads(text)
    except json.JSONDecodeError:
        outcome.reasons.append("shape: the output is not valid JSON")
        return outcome
    _read_candidate(data, outcome, talk_mode, cap, room)
    return outcome


# --------------------------------------------------------------------------- #
# Writing: whole files or nothing
# --------------------------------------------------------------------------- #

PARTIAL_SUFFIX = ".partial"


def _write_bytes(handle: Any, data: bytes) -> None:
    handle.write(data)


def _commit(partial: Path, path: Path) -> None:
    os.replace(partial, path)


def write_atomic(path: Path, payload: Any) -> Path:
    """Write JSON to `.<name>.partial` beside `path`, fsync, rename into place.

    A run killed mid-write leaves the `.partial`, never a short `path`; the next write
    of the same file overwrites it, and a finished run sweeps any left over.
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_name(f".{path.name}{PARTIAL_SUFFIX}")
    data = (json.dumps(payload, indent=2, ensure_ascii=False) + "\n").encode("utf-8")
    with open(partial, "wb") as handle:
        _write_bytes(handle, data)
        handle.flush()
        os.fsync(handle.fileno())
    _commit(partial, path)
    return path


#: Where the claude-cli backend keeps each finished candidate's result, so a rerun
#: picks up where a stopped or killed run left off (`generate_cli`).
CLI_SUBDIR = "cli"


def sweep_partials(run_dir: Path) -> None:
    for directory in (run_dir, run_dir / RUN_SUBDIR, run_dir / RUN_SUBDIR / CLI_SUBDIR):
        if directory.is_dir():
            for leftover in directory.glob(f".*{PARTIAL_SUFFIX}"):
                leftover.unlink()


# --------------------------------------------------------------------------- #
# The report (AC-3, AC-5)
# --------------------------------------------------------------------------- #


def cost_usd(usage: Mapping[str, int], model: str) -> float:
    price_in, price_out = list_prices(model)
    dollars = (
        usage.get("input_tokens", 0) * price_in
        + usage.get("output_tokens", 0) * price_out
        + usage.get("cache_creation_input_tokens", 0) * price_in * CACHE_WRITE_MULTIPLIER
        + usage.get("cache_read_input_tokens", 0) * price_in * CACHE_READ_MULTIPLIER
    )
    return dollars / 1_000_000 * BATCH_DISCOUNT


def honoured(request: RunRequest, outcomes: Sequence[Outcome]) -> dict[str, dict[str, Any]]:
    """Which of count, topics and difficulty the run honoured, and why not (AC-3)."""
    emitted = [o for o in outcomes if o.emitted]
    out: dict[str, dict[str, Any]] = {
        "count": {
            "honoured": len(emitted) == request.count,
            "detail": f"{len(emitted)} of {request.count} asked for were emitted",
        }
    }
    if request.talk is not None:
        out["topics"] = {"honoured": None, "detail": "talk mode: each topic is the concept the model named"}
    else:
        covered = {o.topic for o in emitted}
        missing = [t for t in dict.fromkeys(request.topics) if t not in covered]
        out["topics"] = {
            "honoured": not missing,
            "detail": "every topic has a candidate" if not missing else f"no candidate for: {', '.join(missing)}",
        }
    out["difficulty"] = {
        "honoured": bool(emitted),
        "detail": (
            f"every emitted candidate is tagged {request.difficulty}; whether it is that hard "
            "is judged at review (AC-88)"
            if emitted
            else "no candidate was emitted"
        ),
    }
    return out


NUMBERS_NOTES = {
    FIXTURE: (
        "FIXTURE NUMBERS: tokens come from recorded fixtures, not from a real batch; "
        "they are not a measurement"
    ),
    MEASURED: "tokens as the batch reported them; cost is list price at the batch discount",
    API_EQUIVALENT: (
        "API-EQUIVALENT, NOT BILLED: tokens, wall-clock and total_cost_usd as the claude CLI "
        "reported them for each candidate; the run consumed subscription usage, not dollars"
    ),
}


def build_report(
    *,
    run_id: str,
    request: RunRequest,
    model: str,
    cap: Capacity,
    room: audit.Room,
    backend: str,
    numbers: str,
    batch_id: str,
    resumed: bool,
    worst_case: float | None,
    wall_clock_s: float,
    outcomes: Sequence[Outcome],
    run_dir: Path,
    stopped: str | None = None,
    claude_bin: str | None = None,
) -> dict[str, Any]:
    # The claude-cli backend's dollars are the CLI's own `total_cost_usd`, per
    # candidate, and they are never spend: no list price, no batch discount, and no
    # dollar total anywhere in its report (PQ-69 ruling 5).
    per_candidate = numbers == API_EQUIVALENT
    rows = []
    for o in outcomes:
        row: dict[str, Any] = {
            "custom_id": o.custom_id,
            "id": o.id,
            "status": "emitted" if o.emitted else "reported",
            "topic": o.topic,
            "difficulty_requested": o.difficulty,
            "expect": o.expect,
            "reasons": o.reasons,
            **{k: o.usage.get(k, 0) for k in ("input_tokens", "output_tokens")},
        }
        if per_candidate:
            row["api_equivalent_usd"] = o.measured.get("api_equivalent_usd")
            row["wall_clock_s"] = o.measured.get("wall_clock_s")
        else:
            row["cost_usd"] = round(cost_usd(o.usage, model), 6)
            row["wall_clock_s"] = round(wall_clock_s, 3)
        for k in ("cache_creation_input_tokens", "cache_read_input_tokens"):
            if k in o.usage:
                row[k] = o.usage[k]
        if o.concept is not None:
            row["concept"] = o.concept
        if o.discarded:
            row["discarded"] = o.discarded
        if o.emitted:
            row["path"] = str(run_dir / f"{o.id}.json")
            # For AC-73 at review: the quoted spans in the prose, and every trace value
            # the model wrote, because a value can state an output under any name.
            row["quoted_spans"] = list(dict.fromkeys([*bank.quoted_outputs(o.question), *_value_spans(o.question)]))
        rows.append(row)
    checks = honoured(request, outcomes)
    totals: dict[str, Any] = {
        "requested": request.count,
        "emitted": sum(o.emitted for o in outcomes),
        "reported": sum(not o.emitted for o in outcomes),
        "input_tokens": sum(o.usage.get("input_tokens", 0) for o in outcomes),
        "output_tokens": sum(o.usage.get("output_tokens", 0) for o in outcomes),
    }
    if per_candidate:
        seconds = [r["wall_clock_s"] for r in rows if isinstance(r["wall_clock_s"], (int, float))]
        totals["wall_clock_s"] = round(sum(seconds), 3)
        prices: dict[str, Any] = {"basis": "the claude CLI's total_cost_usd per candidate: API-equivalent, not billed"}
        wall_clock_basis = "one claude process per candidate: each row's wall-clock is that process's duration_ms"
    else:
        totals["cost_usd"] = round(sum(cost_usd(o.usage, model) for o in outcomes), 6)
        totals["wall_clock_s"] = round(wall_clock_s, 3)
        price_in, price_out = list_prices(model)
        prices = {
            "input_per_mtok": price_in,
            "output_per_mtok": price_out,
            "basis": "list-price estimate",
            "batch_discount": BATCH_DISCOUNT,
        }
        wall_clock_basis = "one batch: every candidate's wall-clock is the batch's, from submit to seen ended"
    report: dict[str, Any] = {
        "run_id": run_id,
        "backend": backend,
        "numbers": numbers,
        "numbers_note": NUMBERS_NOTES[numbers],
        "model": model,
        "prices": prices,
        "request": request.to_dict(),
        "capacity": {
            "source_lines": cap.source_lines,
            "source_width": cap.source_width,
            "option_lines": cap.option_lines,
            "option_chars": cap.option_chars,
            "floor_px": round(cap.floor_px, 3),
            "room": {"screen_width_ft": room.screen_width_ft, "screen_height_ft": room.screen_height_ft, "back_row_ft": room.back_row_ft},
        },
        "batch_id": batch_id,
        "resumed": resumed,
        "worst_case_usd": None if worst_case is None else round(worst_case, 4),
        "wall_clock_basis": wall_clock_basis,
        "candidates": rows,
        "totals": totals,
        "honoured": checks,
        "not_honoured": [name for name, check in checks.items() if check["honoured"] is False],
    }
    if stopped is not None:
        report["stopped"] = stopped
    if claude_bin is not None:
        report["claude_bin"] = claude_bin
    return report


def _value_spans(question: Question | None) -> list[str]:
    if question is None:
        return []
    return [v.now for step in question.trace.steps for v in step.values if v.now.strip()]


# --------------------------------------------------------------------------- #
# The run
# --------------------------------------------------------------------------- #


def check_candidates(count: int, max_candidates: int | None) -> None:
    """The claude-cli backend's cap (PQ-69 ruling 4): required, no default, and a
    run over it refuses rather than running part of what was asked."""
    if max_candidates is None:
        raise GenerateError(NO_CANDIDATE_CAP)
    if max_candidates < 1:
        raise GenerateError(f"Refusing to start: a candidate cap is at least 1, not {max_candidates}.")
    if count > max_candidates:
        raise GenerateError(over_candidate_cap(count, max_candidates))


def run(
    request: RunRequest,
    *,
    client_factory: Factory,
    out: Path,
    run_id: str,
    cap_usd: float | None = None,
    max_candidates: int | None = None,
    model: str = DEFAULT_MODEL,
    max_tokens: int = DEFAULT_MAX_TOKENS,
    room: audit.Room = audit.Room(),
    poll_interval_s: float = DEFAULT_POLL_INTERVAL_S,
    clock: Callable[[], float] = time.time,
    sleep: Callable[[float], None] = time.sleep,
) -> dict[str, Any]:
    """One generation run, start to report. Re-running with the same `run_id` resumes.

    Order matters and is the point: validate, build, check the backend's cap, *then*
    build the client, write the state, submit, write the batch id. A refusal
    therefore happens before a client exists, and a resumed run never reaches
    `submit`. Each backend has its own cap and refuses the other's: dollars for
    Batches, a candidate count for claude-cli, which is checked again on a resume
    because a resumed claude-cli run starts processes.
    """
    request.validate()
    if not _RUN_ID.match(run_id):
        raise GenerateError(f"A run id is lower-case letters, digits and dashes, not {run_id!r}.")
    if max_tokens < 1:
        raise GenerateError(f"max_tokens is at least 1, not {max_tokens}.")
    backend, numbers = client_factory.backend, client_factory.numbers
    if backend == CLAUDE_CLI:
        if cap_usd is not None:
            raise GenerateError(CLI_NO_USD)
        check_candidates(request.count, max_candidates)
    elif max_candidates is not None:
        raise GenerateError(BATCHES_NO_MAX)
    cap = capacity(room)
    requests = build_requests(request, cap, model, max_tokens)
    if Path(out).resolve().is_relative_to(QUESTIONS_DIR.resolve()):
        raise GenerateError("Refusing to write candidates into bank/questions/; that is the bank.")
    run_dir = Path(out) / run_id
    state_path = run_dir / RUN_SUBDIR / STATE_FILE
    # The backend and the kind of numbers are part of the identity, so a replayed run
    # id never resumes a real batch, nor a real one a fixture's, nor either backend
    # the other's run.
    identity = {
        "request": request.to_dict(),
        "model": model,
        "max_tokens": max_tokens,
        "backend": backend,
        "numbers": numbers,
    }

    worst: float | None = None
    if state_path.is_file():
        state = json.loads(state_path.read_text(encoding="utf-8"))
        if {k: state.get(k) for k in identity} != identity:
            raise GenerateError(
                f"Run {run_id} was started with a different request; use a new --run-id."
            )
        if not state.get("batch_id"):
            raise GenerateError(
                f"Run {run_id} stopped while submitting, so a batch may already exist for it; "
                "check the Console before starting again under a new --run-id."
            )
        client = client_factory()
        resumed = True
    else:
        if backend == BATCHES:
            worst = check_spend(requests, cap_usd, numbers == FIXTURE)
        client = client_factory()
        state = {**identity, "status": "submitting"}
        if worst is not None:
            state["worst_case_usd"] = round(worst, 4)
        write_atomic(state_path, state)
        state["batch_id"] = client.submit(requests)
        state["submitted_at"] = clock()
        state["status"] = "submitted"
        write_atomic(state_path, state)
        resumed = False

    batch_id = state["batch_id"]
    while client.poll(batch_id) != "ended":
        sleep(poll_interval_s)
    if "ended_seen_at" not in state:
        state["ended_seen_at"] = clock()
        write_atomic(state_path, state)
    wall_clock = max(0.0, state["ended_seen_at"] - state["submitted_at"])

    by_id: dict[str, Mapping[str, Any]] = {}
    for entry in client.results(batch_id):
        by_id.setdefault(str(entry.get("custom_id")), entry)
    stopped = getattr(client, "stopped", None)

    talk_mode = request.talk is not None
    outcomes = []
    for i, r in enumerate(requests):
        cid = r["custom_id"]
        outcome = Outcome(
            custom_id=cid,
            id=f"{run_id}-{cid[1:]}",
            topic=request.topic_for(i),
            difficulty=request.difficulty,
        )
        if stopped is not None and cid not in by_id:
            outcome.reasons.append(NOT_RUN)
            outcomes.append(outcome)
            continue
        outcomes.append(classify(by_id.get(cid), outcome, talk_mode, cap, room))

    for outcome in outcomes:
        if outcome.emitted:
            write_atomic(run_dir / f"{outcome.id}.json", bank.question_to_dict(outcome.question))

    report = build_report(
        run_id=run_id,
        request=request,
        model=model,
        cap=cap,
        room=room,
        backend=backend,
        numbers=numbers,
        batch_id=batch_id,
        resumed=resumed,
        worst_case=worst if worst is not None else state.get("worst_case_usd"),
        wall_clock_s=wall_clock,
        outcomes=outcomes,
        run_dir=run_dir,
        stopped=stopped,
        claude_bin=getattr(client, "binary", None),
    )
    write_atomic(run_dir / RUN_SUBDIR / REPORT_FILE, report)
    sweep_partials(run_dir)
    return report


# --------------------------------------------------------------------------- #
# The command line
# --------------------------------------------------------------------------- #


def default_run_id(now: float | None = None) -> str:
    return "g" + time.strftime("%Y%m%d%H%M%S", time.gmtime(time.time() if now is None else now))


def _row_dollars(row: Mapping[str, Any]) -> str:
    if "cost_usd" in row:
        return f"${row['cost_usd']:.4f}"
    if isinstance(row.get("api_equivalent_usd"), (int, float)):
        return f"API-equivalent ${row['api_equivalent_usd']:.4f}, not billed"
    return "no API-equivalent figure reported"


def print_report(report: Mapping[str, Any], out: Any = None) -> None:
    out = sys.stdout if out is None else out
    if report.get("stopped"):
        print(report["stopped"], file=out)
    print(report["numbers_note"], file=out)
    for row in report["candidates"]:
        line = f"{row['id']}: {row['status']}"
        if row["status"] == "emitted":
            line += f" -> {row['path']}"
        else:
            line += f" ({'; '.join(row['reasons'])})"
        if row.get("discarded"):
            line += f" [discarded: {'; '.join(row['discarded'])}]"
        line += f" | {row['input_tokens']} in / {row['output_tokens']} out, {_row_dollars(row)}, {row['wall_clock_s']} s"
        print(line, file=out)
    totals = report["totals"]
    summary = f"emitted {totals['emitted']} of {totals['requested']}, reported {totals['reported']}; "
    if "cost_usd" in totals:
        summary += f"${totals['cost_usd']:.4f} ({report['prices']['basis']}, batch discount), "
    else:
        summary += f"{totals['input_tokens']} in / {totals['output_tokens']} out tokens of subscription usage, "
    print(summary + f"{totals['wall_clock_s']} s", file=out)
    for name, check in report["honoured"].items():
        mark = "n/a " if check["honoured"] is None else ("ok  " if check["honoured"] else "NOT ")
        print(f"{mark}{name}: {check['detail']}", file=out)


def main(argv: Sequence[str] | None = None) -> int:
    """`python -m popquiz.generate`. Exit 0: every request honoured. 1: the run
    finished and reported something it could not honour, or stopped early. 2:
    refused or failed."""
    parser = argparse.ArgumentParser(prog="generate", description=__doc__.splitlines()[0])
    parser.add_argument("--count", type=int, required=True)
    parser.add_argument("--difficulty", type=int, required=True)
    parser.add_argument("--topic", action="append", default=[], help="repeat for several")
    parser.add_argument("--talk-title")
    parser.add_argument("--talk-abstract")
    parser.add_argument(
        "--backend", choices=BACKENDS, default=BATCHES,
        help="batches (the default): the Message Batches API; claude-cli: headless Claude Code on a subscription",
    )
    parser.add_argument("--cap-usd", type=float, default=None, help="batches: spend cap for a paid run; no default")
    parser.add_argument(
        "--max-candidates", type=int, default=None, help="claude-cli: the most candidates a run may ask for; no default"
    )
    parser.add_argument("--claude-bin", default=None, help="claude-cli: the claude executable; default: claude on PATH")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--max-tokens", type=int, default=DEFAULT_MAX_TOKENS)
    parser.add_argument("--replay", type=Path, help="batches: a recorded batch fixture; no API, no key")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--run-id", default=None, help="resume this run, or name a new one")
    parser.add_argument("--poll-interval-s", type=float, default=DEFAULT_POLL_INTERVAL_S)
    defaults = audit.Room()
    parser.add_argument("--screen-width-ft", type=float, default=defaults.screen_width_ft)
    parser.add_argument("--screen-height-ft", type=float, default=None, help="default: width x 9/16")
    parser.add_argument("--back-row-ft", type=float, default=defaults.back_row_ft)
    args = parser.parse_args(argv)

    if args.screen_height_ft is not None:
        height = args.screen_height_ft
    elif args.screen_width_ft == defaults.screen_width_ft:
        height = defaults.screen_height_ft
    else:
        height = args.screen_width_ft * 9 / 16
    room = audit.Room(args.screen_width_ft, height, args.back_row_ft)

    talk = None
    if args.talk_title is not None or args.talk_abstract is not None:
        talk = Talk(args.talk_title or "", args.talk_abstract or "")
    request = RunRequest(count=args.count, difficulty=args.difficulty, topics=tuple(args.topic), talk=talk)
    run_id = args.run_id or default_run_id()

    # One backend per run, named on the command line. Each factory builds exactly one
    # client type and nothing here builds another after a failure: neither backend
    # ever falls back to the other (PQ-69 ruling 1).
    replay = args.replay
    if args.backend == CLAUDE_CLI:
        if replay is not None:
            print(f"generate: {CLI_NO_REPLAY}", file=sys.stderr)
            return 2
        from popquiz import generate_cli

        cache_dir = Path(args.out) / run_id / RUN_SUBDIR / CLI_SUBDIR
        claude_bin = args.claude_bin
        factory = Factory(
            generate_cli.ClaudeCliClient, lambda: generate_cli.ClaudeCliClient(claude_bin, cache_dir)
        )
    elif args.claude_bin is not None:
        print(f"generate: {BATCHES_NO_CLAUDE_BIN}", file=sys.stderr)
        return 2
    elif replay is not None:
        # `--replay` is the only way to a fixture client, and a fixture run never
        # builds the real one, whatever the environment holds.
        factory = Factory(ReplayBatchClient, lambda: ReplayBatchClient(replay))
    else:
        factory = Factory(AnthropicBatchClient, lambda: AnthropicBatchClient(os.environ.get("ANTHROPIC_API_KEY")))

    try:
        report = run(
            request,
            client_factory=factory,
            out=args.out,
            run_id=run_id,
            cap_usd=args.cap_usd,
            max_candidates=args.max_candidates,
            model=args.model,
            max_tokens=args.max_tokens,
            room=room,
            poll_interval_s=args.poll_interval_s,
        )
    except GenerateError as exc:
        print(f"generate: {exc}", file=sys.stderr)
        return 2
    print_report(report)
    print(f"report: {Path(args.out) / report['run_id'] / RUN_SUBDIR / REPORT_FILE}")
    return 1 if report["not_honoured"] or report.get("stopped") else 0


if __name__ == "__main__":
    sys.exit(main())
