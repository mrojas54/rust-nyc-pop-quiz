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


def normalize(text: str) -> str:
    """Curly single quotes read as the straight one SPEC's patterns spell. Same
    length, so an index into the result is an index into the original."""
    return text.replace("‘", "'").replace("’", "'")


def check(text: str) -> list[Hit]:
    """Every rule that matches `text`, in SPEC's order, quoting the original text."""
    norm = normalize(text)
    hits = []
    for rule in RULES:
        m = rule.regex.search(norm)
        if m:
            hits.append(Hit(rule.lint, rule.group, rule.id, rule.source, text[m.start() : m.end()]))
    return hits


def forbidden_matches(text: str) -> list[Hit]:
    return [h for h in check(text) if h.lint == "forbidden"]


def trope_matches(text: str) -> list[Hit]:
    return [h for h in check(text) if h.lint == "trope"]


def _get(obj: Any, name: str) -> Any:
    if isinstance(obj, dict):
        return obj.get(name)
    return getattr(obj, name, None)


def prose_fields(record: Any) -> list[tuple[str, str]]:
    """The lint's homes in one question (SPEC 11): `explains.what`,
    `explains.takeaway`, each option's `why_tempting`, and `hint`. Never
    `explains.legacy` (never rendered), `source` or `options[].text` (program text).
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
    return fields


def check_prose(record: Any) -> list[Warning]:
    """Every Forbidden and trope match in a question's prose, as warnings. Never
    raises: a malformed record yields what could be read, and a record that cannot
    be read at all yields nothing. A warning blocks neither accept nor affirm."""
    try:
        return [
            Warning(field=name, group=hit.group, pattern=hit.pattern, matched=hit.match)
            for name, text in prose_fields(record)
            for hit in check(text)
        ]
    except Exception:  # noqa: BLE001 - the review screen must always render
        return []
