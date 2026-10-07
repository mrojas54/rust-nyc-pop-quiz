"""Reject exact and normalized duplicates; queue near-duplicates (AC-14 to AC-18, SPEC.md 7.3).

Three checks, in this order, against the history of every question in the bank:

1. **Exact** (AC-14) - the SHA-256 of the source bytes, `bank.source_hash`. A
   byte-identical resubmission is rejected.
2. **Normalized** (AC-15) - the same program with its bindings renamed, or
   reformatted, is rejected.
3. **Near** (AC-16) - Jaccard similarity over the normalized token bigrams at or
   above a configured threshold sends the candidate to the review queue, marked
   `review.near_duplicate_of`. It is neither accepted nor dropped: an organizer
   decides.

Everything that passes the first two is admitted to the bank with `review.status`
unset. That is what this module takes the review queue to be - the contract names
the queue but no place for it, and the migrated questions already wait there - and
the review surface (T-18) reads it from the bank.

**The normalized check is a token-level approximation of the AST rule. That is a
deviation from SPEC 7.3, and here is its extent.** The contract says "normalized
AST (alpha-renamed bindings, formatted)". The standard library has no Rust parser
and `pyproject.toml` is not this ticket's to change, so this module lexes Rust with
a small hand-written lexer and normalizes the token stream instead:

* comments, whitespace and line breaks are dropped, and adjacent punctuation is
  glued back into Rust's compound operators, so `a == b` stays apart from
  `a = = b`;
* the optional commas `rustfmt` adds and removes are dropped, and only where the
  program means the same without them (`_dropped_comma` lists the places);
* names **the program itself declares** - `let` and pattern bindings,
  parameters, closure parameters, functions, types, fields, variants, generics,
  lifetimes, `macro_rules!` names - are renamed to a numbered placeholder in the
  order it first appears, including inline format arguments like `{v:?}`. The
  entry point, names used outside the scope that declares them, and names in
  sources with derives stay;
* every other name is kept exactly: keywords, every standard-library type,
  function, method and macro the program uses without declaring, and declared
  members on the library keep-list (`fn len`). That list is incomplete.

The rule behind all of it: **no normalization may make two different programs
equal.** A normalized duplicate is *rejected*, with no person in the loop, and in
this quiz a program that does not compile is as good a question as one that does
- so a rule that equated `P { a: Q {}, b: 2 }` with the same line missing its
comma would throw away a question. Documentation comments outside recognized
item positions are kept as markers, including comments before parameters.

**A declared name is renamed only where it is in scope.** Every declaration
reaches a token range no wider than its Rust scope: parameters and generics their
function, closure parameters their closure, `let` / `if let` / `while let` / `for` /
match-arm bindings their block or arm, items their module or block (and, inside an
`impl` or `trait`, only their own name). A spelling with any bare use outside every
range of its declarations - `fn f(drop: i32) {}` beside a call to the library's
`drop(1)` - is kept verbatim everywhere, as is a spelling that names a field no
struct in the program declares (`Range { start, end }`). The aim is ranges no
wider than Rust's, so the error is a missed rename rather than a merge. **That aim
is not met yet.** The seven false matches review round 3 found are closed (PQ-30):
a closure in an `if let` head ends at its block, items stop at nested `mod`
bodies, a `macro_rules!` name reaches only its textual scope, a comma-less arm
body is not the next arm's pattern, a path's tail is renamed only where tokens
show it is the program's, a lifetime keeps its quote (`'$1`), and only `impl` and
`trait` functions are reached after a `.`. One known risk remains: a declared
associated item named like a library one off the `_STD_MEMBERS` list - `type Item`
in an `impl Iterator`, or `S::name` where another type declares `name` and `S`
takes the library's - is renamed with it. The rule above is a requirement, not a
property this approximation has established.

What the approximation misses, and what happens instead: statements or items in a
different order, operands swapped around a commutative operator, an expression
rewritten into an equivalent one (`x + x` for `2 * x`), a struct built with field
shorthand in one program and `field: binding` in the other, a trailing comma in
a tuple, a declared member renamed to or from a library name, names preserved
for names used out of scope or derives, and anything a `macro_rules!` body does.
Where tokens cannot tell, the scoping above keeps a name rather than rename it:
a closure in an `if`/`while` head cut short at the block, a `#[macro_use]`
module's macro used after the module, a struct pattern after an or-pattern's `|`,
an item used inside a nested module through `use super::*`, and every spelling
that a kept path tail shares (`T::default()` keeps a program's own `default`).
Each of those is two token streams, so the check says "not a normalized
duplicate" and the near-duplicate check - which sees them as very similar - sends
the pair to an organizer. A miss costs a person a look; a false
match would cost a question silently. The approximation is built to fail in the
first direction.

The near-duplicate bigrams abstract every renamed name to the same placeholder,
where the fingerprint keeps them numbered. Numbered placeholders shift by one the
moment a single statement is inserted near the top, and that would collapse the
similarity of exactly the near-duplicates AC-16 is for.

**The threshold is 0.6 and it is uncalibrated** (D-13). It is a starting guess,
configurable per call and on the command line. Every verdict reports the closest
question and its similarity, so the first real batches are the data that calibrates
it.

**The wording** (AC-18). The one sentence this module says about a candidate that
passed is `UNIQUENESS_STATEMENT`, exactly, and it never claims more than that.
Nothing here says a question has not been asked; it says what was checked and not
found.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import re
import sys
from collections.abc import Callable, Iterable, Sequence
from dataclasses import dataclass
from itertools import pairwise
from pathlib import Path
from typing import Literal, TypeGuard

from popquiz.bank import (
    QUESTIONS_DIR,
    BankError,
    History,
    Question,
    Review,
    append_question,
    history_path,
    load_bank,
    load_history,
    question_from_dict,
    save_history,
    source_hash,
)

# AC-18, SPEC 11: the only words this module uses to say a candidate passed.
UNIQUENESS_STATEMENT = "no exact or normalized duplicate found"

# D-13: a hypothesis, not a measurement. See the module docstring.
NEAR_DUPLICATE_THRESHOLD = 0.6


class DedupeError(Exception):
    """A candidate cannot be checked, or a run was asked to do something unsafe."""


# --------------------------------------------------------------------------- #
# The lexer
# --------------------------------------------------------------------------- #

# The 2021 edition's strict and reserved keywords. `_` is here too: it is the
# wildcard, never a name, so it is never renamed. `union` and `macro_rules` are
# contextual and handled where they matter; `'static` is a lifetime.
KEYWORDS = frozenset(
    [
        "as",
        "break",
        "const",
        "continue",
        "crate",
        "else",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "Self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
        "async",
        "await",
        "dyn",
        "abstract",
        "become",
        "box",
        "do",
        "final",
        "macro",
        "override",
        "priv",
        "typeof",
        "unsized",
        "virtual",
        "yield",
        "try",
        "_",
    ]
)

TokenKind = Literal["ident", "keyword", "lifetime", "char", "string", "number", "punct"]


@dataclass(frozen=True)
class Token:
    """One lexeme. `joint` is whether the next token starts where this one ends.

    Jointness is how Rust tells `==` from `= =`, and it is read for exactly that:
    to recognise `::`, `=>`, `->`, `..` and `||` while reading structure, and to
    glue compound operators back together in the normalized stream. Spacing
    anywhere else never reaches the fingerprint.
    """

    kind: TokenKind
    text: str
    joint: bool = False


_IDENT_START = re.compile(r"[^\W\d]", re.UNICODE)
_IDENT_REST = re.compile(r"\w*", re.UNICODE)
_RAW_STRING = re.compile(r'(?:b|c)?r(#*)"')
_PREFIXED_STRING = re.compile(r'(?:b|c)"')
_RAW_IDENT = re.compile(r"r#([^\W\d]\w*)", re.UNICODE)
_DIGITS = re.compile(r"[0-9_]*")
_RADIX = re.compile(r"0[xob][0-9a-fA-F_]*")
_EXPONENT = re.compile(r"[eE][+-]?[0-9_]*[0-9][0-9_]*")
_SUFFIX = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def _skip_block_comment(source: str, i: int) -> int:
    depth = 0
    n = len(source)
    while i < n:
        if source.startswith("/*", i):
            depth += 1
            i += 2
        elif source.startswith("*/", i):
            depth -= 1
            i += 2
            if depth == 0:
                return i
        else:
            i += 1
    raise DedupeError("unterminated block comment")


def _end_of_quoted(source: str, i: int) -> int:
    """`source[i]` is an opening `"`; returns the index just past the closing one."""
    j = i + 1
    n = len(source)
    while j < n:
        c = source[j]
        if c == "\\":
            j += 2
        elif c == '"':
            return j + 1
        else:
            j += 1
    raise DedupeError("unterminated string literal")


def _end_of_char(source: str, i: int) -> int | None:
    """`source[i]` is `'`. The index past a char literal's closing `'`, or `None`
    when this `'` starts a lifetime or label instead."""
    n = len(source)
    if i + 1 >= n:
        raise DedupeError("a lone ' at the end of the source")
    if source[i + 1] == "\\":
        j = i + 2
        if source.startswith("u{", j):
            close = source.find("}", j)
            if close < 0:
                raise DedupeError("unterminated unicode escape")
            j = close + 1
        elif source.startswith("x", j):
            j += 3
        else:
            j += 1
        if j < n and source[j] == "'":
            return j + 1
        raise DedupeError("unterminated char literal")
    if i + 2 < n and source[i + 2] == "'":
        return i + 3
    return None


def tokenize(source: str) -> list[Token]:
    """Rust source to tokens. Comments and whitespace go; everything else stays.

    Raises `DedupeError` for an unterminated literal or comment: a candidate that
    cannot be lexed is reported, never fingerprinted as something it is not.
    """
    return _lex(source)[0]


def _doc_comment(source: str, i: int) -> str | None:
    """`///` for an outer doc comment starting at `i`, `//!` for an inner one,
    `None` for a plain comment. `////`, `/**/` and `/***` are plain, as in Rust."""
    if source.startswith("//!", i) or source.startswith("/*!", i):
        return "//!"
    if source.startswith("///", i) and not source.startswith("////", i):
        return "///"
    if source.startswith("/**", i) and not (
        source.startswith("/**/", i) or source.startswith("/***", i)
    ):
        return "///"
    return None


def _lex(source: str) -> tuple[list[Token], list[tuple[int, str]]]:
    """The tokens, and where each doc comment sat: `(index of the token it
    precedes, "///" or "//!")`. Normalization needs the second list because a doc
    comment is not always ignorable - see `_kept_doc_comments`."""
    spans: list[tuple[TokenKind, str, int, int]] = []
    docs: list[tuple[int, str]] = []
    i = 0
    n = len(source)

    def emit(kind: TokenKind, start: int, end: int) -> None:
        spans.append((kind, source[start:end], start, end))

    while i < n:
        c = source[i]
        if c.isspace():
            i += 1
            continue
        if source.startswith("//", i) or source.startswith("/*", i):
            doc = _doc_comment(source, i)
            if doc is not None:
                docs.append((len(spans), doc))
            if source.startswith("//", i):
                newline = source.find("\n", i)
                i = n if newline < 0 else newline
            else:
                i = _skip_block_comment(source, i)
            continue

        raw = _RAW_STRING.match(source, i)
        if raw:
            closing = '"' + raw.group(1)
            end = source.find(closing, raw.end())
            if end < 0:
                raise DedupeError("unterminated raw string literal")
            emit("string", i, end + len(closing))
            i = end + len(closing)
            continue
        if _PREFIXED_STRING.match(source, i):
            end = _end_of_quoted(source, i + 1)
            emit("string", i, end)
            i = end
            continue
        if source.startswith("b'", i):
            char_end = _end_of_char(source, i + 1)
            if char_end is None:
                raise DedupeError("malformed byte literal")
            emit("char", i, char_end)
            i = char_end
            continue
        raw_ident = _RAW_IDENT.match(source, i)
        if raw_ident:
            emit("ident", i, raw_ident.end())
            i = raw_ident.end()
            continue
        if c == '"':
            end = _end_of_quoted(source, i)
            emit("string", i, end)
            i = end
            continue
        if c == "'":
            char_end = _end_of_char(source, i)
            if char_end is not None:
                emit("char", i, char_end)
                i = char_end
                continue
            # `_IDENT_START` admits `_`, so `'_` is covered here too.
            if _IDENT_START.match(source, i + 1):
                rest = _IDENT_REST.match(source, i + 2)
                assert rest is not None
                end = rest.end()
                emit("lifetime", i, end)
                i = end
                continue
            raise DedupeError(f"cannot lex the ' at offset {i}")
        if c.isdigit():
            end = _number_end(source, i)
            emit("number", i, end)
            i = end
            continue
        if _IDENT_START.match(c):
            rest = _IDENT_REST.match(source, i + 1)
            assert rest is not None  # The pattern accepts the empty string.
            end = rest.end()
            word = source[i:end]
            emit("keyword" if word in KEYWORDS else "ident", i, end)
            i = end
            continue
        emit("punct", i, i + 1)
        i += 1

    tokens = []
    for k, (kind, text, _start, end) in enumerate(spans):
        joint = k + 1 < len(spans) and spans[k + 1][2] == end
        tokens.append(Token(kind, text, joint))
    return tokens, docs


def _number_end(source: str, i: int) -> int:
    radix = _RADIX.match(source, i)
    if radix:
        j = radix.end()
    else:
        digits = _DIGITS.match(source, i)
        assert digits is not None  # The pattern accepts the empty string.
        j = digits.end()
        # A fractional part, but not a range (`1..2`), a method (`1.max(2)`) or a
        # tuple index chain. `1.` on its own is a float, as in Rust.
        if j < len(source) and source[j] == "." and not source.startswith("..", j):
            after = source[j + 1] if j + 1 < len(source) else ""
            if after.isdigit():
                fraction = _DIGITS.match(source, j + 1)
                assert fraction is not None
                j = fraction.end()
            elif not (after == "_" or _IDENT_START.match(after or "0")):
                j += 1
        exponent = _EXPONENT.match(source, j)
        if exponent:
            j = exponent.end()
    suffix = _SUFFIX.match(source, j)
    return suffix.end() if suffix else j


# --------------------------------------------------------------------------- #
# Reading structure off the token stream
# --------------------------------------------------------------------------- #

_OPENERS = {"(": ")", "[": "]", "{": "}"}
_CLOSERS = frozenset(_OPENERS.values())


def _is(token: Token | None, text: str) -> TypeGuard[Token]:
    return token is not None and token.kind == "punct" and token.text == text


def _kw(token: Token | None, word: str) -> bool:
    return token is not None and token.kind == "keyword" and token.text == word


class _Stream:
    """A token list plus the facts about it every pass needs: bracket partners,
    which colons are halves of `::`, and which `=` starts a `=>`."""

    def __init__(self, tokens: list[Token]) -> None:
        self.tokens = tokens
        self.n = len(tokens)
        self.partner: dict[int, int] = {}
        stack: list[int] = []
        for k, t in enumerate(tokens):
            if t.kind != "punct":
                continue
            if t.text in _OPENERS:
                stack.append(k)
            elif t.text in _CLOSERS:
                # Tolerant of imbalance: an unmatched closer pairs with nothing.
                while stack and _OPENERS[tokens[stack[-1]].text] != t.text:
                    stack.pop()
                if stack:
                    opener = stack.pop()
                    self.partner[opener] = k
                    self.partner[k] = opener
        self.path_sep = [False] * self.n
        for k in range(self.n - 1):
            if _is(tokens[k], ":") and tokens[k].joint and _is(tokens[k + 1], ":"):
                self.path_sep[k] = self.path_sep[k + 1] = True

    def at(self, k: int) -> Token | None:
        return self.tokens[k] if 0 <= k < self.n else None

    def colon(self, k: int) -> bool:
        """A lone `:` - a type annotation or a field - and not half of a `::`."""
        return _is(self.at(k), ":") and not self.path_sep[k]

    def fat_arrow(self, k: int) -> bool:
        t = self.at(k)
        return _is(t, "=") and t.joint and _is(self.at(k + 1), ">")

    def range_dot(self, k: int) -> bool:
        """Whether the `.` at `k` is part of `..`/`..=`, not a field access."""
        before, here, after = self.at(k - 1), self.at(k), self.at(k + 1)
        return (_is(before, ".") and before.joint) or (
            _is(after, ".") and here is not None and here.joint
        )

    def find(self, start: int, end: int, stop: Callable[[int], bool]) -> int:
        """The first index in `[start, end)` at this bracket depth where `stop`
        holds, skipping whole bracket groups. Stops at a closer that leaves the
        group; returns `end` when nothing matches."""
        k = start
        while k < end:
            if stop(k):
                return k
            t = self.tokens[k]
            if t.kind == "punct":
                if t.text in _OPENERS and k in self.partner:
                    k = self.partner[k] + 1
                    continue
                if t.text in _CLOSERS:
                    return k
            k += 1
        return end

    def split(self, start: int, end: int) -> list[tuple[int, int]]:
        """`[start, end)` cut at its depth-0 commas."""
        parts = []
        a = start
        while a <= end:
            b = self.find(a, end, lambda k: _is(self.tokens[k], ","))
            parts.append((a, b))
            if b >= end:
                break
            a = b + 1
        return parts


def _pattern_names(s: _Stream, a: int, b: int) -> set[str]:
    """The names a pattern in `[a, b)` binds.

    An identifier binds when it starts lowercase or with `_`, is not a path segment
    or a tuple-struct, struct or macro name (followed by `(`, `{`, `::` or `!`),
    and is not a field name inside braces (`Point { x: px }` binds `px`). Rust's
    own rule is close to this: an uppercase name in a pattern is a constant, a unit
    struct or a variant.
    """
    names: set[str] = set()
    groups: list[str] = []
    for k in range(a, b):
        t = s.tokens[k]
        if t.kind == "punct":
            if t.text in _OPENERS:
                groups.append(t.text)
            elif t.text in _CLOSERS and groups:
                groups.pop()
            continue
        if t.kind != "ident":
            continue
        bare = t.text.removeprefix("r#")
        if not (bare[0] == "_" or bare[0].islower()):
            continue
        after = s.at(k + 1)
        if (
            after is not None
            and after.kind == "punct"
            and after.text in ("(", "{", "!")
        ):
            continue
        if k + 1 < s.n and s.path_sep[k + 1]:
            continue
        if k > 0 and (s.path_sep[k - 1] or _is(s.at(k - 1), ".")):
            continue
        if groups and groups[-1] == "{" and s.colon(k + 1):
            continue
        names.add(t.text)
    return names


def _cut_at_colon(s: _Stream, a: int, b: int) -> int:
    """Where a parameter's pattern ends: its depth-0 `:`, or the end."""
    return s.find(a, b, s.colon)


def _generic_names(s: _Stream, lt: int, names: set[str]) -> int:
    """Records the type and const parameters of the `<...>` opening at `lt`;
    returns the index of its closing `>`. Lifetimes are renamed wholesale, so they
    need no recording. The `>` of a `->` inside a bound is not a closer."""
    depth = 0
    k = lt
    entry_start = True
    while k < s.n:
        t = s.tokens[k]
        if t.kind == "punct":
            if t.text == "<":
                depth += 1
                if depth == 1:
                    entry_start = True
                    k += 1
                    continue
            elif t.text == ">" and not (
                _is(s.at(k - 1), "-") and s.tokens[k - 1].joint
            ):
                depth -= 1
                if depth == 0:
                    return k
            elif t.text == "," and depth == 1:
                entry_start = True
                k += 1
                continue
            elif t.text in _OPENERS and k in s.partner:
                k = s.partner[k] + 1
                entry_start = False
                continue
        if entry_start and depth == 1:
            if (
                _kw(t, "const")
                and s.at(k + 1) is not None
                and s.tokens[k + 1].kind == "ident"
            ):
                names.add(s.tokens[k + 1].text)
            elif t.kind == "ident":
                names.add(t.text)
        entry_start = False
        k += 1
    return s.n - 1


def _skip_attributes_and_visibility(s: _Stream, a: int, b: int) -> int:
    """Past any `#[...]` attributes and a `pub` / `pub(...)` at the start of an entry."""
    while a < b:
        if _is(s.at(a), "#") and _is(s.at(a + 1), "[") and (a + 1) in s.partner:
            a = s.partner[a + 1] + 1
        elif _kw(s.at(a), "pub"):
            a += 1
            if _is(s.at(a), "(") and a in s.partner:
                a = s.partner[a] + 1
        else:
            break
    return a


def _field_names(s: _Stream, brace: int) -> set[str]:
    fields = set()
    for a, b in s.split(brace + 1, s.partner[brace]):
        a = _skip_attributes_and_visibility(s, a, b)
        t = s.at(a)
        if a < b and t is not None and t.kind == "ident" and s.colon(a + 1):
            fields.add(t.text)
    return fields


def _closure_opens(s: _Stream, k: int) -> bool:
    """Whether the `|` at `k` opens a closure's parameter list rather than being a
    bitwise or, a logical or, or an or-pattern. Decided by what comes before it: a
    closure starts an expression, so it follows an opener, a separator, `=`, `=>`,
    `:` or one of a few keywords."""
    before = s.at(k - 1)
    if before is None:
        return True
    if before.kind == "keyword":
        return before.text in ("move", "return", "async", "in", "break")
    if before.kind != "punct":
        return False
    if before.text in ("{", ","):
        # Unless it is the leading `|` of an or-pattern in a match arm,
        # `{ | A | B => .. }`, which a `=>` before the next separator gives away.
        end = s.find(
            k + 1,
            s.n,
            lambda j: s.fat_arrow(j) or _is(s.tokens[j], ",") or _is(s.tokens[j], ";"),
        )
        return not s.fat_arrow(end)
    if before.text in ("(", "[", ";", "="):
        return True
    if before.text == ":":
        return s.colon(k - 1)
    if before.text == ">":
        return s.fat_arrow(k - 2)
    return False


def _in_condition_head(s: _Stream, k: int) -> bool:
    """Whether the token at `k` stands in the head of an `if`, `while`, `match` or
    `for`, at the head's own depth: there the next `{` opens the block, and no
    expression in the head runs past it."""
    j = k - 1
    while j >= 0:
        t = s.tokens[j]
        if t.kind == "keyword" and t.text in ("if", "while", "match", "in"):
            return True
        if t.kind == "punct":
            if t.text in _CLOSERS and j in s.partner:
                j = s.partner[j] - 1
                continue
            if t.text in _OPENERS or t.text in (";", "}", ",") or (
                t.text == ">" and s.fat_arrow(j - 1)
            ):
                return False
        j -= 1
    return False


def _pattern_path_before(s: _Stream, brace: int) -> bool:
    """Whether the `{` at `brace` is named by a path that begins a pattern."""
    head = brace - 1
    t = s.at(head)
    if t is None or not (t.kind == "ident" or _kw(t, "Self")):
        return False
    while (
        head >= 3
        and s.path_sep[head - 1]
        and s.path_sep[head - 2]
        and s.tokens[head - 3].kind in ("ident", "keyword")
    ):
        head -= 3
    before = s.at(head - 1)
    if before is None:
        return True
    if before.kind != "punct":
        return False
    if before.text == ">":
        return s.fat_arrow(head - 2)
    return before.text in ("{", ",", "}", "@")


def _arm_start(s: _Stream, arrow: int) -> int:
    """Where the match arm ending at the `=>` at `arrow` begins.

    Walks back over whole bracket groups to the arm's boundary: the match body's
    `{`, a `,`, a previous `=>`, or the `}` that ends a previous arm's block. A
    `{...}` group is part of the pattern only when a path names it
    (`Point { x, .. }`) and that path itself starts the pattern: after the match's
    `{`, a `,`, a `=>`, a `}` or an `@`. Anything else before the path - `==` in
    `while a == b {..}`, the `.` of `match s.a {..}`, the `in` of a `for` - makes
    the group a previous arm's comma-less body. An or-pattern's `|` is not taken
    as a start (`a | b {..}` may be a condition), so `A | P { x }` stops early and
    leaves its names unbound: kept, not renamed.
    """
    j = arrow - 1
    while j >= 0:
        t = s.tokens[j]
        if t.kind == "punct":
            if t.text in _CLOSERS:
                opener = s.partner.get(j)
                if opener is None:
                    return j + 1
                if t.text == "}" and not _pattern_path_before(s, opener):
                    return j + 1
                j = opener - 1
                continue
            if t.text in _OPENERS or t.text == ",":
                return j + 1
            if t.text == ">" and s.fat_arrow(j - 1):
                return j + 1
        j -= 1
    return 0


def _macro_rules_bodies(s: _Stream) -> set[int]:
    """Indexes inside `macro_rules!` bodies. Their `=>` are matcher arms, not match
    arms, and their `$x:expr` are fragments, not bindings: they are left alone."""
    inside: set[int] = set()
    for k, t in enumerate(s.tokens):
        if t.kind == "ident" and t.text == "macro_rules" and _is(s.at(k + 1), "!"):
            body = k + 3
            if (
                s.at(k + 2) is not None
                and s.tokens[k + 2].kind == "ident"
                and body in s.partner
            ):
                inside.update(range(body, s.partner[body] + 1))
    return inside


@dataclass(frozen=True)
class _Declared:
    """The names the program itself declares, and where each one reaches.

    `names` are renamed wherever they stand on their own; `members` - fields and
    `impl` or `trait` functions - are also renamed after a `.`; `macros` are
    renamed before `!(`.
    `fields` are the declared field names. `scopes` holds, per spelling, the token
    ranges (inclusive) where a bare use of it may mean one of its declarations;
    `fields_at` marks the tokens that name a field, true where the name is also a
    binding or a use (`P { x }`). `macro_scopes` is where a `name!(..)` call may
    mean the program's `macro_rules! name`: from the definition to the end of the
    block or file around it, the textual scope Rust gives it.

    What a path's tail may reach: `generics` are the type and const parameters'
    spellings, `associated` the names declared inside an `impl` or `trait` and the
    enum variants, `item_braces` the `{` directly around each other item's
    declaration (`None` at top level), and `module_bodies` the `{` of each inline
    module, by its name. `library_impl_items` are the names declared inside an
    `impl` of a trait the program does not declare, which the library fixes.
    `assoc_owners` maps each associated name to the spellings of the types and
    traits that declare it. `impl_owner` maps the `{` of each `impl` or `trait` to
    its owner's spelling, so `Self` can be followed to the type around it.
    """

    names: frozenset[str]
    members: frozenset[str]
    macros: frozenset[str]
    fields: frozenset[str]
    scopes: dict[str, list[tuple[int, int]]]
    fields_at: dict[int, bool]
    macro_scopes: dict[str, list[tuple[int, int]]]
    generics: frozenset[str]
    associated: frozenset[str]
    item_braces: dict[str, set[int | None]]
    module_bodies: dict[str, set[int]]
    library_impl_items: frozenset[str]
    assoc_owners: dict[str, set[str]]
    impl_owner: dict[int, str]
    enclosing: list[int | None]


# Words that, in the tokens before a `{`, make it a block or an item body rather
# than a struct expression. Rust itself forbids a struct literal in the head of an
# `if`, `while`, `match` or `for` for this reason.
_BLOCK_WORDS = frozenset(
    [
        "if",
        "while",
        "match",
        "for",
        "loop",
        "else",
        "fn",
        "impl",
        "trait",
        "struct",
        "enum",
        "union",
        "mod",
        "unsafe",
        "async",
        "extern",
    ]
)


def _enclosing_braces(s: _Stream) -> list[int | None]:
    """For every token, the `{` of the innermost brace group around it."""
    enclosing: list[int | None] = [None] * s.n
    stack: list[int] = []
    for k, t in enumerate(s.tokens):
        enclosing[k] = stack[-1] if stack else None
        if _is(t, "{") and k in s.partner:
            stack.append(k)
        elif _is(t, "}") and stack and s.partner.get(k) == stack[-1]:
            stack.pop()
    return enclosing


def _header_words(s: _Stream, brace: int) -> set[str]:
    """The keywords and names between the `{` at `brace` and the start of its
    statement or item: back to a `;`, a `}` or an opener, over `(...)` and `[...]`."""
    words: set[str] = set()
    j = brace - 1
    while j >= 0:
        t = s.tokens[j]
        if t.kind == "punct":
            if t.text in (")", "]") and j in s.partner:
                j = s.partner[j] - 1
                continue
            if t.text in _OPENERS or t.text in (";", "}", ")", "]"):
                break
        elif t.kind in ("keyword", "ident"):
            words.add(t.text)
        j -= 1
    return words


def _header_keyword(s: _Stream, brace: int, word: str) -> int | None:
    """The index of the first `word` keyword in the header of the `{` at `brace`
    (back to the start of its statement or item), or `None`."""
    found = None
    j = brace - 1
    while j >= 0:
        t = s.tokens[j]
        if t.kind == "punct":
            if t.text in (")", "]") and j in s.partner:
                j = s.partner[j] - 1
                continue
            if t.text in _OPENERS or t.text in (";", "}", ")", "]"):
                break
        elif _kw(t, word):
            found = j
        j -= 1
    return found


def _impl_owner(s: _Stream, brace: int) -> str | None:
    """The type an `impl` (or the name of a `trait`) owns the items in the `{` at
    `brace`: for `impl<..> Trait for Type<..> where .. {` the last identifier of
    `Type`'s path, for `trait NAME {` NAME. `None` when tokens do not show it, and
    for any other brace."""
    trait = _header_keyword(s, brace, "trait")
    if trait is not None:
        name = s.at(trait + 1)
        return name.text if name is not None and name.kind == "ident" else None
    impl = _header_keyword(s, brace, "impl")
    if impl is None:
        return None
    k = impl + 1
    if _is(s.at(k), "<"):
        k = _generic_names(s, k, set()) + 1
    depth = 0
    owner: str | None = None
    while k < brace:
        t = s.tokens[k]
        if t.kind == "punct":
            if t.text == "<":
                depth += 1
            elif t.text == ">" and not (
                _is(s.at(k - 1), "-") and s.tokens[k - 1].joint
            ):
                depth -= 1
            elif t.text in ("(", "[") and k in s.partner:
                k = s.partner[k]
        elif depth == 0:
            if _kw(t, "where"):
                break
            if _kw(t, "for"):
                owner = None
            elif t.kind == "ident":
                owner = t.text
        k += 1
    return owner


def _impl_trait(s: _Stream, brace: int) -> str | None:
    """The trait an `impl Trait for Type {` names, by the last identifier of its
    path, for the `{` at `brace`; `None` for an inherent `impl` or any other brace."""
    impl = _header_keyword(s, brace, "impl")
    if impl is None:
        return None
    k = impl + 1
    if _is(s.at(k), "<"):
        k = _generic_names(s, k, set()) + 1
    depth = 0
    last: str | None = None
    while k < brace:
        t = s.tokens[k]
        if t.kind == "punct":
            if t.text == "<":
                depth += 1
            elif t.text == ">" and not (
                _is(s.at(k - 1), "-") and s.tokens[k - 1].joint
            ):
                depth -= 1
            elif t.text in ("(", "[") and k in s.partner:
                k = s.partner[k]
        elif depth == 0:
            if _kw(t, "for"):
                return last
            if t.kind == "ident":
                last = t.text
        k += 1
    return None


class _Collector:
    """Gathers declarations, and for each the range where its bare name reaches.

    Token-level scoping, not name resolution. Every range is at most the Rust
    scope: a function's parameters and generics its signature and body; a
    closure's parameters the closure; a `let` its pattern and the rest of its
    block after the statement; an `if let` or `while let` its pattern and block; a
    `for` its pattern and body; a match arm its pattern, guard and body; an item its
    module or block, or - inside an `impl` or `trait` - only its own name, because
    a method or associated item is never reached by its bare name. A name used
    outside every range of its declarations is kept (`_unscoped_names`). The ranges
    are meant to be drawn smaller where tokens cannot tell; the module docstring
    lists the known places where they are still drawn wider.
    """

    def __init__(self, s: _Stream) -> None:
        self.s = s
        self.names: set[str] = set()
        self.members: set[str] = set()
        self.macros: set[str] = set()
        self.fields: set[str] = set()
        self.scopes: dict[str, list[tuple[int, int]]] = {}
        self.macro_scopes: dict[str, list[tuple[int, int]]] = {}
        self.generic_names: set[str] = set()
        self.associated: set[str] = set()
        self.assoc_owners: dict[str, set[str]] = {}
        self.impl_owner: dict[int, str] = {}
        # Names declared inside an `impl` of a trait the program does not declare.
        self.library_impl_items: set[str] = set()
        self.user_traits: set[str] = set()
        for k, t in enumerate(s.tokens):
            after = s.at(k + 1)
            if _kw(t, "trait") and after is not None and after.kind == "ident":
                self.user_traits.add(after.text)
        self.item_braces: dict[str, set[int | None]] = {}
        self.patterns: list[tuple[int, int]] = []
        self.bodies: set[int] = set()
        self.enclosing = _enclosing_braces(s)
        # The `{` of every inline `mod NAME { .. }`, by name. A module is its own
        # namespace: the items around it do not reach inside by their bare names.
        self.module_bodies: dict[str, set[int]] = {}
        for k, t in enumerate(s.tokens):
            name = s.at(k + 1)
            if (
                _kw(t, "mod")
                and name is not None
                and name.kind == "ident"
                and _is(s.at(k + 2), "{")
                and (k + 2) in s.partner
            ):
                self.module_bodies.setdefault(name.text, set()).add(k + 2)

    def scope(self, names: Iterable[str], lo: int, hi: int) -> None:
        for name in names:
            self.scopes.setdefault(name, []).append((lo, hi))

    def bind(self, a: int, b: int) -> set[str]:
        """The names the pattern in `[a, b)` binds, recorded as declared."""
        self.patterns.append((a, b))
        bound = _pattern_names(self.s, a, b)
        self.names |= bound
        return bound

    def item_end(self, start: int) -> int:
        """Where the item or closure whose header continues at `start` ends: the
        `}` of its body, or its `;`."""
        s = self.s
        end = s.find(
            start, s.n, lambda x: _is(s.tokens[x], "{") or _is(s.tokens[x], ";")
        )
        if _is(s.at(end), "{") and end in s.partner:
            self.bodies.add(end)
            return s.partner[end]
        return min(end, s.n - 1)

    def item(self, d: int) -> bool:
        """The item named at `d`: reached throughout its module or block, and by
        its bare name nowhere else. True when it is an associated item - inside an
        `impl` or `trait` - which only a path or a `.` reaches."""
        s = self.s
        name = s.tokens[d].text
        self.names.add(name)
        brace = self.enclosing[d]
        if brace is not None and _header_words(s, brace) & {"impl", "trait"}:
            self.scope([name], d, d)
            self.associated.add(name)
            owner = _impl_owner(s, brace)
            if owner is not None:
                self.assoc_owners.setdefault(name, set()).add(owner)
                self.impl_owner[brace] = owner
            trait = _impl_trait(s, brace)
            if trait is not None and trait not in self.user_traits:
                self.library_impl_items.add(name)
            return True
        self.item_braces.setdefault(name, set()).add(brace)
        if brace is None:
            self.scope_outside_modules(name, d, 0, s.n - 1)
        else:
            self.scope_outside_modules(name, d, brace, s.partner[brace])
        return False

    def scope_outside_modules(self, name: str, d: int, lo: int, hi: int) -> None:
        """`name`, declared at `d`, reaches `[lo, hi]` except the bodies of the
        modules nested in it: inside one, the bare name means that module's own
        item or the library's, never this one."""
        holes = sorted(
            (b, self.s.partner[b])
            for bodies in self.module_bodies.values()
            for b in bodies
            if lo < b and self.s.partner[b] <= hi and not b <= d <= self.s.partner[b]
        )
        a = lo
        for b, e in holes:
            if b < a:
                continue  # inside a hole already cut
            if a <= b - 1:
                self.scope([name], a, b - 1)
            a = e + 1
        if a <= hi:
            self.scope([name], a, hi)

    def generics(self, lt: int) -> tuple[int, set[str]]:
        """The type and const parameters of the `<...>` at `lt`, and its `>`."""
        found: set[str] = set()
        gt = _generic_names(self.s, lt, found)
        self.names |= found
        self.generic_names |= found
        return gt, found

    def data_type(self, k: int, kind: str) -> None:
        """A `struct`, `enum` or `union` named at `k`: its generics, fields and
        variants."""
        s = self.s
        self.item(k)
        j = k + 1
        found: set[str] = set()
        if _is(s.at(j), "<"):
            gt, found = self.generics(j)
            j = gt + 1
        end = self.item_end(j)
        self.scope(found, k, end)
        if not (_is(s.at(end), "}") and end in s.partner):
            return  # no body, or an unmatched `}` in malformed source
        body = s.partner[end]
        if kind != "enum":
            declared = _field_names(s, body)
            self.members |= declared
            self.fields |= declared
            return
        for a, b in s.split(body + 1, end):
            a = _skip_attributes_and_visibility(s, a, b)
            t = s.at(a)
            if a < b and t is not None and t.kind == "ident":
                self.names.add(t.text)
                self.associated.add(t.text)
                self.assoc_owners.setdefault(t.text, set()).add(s.tokens[k].text)
                self.scope([t.text], a, a)
                if _is(s.at(a + 1), "{") and (a + 1) in s.partner:
                    declared = _field_names(s, a + 1)
                    self.members |= declared
                    self.fields |= declared

    def function(self, k: int) -> None:
        """A `fn` named at `k + 1`: its parameters and generics reach its body.
        Only a method or associated function is a member: a `.name()` call never
        reaches a free `fn name`, and may well be the library's method."""
        s = self.s
        if self.item(k + 1):
            self.members.add(s.tokens[k + 1].text)
        j = k + 2
        found: set[str] = set()
        if _is(s.at(j), "<"):
            gt, found = self.generics(j)
            j = gt + 1
        if not (_is(s.at(j), "(") and j in s.partner):
            self.scope(found, k + 2, j)
            return
        close = s.partner[j]
        params: set[str] = set()
        for a, b in s.split(j + 1, close):
            params |= self.bind(a, _cut_at_colon(s, a, b))
        end = self.item_end(close + 1)
        self.scope(params, j, end)
        self.scope(found, k + 2, end)

    def let(self, k: int) -> None:
        s = self.s
        tokens = s.tokens
        pattern_end = s.find(
            k + 1,
            s.n,
            lambda j: (
                _is(tokens[j], "=")
                or _is(tokens[j], ";")
                or s.colon(j)
                or _kw(tokens[j], "else")
            ),
        )
        bound = self.bind(k + 1, pattern_end)
        self.scope(bound, k + 1, pattern_end - 1)
        before = s.at(k - 1)
        conditional = (
            _kw(before, "if")
            or _kw(before, "while")
            or (_is(before, "&") and _is(s.at(k - 2), "&") and s.tokens[k - 2].joint)
        )
        if conditional:
            # `if let` / `while let`: the block after the condition, not the
            # initializer and not an `else`.
            brace = s.find(pattern_end, s.n, lambda j: _is(tokens[j], "{"))
            if _is(s.at(brace), "{") and brace in s.partner:
                self.scope(bound, brace, s.partner[brace])
            return
        # A `let` statement: the rest of its block, after its own `;`, so never
        # its own initializer or `else`.
        block = self.enclosing[k]
        block_end = s.partner[block] if block is not None else s.n
        statement_end = s.find(k + 1, block_end, lambda j: _is(tokens[j], ";"))
        if statement_end + 1 <= block_end - 1:
            self.scope(bound, statement_end + 1, block_end - 1)

    def for_loop(self, k: int) -> None:
        s = self.s
        tokens = s.tokens
        end = s.find(
            k + 1,
            s.n,
            lambda j: _kw(tokens[j], "in") or _is(tokens[j], "{") or _is(tokens[j], ";"),
        )
        if not _kw(s.at(end), "in"):
            return
        bound = self.bind(k + 1, end)
        self.scope(bound, k + 1, end - 1)
        brace = s.find(end + 1, s.n, lambda j: _is(tokens[j], "{"))
        if _is(s.at(brace), "{") and brace in s.partner:
            self.scope(bound, brace, s.partner[brace])

    def match_arm(self, arrow: int) -> None:
        """The arm whose `=>` is at `arrow`: its pattern reaches its guard and body,
        and the body ends at its `,`, or at the next arm when a block-like body
        leaves the comma out."""
        s = self.s
        tokens = s.tokens
        start = _arm_start(s, arrow)
        guard = s.find(start, arrow, lambda j: _kw(tokens[j], "if"))
        bound = self.bind(start, guard)
        body = arrow + 2
        if _is(s.at(body), "{") and body in s.partner:
            end = s.partner[body]
        else:
            end = s.find(body, s.n, lambda j: _is(tokens[j], ",")) - 1
        following = s.find(body, s.n, s.fat_arrow)
        if following < s.n and s.fat_arrow(following):
            end = min(end, _arm_start(s, following) - 1)
        self.scope(bound, start, end)

    def closure(self, k: int) -> None:
        """The closure whose parameter list opens at the `|` at `k`: its parameters
        reach its body, which is a block after `-> T`, or else runs to the next `,`
        or `;` or the end of the group around it. In the head of an `if let` or
        `while let` the body also ends at the `{` that opens the block, so
        `|x| if x {..}` there is cut short: its `x` is kept rather than renamed."""
        s = self.s
        tokens = s.tokens
        if tokens[k].joint and _is(s.at(k + 1), "|"):
            return  # `||`: no parameters
        close = s.find(k + 1, s.n, lambda j: _is(tokens[j], "|"))
        bound: set[str] = set()
        for a, b in s.split(k + 1, close):
            bound |= self.bind(a, _cut_at_colon(s, a, b))
        after = close + 1
        if _is(s.at(after), "-") and s.tokens[after].joint and _is(s.at(after + 1), ">"):
            end = self.item_end(after + 2)
        else:
            block = _is(s.at(after), "{") and after in s.partner
            if block:
                self.bodies.add(after)
            end = s.find(
                after, s.n, lambda j: _is(tokens[j], ",") or _is(tokens[j], ";")
            ) - 1
            if _in_condition_head(s, k):
                if block:
                    end = min(end, s.partner[after])
                else:
                    brace = s.find(after, s.n, lambda j: _is(tokens[j], "{"))
                    end = min(end, brace - 1)
        self.scope(bound, k, end)

    def field_positions(self, skip: set[int]) -> dict[int, bool]:
        """The tokens that name a field: an entry's leading name followed by a lone
        `:` in any brace group, and in a struct expression or pattern also a name
        standing alone (`P { x }`, where it is the field and a binding or use).
        A brace group is a struct expression or pattern when a path names it and it
        is not a block: inside a pattern always; elsewhere unless it is a known
        body or its statement opens with a block or item keyword."""
        s = self.s
        in_pattern = [False] * (s.n + 1)
        for a, b in self.patterns:
            for j in range(a, b):
                in_pattern[j] = True
        positions: dict[int, bool] = {}
        for brace, close in s.partner.items():
            if brace > close or not _is(s.tokens[brace], "{") or brace in skip:
                continue
            before = s.at(brace - 1)
            named = before is not None and (
                before.kind == "ident"
                or _kw(before, "Self")
                or (_is(before, ">") and not s.fat_arrow(brace - 2))
            )
            struct = named and (
                in_pattern[brace]
                or (
                    brace not in self.bodies
                    and not (_header_words(s, brace) & _BLOCK_WORDS)
                )
            )
            for a, b in s.split(brace + 1, close):
                a = _skip_attributes_and_visibility(s, a, b)
                while a < b and (_kw(s.at(a), "ref") or _kw(s.at(a), "mut")):
                    a += 1
                t = s.at(a)
                if a >= b or t is None or t.kind != "ident":
                    continue
                if s.colon(a + 1):
                    positions[a] = False
                elif struct and a + 1 == b:
                    positions[a] = True
        return positions


def _declared(s: _Stream) -> _Declared:
    c = _Collector(s)
    skip = _macro_rules_bodies(s)
    tokens = s.tokens

    for k, t in enumerate(tokens):
        if k in skip:
            continue
        if t.kind == "keyword":
            word = t.text
            if word == "let":
                c.let(k)
            elif word == "for" and not _is(s.at(k + 1), "<"):
                c.for_loop(k)
            elif word == "fn":
                name = s.at(k + 1)
                if name is not None and name.kind == "ident":
                    c.function(k)
            elif word in ("struct", "enum"):
                name = s.at(k + 1)
                if name is not None and name.kind == "ident":
                    c.data_type(k + 1, word)
            elif word in ("trait", "type", "mod", "const", "static"):
                j = k + 2 if word == "static" and _kw(s.at(k + 1), "mut") else k + 1
                name = s.at(j)
                if name is not None and name.kind == "ident":
                    c.item(j)
                    if _is(s.at(j + 1), "<"):
                        gt, found = c.generics(j + 1)
                        c.scope(found, j, c.item_end(gt + 1))
            elif word == "impl" and _is(s.at(k + 1), "<"):
                gt, found = c.generics(k + 1)
                c.scope(found, k, c.item_end(gt + 1))
            elif word == "use":
                end = s.find(k + 1, s.n, lambda j: _is(tokens[j], ";"))
                for j in range(k + 1, min(end, s.n)):
                    alias = s.at(j + 1)
                    if (
                        _kw(tokens[j], "as")
                        and alias is not None
                        and alias.kind == "ident"
                    ):
                        c.item(j + 1)
        elif t.kind == "ident":
            after = s.at(k + 1)
            if (
                t.text == "union"
                and after is not None
                and after.kind == "ident"
                and (_is(s.at(k + 2), "{") or _is(s.at(k + 2), "<"))
            ):
                c.data_type(k + 1, "union")
            elif t.text == "macro_rules" and _is(after, "!"):
                name = s.at(k + 2)
                if name is not None and name.kind == "ident":
                    c.macros.add(name.text)
                    c.scope([name.text], k + 2, k + 2)
                    # Textual scope: a call before the definition, or outside the
                    # block that holds it, is someone else's macro.
                    block = c.enclosing[k]
                    end = s.partner[block] - 1 if block is not None else s.n - 1
                    c.macro_scopes.setdefault(name.text, []).append((k + 2, end))
        elif t.kind == "punct":
            if s.fat_arrow(k):
                c.match_arm(k)
            elif t.text == "|" and _closure_opens(s, k):
                c.closure(k)

    return _Declared(
        frozenset(c.names),
        frozenset(c.members),
        frozenset(c.macros),
        frozenset(c.fields),
        c.scopes,
        c.field_positions(skip),
        c.macro_scopes,
        frozenset(c.generic_names),
        frozenset(c.associated),
        c.item_braces,
        c.module_bodies,
        frozenset(c.library_impl_items),
        c.assoc_owners,
        c.impl_owner,
        c.enclosing,
    )


# --------------------------------------------------------------------------- #
# Normalization
# --------------------------------------------------------------------------- #

# The formatting macros, and which argument is the format string: `println!("..")`
# is the first, `write!(f, "..")` the second, `assert_eq!(a, b, "..")` the third.
# Only that one literal has its inline arguments renamed. Any other string literal
# is kept byte for byte, because a program prints it: renaming `{y}` inside a
# literal that is printed as-is would make two programs with different output equal.
_FORMAT_ARGUMENT = {
    **dict.fromkeys(
        (
            "print",
            "println",
            "eprint",
            "eprintln",
            "format",
            "format_args",
            "panic",
            "unreachable",
            "todo",
            "unimplemented",
        ),
        0,
    ),
    **dict.fromkeys(("write", "writeln", "assert", "debug_assert"), 1),
    **dict.fromkeys(
        ("assert_eq", "assert_ne", "debug_assert_eq", "debug_assert_ne"), 2
    ),
}

_FORMAT_PIECE = re.compile(r"\{\{|\}\}|\{([^{}]*)\}")
_NAME = re.compile(r"[^\W\d]\w*\Z", re.UNICODE)
_NAMED_WIDTH = re.compile(r"([^\W\d]\w*)(?=\$)", re.UNICODE)


def _format_strings(s: _Stream, declared: _Declared) -> set[int]:
    """Indexes of the string literals that are format strings."""
    found: set[int] = set()
    for k, t in enumerate(s.tokens):
        if (
            t.kind != "ident"
            or t.text not in _FORMAT_ARGUMENT
            or t.text in declared.macros
        ):
            continue
        bang, opener = s.at(k + 1), k + 2
        if (
            not _is(bang, "!")
            or opener not in s.partner
            or s.tokens[opener].text not in _OPENERS
        ):
            continue
        parts = s.split(opener + 1, s.partner[opener])
        position = _FORMAT_ARGUMENT[t.text]
        if position < len(parts):
            a, b = parts[position]
            if a < b and s.tokens[a].kind == "string":
                found.add(a)
    return found


def _is_macro_call(s: _Stream, k: int) -> bool:
    """`name!(`, `name![` or `name!{` - and not `name != ...`."""
    bang = s.at(k + 1)
    return (
        _is(bang, "!")
        and not (bang.joint and _is(s.at(k + 2), "="))
        and s.at(k + 2) is not None
        and s.tokens[k + 2].kind == "punct"
        and s.tokens[k + 2].text in _OPENERS
    )


# Rust's compound punctuation. The lexer keeps punctuation one character at a
# time; the output glues adjacent characters back into these, as rustc's parser
# does, so `a == b` and `a = = b` - which does not compile - stay two programs.
_COMPOUND = frozenset(
    [
        "==",
        "!=",
        "<=",
        ">=",
        "&&",
        "||",
        "+=",
        "-=",
        "*=",
        "/=",
        "%=",
        "^=",
        "&=",
        "|=",
        "<<",
        ">>",
        "<<=",
        ">>=",
        "::",
        "->",
        "=>",
        "..",
        "...",
        "..=",
    ]
)

# Methods and fields the standard library gives its own types. A program that
# declares a member by one of these names may also call the library's - its own
# `fn len` beside `self.items.len()` - and renaming both would make it equal to a
# program that calls a `size` the library never had. So a declared member with one
# of these names is never renamed, anywhere. The list is not complete and cannot
# be: a rarer name still carries the risk, and it is named in `bank/README.md`.
_STD_MEMBERS = frozenset(
    [
        "abs",
        "add",
        "all",
        "and_then",
        "any",
        "as_bytes",
        "as_mut",
        "as_ref",
        "as_slice",
        "as_str",
        "borrow",
        "borrow_mut",
        "bytes",
        "capacity",
        "chain",
        "char_indices",
        "chars",
        "checked_add",
        "checked_sub",
        "chunks",
        "clamp",
        "clear",
        "clone",
        "cloned",
        "cmp",
        "collect",
        "concat",
        "contains",
        "contains_key",
        "copied",
        "count",
        "dedup",
        "dedup_by_key",
        "deref",
        "deref_mut",
        "div",
        "drain",
        "end",
        "ends_with",
        "entry",
        "enumerate",
        "eq",
        "err",
        "expect",
        "extend",
        "fill",
        "filter",
        "filter_map",
        "find",
        "first",
        "flat_map",
        "flatten",
        "fmt",
        "fold",
        "for_each",
        "from",
        "ge",
        "get",
        "get_mut",
        "gt",
        "hash",
        "index",
        "insert",
        "into",
        "into_iter",
        "is_empty",
        "is_err",
        "is_none",
        "is_ok",
        "is_some",
        "iter",
        "iter_mut",
        "join",
        "keys",
        "last",
        "le",
        "len",
        "lines",
        "lt",
        "map",
        "map_err",
        "max",
        "max_by_key",
        "min",
        "min_by_key",
        "mul",
        "ne",
        "neg",
        "next",
        "not",
        "ok",
        "or_else",
        "or_insert",
        "or_insert_with",
        "parse",
        "partial_cmp",
        "peek",
        "pop",
        "position",
        "pow",
        "product",
        "push",
        "push_str",
        "rem",
        "remove",
        "replace",
        "reserve",
        "resize",
        "retain",
        "rev",
        "reverse",
        "saturating_sub",
        "skip",
        "sort",
        "sort_by",
        "sort_by_key",
        "sort_unstable",
        "split",
        "split_at",
        "split_off",
        "start",
        "starts_with",
        "step_by",
        "sub",
        "sum",
        "swap",
        "take",
        "to_lowercase",
        "to_owned",
        "to_string",
        "to_uppercase",
        "to_vec",
        "trim",
        "truncate",
        "unwrap",
        "unwrap_or",
        "unwrap_or_default",
        "unwrap_or_else",
        "values",
        "values_mut",
        "windows",
        "wrapping_add",
        "zip",
    ]
)

# Macros whose arguments take a trailing comma with the same meaning. A macro
# the program defines may not: its rules can treat `m!(1, 2,)` as another input.
_TRAILING_COMMA_MACROS = frozenset(
    [
        "print",
        "println",
        "eprint",
        "eprintln",
        "format",
        "format_args",
        "panic",
        "write",
        "writeln",
        "vec",
        "assert_eq",
        "assert_ne",
        "debug_assert_eq",
        "debug_assert_ne",
        "dbg",
        "matches",
        "unreachable",
        "todo",
        "unimplemented",
    ]
)

# What can stand just before an expression begins: where a `[` opens an array
# rather than indexing one, and where a path followed by `{` is a struct literal.
_EXPRESSION_STARTS = frozenset(["=", "(", "[", "{", ",", ";", "&", "*", "!"])


def _starts_expression(s: _Stream, k: int) -> bool:
    """Whether an expression may begin at `k`, judged by the token before it."""
    before = s.at(k - 1)
    if before is None:
        return True
    if before.kind == "keyword":
        return before.text in ("return", "break")
    if before.kind != "punct":
        return False
    if before.text == ":":
        return s.colon(k - 1)
    if before.text == ">":
        return s.fat_arrow(k - 2)
    return before.text in _EXPRESSION_STARTS


def _path_head(s: _Stream, k: int) -> int:
    """The first segment of the path whose last segment is at `k`."""
    while k >= 3 and s.path_sep[k - 1] and s.path_sep[k - 2]:
        segment = s.tokens[k - 3]
        if segment.kind not in ("ident", "keyword"):
            break
        k -= 3
    return k


def _list_bodies(s: _Stream) -> set[int]:
    """The `{` of every struct, enum and union definition, match and struct
    literal: the braces whose last comma is optional. A block is none of these, and
    `{ f(), }` does not compile where `{ f() }` does."""
    bodies: set[int] = set()
    for k, t in enumerate(s.tokens):
        if (
            t.kind == "keyword"
            and t.text in ("struct", "enum")
            or (
                t.kind == "ident"
                and t.text == "union"
                and s.at(k + 1) is not None
                and s.tokens[k + 1].kind == "ident"
            )
        ):
            j = k + 2
            if _is(s.at(j), "<"):
                j = _generic_names(s, j, set()) + 1
            body = s.find(
                j,
                s.n,
                lambda x: s.tokens[x].kind == "punct" and s.tokens[x].text in "{;(",
            )
            if _is(s.at(body), "{"):
                bodies.add(body)
        elif _kw(t, "match"):
            body = s.find(k + 1, s.n, lambda x: _is(s.tokens[x], "{"))
            if _is(s.at(body), "{"):
                bodies.add(body)
        elif _is(t, "{") and k > 0:
            before = s.tokens[k - 1]
            if (before.kind == "ident" or _kw(before, "Self")) and _starts_expression(
                s, _path_head(s, k - 1)
            ):
                bodies.add(k)
    return bodies


def _user_macro_arguments(s: _Stream, declared: _Declared) -> set[int]:
    inside: set[int] = set()
    for k, t in enumerate(s.tokens):
        if t.kind == "ident" and t.text in declared.macros and _is_macro_call(s, k):
            opener = k + 2
            if opener in s.partner:
                inside.update(range(opener, s.partner[opener] + 1))
    return inside


def _entry_start(s: _Stream, opener: int, comma: int) -> int:
    """Where the list entry that ends at `comma` begins, inside the group at `opener`."""
    start = opener + 1
    for a, b in s.split(opener + 1, comma):
        start = a
    return start


def _dropped_comma(s: _Stream, k: int, bodies: set[int], user_macro: set[int]) -> bool:
    """The commas rustfmt adds and removes - and only where the program means the
    same with or without them.

    That second half is the whole design. A rule that dropped a comma a program
    needs would make one that does not compile equal to one that does, and in this
    quiz *does not compile* is an answer. So a comma is dropped only:

    * before the `]` of an array (not an index: `v[0,]` does not compile), and not
      of a repeat expression `[x; n]`;
    * before the `)` of a call, a function's parameters or a tuple struct - where
      `f(a,)` is `f(a)` - but never of a tuple or parentheses, since `(x,)` is a
      tuple and `(x)` is not;
    * before the `}` of a struct or enum definition, a struct literal (not after
      `..base`, which may not be followed by one) or a match;
    * after the `}` of a match arm's block, `=> { .. },`;

    and never inside the arguments of a macro the program defines, or of a macro
    not known to take a trailing comma.
    """
    if k in user_macro:
        return False
    after = s.at(k + 1)
    before = s.at(k - 1)
    # A trailing separator requires an entry. Removing a lone or repeated comma
    # would turn an invalid list into a valid empty or single-entry list.
    if before is None or (
        before.kind == "punct" and before.text in ("(", "[", "{", ",")
    ):
        return False
    if _is(before, "}") and (k - 1) in s.partner and s.fat_arrow(s.partner[k - 1] - 2):
        return True
    if after is None or after.kind != "punct" or after.text not in _CLOSERS:
        return False
    opener = s.partner.get(k + 1)
    if opener is None:
        return False
    if after.text == "]":
        if not _starts_expression(s, opener):
            return False
        return s.find(opener + 1, k, lambda j: _is(s.tokens[j], ";")) >= k
    if after.text == ")":
        callee = s.at(opener - 1)
        if callee is None:
            return False
        if _is(callee, "!"):
            name = s.at(opener - 2)
            return name is not None and name.text in _TRAILING_COMMA_MACROS
        return callee.kind == "ident" or _is(callee, ")") or _is(callee, "]")
    if opener not in bodies:
        return False
    entry = _entry_start(s, opener, k)
    return not (_is(s.at(entry), ".") and _is(s.at(entry + 1), "."))


def _compound_length(s: _Stream, k: int) -> int:
    """How many single-character tokens from `k` form one compound: 1, 2 or 3."""
    for length in (3, 2):
        parts = s.tokens[k : k + length]
        if (
            len(parts) == length
            and all(p.kind == "punct" for p in parts)
            and all(p.joint for p in parts[:-1])
            and "".join(p.text for p in parts) in _COMPOUND
        ):
            return length
    return 1


def _inline_argument_names(literal: str) -> set[str]:
    """The names a format string captures: `{v:?}`'s `v` and `{:>w$}`'s `w`."""
    names: set[str] = set()
    for match in _FORMAT_PIECE.finditer(literal):
        spec = match.group(1)
        if spec is None:
            continue
        argument, _, form = spec.partition(":")
        if _NAME.match(argument):
            names.add(argument)
        names.update(_NAMED_WIDTH.findall(form))
    return names


def _unscoped_names(
    s: _Stream, declared: _Declared, format_strings: set[int]
) -> set[str]:
    """Declared spellings to keep, because some bare use of one is outside the
    reach of every declaration of it.

    Such a use means something the program does not declare - the library's
    `drop`, a field of the library's `Range` - so renaming the spelling would make
    that use the same token as any other name, resolved or not. This declines the
    rename instead of resolving the name. A use after `.`, after `::` or before `!`
    is left to the member and path rules; a macro call outside the textual scope of
    every `macro_rules!` of its name protects the name. A field position needs a declared
    field of that spelling, and, in `P { x }`, a declaration in reach as well.
    """
    candidates = declared.names | declared.members | declared.macros
    protected: set[str] = set()

    def reached(name: str, k: int) -> bool:
        return any(lo <= k <= hi for lo, hi in declared.scopes.get(name, ()))

    for k, t in enumerate(s.tokens):
        if t.kind == "string" and k in format_strings:
            for name in _inline_argument_names(t.text):
                if name in candidates and not reached(name, k):
                    protected.add(name)
            continue
        if t.kind != "ident" or t.text not in candidates or t.text in protected:
            continue
        if _is(s.at(k - 1), ".") and not s.range_dot(k - 1):
            continue
        if k > 0 and s.path_sep[k - 1]:
            continue
        if _is_macro_call(s, k):
            # Only a `macro_rules!` of this spelling can be meant, and only where
            # its textual scope reaches.
            if t.text in declared.macros and not any(
                lo <= k <= hi for lo, hi in declared.macro_scopes.get(t.text, ())
            ):
                protected.add(t.text)
            continue
        shorthand = declared.fields_at.get(k)
        if shorthand is None:
            fine = reached(t.text, k)
        else:
            fine = t.text in declared.fields and (not shorthand or reached(t.text, k))
        if not fine:
            protected.add(t.text)
    return protected


_PATH_KEYWORDS = frozenset(["Self", "self", "crate", "super"])


def _tail_reaches(s: _Stream, declared: _Declared, k: int) -> bool:
    """Whether the path tail at `k` can be the program's own declaration of its
    name, given the segment before it. Token-level, so it answers no whenever it
    cannot tell: after a generic parameter (`I::Item`, `T::default()` are the
    bound trait's), after `Self` unless the name is an associated item, after a
    module or `crate`/`self`/`super` unless the name is an item declared directly
    in a module (a `use` re-export of the library's is not), and after a type
    unless the name is an associated item or a variant of that very type."""
    head = s.at(k - 3)
    name = s.tokens[k].text
    if head is None or head.kind not in ("ident", "keyword"):
        return False
    if head.text in ("crate", "self", "super"):
        modules = {b for bodies in declared.module_bodies.values() for b in bodies}
        return any(
            b is None or b in modules for b in declared.item_braces.get(name, ())
        )
    if head.text == "Self":
        brace = declared.enclosing[k]
        while brace is not None and brace not in declared.impl_owner:
            brace = declared.enclosing[brace]
        return (
            brace is not None
            and declared.impl_owner[brace] in declared.assoc_owners.get(name, set())
        )
    if head.kind != "ident" or head.text in declared.generics:
        return False
    if head.text in declared.module_bodies:
        return bool(
            declared.module_bodies[head.text] & declared.item_braces.get(name, set())
        )
    return head.text in declared.assoc_owners.get(name, set())


def _unreached_path_tails(
    s: _Stream, declared: _Declared, protected: set[str]
) -> set[str]:
    """Declared spellings to keep because a path tail of that spelling is kept.

    A tail is renamed only when its head is and `_tail_reaches` says the name is
    the program's. A tail that is kept verbatim after a head the program declares
    - or after `Self`, `self`, `crate` or `super` - may be the library's name, or
    the program's reached a way tokens cannot follow; either way, renaming its
    declaration elsewhere would cut the link between the two and make the
    program the same tokens as one that names something else."""
    candidates = declared.names | declared.members | declared.macros
    renamable = candidates - protected
    renamed: set[int] = set()
    found: set[str] = set()
    for k in range(3, s.n):
        t = s.tokens[k]
        if t.kind != "ident" or not s.path_sep[k - 1]:
            continue
        head = s.tokens[k - 3]
        if head.text in _PATH_KEYWORDS:
            head_declared = head_renamed = True
        else:
            head_declared = head.kind == "ident" and head.text in candidates
            head_renamed = head.text in renamable and (
                k < 4 or not s.path_sep[k - 4] or (k - 3) in renamed
            )
        if head_renamed and t.text in renamable and _tail_reaches(s, declared, k):
            renamed.add(k)
        elif head_declared and t.text in renamable:
            found.add(t.text)
    return found


def _kept_doc_comments(
    tokens: list[Token], docs: list[tuple[int, str]]
) -> dict[int, list[str]]:
    """The doc comments that change what the program is, by the token they precede.

    Only outer docs immediately before recognized item keywords are dropped.
    All other positions retain a marker: dangling docs, docs on parameters and
    inner docs can affect compilation. This is conservative about unfamiliar
    item syntax; it is not a Rust attribute parser.
    """
    kept: dict[int, list[str]] = {}
    for position, kind in docs:
        # Only discard docs immediately before an item keyword. In particular,
        # a parameter or expression is not an item a doc comment can document.
        item = (
            position < len(tokens)
            and tokens[position].kind == "keyword"
            and (
                tokens[position].text
                in {"fn", "struct", "enum", "trait", "impl", "mod", "type"}
            )
        )
        if kind == "//!" or not item:
            kept.setdefault(position, []).append(kind)
    return kept


def _normalize(
    tokens: list[Token], docs: list[tuple[int, str]], *, numbered: bool
) -> list[str]:
    s = _Stream(tokens)
    declared = _declared(s)
    kept_members = declared.members & _STD_MEMBERS
    format_strings = _format_strings(s, declared)
    protected = kept_members | {"main"} | _unscoped_names(s, declared, format_strings)
    # Derives can expose names (Debug) or interpret them (other derives). Without
    # expanding those macros, alpha-renaming is not evidence of equivalence.
    if any(t.text == "derive" for t in tokens):
        protected |= declared.names | declared.members | declared.macros
    # A library trait fixes the names declared in its `impl`.
    protected |= declared.library_impl_items
    # A protected head keeps its tails verbatim, which may protect more: repeat
    # until nothing changes. Each round only grows `protected`.
    while extra := _unreached_path_tails(s, declared, protected) - protected:
        protected |= extra
    renamable = (declared.names | declared.members | declared.macros) - protected
    members = declared.members - protected
    bodies = _list_bodies(s)
    user_macro = _user_macro_arguments(s, declared)
    kept_docs = _kept_doc_comments(tokens, docs)

    order: dict[str, int] = {}

    def placeholder(name: str) -> str:
        if name not in order:
            order[name] = len(order) + 1
        return f"${order[name]}" if numbered else "$"

    renamed_at: set[int] = set()
    out: list[str] = []
    k = 0
    while k < len(tokens):
        out.extend(kept_docs.get(k, ()))
        t = tokens[k]
        if t.kind == "punct":
            if t.text == "," and _dropped_comma(s, k, bodies, user_macro):
                k += 1
                continue
            length = _compound_length(s, k)
            out.append("".join(p.text for p in tokens[k : k + length]))
            k += length
            continue
        if t.kind == "lifetime":
            # A lifetime keeps its quote, so `'a` and a name `a` never share text.
            out.append(
                t.text if t.text in ("'static", "'_") else "'" + placeholder(t.text)
            )
        elif t.kind == "string" and k in format_strings:
            out.append(
                _rename_inline_arguments(
                    t.text, declared.names - protected, placeholder
                )
            )
        elif t.kind != "ident":
            out.append(t.text)
        else:
            name = t.text
            before = s.at(k - 1)
            if _is(before, ".") and not s.range_dot(k - 1):
                rename = name in members
            elif k > 0 and s.path_sep[k - 1]:
                segment = s.at(k - 3)
                rename = (
                    name in renamable
                    and (
                        (k - 3) in renamed_at
                        or (segment is not None and segment.text in _PATH_KEYWORDS)
                    )
                    and _tail_reaches(s, declared, k)
                )
            elif _is_macro_call(s, k):
                rename = name in declared.macros and name in renamable
            else:
                rename = name in renamable
            if rename:
                renamed_at.add(k)
                out.append(placeholder(name))
            else:
                out.append(name)
        k += 1
    out.extend(kept_docs.get(len(tokens), ()))
    return out


def _rename_inline_arguments(
    literal: str, renamable: frozenset[str], placeholder: Callable[[str], str]
) -> str:
    """`{v:?}` to `{$1:?}` inside a format string, and `{:>w$}` to `{:>$2$}`.
    `{{` and `}}` are escapes and are left alone; so are positional arguments."""

    def piece(match: re.Match[str]) -> str:
        spec = match.group(1)
        if spec is None:
            return match.group(0)
        argument, colon, form = spec.partition(":")
        if _NAME.match(argument) and argument in renamable:
            argument = placeholder(argument)
        form = _NAMED_WIDTH.sub(
            lambda w: (
                placeholder(w.group(1)) if w.group(1) in renamable else w.group(1)
            ),
            form,
        )
        return "{" + argument + colon + form + "}"

    return _FORMAT_PIECE.sub(piece, literal)


def normalized_tokens(source: str) -> list[str]:
    """The token stream the fingerprint hashes: comments, layout and the optional
    commas gone, compound operators glued, every declared name a numbered
    placeholder in order of first appearance. Raises `DedupeError` if the source
    cannot be lexed."""
    tokens, docs = _lex(source)
    return _normalize(tokens, docs, numbered=True)


# --------------------------------------------------------------------------- #
# The three stores
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class Entry:
    """What the history records for one program (SPEC 3.3): the three stores."""

    exact_hash: str
    fingerprint: str
    bigrams: tuple[str, ...]


def _fingerprint(stream: Sequence[str]) -> str:
    encoded = json.dumps(list(stream), ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(encoded.encode("utf-8")).hexdigest()


def _bigrams(stream: Sequence[str]) -> tuple[str, ...]:
    """Distinct adjacent pairs, sorted, each written `"a b"`.

    The join is unambiguous: outside a string or char literal no token contains a
    space, and a literal token always carries both of its quotes, so no two
    different pairs can spell the same string.
    """
    return tuple(sorted({f"{a} {b}" for a, b in pairwise(stream)}))


def entry_for(source: str) -> Entry:
    """The three stores for one program. Raises `DedupeError` if it cannot be lexed."""
    tokens, docs = _lex(source)
    return Entry(
        exact_hash=source_hash(source),
        fingerprint=_fingerprint(_normalize(tokens, docs, numbered=True)),
        bigrams=_bigrams(_normalize(tokens, docs, numbered=False)),
    )


def similarity(a: Iterable[str], b: Iterable[str]) -> float:
    """Jaccard similarity of two bigram sets. Two empty sets share nothing: 0.0."""
    left, right = set(a), set(b)
    union = left | right
    return len(left & right) / len(union) if union else 0.0


# --------------------------------------------------------------------------- #
# The history (SPEC 3.3, AC-17)
# --------------------------------------------------------------------------- #


def _holds(history: History, question_id: str) -> bool:
    return any(
        question_id in store
        for store in (
            history.exact_hashes,
            history.ast_fingerprints,
            history.token_bigrams,
        )
    )


def recorded_ids(history: History) -> tuple[str, ...]:
    """Every question id the history holds an entry for, in the order recorded."""
    seen: dict[str, None] = {}
    for store in (
        history.exact_hashes,
        history.ast_fingerprints,
        history.token_bigrams,
    ):
        for question_id in store:
            seen.setdefault(question_id, None)
    return tuple(seen)


def history_size(history: History) -> int:
    """How many programs the history holds: the number on the organizer's first
    screen and in every run report (AC-17).

    Not `History.total()`, which adds the three stores together and so counts each
    program three times once the stores agree.
    """
    return len(recorded_ids(history))


def entry_in(history: History, question_id: str) -> Entry | None:
    """The recorded entry for one question, or `None` unless all three stores hold it."""
    if not all(
        question_id in store
        for store in (
            history.exact_hashes,
            history.ast_fingerprints,
            history.token_bigrams,
        )
    ):
        return None
    return Entry(
        exact_hash=history.exact_hashes[question_id],
        fingerprint=history.ast_fingerprints[question_id],
        bigrams=tuple(history.token_bigrams[question_id]),
    )


def _with_entry(history: History, question_id: str, entry: Entry) -> History:
    return History(
        exact_hashes={**history.exact_hashes, question_id: entry.exact_hash},
        ast_fingerprints={**history.ast_fingerprints, question_id: entry.fingerprint},
        token_bigrams={**history.token_bigrams, question_id: list(entry.bigrams)},
        version=history.version,
    )


def record(
    history: History, question_id: str, source: str, *, replace: bool = False
) -> History:
    """A new history with one program recorded under `question_id`.

    Recording the same program twice is a no-op. Recording a *different* program
    under an id the history already holds is refused: ids are never reused, and a
    history that quietly swapped one program for another would forget the first.
    The one legitimate case is an organizer's edit (SPEC 7.4), which re-records the
    edited source with `replace=True` after checking it with `excluding=` its own id.
    """
    entry = entry_for(source)
    recorded = entry_in(history, question_id)
    if recorded == entry:
        return history
    if _holds(history, question_id) and not replace:
        raise DedupeError(
            f"{question_id} is already in the history with a different program. Ids "
            "are never reused; an edited question is re-recorded with replace=True."
        )
    return _with_entry(history, question_id, entry)


def sync_with_bank(
    history: History, questions: Iterable[Question]
) -> tuple[History, tuple[str, ...], tuple[str, ...]]:
    """Make the history describe the bank: `(history, backfilled, refreshed)`.

    A bank question with no entry is *backfilled*. One whose entry no longer matches
    its source - the question was re-authored, or this module's normalization
    changed - is *refreshed*. Entries for ids not in the bank are kept: the history
    is memory, and forgetting is not dedupe's call.

    So the history can always be rebuilt from the bank, and a history reset to the
    empty shape - which re-running the MVP migration does - heals on the next run.
    """
    backfilled: list[str] = []
    refreshed: list[str] = []
    for question in questions:
        entry = entry_for(question.source)
        recorded = entry_in(history, question.id)
        if recorded == entry:
            continue
        if _holds(history, question.id):
            refreshed.append(question.id)
        else:
            backfilled.append(question.id)
        history = _with_entry(history, question.id, entry)
    return history, tuple(backfilled), tuple(refreshed)


# --------------------------------------------------------------------------- #
# The check (AC-14, AC-15, AC-16)
# --------------------------------------------------------------------------- #

VerdictKind = Literal[
    "exact_duplicate", "normalized_duplicate", "near_duplicate", "cleared"
]

_ADMITTED = frozenset({"near_duplicate", "cleared"})


@dataclass(frozen=True)
class Verdict:
    """What dedupe found for one candidate.

    `duplicate_of` is the question a rejection or a near-duplicate mark points at.
    `closest` and `similarity` are the most similar recorded question and its score,
    on every verdict that reached the near check: the threshold is uncalibrated, and
    these are the numbers that will calibrate it.
    """

    candidate_id: str
    kind: VerdictKind
    threshold: float
    duplicate_of: str | None = None
    closest: str | None = None
    similarity: float | None = None

    @property
    def admitted(self) -> bool:
        """Whether the candidate joins the review queue (it was not rejected)."""
        return self.kind in _ADMITTED

    @property
    def statement(self) -> str | None:
        """The uniqueness statement, for a candidate that has earned one (AC-18)."""
        return UNIQUENESS_STATEMENT if self.admitted else None

    def describe(self, *, written: bool = True) -> str:
        """One line for the run report."""
        about = self.candidate_id
        threshold = f"threshold {self.threshold:.2f}, uncalibrated"
        queue = (
            "added to the review queue" if written else "would join the review queue"
        )
        if self.kind == "exact_duplicate":
            return (
                f"{about}: rejected — exact duplicate of {self.duplicate_of}: "
                "the source is byte-identical"
            )
        if self.kind == "normalized_duplicate":
            return (
                f"{about}: rejected — normalized duplicate of {self.duplicate_of}: the "
                "same tokens once declared names are renamed and comments, layout and "
                "formatting commas are ignored"
            )
        if self.kind == "near_duplicate":
            return (
                f"{about}: {UNIQUENESS_STATEMENT}; near-duplicate of {self.duplicate_of} "
                f"at similarity {self.similarity:.2f} ({threshold}) — {queue}, marked "
                f"near-duplicate of {self.duplicate_of}"
            )
        if self.closest is None:
            return f"{about}: {UNIQUENESS_STATEMENT}; the history is empty — {queue}"
        return (
            f"{about}: {UNIQUENESS_STATEMENT}; closest is {self.closest} at similarity "
            f"{self.similarity:.2f} ({threshold}) — {queue}"
        )

    def to_dict(self) -> dict[str, object]:
        out: dict[str, object] = {
            "candidate": self.candidate_id,
            "verdict": self.kind,
            "admitted": self.admitted,
        }
        optional = {
            "statement": self.statement,
            "duplicate_of": self.duplicate_of,
            "closest": self.closest,
            "similarity": None
            if self.similarity is None
            else round(self.similarity, 4),
        }
        out.update({key: value for key, value in optional.items() if value is not None})
        return out


def _require_threshold(threshold: float) -> None:
    if (
        isinstance(threshold, bool)
        or not isinstance(threshold, (int, float))
        or not 0.0 < threshold <= 1.0
    ):
        raise DedupeError(
            f"the near-duplicate threshold must be a number above 0 and at most 1, "
            f"not {threshold!r}"
        )


def _check_entry(
    candidate_id: str,
    entry: Entry,
    history: History,
    threshold: float,
    excluding: str | None,
) -> Verdict:
    ids = [qid for qid in sorted(recorded_ids(history)) if qid != excluding]
    for qid in ids:
        if history.exact_hashes.get(qid) == entry.exact_hash:
            return Verdict(candidate_id, "exact_duplicate", threshold, duplicate_of=qid)
    for qid in ids:
        if history.ast_fingerprints.get(qid) == entry.fingerprint:
            return Verdict(
                candidate_id, "normalized_duplicate", threshold, duplicate_of=qid
            )

    closest: str | None = None
    best = 0.0
    for qid in ids:
        score = similarity(entry.bigrams, history.token_bigrams.get(qid, ()))
        if closest is None or score > best:
            closest, best = qid, score
    if closest is not None and best >= threshold:
        return Verdict(
            candidate_id,
            "near_duplicate",
            threshold,
            duplicate_of=closest,
            closest=closest,
            similarity=best,
        )
    return Verdict(
        candidate_id,
        "cleared",
        threshold,
        closest=closest,
        similarity=None if closest is None else best,
    )


def check(
    candidate_id: str,
    source: str,
    history: History,
    *,
    threshold: float = NEAR_DUPLICATE_THRESHOLD,
    excluding: str | None = None,
) -> Verdict:
    """Check one program against the history. Pure: reads nothing, writes nothing.

    Exact first, then normalized, then near; the first that matches decides. Ties
    go to the lowest id, so two runs over one history agree. `excluding` leaves one
    id's entry out of the comparison - an organizer's edit is not a duplicate of the
    question it edits (SPEC 7.4).
    """
    _require_threshold(threshold)
    return _check_entry(candidate_id, entry_for(source), history, threshold, excluding)


# --------------------------------------------------------------------------- #
# The run: the history, the review queue, the report
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class RunReport:
    """What one dedupe run found and did, with the history's size before and after
    (AC-17). `growth` counts backfilled bank questions and admitted candidates."""

    bank_dir: str
    threshold: float
    dry_run: bool
    verdicts: tuple[Verdict, ...]
    history_before: int
    history_after: int
    backfilled: tuple[str, ...] = ()
    refreshed: tuple[str, ...] = ()

    @property
    def growth(self) -> int:
        return self.history_after - self.history_before

    @property
    def admitted(self) -> tuple[str, ...]:
        return tuple(v.candidate_id for v in self.verdicts if v.admitted)

    def lines(self) -> list[str]:
        out = [v.describe(written=not self.dry_run) for v in self.verdicts]
        backfilled, admitted = len(self.backfilled), len(self.admitted)
        if self.dry_run:
            out.append(
                f"history: {self.history_before} programs recorded; this run would make "
                f"it {self.history_after} ({self.growth:+d}: {backfilled} to backfill "
                f"from the bank, {admitted} to admit) — dry run, nothing written"
            )
        else:
            out.append(
                f"history: {self.history_before} → {self.history_after} programs "
                f"recorded ({self.growth:+d}: {backfilled} backfilled from the bank, "
                f"{admitted} admitted)"
            )
        if self.refreshed:
            out.append(
                "refreshed: "
                + ", ".join(self.refreshed)
                + " — the recorded entry no longer matched the bank's source"
            )
        return out

    def to_dict(self) -> dict[str, object]:
        return {
            "bank": self.bank_dir,
            "dry_run": self.dry_run,
            "threshold": self.threshold,
            "threshold_calibrated": False,
            "verdicts": [v.to_dict() for v in self.verdicts],
            "history": {
                "before": self.history_before,
                "after": self.history_after,
                "growth": self.growth,
                "backfilled": list(self.backfilled),
                "refreshed": list(self.refreshed),
            },
        }


def _require_fresh(
    candidates: Sequence[Question], bank_ids: set[str], history_ids: set[str]
) -> None:
    """Every candidate arrives with an unused id and unjudged, or nothing is written."""
    seen: set[str] = set()
    for candidate in candidates:
        qid = candidate.id
        if qid in bank_ids or qid in history_ids:
            raise DedupeError(
                f"{qid}: the bank already holds this id. Ids are never reused, so a "
                "candidate needs one of its own."
            )
        if qid in seen:
            raise DedupeError(f"{qid}: appears twice in this run")
        seen.add(qid)
        if candidate.used is not None:
            raise DedupeError(
                f"{qid}: a candidate cannot carry `used` - only the release transition "
                "writes it (G-10)"
            )
        review = candidate.review
        if review is not None and (
            review.status is not None
            or review.affirmed_by
            or review.affirmed_at
            or review.near_duplicate_of
        ):
            raise DedupeError(
                f"{qid}: a candidate arrives unjudged, and this one already carries a "
                "review status, an affirmation or a near-duplicate mark"
            )


def _into_review_queue(candidate: Question, verdict: Verdict) -> Question:
    """The record the bank receives. A near-duplicate carries its mark (SPEC 3.1,
    the Orchestrator's ruling F-8 that dedupe writes it); nothing else under
    `review` is touched, so the status stays unset: not accepted, not dropped."""
    if verdict.kind != "near_duplicate":
        return candidate
    review = candidate.review or Review()
    return dataclasses.replace(
        candidate,
        review=dataclasses.replace(review, near_duplicate_of=verdict.duplicate_of),
    )


def run(
    bank_dir: Path | str,
    candidates: Sequence[Question],
    *,
    threshold: float = NEAR_DUPLICATE_THRESHOLD,
    write: bool = True,
) -> RunReport:
    """Check candidates against the bank, admit the ones that pass, grow the history.

    The history is synced with the bank first, so every check covers every question
    in the bank whatever its review status - a question an organizer rejected coming
    back is still a resubmission. Candidates are checked **in order**, and each one
    admitted is recorded before the next is checked, so two near-identical
    candidates in one batch are caught against each other.

    Admitted candidates are appended to the bank with `review.status` unset, which
    is the review queue; rejected ones are written nowhere. Question files are
    written before the history, and the history once, at the end: a run that dies
    between the two leaves a history the next run's sync completes. With
    `write=False` the report is the same and nothing touches the disk.
    """
    _require_threshold(threshold)
    bank_dir = Path(bank_dir)
    if not (bank_dir / QUESTIONS_DIR).is_dir() and not history_path(bank_dir).is_file():
        # A mistyped path would otherwise be an empty bank: nothing to compare
        # with, every candidate admitted, and a second bank started by accident.
        raise DedupeError(
            f"{bank_dir} is not a bank: it holds neither {QUESTIONS_DIR}/ nor "
            f"{history_path(bank_dir).name}"
        )
    questions = load_bank(bank_dir)
    on_disk = load_history(bank_dir)
    history, backfilled, refreshed = sync_with_bank(on_disk, questions)
    _require_fresh(candidates, {q.id for q in questions}, set(recorded_ids(history)))

    # Lex every candidate before anything is written, so one that cannot be lexed
    # stops the run with the bank untouched.
    entries = [entry_for(candidate.source) for candidate in candidates]

    verdicts: list[Verdict] = []
    queued: list[Question] = []
    for candidate, entry in zip(candidates, entries):
        verdict = _check_entry(candidate.id, entry, history, threshold, None)
        verdicts.append(verdict)
        if verdict.admitted:
            queued.append(_into_review_queue(candidate, verdict))
            history = _with_entry(history, candidate.id, entry)

    if write:
        for question in queued:
            append_question(bank_dir, question)
        if history != on_disk:
            save_history(bank_dir, history)

    return RunReport(
        bank_dir=str(bank_dir),
        threshold=threshold,
        dry_run=not write,
        verdicts=tuple(verdicts),
        history_before=history_size(on_disk),
        history_after=history_size(history),
        backfilled=backfilled,
        refreshed=refreshed,
    )


def status(
    bank_dir: Path | str, *, threshold: float = NEAR_DUPLICATE_THRESHOLD
) -> RunReport:
    """The history as the review surface's first screen shows it (AC-17): its size
    on disk, and what the next run would make it. Writes nothing."""
    return run(bank_dir, (), threshold=threshold, write=False)


# --------------------------------------------------------------------------- #
# The command line
# --------------------------------------------------------------------------- #


def _load_candidate(path: Path) -> Question:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise DedupeError(f"{path}: {error.strerror or error}") from error
    except ValueError as error:
        raise DedupeError(f"{path}: not JSON ({error})") from error
    if not isinstance(data, dict):
        raise DedupeError(f"{path}: expected one question object")
    try:
        return question_from_dict(data)
    except KeyError as error:
        raise DedupeError(f"{path}: missing field {error}") from error
    except (BankError, TypeError) as error:
        raise DedupeError(f"{path}: {error}") from error


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="python -m popquiz.dedupe",
        description=(
            "Check candidate questions against the bank's history (SPEC 7.3). Exact "
            "and normalized duplicates are rejected; the rest join the review queue, "
            "near-duplicates marked. With no candidates, report the history and "
            "write nothing."
        ),
    )
    parser.add_argument(
        "candidates",
        nargs="*",
        type=Path,
        metavar="CANDIDATE.json",
        help="one candidate question per file, in the SPEC 3.1 shape",
    )
    parser.add_argument(
        "--bank",
        type=Path,
        default=Path(__file__).resolve().parents[3] / "bank",
        help="the bank directory (default: this repository's bank/)",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=NEAR_DUPLICATE_THRESHOLD,
        help=f"near-duplicate threshold, above 0 and at most 1 "
        f"(default {NEAR_DUPLICATE_THRESHOLD}, uncalibrated)",
    )
    parser.add_argument("--dry-run", action="store_true", help="report, write nothing")
    parser.add_argument("--json", action="store_true", help="print the report as JSON")
    args = parser.parse_args(argv)

    try:
        candidates = [_load_candidate(path) for path in args.candidates]
        report = run(
            args.bank,
            candidates,
            threshold=args.threshold,
            write=bool(candidates) and not args.dry_run,
        )
    except (BankError, DedupeError) as error:
        print(f"dedupe: {error}", file=sys.stderr)
        return 2

    if args.json:
        print(json.dumps(report.to_dict(), indent=2, ensure_ascii=False))
    else:
        print("\n".join(report.lines()))
    return 0 if len(report.admitted) == len(report.verdicts) else 1


if __name__ == "__main__":
    raise SystemExit(main())
