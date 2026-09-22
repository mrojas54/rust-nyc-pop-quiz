"""The receipt: what the room is shown about how the answer was established (SPEC 7.5).

A list of the steps that were taken, one line each, each carrying a ✓ that marks the
step as done. It does not say the answer is *verified* - "saying verified is different
than verifying" (the client, D-25). A line renders only if the record holds the step
it names, so the list can never claim more than was done (G-7, G-2).

**This module is a pure function of a `verified` record and nothing else.** No
toolchain, no filesystem, no clock, no network - `receipt_lines` reads its argument
and returns strings, which is what makes AC-13 provable: the receipt renders with
the toolchain absent, in a sixty-second suite that has no `rustc`.

The same function is needed in JavaScript for the wall and take-it-home (T-05). The
contract asks for one function and two languages cannot share one, so the cases live
as data in `bank/fixtures/receipts/` and both implementations test against the same
files. That is how the build keeps them one function rather than two that drift.

Why the machine detail is not here: the compiler line, the edition, the target triple
and the Miri configuration stay off the wall and appear on take-it-home under *How we
know* (SPEC 7.5, 13, AC-87). `receipt_detail` returns those as facts, not as
sentences - the wording is participant-facing copy on a surface T-12 renders and T-22
freezes, and this module does not author it.
"""

from __future__ import annotations

from dataclasses import dataclass

from popquiz.bank import Flags, Miri, Verified, receipt_class

# SPEC 7.5. The heading is not a step, so it is not one of the returned lines: it
# carries the machine provenance marker (AC-74), which is the renderer's business,
# and the AC-43 assertions about punctuation and claim words are about the steps.
RECEIPT_HEADING = "How we know"

# The mark that says a step was done. A *done* step, not a passed one - "Nothing ran"
# carries one too (SPEC 7.5). It is also the non-colour signal the list is read by,
# so it is never decoration (AC-40).
DONE = "✓"

# Words a receipt line may never contain, and the punctuation it may never end in
# (AC-43). Here rather than only in the test, so the rule sits beside the strings it
# governs and a later edit to the copy meets it.
FORBIDDEN_IN_A_LINE = ("verified", "established", "proves", "always", "guaranteed")
TERMINAL_PUNCTUATION = (".", "!", "?")


@dataclass(frozen=True)
class ReceiptDetail:
    """What take-it-home shows beneath the wall's list (SPEC 13, AC-87).

    Facts, not copy. For a complete record every field is what the verifier
    recorded. For a `legacy` record `target_triple` is `None` - the August batch
    recorded none and nothing is back-filled (D-16, G-2) - and
    `miri_ran_outside_verifier` is true, because that pass was run outside the repo.
    The page renders `None` as *not recorded* and the flag as the Miri row saying
    the check was run separately; those strings belong to the surface, not here.

    `ran` is false for a does-not-compile record, which has no runs, no Miri result
    and nothing to say about determinism (D-22).
    """

    compiler: str
    edition: str
    legacy: bool
    ran: bool
    target_triple: str | None = None
    flags: Flags | None = None
    miri: Miri | None = None
    compile_error_code: tuple[str, ...] | None = None
    miri_ran_outside_verifier: bool = False


def receipt_lines(verified: Verified | None) -> list[str] | None:
    """The receipt's step lines for one record, or `None` if it renders no receipt.

    Precedence is SPEC 7.5's: does-not-compile first, otherwise the list for a
    record that ran, so each record renders exactly one of the two lists and there
    is no third. A record that is none of complete, `legacy` or does-not-compile
    gets `None` and cannot be scheduled.

    A `legacy` record renders **the same list** as a complete one (D-25). Nothing
    in this function reads `legacy`, which is why that is true by construction
    rather than by a branch somebody has to keep in step.
    """
    kind = receipt_class(verified)
    if kind is None:
        return None
    assert verified is not None  # receipt_class returns None for None

    if kind == "does_not_compile":
        # Nothing ran, so there is no run count and no Miri line - both would be
        # claims about an execution that never happened (D-22).
        codes = ", ".join(verified.compile_error_code or ())
        return [
            f"{DONE} Compiler refused it",
            f"{DONE} Error {codes}",
            f"{DONE} Nothing ran",
        ]

    lines: list[str] = []
    runs = verified.runs

    # Each line is gated on the record holding the step it names. The gates are
    # separate, and stay separate, because a record that ran but whose output varied
    # must render "Compiled" without "Output never varied" - the verifier rejects
    # such a program (AC-8), and the receipt must not be the thing that hides it if
    # one ever reaches here.
    if runs is not None:
        # `stdout` may be empty: a program whose answer is a panic printed nothing
        # and still compiled and ran (SPEC 7.5).
        lines.append(f"{DONE} Compiled")
        # SPEC 7.5 gives this template verbatim, so it renders verbatim. At a count
        # of one it reads "Ran 1 times"; inventing a singular would be authoring
        # participant-facing copy, which belongs to SPEC 11 and T-22's freeze.
        lines.append(f"{DONE} Ran {runs.count} times")
        if runs.byte_identical:
            lines.append(f"{DONE} Output never varied")

    miri = verified.miri
    if miri is not None and miri.output_matched:
        # One line either way (SPEC 7.5). `clean` false is not a failure to hide: it
        # is how the record says undefined behaviour was the declared answer (AC-9).
        # Neither wording claims more than a Miri run gives - absence of UB on the
        # paths actually executed (AC-43, PHILOSOPHY 4).
        if miri.clean:
            lines.append(f"{DONE} Miri ran clean")
        else:
            lines.append(f"{DONE} Miri flagged undefined behavior")

    return lines


def receipt_detail(verified: Verified | None) -> ReceiptDetail | None:
    """The machine facts take-it-home shows beneath the list (SPEC 13, AC-87).

    `None` for a record that renders no receipt, so the two functions agree about
    which records have a *How we know* section at all.

    Nothing is substituted or inferred. A legacy record's `target_triple` comes back
    `None` because the record holds none, and `miri_ran_outside_verifier` is set
    from `legacy` because D-16 is what defines that pass as having run outside the
    verifier. The page says *not recorded*; this says nothing.
    """
    kind = receipt_class(verified)
    if kind is None:
        return None
    assert verified is not None

    return ReceiptDetail(
        compiler=verified.rustc,
        edition=verified.edition,
        legacy=verified.legacy,
        ran=kind == "ran",
        target_triple=verified.target_triple,
        flags=verified.flags,
        miri=verified.miri,
        compile_error_code=verified.compile_error_code,
        miri_ran_outside_verifier=verified.legacy and verified.miri is not None,
    )


def line_is_well_formed(line: str) -> bool:
    """Whether one rendered line obeys AC-43's two rules.

    Exported so the wall's own tests and `bank-audit` can reach the same check
    rather than each spelling out the word list. It checks the two mechanical rules
    - no terminal punctuation, none of the claim words - and **not** whether the
    line is honest about the record, which is what the fixture set is for.
    """
    lowered = line.lower()
    return not line.endswith(TERMINAL_PUNCTUATION) and not any(
        word in lowered for word in FORBIDDEN_IN_A_LINE
    )
