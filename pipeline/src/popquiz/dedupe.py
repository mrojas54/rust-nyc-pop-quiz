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

* comments and whitespace are dropped, and punctuation is kept one character at a
  time, so spacing and line breaks never matter;
* the two commas `rustfmt` adds and removes are dropped - a trailing comma before a
  closing bracket, and the comma after a block match arm;
* every name **the program itself declares** - `let` and pattern bindings,
  parameters, closure parameters, functions, types, fields, variants, generics,
  lifetimes, `macro_rules!` names - is renamed to a numbered placeholder in the
  order it first appears, including inline format arguments like `{v:?}`;
* every other name is kept exactly: keywords, and every standard-library type,
  function, method and macro the program uses without declaring.

That last rule is the one to keep. A normalized duplicate is *rejected*, with no
person in the loop, so the failure to avoid is two different programs coming out
equal. Two programs that call different library functions can never be made equal
here, because a name the program did not declare is never renamed.

What the approximation misses, and what happens instead: statements or items in a
different order, operands swapped around a commutative operator, an expression
rewritten into an equivalent one (`x + x` for `2 * x`), a struct built with field
shorthand in one program and `field: binding` in the other, and anything a
`macro_rules!` body does. Each of those is two token streams, so the check says
"not a normalized duplicate" and the near-duplicate check - which sees them as very
similar - sends the pair to an organizer. A miss costs a person a look; a false
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
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

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
    """
    as break const continue crate else enum extern false fn for if impl in let loop
    match mod move mut pub ref return self Self static struct super trait true type
    unsafe use where while async await dyn abstract become box do final macro
    override priv typeof unsized virtual yield try _
    """.split()
)

TokenKind = Literal["ident", "keyword", "lifetime", "char", "string", "number", "punct"]


@dataclass(frozen=True)
class Token:
    """One lexeme. `joint` is whether the next token starts where this one ends.

    Jointness is read only to recognise the multi-character operators that Rust
    requires to be written without a space - `::`, `=>`, `->`, `..`, `||` - and never
    reaches the fingerprint, which is why reformatting cannot change a fingerprint
    through it.
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
    spans: list[tuple[TokenKind, str, int, int]] = []
    i = 0
    n = len(source)

    def emit(kind: TokenKind, start: int, end: int) -> None:
        spans.append((kind, source[start:end], start, end))

    while i < n:
        c = source[i]
        if c.isspace():
            i += 1
            continue
        if source.startswith("//", i):
            newline = source.find("\n", i)
            i = n if newline < 0 else newline
            continue
        if source.startswith("/*", i):
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
            end = _end_of_char(source, i + 1)
            if end is None:
                raise DedupeError("malformed byte literal")
            emit("char", i, end)
            i = end
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
            end = _end_of_char(source, i)
            if end is not None:
                emit("char", i, end)
                i = end
                continue
            # `_IDENT_START` admits `_`, so `'_` is covered here too.
            if _IDENT_START.match(source, i + 1):
                end = _IDENT_REST.match(source, i + 2).end()
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
            end = _IDENT_REST.match(source, i + 1).end()
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
    return tokens


def _number_end(source: str, i: int) -> int:
    radix = _RADIX.match(source, i)
    if radix:
        j = radix.end()
    else:
        j = _DIGITS.match(source, i).end()
        # A fractional part, but not a range (`1..2`), a method (`1.max(2)`) or a
        # tuple index chain. `1.` on its own is a float, as in Rust.
        if j < len(source) and source[j] == "." and not source.startswith("..", j):
            after = source[j + 1] if j + 1 < len(source) else ""
            if after.isdigit():
                j = _DIGITS.match(source, j + 1).end()
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


def _is(token: Token | None, text: str) -> bool:
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
        return (_is(before, ".") and before.joint) or (_is(after, ".") and here.joint)

    def find(self, start: int, end: int, stop) -> int:
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
        bare = t.text[2:] if t.text.startswith("r#") else t.text
        if not (bare[0] == "_" or bare[0].islower()):
            continue
        after = s.at(k + 1)
        if after is not None and after.kind == "punct" and after.text in ("(", "{", "!"):
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
            elif t.text == ">" and not (_is(s.at(k - 1), "-") and s.tokens[k - 1].joint):
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
            if _kw(t, "const") and s.at(k + 1) is not None and s.tokens[k + 1].kind == "ident":
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


def _variants(s: _Stream, brace: int, names: set[str], members: set[str]) -> None:
    for a, b in s.split(brace + 1, s.partner[brace]):
        a = _skip_attributes_and_visibility(s, a, b)
        t = s.at(a)
        if a < b and t is not None and t.kind == "ident":
            names.add(t.text)
            if _is(s.at(a + 1), "{") and (a + 1) in s.partner:
                members.update(_field_names(s, a + 1))


def _item_body(s: _Stream, k: int, kind: str, names: set[str], members: set[str]) -> None:
    """After a `struct`/`enum`/`union` name at `k`: generics, then the body."""
    j = k + 1
    if _is(s.at(j), "<"):
        j = _generic_names(s, j, names) + 1
    body = s.find(j, s.n, lambda x: _is(s.tokens[x], "{") or _is(s.tokens[x], ";") or _is(s.tokens[x], "("))
    if body < s.n and _is(s.at(body), "{") and body in s.partner:
        if kind == "enum":
            _variants(s, body, names, members)
        else:
            members.update(_field_names(s, body))


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
    if before.text in ("(", "[", "{", ",", ";", "="):
        return True
    if before.text == ":":
        return s.colon(k - 1)
    if before.text == ">":
        return s.fat_arrow(k - 2)
    return False


def _arm_start(s: _Stream, arrow: int) -> int:
    """Where the match arm ending at the `=>` at `arrow` begins.

    Walks back over whole bracket groups to the arm's boundary: the match body's
    `{`, a `,`, a previous `=>`, or the `}` that ends a previous arm's block. A
    `{...}` group is part of the pattern when a path names it (`Point { x, .. }`),
    and ends a previous arm when a keyword or a `=>` precedes it, or when it is the
    body of an `if`, `while` or `match` on a single name.
    """
    j = arrow - 1
    while j >= 0:
        t = s.tokens[j]
        if t.kind == "punct":
            if t.text in _CLOSERS:
                opener = s.partner.get(j)
                if opener is None:
                    return j + 1
                if t.text == "}":
                    before = s.at(opener - 1)
                    named_by_path = before is not None and before.kind == "ident" and not (
                        s.at(opener - 2) is not None
                        and s.tokens[opener - 2].kind == "keyword"
                        and s.tokens[opener - 2].text in ("if", "while", "match")
                    )
                    if not named_by_path:
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
            if s.at(k + 2) is not None and s.tokens[k + 2].kind == "ident" and body in s.partner:
                inside.update(range(body, s.partner[body] + 1))
    return inside


@dataclass(frozen=True)
class _Declared:
    """The names the program itself declares.

    `names` are renamed wherever they stand on their own; `members` - fields and
    functions - are also renamed after a `.`; `macros` are renamed before `!(`.
    """

    names: frozenset[str]
    members: frozenset[str]
    macros: frozenset[str]


def _declared(s: _Stream) -> _Declared:
    names: set[str] = set()
    members: set[str] = set()
    macros: set[str] = set()
    skip = _macro_rules_bodies(s)
    tokens = s.tokens

    for k, t in enumerate(tokens):
        if k in skip:
            continue
        if t.kind == "keyword":
            word = t.text
            if word == "let":
                end = s.find(
                    k + 1,
                    s.n,
                    lambda j: _is(tokens[j], "=")
                    or _is(tokens[j], ";")
                    or s.colon(j)
                    or _kw(tokens[j], "else"),
                )
                names |= _pattern_names(s, k + 1, end)
            elif word == "for" and not _is(s.at(k + 1), "<"):
                end = s.find(
                    k + 1,
                    s.n,
                    lambda j: _kw(tokens[j], "in") or _is(tokens[j], "{") or _is(tokens[j], ";"),
                )
                if _kw(s.at(end), "in"):
                    names |= _pattern_names(s, k + 1, end)
            elif word == "fn":
                name = s.at(k + 1)
                if name is None or name.kind != "ident":
                    continue
                names.add(name.text)
                members.add(name.text)
                j = k + 2
                if _is(s.at(j), "<"):
                    j = _generic_names(s, j, names) + 1
                if _is(s.at(j), "(") and j in s.partner:
                    for a, b in s.split(j + 1, s.partner[j]):
                        names |= _pattern_names(s, a, _cut_at_colon(s, a, b))
            elif word in ("struct", "enum"):
                name = s.at(k + 1)
                if name is not None and name.kind == "ident":
                    names.add(name.text)
                    _item_body(s, k + 1, word, names, members)
            elif word in ("trait", "type", "mod", "const", "static"):
                j = k + 2 if word == "static" and _kw(s.at(k + 1), "mut") else k + 1
                name = s.at(j)
                if name is not None and name.kind == "ident":
                    names.add(name.text)
                    if _is(s.at(j + 1), "<"):
                        _generic_names(s, j + 1, names)
            elif word == "impl" and _is(s.at(k + 1), "<"):
                _generic_names(s, k + 1, names)
            elif word == "use":
                end = s.find(k + 1, s.n, lambda j: _is(tokens[j], ";"))
                for j in range(k + 1, min(end, s.n)):
                    alias = s.at(j + 1)
                    if _kw(tokens[j], "as") and alias is not None and alias.kind == "ident":
                        names.add(alias.text)
        elif t.kind == "ident":
            after = s.at(k + 1)
            if (
                t.text == "union"
                and after is not None
                and after.kind == "ident"
                and (_is(s.at(k + 2), "{") or _is(s.at(k + 2), "<"))
            ):
                names.add(after.text)
                _item_body(s, k + 1, "union", names, members)
            elif t.text == "macro_rules" and _is(after, "!"):
                name = s.at(k + 2)
                if name is not None and name.kind == "ident":
                    macros.add(name.text)
        elif t.kind == "punct":
            if s.fat_arrow(k):
                start = _arm_start(s, k)
                guard = s.find(start, k, lambda j: _kw(tokens[j], "if"))
                names |= _pattern_names(s, start, guard)
            elif t.text == "|" and _closure_opens(s, k):
                if t.joint and _is(s.at(k + 1), "|"):
                    continue  # `||`: no parameters
                close = s.find(k + 1, s.n, lambda j: _is(tokens[j], "|"))
                for a, b in s.split(k + 1, close):
                    names |= _pattern_names(s, a, _cut_at_colon(s, a, b))

    return _Declared(frozenset(names), frozenset(members), frozenset(macros))


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
        ("print", "println", "eprint", "eprintln", "format", "format_args", "panic",
         "unreachable", "todo", "unimplemented"),
        0,
    ),
    **dict.fromkeys(("write", "writeln", "assert", "debug_assert"), 1),
    **dict.fromkeys(("assert_eq", "assert_ne", "debug_assert_eq", "debug_assert_ne"), 2),
}

_FORMAT_PIECE = re.compile(r"\{\{|\}\}|\{([^{}]*)\}")
_NAME = re.compile(r"[^\W\d]\w*\Z", re.UNICODE)
_NAMED_WIDTH = re.compile(r"([^\W\d]\w*)(?=\$)", re.UNICODE)


def _format_strings(s: _Stream, declared: _Declared) -> set[int]:
    """Indexes of the string literals that are format strings."""
    found: set[int] = set()
    for k, t in enumerate(s.tokens):
        if t.kind != "ident" or t.text not in _FORMAT_ARGUMENT or t.text in declared.macros:
            continue
        bang, opener = s.at(k + 1), k + 2
        if not _is(bang, "!") or opener not in s.partner or s.tokens[opener].text not in _OPENERS:
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


def _dropped_comma(s: _Stream, k: int, commas_in: dict[int, int]) -> bool:
    """The two commas rustfmt moves. A trailing comma before `]` or `}`, or before
    `)` when the group already holds another comma - so `(x,)` stays a one-element
    tuple and `(x)` stays a parenthesized `x`. And the comma after a block arm's
    `}` (rustfmt removes it), which the next token not being a closer identifies;
    nowhere else in valid Rust can a comma after `}` be absent."""
    after = s.at(k + 1)
    if _is(after, "]") or _is(after, "}"):
        return True
    if _is(after, ")"):
        return commas_in.get(s.partner.get(k + 1, -1), 0) >= 2
    before = s.at(k - 1)
    return _is(before, "}") and not (after is not None and after.kind == "punct" and after.text in _CLOSERS)


def _normalize(tokens: list[Token], *, numbered: bool) -> list[str]:
    s = _Stream(tokens)
    declared = _declared(s)
    renamable = declared.names | declared.members | declared.macros
    format_strings = _format_strings(s, declared)

    commas_in: dict[int, int] = {}
    for opener, closer in s.partner.items():
        if opener < closer and s.tokens[opener].text == "(":
            commas_in[opener] = len(s.split(opener + 1, closer)) - 1

    order: dict[str, int] = {}

    def placeholder(name: str) -> str:
        if name not in order:
            order[name] = len(order) + 1
        return f"${order[name]}" if numbered else "$"

    renamed_at: set[int] = set()
    out: list[str] = []
    for k, t in enumerate(tokens):
        if t.kind == "punct" and t.text == "," and _dropped_comma(s, k, commas_in):
            continue
        if t.kind == "lifetime":
            out.append(t.text if t.text in ("'static", "'_") else placeholder(t.text))
            continue
        if t.kind == "string" and k in format_strings:
            out.append(_rename_inline_arguments(t.text, renamable, placeholder))
            continue
        if t.kind != "ident":
            out.append(t.text)
            continue

        name = t.text
        before = s.at(k - 1)
        if _is(before, ".") and not s.range_dot(k - 1):
            rename = name in declared.members
        elif k > 0 and s.path_sep[k - 1]:
            segment = s.at(k - 3)
            rename = name in renamable and (
                (k - 3) in renamed_at
                or (segment is not None and segment.text in ("Self", "self", "crate", "super"))
            )
        elif _is_macro_call(s, k):
            rename = name in declared.macros
        else:
            rename = name in renamable

        if rename:
            renamed_at.add(k)
            out.append(placeholder(name))
        else:
            out.append(name)
    return out


def _rename_inline_arguments(literal: str, renamable: frozenset[str], placeholder) -> str:
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
            lambda w: placeholder(w.group(1)) if w.group(1) in renamable else w.group(1), form
        )
        return "{" + argument + colon + form + "}"

    return _FORMAT_PIECE.sub(piece, literal)


def normalized_tokens(source: str) -> list[str]:
    """The token stream the fingerprint hashes: comments, layout and rustfmt's
    commas gone, every declared name a numbered placeholder in order of first
    appearance. Raises `DedupeError` if the source cannot be lexed."""
    return _normalize(tokenize(source), numbered=True)


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
    return tuple(sorted({f"{a} {b}" for a, b in zip(stream, stream[1:])}))


def entry_for(source: str) -> Entry:
    """The three stores for one program. Raises `DedupeError` if it cannot be lexed."""
    tokens = tokenize(source)
    return Entry(
        exact_hash=source_hash(source),
        fingerprint=_fingerprint(_normalize(tokens, numbered=True)),
        bigrams=_bigrams(_normalize(tokens, numbered=False)),
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
        for store in (history.exact_hashes, history.ast_fingerprints, history.token_bigrams)
    )


def recorded_ids(history: History) -> tuple[str, ...]:
    """Every question id the history holds an entry for, in the order recorded."""
    seen: dict[str, None] = {}
    for store in (history.exact_hashes, history.ast_fingerprints, history.token_bigrams):
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
        for store in (history.exact_hashes, history.ast_fingerprints, history.token_bigrams)
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

VerdictKind = Literal["exact_duplicate", "normalized_duplicate", "near_duplicate", "cleared"]

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
        queue = "added to the review queue" if written else "would join the review queue"
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
            "similarity": None if self.similarity is None else round(self.similarity, 4),
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
            return Verdict(candidate_id, "normalized_duplicate", threshold, duplicate_of=qid)

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


def status(bank_dir: Path | str, *, threshold: float = NEAR_DUPLICATE_THRESHOLD) -> RunReport:
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
