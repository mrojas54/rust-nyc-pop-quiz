"""The terminal preview preserves the room's explicit reveal boundary."""

from dataclasses import replace
from pathlib import Path

from popquiz.bank import Trace, load_bank
from popquiz.preview_tui import Preview, render

BANK = Path(__file__).resolve().parents[2] / "bank"


def test_question_does_not_render_explanation_or_receipt():
    state = Preview(load_bank(BANK))
    text = "\n".join(line.text for line in render(state, 90))
    assert state.question.source.splitlines()[0] in text
    assert all(option.text in text for option in state.question.options)
    assert state.question.explains.what not in text
    assert "How we know" not in text
    assert "Correct answer" not in text


def test_stepping_cannot_reveal_resolving_step():
    state = Preview(load_bank(BANK))
    state.handle("t")
    for _ in range(100):
        state.handle("right")
    assert state.step == len(state.question.trace.steps) - 2
    assert state.mode == "trace"
    text = "\n".join(line.text for line in render(state, 100))
    assert "How we know" not in text
    assert "Correct answer" not in text
    state.handle("r")
    assert state.mode == "reveal"
    assert state.step == len(state.question.trace.steps) - 1
    assert "Correct answer" in "\n".join(line.text for line in render(state, 100))


def test_returning_from_reveal_and_switching_candidate_reset_boundary():
    state = Preview(load_bank(BANK))
    state.handle("r")
    state.handle("t")
    assert state.step <= len(state.question.trace.steps) - 2
    state.handle("]")
    assert state.mode == "question"
    assert state.step == 0
    assert state.index == 1


def test_missing_or_single_step_trace_is_honest():
    question = load_bank(BANK)[0]
    for steps in ((), question.trace.steps[:1]):
        state = Preview((replace(question, trace=Trace(steps)),))
        state.handle("t")
        state.handle("right")
        text = "\n".join(line.text for line in render(state, 70))
        assert "No teaching steps" in text
        assert "Correct answer" not in text


def test_narrow_render_keeps_all_content_reachable():
    state = Preview(load_bank(BANK))
    state.handle("r")
    rows = render(state, 36)
    assert all(len(row.text) <= 36 for row in rows)
    assert any("How we know" in row.text for row in rows)


def test_preview_never_writes_bank():
    before = {p: p.read_bytes() for p in BANK.rglob("*.json")}
    state = Preview(load_bank(BANK))
    for key in ("t", "right", "left", "r", "q", "]", "[", "t"):
        state.handle(key)
        render(state, 90)
    assert all(p.read_bytes() == content for p, content in before.items())
