"""Where tonight's correct answer sits: a pure function of the date (SPEC G-1, AC-23).

This module is the whole slot path, and it is deliberately tiny. It imports
`hashlib` and `datetime` and nothing else, holds two functions, and has no state:
`bank-audit` lints exactly that (`popquiz.audit.slot_path_violations`), so the
module cannot quietly grow an input.

**Why the date and nothing else.** Any rule that reads history - the least-used
letter, "never the same as last month", a quota - constrains the next answer, and
a constrained answer is information an attendee who remembers past meetups has for
free. Enforcing even counts makes every fifth meetup fully determined; it measured
45.7 % against a 20 % baseline, worse than the always-A bug it replaced
(PHILOSOPHY.md section 2). Under a uniform draw from the date, past frequency
carries zero information, so the best guess is 1 in 5 and stays there.

The price is that the letters will not look even. B comes up twice in a row; some
letter goes missing for half a year. That is what random looks like and it is not
exploitable. Do not "fix" it.

**Why this is its own module.** In `mvp/tools/build_deck.py` the pure function sat
one function above `slot_for_meetup`, a wrapper that wrote the ledger - the
adjacency behind four regressions of this guardrail (sequence/run-state.md,
finding 4). Here the slot path shares a file with nothing that can read the bank,
and `popquiz.audit`, which does read it, imports this module and never the other
way round. `slot_for_meetup` is not ported under any name; the used-question
record's only writer is the release transition (T-11, G-10).

Ported from `build_deck.py:75-84` (`_rng`, verbatim) and `:162-172`
(`slot_for_day`). On a date it is identical to the MVP given that date's ISO string
(`tests/test_slot.py` compares them over two years). It is narrower in two ways,
both deliberate: it takes a `datetime.date` rather than any string, and
`n_options` must be 5.
"""

from __future__ import annotations

import hashlib
from datetime import date


def _rng(*parts):
    """Deterministic, well-mixed stream. Derived from a real hash, not from
    arithmetic on the inputs - close-together seeds fed straight into an LCG
    produced a visible B,D,B,D pattern on the MVP's first build."""
    seed = int(hashlib.blake2b("|".join(map(str, parts)).encode(), digest_size=8).hexdigest(), 16)

    def nxt(bound):
        nonlocal seed
        seed = (seed * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        return (seed >> 33) % bound

    return nxt


def slot_for_day(day: date, n_options: int = 5) -> int:
    """Where this meetup's correct answer sits, 0..4. Uniform over the five options.

    PURE, and it must stay pure. It takes the date and nothing else - no ledger,
    no counts, no previous slot, no bank state. That signature is the security
    property: a function that cannot see history cannot leak it. If you are about
    to add a parameter here, read PHILOSOPHY.md section 2 first; the audit will
    refuse it anyway.

    Keyed on the day rather than the question, so swapping tonight's question
    does not move the answer: the slot belongs to the night.

    `day` must be a `datetime.date` and not a `datetime`, which is a subclass
    whose string form carries a time and would draw a different slot for the same
    night. `n_options` is the constant 5 (SPEC G-1) and is never derived from
    data; it is a parameter only so the signature reads as the contract states it.
    """
    if type(day) is not date:
        raise TypeError(f"slot_for_day takes a datetime.date and nothing else, not {day!r}")
    if n_options != 5:
        raise ValueError(f"n_options is the constant 5 (SPEC G-1), not {n_options!r}")
    return _rng("meetup-slot", day.isoformat())(n_options)
