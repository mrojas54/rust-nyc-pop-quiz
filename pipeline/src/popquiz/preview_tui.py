"""Read-only organizer prototype: question, teaching trace, explicit reveal."""

from __future__ import annotations

import argparse
import curses
import sys
import textwrap
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from popquiz.bank import BankError, Question, correct_index, load_bank
from popquiz.receipt import receipt_lines

Mode = Literal["question", "trace", "reveal"]
Style = Literal["normal", "heading", "active", "dim"]


@dataclass(frozen=True)
class Row:
    text: str
    style: Style = "normal"


@dataclass
class Preview:
    questions: tuple[Question, ...]
    index: int = 0
    mode: Mode = "question"
    step: int = 0

    def __post_init__(self) -> None:
        if not self.questions:
            raise ValueError(
                "No candidates found. Choose a bank with question JSON files."
            )

    @property
    def question(self) -> Question:
        return self.questions[self.index]

    def handle(self, key: str) -> None:
        total = len(self.question.trace.steps)
        if key == "1":
            self.mode = "question"
        elif key in ("t", "\t"):
            self.mode = "question" if self.mode == "trace" else "trace"
            self.step = min(self.step, max(0, total - 2))
        elif key == "r":
            self.mode = "reveal"
            self.step = max(0, total - 1)
        elif key in ("left", "right") and self.mode != "question":
            maximum = max(0, total - (2 if self.mode == "trace" else 1))
            delta = 1 if key == "right" else -1
            self.step = max(0, min(maximum, self.step + delta))
        elif key in ("[", "]"):
            delta = 1 if key == "]" else -1
            self.index = (self.index + delta) % len(self.questions)
            self.mode = "question"
            self.step = 0


def render(state: Preview, width: int) -> list[Row]:
    """Create scrollable rows; the last teaching step is reserved for reveal."""
    width = max(1, width)
    rows: list[Row] = []

    def add(text: str = "", style: Style = "normal") -> None:
        # Keep terminal control characters in authored content inert.
        safe = "".join(c if c.isprintable() or c == "\n" else " " for c in text)
        for paragraph in safe.split("\n"):
            for line in textwrap.wrap(
                paragraph, width, replace_whitespace=False, drop_whitespace=False
            ) or [""]:
                rows.append(Row(line, style))

    question = state.question
    add(
        f"{question.id} / {question.topic}   Candidate {state.index + 1} of {len(state.questions)}",
        "heading",
    )
    add(
        "  ".join(
            f"[{m.title()}]" if state.mode == m else m.title()
            for m in ("question", "trace", "reveal")
        )
    )
    add()
    steps = question.trace.steps
    step = None
    if state.mode == "reveal" and steps or state.mode == "trace" and len(steps) >= 2:
        step = steps[state.step]
    add(
        "What happens when this program runs?"
        if state.mode == "question"
        else "Let's walk it.",
        "heading",
    )
    add()
    for number, source in enumerate(question.source.splitlines(), 1):
        active = step is not None and number in step.lines
        style: Style = "active" if active else "normal"
        if step and not active and not step.focus[0] <= number <= step.focus[1]:
            style = "dim"
        prefix = f"{'>' if active else ' '} {number:2}  "
        # Continuations are visibly indented rather than truncating long code.
        chunks = textwrap.wrap(
            source.expandtabs(4),
            max(1, width - len(prefix)),
            replace_whitespace=False,
            drop_whitespace=False,
        ) or [""]
        for n, chunk in enumerate(chunks):
            add((prefix if n == 0 else " " * len(prefix)) + chunk, style)
    add()
    if state.mode == "question":
        add("Choices / bank order, not meetup letter assignment", "dim")
        for index, option in enumerate(question.options):
            add(f"{chr(65 + index)}  {option.text}")
        add()
        add("Press t to walk through the teaching trace.", "heading")
    elif step:
        add(
            f"Step {state.step + 1} of {len(steps)}"
            + (" / Pause here." if step.pivot else ""),
            "heading",
        )
        add("Host narration / authored teaching walkthrough", "dim")
        add(step.note)
        if step.values:
            add()
            add("Illustrated values / authored, not debugger captures", "dim")
            for value in step.values:
                add(f"{value.name}: {value.was} -> {value.now}")
        if state.mode == "trace" and state.step == len(steps) - 2:
            add()
            add("End of teaching steps. Press r to preview the reveal.", "heading")
    else:
        add("No teaching steps available before reveal. At least two steps are needed.")
    if state.mode == "reveal":
        add()
        try:
            correct = correct_index(question)
            answer = (
                question.options[correct].text
                if correct is not None
                else "Unavailable from the recorded evidence"
            )
        except BankError as error:
            answer = str(error)
        add(f"Correct answer / {answer}", "heading")
        add(
            "From the existing verification record; no new verification was run.", "dim"
        )
        add()
        add("What happened", "heading")
        add(question.explains.what)
        add()
        add("What to remember", "heading")
        add(question.explains.takeaway)
        add()
        add("How we know / recorded evidence", "heading")
        for line in receipt_lines(question.verified) or [
            "No complete receipt recorded"
        ]:
            add(line)
        if question.verified and question.verified.legacy:
            add("Legacy record: Miri was run outside the verifier.")
        add("Miri evidence covers executed paths only.", "dim")
        add()
        add("Why the other choices are tempting", "heading")
        for option in question.options:
            if option.why_tempting:
                add(option.text, "heading")
                add(option.why_tempting)
                add()
    return rows


def run(screen: curses.window, state: Preview) -> None:
    try:
        curses.curs_set(0)
    except curses.error:
        pass
    screen.keypad(True)
    offset = 0
    styles = {
        "normal": curses.A_NORMAL,
        "heading": curses.A_BOLD,
        "active": curses.A_REVERSE | curses.A_BOLD,
        "dim": curses.A_DIM,
    }

    def write(y: int, text: str, style: int = 0) -> None:
        height, width = screen.getmaxyx()
        if 0 <= y < height and width > 1:
            try:
                screen.addnstr(y, 0, text, width - 1, style)
            except curses.error:
                pass  # Resize or a wide glyph at the terminal edge.

    while True:
        screen.erase()
        height, width = screen.getmaxyx()
        if height < 12 or width < 36:
            write(0, "Resize to at least 36 x 12. q quits.")
        else:
            rows = render(state, width - 2)
            available = height - 5
            offset = min(offset, max(0, len(rows) - available))
            write(
                0,
                "RUST NYC / ORGANIZER PREVIEW                         PROTOTYPE",
                curses.A_BOLD,
            )
            write(
                1,
                "Read-only / review exposes answers / bank and used ledger unchanged",
                curses.A_DIM,
            )
            for y, row in enumerate(rows[offset : offset + available], 3):
                write(y, row.text, styles[row.style])
            write(
                height - 2,
                "1 Question  t Trace  r Reveal  <- -> Step  [ ] Candidate  q Quit",
            )
            write(
                height - 1,
                f"Up/Down or j/k: scroll / PgUp/PgDn / lines {offset + 1}-{min(offset + available, len(rows))} of {len(rows)}",
                curses.A_DIM,
            )
        screen.refresh()
        key = screen.get_wch()
        if key in ("q", "\x1b"):
            return
        if key in (curses.KEY_DOWN, "j"):
            offset += 1
        elif key in (curses.KEY_UP, "k"):
            offset = max(0, offset - 1)
        elif key == curses.KEY_NPAGE:
            offset += max(1, height - 5)
        elif key == curses.KEY_PPAGE:
            offset = max(0, offset - max(1, height - 5))
        else:
            action = (
                {curses.KEY_LEFT: "left", curses.KEY_RIGHT: "right"}.get(key)
                if isinstance(key, int)
                else key
            )
            if action is not None:
                before = (state.mode, state.step, state.index)
                state.handle(action)
                if before != (state.mode, state.step, state.index):
                    offset = 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--bank", type=Path, default=Path(__file__).resolve().parents[3] / "bank"
    )
    parser.add_argument("--question", help="Open this candidate first")
    parser.add_argument(
        "--snapshot",
        choices=("question", "trace", "reveal"),
        help="Print a view without a terminal",
    )
    parser.add_argument(
        "--step", type=int, default=1, help="1-based trace step for a snapshot"
    )
    args = parser.parse_args()
    try:
        state = Preview(load_bank(args.bank))
        if args.question:
            state.index = next(
                i
                for i, question in enumerate(state.questions)
                if question.id == args.question
            )
        if args.snapshot:
            if args.snapshot != "question":
                state.handle("r" if args.snapshot == "reveal" else "t")
            if args.snapshot == "trace":
                state.step = max(
                    0, min(args.step - 1, len(state.question.trace.steps) - 2)
                )
            print("PROTOTYPE / READ-ONLY / existing bank content")
            print("\n".join(row.text for row in render(state, 90)))
        else:
            if not sys.stdin.isatty() or not sys.stdout.isatty():
                parser.error(
                    "Open an interactive terminal, or use --snapshot question."
                )
            curses.wrapper(run, state)
    except (BankError, OSError, ValueError, StopIteration) as error:
        parser.error(str(error) or "The requested candidate was not found.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
