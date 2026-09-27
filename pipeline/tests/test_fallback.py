"""The static fallback and its host sheet (SPEC 12, D-24): AC-102, with AC-97 and G-7.

The half of AC-102 a hermetic suite can prove: the one file makes no network
reference of any kind and carries the vendored fonts as data URIs; the host sheet
holds every beat and every step's words verbatim, in phase order, and nothing else;
both come out of one call. The keyboard driver and the live/static parity are
`web/test/wall-static.test.js`; the file opened in a real browser with the network
blocked is the `test-full` run recorded in room/README.md.

Everything is built from `bank/questions/q3.json` and the receipt fixtures. Nothing
here writes down what a program prints (CLAUDE.md): the correct letter and the
receipt come out of `bank.correct_index` and `receipt.receipt_lines`.
"""

from __future__ import annotations

import base64
import dataclasses
import json
import pathlib
import re

import pytest

from popquiz import fallback
from popquiz.bank import BankError, Trace, load_question, verified_from_dict
from popquiz.receipt import receipt_lines

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
BANK = REPO / "bank"
WEB = REPO / "web"
RECEIPTS = BANK / "fixtures" / "receipts"
BAKE_FIXTURE = WEB / "wall" / "fixtures" / "q3-static.json"


@pytest.fixture(scope="module")
def q3():
    return load_question(BANK, "q3")


@pytest.fixture(scope="module")
def page(q3):
    return fallback.build_html(q3)


def _without_bake(html: str) -> str:
    """The file minus the baked record's literal - the one place the program, the
    options and the take-it-home link live as strings. Those are data the wall
    prints, never fetched; a `http://` inside a Rust string literal is text."""
    cut, n = re.subn(
        r'<script type="application/json" id="pq-static">.*?</script>', "", html, flags=re.S
    )
    assert n == 1, "the baked literal is missing or doubled"
    return cut


# --------------------------------------------------------------------------- #
# The one file: no network request of any kind (AC-102)
# --------------------------------------------------------------------------- #

# Every way a page names a resource on a network. `url(` and `src=`/`href=` are
# allowed only when they carry a data URI.
NETWORK = {
    "http(s) scheme": re.compile(r"https?://", re.I),
    "ws(s) scheme": re.compile(r"wss?://", re.I),
    "protocol-relative reference": re.compile(r"""(url\(\s*["']?|(src|href|action)\s*=\s*["']?)//""", re.I),
    "url() not a data URI": re.compile(r"""url\(\s*(?!["']?data:)""", re.I),
    "src/href/action attribute": re.compile(r"""\s(src|href|action|srcset|poster)\s*=""", re.I),
    "@import": re.compile(r"@import", re.I),
    "<link>": re.compile(r"<link\b", re.I),
    "<iframe>/<object>/<embed>": re.compile(r"<(iframe|object|embed)\b", re.I),
}


def test_the_file_names_no_network_resource_outside_the_baked_record(page):
    rest = _without_bake(page)
    for what, pattern in NETWORK.items():
        hits = [rest[max(0, m.start() - 50) : m.end() + 30] for m in pattern.finditer(rest)]
        assert not hits, f"{what}: {hits[:3]}"


def test_the_file_forbids_every_request_itself(page):
    """Defence in depth: a CSP that the browser enforces, so a reference the scan
    above missed still cannot leave the laptop - fetch and WebSocket included."""
    meta = re.search(r'<meta http-equiv="Content-Security-Policy" content="([^"]+)">', page)
    assert meta, "no CSP meta"
    policy = dict(
        (d.split()[0], d.split()[1:]) for d in (p.strip() for p in meta.group(1).split(";")) if d
    )
    assert policy["default-src"] == ["'none'"]
    assert policy["connect-src"] == ["'none'"]
    assert policy["font-src"] == ["data:"]
    assert policy["img-src"] == ["data:"]
    assert policy["script-src"] == ["'unsafe-inline'"]
    assert policy["style-src"] == ["'unsafe-inline'"]
    # The CSP must come before anything it governs.
    assert page.index("Content-Security-Policy") < page.index("<style>") < page.index("<script")


def test_the_vendored_fonts_are_inlined_as_data_uris(page):
    """D-14: the four faces fonts.css declares, byte for byte, and no font file
    reference left."""
    uris = re.findall(r'url\("data:font/ttf;base64,([A-Za-z0-9+/=]+)"\)', page)
    faces = re.findall(r'url\("fonts/([^"]+)"\)', (WEB / "shared" / "fonts.css").read_text())
    assert len(faces) == 4
    assert sorted(base64.b64decode(u) for u in uris) == sorted(
        (WEB / "shared" / "fonts" / f).read_bytes() for f in faces
    )
    assert "fonts/" not in _without_bake(page)


def test_the_file_carries_the_wall_code_and_marks_the_wrap_static(page):
    for name in fallback.JS_FILES:
        # The first real line of each file, to prove the whole set is inlined in order.
        assert (WEB / name).read_text(encoding="utf-8").strip() in page, name
    order = [page.index((WEB / n).read_text(encoding="utf-8").strip()) for n in fallback.JS_FILES]
    assert order == sorted(order), "the scripts are out of index.html's order"
    assert 'id="wallWrap" data-mode="static"' in page
    # two script elements: the baked literal and the code (JS comments may say "<script>")
    assert len(re.findall(r"^<script", page, flags=re.M)) == 2


def test_the_baked_record_is_the_bake_and_cannot_close_its_script(q3, page):
    literal = re.search(
        r'<script type="application/json" id="pq-static">\n(.*?)\n</script>', page, flags=re.S
    ).group(1)
    assert "<" not in literal
    assert json.loads(literal) == fallback.bake(q3)


# --------------------------------------------------------------------------- #
# The bake: the pipeline's twins, never the browser's derivation (G-7)
# --------------------------------------------------------------------------- #


def test_the_committed_bake_fixture_is_the_bake_of_q3(q3):
    """web/wall/fixtures/q3-static.json feeds the JS parity test; it is generated,
    never edited. Regenerate: `python -m popquiz.fallback q3 --bake > …`."""
    assert json.loads(BAKE_FIXTURE.read_text(encoding="utf-8")) == fallback.bake(q3)


def test_the_bake_carries_the_derived_answer_and_the_receipt_twin(q3):
    b = fallback.bake(q3)
    assert [o["letter"] for o in b["options"]] == ["A", "B", "C", "D", "E"]
    assert [o["text"] for o in b["options"]] == [o.text for o in q3.options]
    # the correct option is the one whose text is the verified output (bank.correct_index)
    stdout = q3.verified.stdout.removesuffix("\n")
    assert next(o for o in b["options"] if o["letter"] == b["correct"])["text"] == stdout
    assert b["receipt"] == {"heading": "How we know", "lines": receipt_lines(q3.verified)}
    assert len(b["trace"]) == len(q3.trace.steps)
    assert all(set(s) == {"lines", "focus", "note", "values", "pivot"} for s in b["trace"])
    # nothing the wall must never show: no explanation, no hint, no why_tempting
    text = json.dumps(b, ensure_ascii=False)
    for secret in [q3.hint, q3.explains.what, q3.explains.takeaway,
                   *(o.why_tempting for o in q3.options if o.why_tempting)]:
        assert secret not in text


def _cases():
    return sorted(p for p in RECEIPTS.glob("*.json"))


@pytest.mark.parametrize("path", _cases(), ids=lambda p: p.stem)
def test_the_baked_receipt_is_the_fixture_lines(q3, path):
    """G-7, pinned against bank/fixtures/receipts/: the bake's receipt for every
    fixture record is its `expected_lines`, and a record that renders none is
    refused rather than baked with an empty list."""
    case = json.loads(path.read_text(encoding="utf-8"))
    verified = verified_from_dict(case["verified"])
    if case.get("expected_lines") is None:
        assert receipt_lines(verified) is None
        with pytest.raises(BankError, match="renders no receipt"):
            fallback.bake(dataclasses.replace(q3, verified=verified))
        return
    # The bake has no receipt function of its own: it calls the one twin. (The
    # fixtures' synthetic stdouts match no q3 option, so the whole bake is pinned
    # on q3 above; here the function it calls is pinned on every case.)
    assert fallback.receipt_lines is receipt_lines
    assert receipt_lines(verified) == case["expected_lines"]


def test_a_record_the_room_would_refuse_has_no_fallback(q3):
    with pytest.raises(BankError, match="fewer than two steps"):
        fallback.bake(dataclasses.replace(q3, trace=Trace(steps=q3.trace.steps[:1])))
    with pytest.raises(BankError, match="no option matches"):
        fallback.bake(dataclasses.replace(
            q3, verified=dataclasses.replace(q3.verified, stdout="nothing like any option\n")))
    with pytest.raises(BankError, match="names no stdout"):
        last = dataclasses.replace(q3.trace.steps[-1], values=())
        fallback.bake(dataclasses.replace(q3, trace=Trace(steps=(*q3.trace.steps[:-1], last))))
    # AC-24: exactly one does-not-compile option
    as_output = dataclasses.replace(q3.options[3], kind="output")
    with pytest.raises(BankError, match="0 does-not-compile options"):
        fallback.bake(dataclasses.replace(q3, options=(*q3.options[:3], as_output, q3.options[4])))
    # AC-95: every incorrect option carries its why_tempting, or the sheet has a hole
    for blank in (None, "  "):
        bare = dataclasses.replace(q3.options[1], why_tempting=blank)
        with pytest.raises(BankError, match="option B has no why_tempting"):
            fallback.bake(dataclasses.replace(q3, options=(q3.options[0], bare, *q3.options[2:])))
        with pytest.raises(BankError, match="option B has no why_tempting"):
            fallback.host_sheet(dataclasses.replace(q3, options=(q3.options[0], bare, *q3.options[2:])))


# --------------------------------------------------------------------------- #
# The host sheet (AC-102, D-24): the beats and the notes, verbatim, and no more
# --------------------------------------------------------------------------- #


def _copy_js() -> dict[str, str]:
    src = (WEB / "shared" / "copy.js").read_text(encoding="utf-8")
    return dict(re.findall(r'^\s+([a-z_]+): "((?:[^"\\]|\\.)*)",?\s*$', src, flags=re.M))


def test_the_sheet_labels_are_copy_js_strings():
    """The sheet authors no label: each is SPEC 11's, as web/shared/copy.js holds it."""
    copy = _copy_js()
    for phase, label in fallback.HOST_PHASE_LABELS.items():
        assert copy[f"host_phase_{phase}"] == label
    assert copy["host_reveal_beat_what"] == fallback.BEAT_WHAT
    assert copy["host_reveal_beat_remember"] == fallback.BEAT_REMEMBER
    assert copy["wall_trace_step"] == fallback.STEP_LABEL


def _structural(line: str, q) -> bool:
    letters_and_texts = {f"{l} · {o.text}" for l, o in zip("ABCDE", q.options)}
    return (
        line in fallback.HOST_PHASE_LABELS.values()
        or line in (fallback.BEAT_WHAT, fallback.BEAT_REMEMBER)
        or re.fullmatch(r"Step \d+ of \d+", line) is not None
        or line in letters_and_texts
    )


def test_the_sheet_holds_every_beat_and_note_verbatim_in_phase_order_and_nothing_else(q3):
    sheet = fallback.host_sheet(q3)
    assert isinstance(sheet, str) and "<" not in sheet, "plain text, no markup"
    steps = q3.trace.steps
    correct = fallback.LETTERS.index(fallback.bake(q3)["correct"])
    expected = [
        *(s.note for s in steps[:-1]),          # work: 0..M-2
        steps[-1].note,                         # reveal: the resolving step
        q3.explains.what,
        *(o.why_tempting for i, o in enumerate(q3.options) if i != correct),
        q3.explains.takeaway,
    ]
    prose = [l for l in sheet.split("\n") if l and not _structural(l, q3)]
    assert prose == expected
    # no other prose: not the hint, not the legacy explanation
    assert q3.hint not in sheet
    assert q3.explains.legacy not in sheet


def test_the_sheet_has_one_block_per_phase_in_the_rooms_order(q3):
    blocks = fallback.host_sheet(q3).rstrip("\n").split("\n\n")
    assert [b.split("\n")[0] for b in blocks] == [fallback.HOST_PHASE_LABELS[p] for p in fallback.PHASES]
    m = len(q3.trace.steps)
    work = blocks[fallback.PHASES.index("work")].split("\n")
    assert work[1::2] == [f"Step {i + 1} of {m}" for i in range(m - 1)], "work walks 0..M-2 only"
    reveal = blocks[fallback.PHASES.index("reveal")].split("\n")
    assert reveal[1] == f"Step {m} of {m}", "reveal starts at the resolving step"
    for p in ("idle", "live", "closed", "split", "released"):
        assert blocks[fallback.PHASES.index(p)] == fallback.HOST_PHASE_LABELS[p]


# --------------------------------------------------------------------------- #
# One call, both files; the CLI
# --------------------------------------------------------------------------- #


def test_write_fallback_writes_the_file_and_the_sheet_beside_it(q3, tmp_path):
    page, sheet = fallback.write_fallback(q3, tmp_path / "meetup" / "q3.html")
    assert page == tmp_path / "meetup" / "q3.html"
    assert sheet == tmp_path / "meetup" / "q3.host-sheet.txt"
    assert page.read_text(encoding="utf-8") == fallback.build_html(q3)
    assert sheet.read_text(encoding="utf-8") == fallback.host_sheet(q3)


def test_a_refused_record_writes_neither_file(q3, tmp_path):
    bad = dataclasses.replace(q3, trace=Trace(steps=q3.trace.steps[:1]))
    with pytest.raises(BankError):
        fallback.write_fallback(bad, tmp_path / "q3.html")
    assert list(tmp_path.iterdir()) == []


def test_the_cli_builds_from_the_bank(tmp_path, capsys):
    out = tmp_path / "q3.html"
    assert fallback.main(["q3", "--out", str(out), "--bank", str(BANK)]) == 0
    assert out.is_file() and (tmp_path / "q3.host-sheet.txt").is_file()
    assert fallback.main(["nope", "--out", str(out), "--bank", str(BANK)]) == 1
    assert "no question 'nope'" in capsys.readouterr().err
