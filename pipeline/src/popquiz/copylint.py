"""The two copy lints over question prose: SPEC 11's Forbidden row (G-5, AC-98) and
the 11.1 trope check (D-23, AC-42).

The Python twin of `web/test/copylint.js`. The patterns below are SPEC's own text,
character for character; `tests/test_copylint.py` reads SPEC.md and fails if they
differ, and the JavaScript suite pins its copy the same way, so the two cannot drift
from SPEC or from each other. Both run the fixtures in `bank/fixtures/copy-lint/`.

**Same behaviour as JavaScript's `i` without `u`.** Compiled with
`re.ASCII | re.IGNORECASE`, so `\\b` and `\\w` are ASCII and case folds ASCII only,
as in JavaScript. JavaScript's `\\s` is wider than ASCII (NBSP, the U+2000 block,
U+FEFF, ...), so each `\\s` in a pattern is compiled as that explicit class; the held
source text stays verbatim. The one difference that cannot be removed: a `{0,N}` span
counts UTF-16 code units in JavaScript and code points here, so a run of astral
characters (emoji) near the bound can match here and not there.
`bank/fixtures/copy-lint/dialect.json` records it.

**Consequence.** Over question prose a match is a warning beside the text on the
review screen (SPEC 7.4): `check_prose` is the hook the review screen (T-18) calls,
and it never raises. Over the copy module the same patterns fail the build; that
check runs in `web/test/copylint.test.js`.

This is not the receipt's rule (`receipt.FORBIDDEN_IN_A_LINE`, D-25), which is a
different lint over a different string.
"""

from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass
from typing import Any

# SPEC 11, the Forbidden row. The one normative list.
FORBIDDEN: tuple[str, ...] = (
    r"turn to",
    r"ask (someone|the person|your neighbou?r)",
    r"find someone",
    r"volunteer",
    r"who (said|picked|chose)",
    r"\bwrong\b",
    r"\bincorrect\b",
    r"✗",
    r"argu",
)

# SPEC 11.1, grouped as the spec groups them.
TROPES: dict[str, tuple[str, ...]] = {
    "contrast": (
        r"\b(it'?s|it is|that'?s|that is|this is) not\b[^.!?]{0,90}(—|;|,|:)\s*(it'?s|it is|that'?s|that is|this is|but)\b",
        r"\bnot (just|only|merely|simply)\b",
        r"(^|[.!?]\s+)not (the|a|an|your|our|every|any)\b",
        r"\bnot\b[^.!?]{0,40},\s*not\b[^.!?]{0,40},\s*not\b",
        r"\b(listen|look|read|think),? (don'?t|do not)\b",
    ),
    "filler": (
        r"\b(genuinely|truly|honestly|quietly|extremely|deeply|fundamentally|literally)\b",
        r"\bdoing (all|the) (the )?work\b",
    ),
    "signpost": (
        r"\bworth (a|the|stopping|talking|noting|remembering)\b",
        r"\b(here'?s|here is) the (thing|whole idea)\b",
        r"\bthe (important|key) (word|thing|part)\b",
        r"\bif you remember one thing\b",
    ),
    "reassurance": (
        r"\bthat'?s (fine|okay|ok|totally fine)\b",
        r"\bit'?s (fine|okay|ok|normal) to\b",
        r"\bnothing is missing\b",
        r"\bdon'?t worry\b",
    ),
    "flattery": (r"\bmost (interesting|impressive|clever|insightful)\b",),
}

# JavaScript's `\s` (ECMA-262 WhiteSpace + LineTerminator), spelled out.
JS_WHITESPACE = (
    "[\\t\\n\\v\\f\\r \\u00a0\\u1680\\u2000-\\u200a"
    "\\u2028\\u2029\\u202f\\u205f\\u3000\\ufeff]"
)

_FLAGS = re.ASCII | re.IGNORECASE


def _compile(source: str) -> re.Pattern[str]:
    # No pattern puts `\s` inside a character class, so a textual rewrite is exact.
    return re.compile(source.replace(r"\s", JS_WHITESPACE), _FLAGS)


@dataclass(frozen=True)
class Rule:
    lint: str  # "forbidden" | "trope"
    group: str  # "forbidden" or the 11.1 group
    id: str  # "forbidden/N" or "<group>/N", N from 1 in SPEC's order
    source: str
    regex: re.Pattern[str]


RULES: tuple[Rule, ...] = tuple(
    [Rule("forbidden", "forbidden", f"forbidden/{i}", s, _compile(s)) for i, s in enumerate(FORBIDDEN, 1)]
    + [
        Rule("trope", group, f"{group}/{i}", s, _compile(s))
        for group, sources in TROPES.items()
        for i, s in enumerate(sources, 1)
    ]
)


@dataclass(frozen=True)
class Hit:
    lint: str
    group: str
    id: str
    pattern: str
    match: str


@dataclass(frozen=True)
class Warning:  # noqa: A001 - the review screen's word for it
    """One match in one field of a question, for the review screen to show beside it."""

    field: str
    group: str
    pattern: str
    matched: str
    id: str  # the rule's id, `<group>/N`: what a log may print, where `matched` may not


_ZERO_WIDTH = re.compile("[\u200b-\u200d\ufeff]")
# JavaScript's class, not `str.isspace`: U+001C..U+001F are Python whitespace and
# not JavaScript's, and the two twins must collapse the same characters.
_WHITESPACE_RUN = re.compile(JS_WHITESPACE + "+")
_APOSTROPHE = re.compile("[\u2018\u2019\u02bc`]")
_SPACED_HYPHEN = re.compile(" - ")
_DASH = re.compile("-{2,}|[\u2013\u2014]")


def normalize(text: str) -> str:
    """The text as SPEC's patterns spell it, so a variant cannot walk past them.
    In order, the same as `copylint.js`: NFKC (fullwidth letters, look-alikes and
    NBSP to their plain forms); zero-width characters (U+200B..U+200D, U+FEFF)
    removed; every run of JavaScript whitespace, newlines included, to one space,
    and the ends trimmed; U+2018, U+2019, U+02BC and the backtick to `'`; `--`
    (and longer), U+2013, U+2014 and a spaced hyphen ` - ` to U+2014, the dash the
    contrast pattern spells. Lengths change: a match is quoted from this text."""
    text = unicodedata.normalize("NFKC", text)
    text = _ZERO_WIDTH.sub("", text)
    text = _WHITESPACE_RUN.sub(" ", text).strip(" ")
    text = _APOSTROPHE.sub("'", text)
    text = _SPACED_HYPHEN.sub(" \u2014 ", text)
    return _DASH.sub("\u2014", text)


def check(text: str) -> list[Hit]:
    """Every rule that matches `text`, in SPEC's order, quoting the normalized text."""
    norm = normalize(text)
    return [
        Hit(rule.lint, rule.group, rule.id, rule.source, m.group(0))
        for rule in RULES
        if (m := rule.regex.search(norm))
    ]


def check_raw(text: str) -> list[str]:
    """The ids of every rule whose compiled pattern matches `text` as given, with no
    normalizing: the engine alone, for the dialect fixtures."""
    return [rule.id for rule in RULES if rule.regex.search(text)]


def forbidden_matches(text: str) -> list[Hit]:
    return [h for h in check(text) if h.lint == "forbidden"]


def trope_matches(text: str) -> list[Hit]:
    return [h for h in check(text) if h.lint == "trope"]


def _get(obj: Any, name: str) -> Any:
    if isinstance(obj, dict):
        return obj.get(name)
    return getattr(obj, name, None)


def prose_fields(record: Any) -> list[tuple[str, str]]:
    """The lint's homes in one question: SPEC 11's four, `explains.what`,
    `explains.takeaway`, each option's `why_tempting` and `hint`, and every trace
    step's `note`, which the wall, the host and take-it-home all show. AC-98 covers
    every participant-facing string and outranks SPEC's list (F-47, ruled
    2026-10-16; the list's amendment goes upstream). Never `explains.legacy` (never
    rendered), `source` or `options[].text` (program text).
    Accepts a record as JSON (dict) or a `bank.Question`; skips what is missing."""
    fields: list[tuple[str, str]] = []
    if record is None:
        return fields
    explains = _get(record, "explains")
    if explains is not None:
        for name in ("what", "takeaway"):
            value = _get(explains, name)
            if isinstance(value, str):
                fields.append((f"explains.{name}", value))
    options = _get(record, "options")
    if isinstance(options, (list, tuple)):
        for i, option in enumerate(options):
            value = _get(option, "why_tempting")
            if isinstance(value, str):
                fields.append((f"options[{i}].why_tempting", value))
    hint = _get(record, "hint")
    if isinstance(hint, str):
        fields.append(("hint", hint))
    trace = _get(record, "trace")
    steps = _get(trace, "steps") if trace is not None else None
    if isinstance(steps, (list, tuple)):
        for i, step in enumerate(steps):
            value = _get(step, "note")
            if isinstance(value, str):
                fields.append((f"trace.steps[{i}].note", value))
    return fields


def check_prose(record: Any) -> list[Warning]:
    """Every Forbidden and trope match in a question's prose, as warnings. Never
    raises: a malformed record yields what could be read, and a record that cannot
    be read at all yields nothing. A warning blocks neither accept nor affirm."""
    try:
        return [
            Warning(field=name, group=hit.group, pattern=hit.pattern, matched=hit.match, id=hit.id)
            for name, text in prose_fields(record)
            for hit in check(text)
        ]
    except Exception:  # noqa: BLE001 - the review screen must always render
        return []
