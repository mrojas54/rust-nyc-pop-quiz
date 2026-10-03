"""The static fallback and its host sheet (SPEC 12, D-21, D-24, AC-102).

When the room cannot run, the segment still can: the wall's seven views for one
question as **one HTML file** that opens on the organizer's laptop with no network,
stepped from the keyboard, and a plain-text **host sheet** beside it holding the
three beats and each step's words, because there is no host phone to read them from.

Three functions, and the split is the point:

* `bake(question)` - the record reduced to what the wall draws: options in the
  order given, the source, the full trace, the correct letter (`bank.correct_index`)
  and the receipt lines (`receipt.receipt_lines`, the Python twin of the room's
  `answers::receipt_lines`, G-7). The browser never derives either; it prints what
  the pipeline baked, exactly as the live wall prints what the room sends.
* `build_html(question)` - one file: the wall's own CSS and JS from `web/`, the
  vendored fonts as base64 data URIs (D-14), the bake as a JSON literal, and a
  Content-Security-Policy that forbids every request, so "no network request of
  any kind" is enforced by the browser rather than hoped for.
* `host_sheet(question)` - the beats and notes verbatim, one block per phase in
  the room's order. It adds no words: every line is affirmed text or a label
  (AC-72, AC-95).

`write_fallback` writes both beside each other in one call; `popquiz schedule`
(T-20) calls it for the scheduled question. Both files hold the answer, live on the
organizer's laptop and are never served - no room route carries either.

**The options are rendered in the order they are given.** Arranging them into the
meetup date's slot (AC-23) is the caller's job, before the room and this file both
see the question, so the two can never disagree about where the answer sits.
"""

from __future__ import annotations

import argparse
import base64
import json
import re
import sys
from collections.abc import Sequence
from pathlib import Path
from typing import Any

from popquiz.bank import BankError, Question, correct_index, load_question, receipt_class
from popquiz.receipt import RECEIPT_HEADING, receipt_lines

LETTERS = ("A", "B", "C", "D", "E")

#: Where the released wall links to: take-it-home, which SPEC 13 names `/last`
#: (the loved prototype's link, and the wall's QR golden). The host is T-09's, so
#: this is a hypothesis until it deploys; T-20 passes the real one.
DEFAULT_HOME_LINK = "https://popquiz.rustnyc.org/last"

REPO = Path(__file__).resolve().parents[3]
WEB = REPO / "web"

#: The page's files in index.html's order; the static driver last.
CSS_FILES = ("shared/tokens.css", "shared/components.css", "wall/wall.css")
JS_FILES = (
    "shared/dom.js",
    "shared/phase.js",
    "shared/check.js",
    "shared/well.js",
    "shared/trace.js",
    "shared/typemodel.js",
    "shared/copy.js",
    "wall/qr.js",
    "wall/wall.js",
    "wall/fallback/static.js",
)

#: The file forbids every request, fetch and WebSocket included. Inline style and
#: script only; fonts and images from data URIs only.
CSP = (
    "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; "
    "font-src data:; img-src data:; connect-src 'none'; base-uri 'none'; form-action 'none'"
)

# SPEC 11, the host's strings the sheet uses as labels. Copied from
# web/shared/copy.js; `test_fallback.py` holds them equal to it, so none is
# authored here.
HOST_PHASE_LABELS = {
    "idle": "before the question",
    "live": "question live",
    "closed": "answers closed",
    "split": "the split",
    "work": "walking it through",
    "reveal": "the answer",
    "released": "released",
}
PHASES = ("idle", "live", "closed", "split", "work", "reveal", "released")
BEAT_WHAT = "What happens"
BEAT_REMEMBER = "What to remember"
STEP_LABEL = "Step ‹N› of ‹M›"


# --------------------------------------------------------------------------- #
# The bake
# --------------------------------------------------------------------------- #


def _refuse(question: Question, why: str) -> BankError:
    return BankError(f"{question.id}: {why}, so it has no static fallback")


def bake(question: Question, *, home_link: str = DEFAULT_HOME_LINK) -> dict[str, Any]:
    """The record as the static wall reads it.

    Refuses what the room's `answers::load` refuses, for the same reasons: not five
    options, not exactly one *does not compile* (AC-24), no receipt, no single
    derivable correct option (G-2), a trace too short to walk (D-10), a record that
    ran whose final step names no `stdout`, or an incorrect option with no
    `why_tempting` (AC-95). A file built from any of those would show a room a
    question the room itself would not run, and a sheet with a blank middle beat.
    """
    if len(question.options) != len(LETTERS):
        raise _refuse(question, f"it has {len(question.options)} options, not five")
    dnc = sum(1 for o in question.options if o.kind == "does_not_compile")
    if dnc != 1:
        raise _refuse(question, f"it has {dnc} does-not-compile options, not exactly one")
    lines = receipt_lines(question.verified)
    if lines is None:
        raise _refuse(question, "it renders no receipt")
    correct = correct_index(question)
    if correct is None:
        raise _refuse(question, "no option matches the verified answer")
    steps = question.trace.steps
    if len(steps) < 2:
        raise _refuse(question, "its trace has fewer than two steps to walk")
    if receipt_class(question.verified) == "ran" and not question.trace.names_stdout_last():
        raise _refuse(question, "its trace's final step names no stdout")
    for i, option in enumerate(question.options):
        if i != correct and not (option.why_tempting or "").strip():
            raise _refuse(question, f"option {LETTERS[i]} has no why_tempting text")

    return {
        "id": question.id,
        "source": question.source,
        "options": [
            {"letter": letter, "text": option.text}
            for letter, option in zip(LETTERS, question.options, strict=True)
        ],
        "correct": LETTERS[correct],
        "receipt": {"heading": RECEIPT_HEADING, "lines": lines},
        "trace": [
            {
                "lines": list(step.lines),
                "focus": list(step.focus),
                "note": step.note,
                "values": [{"name": v.name, "was": v.was, "now": v.now} for v in step.values],
                "pivot": step.pivot,
            }
            for step in steps
        ],
        "home_link": home_link,
    }


# --------------------------------------------------------------------------- #
# The one file
# --------------------------------------------------------------------------- #

_CSS_COMMENT = re.compile(r"/\*.*?\*/", re.S)
_IMPORT_FONTS = re.compile(r'@import\s+url\("fonts\.css"\);')
_TTF_FALLBACK = re.compile(r',\s*url\("fonts/[^"]+\.ttf"\)\s*format\("truetype"\)')
_FONT_URL = re.compile(r'url\("fonts/([^"]+)"\)')


def _fonts_css(web: Path) -> str:
    """fonts.css with each vendored face inlined as a data URI (D-14)."""

    def inline(match: re.Match[str]) -> str:
        name = match.group(1)
        data = (web / "shared" / "fonts" / name).read_bytes()
        kind = "woff2" if name.endswith(".woff2") else "ttf"
        return f'url("data:font/{kind};base64,{base64.b64encode(data).decode("ascii")}")'

    css = (web / "shared" / "fonts.css").read_text(encoding="utf-8")
    # The single-file deck needs one copy per face, not a legacy URL fallback.
    css = _TTF_FALLBACK.sub("", css)
    out, n = _FONT_URL.subn(inline, css)
    if n == 0:
        raise BankError("web/shared/fonts.css declares no vendored font")
    return out


def _styles(web: Path) -> str:
    parts = []
    for name in CSS_FILES:
        css = (web / name).read_text(encoding="utf-8")
        if name == "shared/tokens.css":
            css, n = _IMPORT_FONTS.subn(lambda _m: _fonts_css(web), css)
            if n != 1:
                raise BankError("web/shared/tokens.css no longer imports fonts.css once")
        parts.append(_CSS_COMMENT.sub("", css))
    return "\n".join(parts)


def _script(web: Path) -> str:
    parts = []
    for name in JS_FILES:
        js = (web / name).read_text(encoding="utf-8")
        # Closing a script element early is the one way inline JS breaks a page.
        if "</script" in js.lower():
            raise BankError(f"web/{name} contains </script and cannot be inlined")
        parts.append(js)
    return "\n".join(parts)


def _json_literal(value: Any) -> str:
    """JSON safe inside a <script> element: no `</` and no `<!--` can close it."""
    return (
        json.dumps(value, ensure_ascii=False, indent=1)
        .replace("<", "\\u003c")
        .replace(" ", "\\u2028")
        .replace(" ", "\\u2029")
    )


def build_html(
    question: Question, *, home_link: str = DEFAULT_HOME_LINK, web: Path = WEB
) -> str:
    """The static fallback for one question: one self-contained HTML file."""
    baked = bake(question, home_link=home_link)
    return (
        "<!DOCTYPE html>\n"
        '<html lang="en">\n<head>\n<meta charset="utf-8">\n'
        f'<meta http-equiv="Content-Security-Policy" content="{CSP}">\n'
        '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
        "<title>Rust NYC Pop Quiz</title>\n"
        f"<style>\n{_styles(web)}\n</style>\n</head>\n"
        '<body class="wall-page">\n'
        '<div class="wall-stage"><div class="wall-wrap" id="wallWrap" data-mode="static"></div></div>\n'
        f'<script type="application/json" id="pq-static">\n{_json_literal(baked)}\n</script>\n'
        f"<script>\n{_script(web)}\n</script>\n"
        "</body>\n</html>\n"
    )


# --------------------------------------------------------------------------- #
# The host sheet
# --------------------------------------------------------------------------- #


def _step_label(at: int, m: int) -> str:
    return STEP_LABEL.replace("‹N›", str(at + 1)).replace("‹M›", str(m))


def host_sheet(question: Question) -> str:
    """The host's script with no host phone: plain text, one block per phase.

    `work` holds each walked step's label and `note` (steps 0..M-2, D-10). `reveal`
    holds the final step's note and the three beats: *What happens*, then every
    incorrect option's `why_tempting` under that option's letter and text - the
    room would pick the most-chosen one, and with no phones the host picks it from
    the room - then *What to remember*. Every other phase is its label alone.
    Nothing else: no hint, no `explains.legacy`, no sentence of the sheet's own.
    """
    baked = bake(question)  # the same refusals as the file it sits beside
    steps = question.trace.steps
    m = len(steps)
    correct = LETTERS.index(baked["correct"])

    blocks: list[list[str]] = []
    for phase in PHASES:
        block = [HOST_PHASE_LABELS[phase]]
        if phase == "work":
            for at, step in enumerate(steps[:-1]):
                block += [_step_label(at, m), step.note]
        elif phase == "reveal":
            block += [_step_label(m - 1, m), steps[-1].note]
            block += [BEAT_WHAT, question.explains.what]
            for i, option in enumerate(question.options):
                if i != correct:
                    block += [f"{LETTERS[i]} · {option.text}", option.why_tempting]
            block += [BEAT_REMEMBER, question.explains.takeaway]
        blocks.append(block)
    return "\n\n".join("\n".join(b) for b in blocks) + "\n"


# --------------------------------------------------------------------------- #
# The call T-20 makes
# --------------------------------------------------------------------------- #


def sheet_path(out: Path) -> Path:
    """`q3.html` -> `q3.host-sheet.txt`, beside it."""
    out = Path(out)
    return out.with_name(f"{out.stem}.host-sheet.txt")


def write_fallback(
    question: Question, out: Path, *, home_link: str = DEFAULT_HOME_LINK, web: Path = WEB
) -> tuple[Path, Path]:
    """Write the fallback file and its host sheet beside it. Both or neither: each
    is rendered before either is written, so a refusal leaves nothing behind."""
    out = Path(out)
    page = build_html(question, home_link=home_link, web=web)
    sheet = host_sheet(question)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(page, encoding="utf-8")
    sheet_file = sheet_path(out)
    sheet_file.write_text(sheet, encoding="utf-8")
    return out, sheet_file


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="popquiz.fallback",
        description="Write one question's static fallback file and its host sheet (SPEC 12).",
    )
    parser.add_argument("question_id")
    parser.add_argument("--out", type=Path, help="the HTML file; the sheet is written beside it")
    parser.add_argument("--bank", type=Path, default=REPO / "bank", help="the bank directory")
    parser.add_argument("--home-link", default=DEFAULT_HOME_LINK, help="the take-it-home link")
    parser.add_argument("--bake", action="store_true", help="print the bake as JSON and write nothing")
    args = parser.parse_args(argv)

    try:
        question = load_question(args.bank, args.question_id)
        if args.bake:
            sys.stdout.write(
                json.dumps(bake(question, home_link=args.home_link), ensure_ascii=False, indent=2) + "\n"
            )
            return 0
        if args.out is None:
            parser.error("--out is required unless --bake is given")
        page, sheet = write_fallback(question, args.out, home_link=args.home_link)
    except BankError as e:
        print(f"popquiz.fallback: {e}", file=sys.stderr)
        return 1
    print(page)
    print(sheet)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
