"""The generator (T-16, AC-1 to AC-5) on recorded fixtures. No API, no key, no network.

Every test here runs with `ANTHROPIC_API_KEY` (and the SDK's other credential and
endpoint variables) removed and with the socket layer patched to fail, so a test that
reached for the network would fail rather than quietly spend. The fixtures in
`fixtures/generate/` are hand-built in the SDK's batch result shape; none is a
recording, and none says which option is correct (see their README).
"""

from __future__ import annotations

import ast
import dataclasses
import json
import math
import socket
import sys
from pathlib import Path

import pytest

from fixtures.verify.verify_cases import RECORDINGS
from popquiz import audit, bank, copylint, generate, sandbox, verify
from popquiz.generate import GenerateError, ReplayBatchClient, RunRequest, Talk
from popquiz.runner import StubRunner

HERE = Path(__file__).parent
FIXTURES = HERE / "fixtures" / "generate"
SRC = HERE.parent / "src" / "popquiz"

TOPICS = ("ownership", "iterators", "integers")


@pytest.fixture(autouse=True)
def no_network_no_key(monkeypatch):
    """Ruling 1: nothing in this file can reach a network or find a credential."""
    for name in ("ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_BASE_URL", "ANTHROPIC_PROFILE"):
        monkeypatch.delenv(name, raising=False)

    def refuse(*args, **kwargs):
        raise AssertionError("a test in test_generate.py tried to use the network")

    monkeypatch.setattr(socket.socket, "connect", refuse)
    monkeypatch.setattr(socket.socket, "connect_ex", refuse)
    monkeypatch.setattr(socket, "create_connection", refuse)
    monkeypatch.setattr(socket, "getaddrinfo", refuse)


def replay(name: str, **kwargs) -> ReplayBatchClient:
    return ReplayBatchClient(FIXTURES / f"{name}.json", **kwargs)


def go(tmp_path, client, request=None, **kwargs):
    request = request or RunRequest(count=3, difficulty=2, topics=TOPICS)
    kwargs.setdefault("run_id", "t1")
    kwargs.setdefault("fixture", True)
    return generate.run(request, client_factory=lambda: client, out=tmp_path, **kwargs)


def emitted_files(tmp_path, run_id="t1"):
    return sorted((tmp_path / run_id).glob("*.json"))


# --------------------------------------------------------------------------- #
# Ruling 1: the default test path cannot construct the real client
# --------------------------------------------------------------------------- #


def test_the_real_client_refuses_without_a_key_before_importing_the_sdk(monkeypatch):
    monkeypatch.delitem(sys.modules, "anthropic", raising=False)
    with pytest.raises(GenerateError, match="ANTHROPIC_API_KEY is not set"):
        generate.AnthropicBatchClient(None)
    with pytest.raises(GenerateError):
        generate.AnthropicBatchClient("")
    assert "anthropic" not in sys.modules


def test_the_module_does_not_import_the_sdk():
    """`import anthropic` lives inside the real client only, so importing the
    generator - which every test here does - never loads it."""
    tree = ast.parse((SRC / "generate.py").read_text(encoding="utf-8"))
    top_level = [n for n in tree.body if isinstance(n, (ast.Import, ast.ImportFrom))]
    names = [a.name for n in top_level if isinstance(n, ast.Import) for a in n.names]
    names += [n.module or "" for n in top_level if isinstance(n, ast.ImportFrom)]
    assert not [n for n in names if n.split(".")[0] == "anthropic"]


def test_a_paid_run_with_no_key_refuses_and_writes_nothing(tmp_path, capsys):
    code = generate.main(
        ["--count", "1", "--topic", "x", "--difficulty", "2", "--cap-usd", "5", "--out", str(tmp_path), "--run-id", "k1"]
    )
    assert code == 2
    assert generate.NO_KEY in capsys.readouterr().err
    # The client is built before the state is written, so a refused client leaves no run.
    assert not (tmp_path / "k1").exists()


def test_replay_with_a_key_set_never_builds_the_real_client(tmp_path, monkeypatch):
    """Ruling 13 of the plan review: `--replay` is the only switch, and it wins."""
    monkeypatch.setenv("ANTHROPIC_API_KEY", "sk-test-not-a-key")

    def boom(*args, **kwargs):
        raise AssertionError("AnthropicBatchClient was built on a fixture run")

    monkeypatch.setattr(generate, "AnthropicBatchClient", boom)
    code = generate.main(
        ["--replay", str(FIXTURES / "clean.json"), "--count", "3", *sum((["--topic", t] for t in TOPICS), []),
         "--difficulty", "2", "--out", str(tmp_path), "--run-id", "r1"]
    )
    assert code == 0
    assert len(emitted_files(tmp_path, "r1")) == 3


# --------------------------------------------------------------------------- #
# Ruling 4: refuse first, spend never
# --------------------------------------------------------------------------- #


def test_a_paid_run_with_no_cap_refuses_before_building_a_client(tmp_path):
    built = []
    with pytest.raises(GenerateError) as err:
        generate.run(
            RunRequest(count=3, difficulty=2, topics=TOPICS),
            client_factory=lambda: built.append(1) or replay("clean"),
            out=tmp_path,
            run_id="c1",
            fixture=False,
        )
    assert str(err.value) == generate.NO_CAP
    assert built == []
    assert not (tmp_path / "c1").exists()


def test_a_paid_run_over_its_cap_refuses_before_building_a_client(tmp_path):
    request = RunRequest(count=40, difficulty=2, topics=TOPICS)
    built = []
    with pytest.raises(GenerateError) as err:
        generate.run(request, client_factory=lambda: built.append(1), out=tmp_path, run_id="c2", fixture=False, cap_usd=1.0)
    assert str(err.value).startswith("Refusing to submit: the worst case for 40 requests is $")
    assert str(err.value).endswith("over the $1.00 cap.")
    assert built == []


def test_a_paid_run_under_its_cap_submits(tmp_path):
    client = replay("clean")
    report = go(tmp_path, client, fixture=False, cap_usd=10.0)
    assert client.submit_calls == 1
    assert report["numbers"] == "measured"


def test_the_cap_boundary_is_inclusive_of_the_worst_case(tmp_path):
    """Exactly at the cap submits; a cent under refuses."""
    request = RunRequest(count=3, difficulty=2, topics=TOPICS)
    cap = generate.capacity()
    worst = generate.worst_case_usd(generate.build_requests(request, cap, generate.DEFAULT_MODEL, generate.DEFAULT_MAX_TOKENS))
    go(tmp_path, replay("clean"), fixture=False, cap_usd=worst, run_id="b1")
    with pytest.raises(GenerateError):
        go(tmp_path, replay("clean"), fixture=False, cap_usd=worst - 0.01, run_id="b2")


def test_a_fixture_run_needs_no_cap_but_honours_one_given(tmp_path):
    assert go(tmp_path, replay("clean"), run_id="f1")["worst_case_usd"] > 0
    client = replay("clean")
    with pytest.raises(GenerateError, match="over the \\$0.01 cap"):
        go(tmp_path, client, run_id="f2", cap_usd=0.01)
    assert client.submit_calls == 0


@pytest.mark.parametrize("cap", [math.nan, math.inf, 0.0, -1.0])
def test_a_cap_that_is_not_a_positive_amount_refuses(tmp_path, cap):
    """`nan` compares false with everything, so a naive `worst > cap` passes it."""
    built = []
    with pytest.raises(GenerateError, match="a spend cap is a positive amount"):
        generate.run(
            RunRequest(count=3, difficulty=2, topics=TOPICS),
            client_factory=lambda: built.append(1),
            out=tmp_path,
            run_id="n1",
            fixture=False,
            cap_usd=cap,
        )
    assert built == []


def test_the_worst_case_reads_max_tokens_from_the_requests_built():
    request = RunRequest(count=2, difficulty=2, topics=TOPICS)
    cap = generate.capacity()
    small = generate.build_requests(request, cap, generate.DEFAULT_MODEL, 1000)
    large = generate.build_requests(request, cap, generate.DEFAULT_MODEL, 2000)
    price_out = generate.LIST_PRICES_PER_MTOK[generate.DEFAULT_MODEL][1]
    delta = generate.worst_case_usd(large) - generate.worst_case_usd(small)
    assert delta == pytest.approx(2 * 1000 * price_out / 1e6 * generate.BATCH_DISCOUNT)


def test_the_worst_case_bounds_input_by_bytes_at_the_batch_discount():
    request = RunRequest(count=1, difficulty=2, topics=TOPICS)
    reqs = generate.build_requests(request, generate.capacity(), generate.DEFAULT_MODEL, 100)
    price_in, price_out = generate.LIST_PRICES_PER_MTOK[generate.DEFAULT_MODEL]
    in_bytes = len(json.dumps(reqs[0]["params"], ensure_ascii=False).encode("utf-8"))
    assert generate.worst_case_usd(reqs) == pytest.approx((100 * price_out + in_bytes * price_in) / 1e6 * 0.5)


def test_a_model_with_no_list_price_refuses(tmp_path):
    with pytest.raises(GenerateError, match="no list price for claude-made-up"):
        go(tmp_path, replay("clean"), fixture=False, cap_usd=100.0, model="claude-made-up")


def test_prices_cannot_be_set_on_the_command_line():
    """A price typed at a prompt would let the cap be argued down. There is no flag."""
    source = (SRC / "generate.py").read_text(encoding="utf-8")
    assert "--price" not in source
    assert generate.BATCH_DISCOUNT == 0.5


def test_the_default_model_and_its_price():
    """The client's 2026-10-10 ruling: Opus 5.5, at the skill's list price."""
    assert generate.DEFAULT_MODEL == "claude-opus-5-5"
    assert generate.LIST_PRICES_PER_MTOK["claude-opus-5-5"] == (4.0, 20.0)


def test_no_request_asks_for_prompt_caching():
    """No cache_control, so no cache-write premium hides outside the worst case."""
    reqs = generate.build_requests(RunRequest(count=2, difficulty=2, topics=TOPICS), generate.capacity(), "claude-opus-5-5", 100)
    assert "cache_control" not in json.dumps(reqs)


# --------------------------------------------------------------------------- #
# Ruling 5: the brief
# --------------------------------------------------------------------------- #


def built_system_prompt(tmp_path, request=None, fixture="clean"):
    client = replay(fixture)
    go(tmp_path, client, request=request)
    systems = {r["params"]["system"] for r in client.submitted}
    assert len(systems) == 1, "every request in a run carries the same brief"
    return systems.pop()


def test_the_brief_carries_the_rooms_three_capacity_facts(tmp_path):
    room = audit.Room()
    lines = math.floor(audit.READING_AREA[1] / (audit.floor_px(room) * audit.LINE_HEIGHT))
    assert lines == 7  # SPEC 5.2's worked figure at the default guess
    brief = built_system_prompt(tmp_path)
    assert f"at most {lines} lines" in brief
    assert "Each option is a single line" in brief
    assert f"No option is longer than {audit.OPTION_MAX_CHARS} characters" in brief
    assert audit.OPTION_MAX_CHARS == 29


def test_the_capacity_follows_the_configured_room():
    """A bigger screen, nearer seats: more lines. Read from the room, never fixed."""
    near = generate.capacity(audit.Room(screen_width_ft=20.0, screen_height_ft=11.25, back_row_ft=15.0))
    assert near.source_lines > generate.capacity().source_lines


def test_a_source_at_the_stated_capacity_fits_and_one_past_it_does_not():
    """The stated limits are the ones `audit.fit` holds a candidate to."""
    cap = generate.capacity()
    at = "\n".join(["x" * cap.source_width] * cap.source_lines) + "\n"
    assert audit.fit(at).fits
    assert not audit.fit(at + "y\n").fits
    assert not audit.fit("x" * (cap.source_width + 1) + "\n").fits


def test_the_brief_carries_every_copylint_pattern_from_the_module(tmp_path):
    brief = built_system_prompt(tmp_path)
    tropes = [r for r in copylint.RULES if r.lint == "trope"]
    assert len(tropes) == sum(len(v) for v in copylint.TROPES.values())
    for rule in copylint.RULES:
        assert rule.source in brief, f"{rule.id} is missing from the brief"


def test_the_brief_carries_the_style_rules(tmp_path):
    brief = built_system_prompt(tmp_path)
    for phrase in ("read aloud", "contractions", "each claim once", "no reassurance", "no flattery", "no signposting"):
        assert phrase in brief


def test_the_patterns_are_rendered_from_copylint_not_copied():
    """A copied list would survive a change to SPEC 11.1; this reads the module."""
    source = (SRC / "generate.py").read_text(encoding="utf-8")
    assert "copylint.RULES" in source
    # The short Forbidden entries ("turn to", "argu") occur inside ordinary words, so
    # only the patterns long enough to be unmistakable are looked for.
    distinctive = [r for r in copylint.RULES if len(r.source) >= 16]
    assert len(distinctive) >= len(copylint.RULES) // 2
    for rule in distinctive:
        assert rule.source not in source, f"{rule.id} is copied into generate.py"


def test_requests_use_structured_outputs_and_adaptive_thinking(tmp_path):
    client = replay("clean")
    go(tmp_path, client)
    params = client.submitted[0]["params"]
    assert params["model"] == generate.DEFAULT_MODEL
    assert params["thinking"] == {"type": "adaptive"}
    assert params["output_config"]["effort"] == "high"
    schema = params["output_config"]["format"]["schema"]
    assert params["output_config"]["format"]["type"] == "json_schema"
    assert [r["custom_id"] for r in client.submitted] == ["c01", "c02", "c03"]
    assert "tool_choice" not in params

    def closed(node):
        if isinstance(node, dict):
            if node.get("type") == "object":
                assert node.get("additionalProperties") is False
            for value in node.values():
                closed(value)

    closed(schema)


def test_the_schema_asks_for_no_answer():
    """Ruling 7: nothing in what the model returns names a correct option, an
    output, a letter or a position. The trace is asked for; its stdout row is not."""
    for talk_mode in (False, True):
        text = json.dumps(generate.output_schema(talk_mode)).lower()
        assert "trace" in text
        for word in ("correct", "answer", "stdout", "expected_output", "letter", "position", "slot"):
            assert word not in text


# --------------------------------------------------------------------------- #
# AC-3: count, topics, difficulty - honoured or reported
# --------------------------------------------------------------------------- #


def test_ac3_a_clean_run_yields_n_candidates_tagged_with_the_topics_and_difficulty(tmp_path):
    report = go(tmp_path, replay("clean"))
    files = emitted_files(tmp_path)
    assert len(files) == 3
    records = [bank.question_from_dict(json.loads(p.read_text(encoding="utf-8"))) for p in files]
    assert [q.topic for q in records] == list(TOPICS)
    assert {q.difficulty_requested for q in records} == {2}
    assert report["not_honoured"] == []
    assert all(c["honoured"] for c in report["honoured"].values())


def test_ac3_a_short_run_names_count_and_the_topic_it_missed(tmp_path):
    report = go(tmp_path, replay("one-errored"))
    assert report["not_honoured"] == ["count", "topics"]
    assert "2 of 3" in report["honoured"]["count"]["detail"]
    assert "iterators" in report["honoured"]["topics"]["detail"]
    assert report["honoured"]["difficulty"]["honoured"] is True


def test_ac3_more_topics_than_questions_is_reported(tmp_path):
    request = RunRequest(count=3, difficulty=2, topics=(*TOPICS, "lifetimes"))
    report = go(tmp_path, replay("clean"), request=request)
    assert "lifetimes" in report["honoured"]["topics"]["detail"]
    assert "topics" in report["not_honoured"]


def test_ac3_nothing_emitted_names_all_three(tmp_path):
    report = go(tmp_path, replay("over-capacity"))
    assert report["not_honoured"] == ["count", "topics", "difficulty"]


def test_ac3_the_cli_exits_1_when_something_was_not_honoured(tmp_path, capsys):
    code = generate.main(
        ["--replay", str(FIXTURES / "one-expired.json"), "--count", "3", *sum((["--topic", t] for t in TOPICS), []),
         "--difficulty", "2", "--out", str(tmp_path), "--run-id", "x1"]
    )
    assert code == 1
    out = capsys.readouterr().out
    assert "NOT count" in out and "expired" in out


@pytest.mark.parametrize(
    "request_kwargs",
    [
        {"count": 0, "difficulty": 2, "topics": ("x",)},
        {"count": 1, "difficulty": 9, "topics": ("x",)},
        {"count": 1, "difficulty": 2, "topics": ()},
        {"count": 1, "difficulty": 2, "topics": ("x",), "talk": Talk("t", "a")},
        {"count": 1, "difficulty": 2, "talk": Talk("t", " ")},
    ],
)
def test_a_malformed_request_is_refused_before_anything(tmp_path, request_kwargs):
    with pytest.raises(GenerateError):
        go(tmp_path, replay("clean"), request=RunRequest(**request_kwargs))
    assert not (tmp_path / "t1").exists()


# --------------------------------------------------------------------------- #
# Ruling 6: report, never trim
# --------------------------------------------------------------------------- #


def rows(report):
    return {r["custom_id"]: r for r in report["candidates"]}


def test_errored_and_expired_requests_are_reported_and_not_emitted(tmp_path):
    errored = rows(go(tmp_path, replay("one-errored"), run_id="e1"))
    assert errored["c02"]["status"] == "reported"
    assert errored["c02"]["reasons"] == ["errored: overloaded_error: Overloaded"]
    assert not (tmp_path / "e1" / "e1-02.json").exists()

    expired = rows(go(tmp_path, replay("one-expired"), run_id="e2"))
    assert expired["c03"]["reasons"] == ["expired"]
    assert not (tmp_path / "e2" / "e2-03.json").exists()
    assert [p.name for p in emitted_files(tmp_path, "e2")] == ["e2-01.json", "e2-02.json"]


def test_each_broken_capacity_fact_is_reported_with_its_fact(tmp_path):
    report = rows(go(tmp_path, replay("over-capacity")))
    assert report["c01"]["reasons"] == ["capacity: the source is 11 lines; the room holds 7 at the floor"]
    assert report["c02"]["reasons"] == ["capacity: option 1 is 31 characters; the most is 29"]
    assert report["c03"]["reasons"] == ["capacity: option 1 spans 2 lines; each option is one line"]
    assert emitted_files(tmp_path) == []


def test_nothing_is_trimmed_wrapped_or_joined(tmp_path, monkeypatch):
    """Mutation guard: a classifier that shortened the long option or joined the
    two-line one would emit them. They must stay reported, and nothing emitted may
    differ from what the model wrote."""
    report = go(tmp_path, replay("invalid-shape"))
    fixture = json.loads((FIXTURES / "invalid-shape.json").read_text(encoding="utf-8"))
    model_text = fixture["results"][2]["result"]["message"]["content"][1]["text"]
    written = json.loads((tmp_path / "t1" / "t1-03.json").read_text(encoding="utf-8"))
    model = json.loads(model_text)
    assert written["source"] == model["source"]
    assert [o["text"] for o in written["options"]] == [o["text"] for o in model["options"]]
    assert rows(report)["c03"]["status"] == "emitted"


def test_invalid_shapes_are_reported(tmp_path):
    report = rows(go(tmp_path, replay("invalid-shape")))
    assert any("expected exactly 1" in r for r in report["c01"]["reasons"])
    assert report["c02"]["reasons"] == [
        "cut off at max_tokens; the output is incomplete",
        "shape: the output is not valid JSON",
    ]
    assert [p.name for p in emitted_files(tmp_path)] == ["t1-03.json"]


def _candidate(**overrides):
    base = {
        "source": "fn main() {\n    println!(\"{}\", 1);\n}\n",
        "options": [
            {"text": t, "kind": k, "why_tempting": "Because."}
            for t, k in (("1", "output"), ("2", "output"), ("3", "output"), ("does not compile", "does_not_compile"), ("panics at runtime", "panic"))
        ],
        "hint": "Read it once.",
        "explains": {"what": "It prints.", "takeaway": "Printing prints."},
        "expect": "ran",
        "trace": {"steps": [
            {"lines": [2], "focus": [1, 3], "note": "One print.", "values": [], "pivot": False},
            {"lines": [3], "focus": [3, 3], "note": "The program ends.", "values": [], "pivot": False},
        ]},
    }
    base.update(overrides)
    return base


def _classify(data, talk_mode=False):
    entry = {"custom_id": "c01", "result": {"type": "succeeded", "message": {
        "content": [{"type": "text", "text": json.dumps(data)}], "stop_reason": "end_turn",
        "usage": {"input_tokens": 1, "output_tokens": 1}}}}
    outcome = generate.Outcome(custom_id="c01", id="t-01", topic=None if talk_mode else "x", difficulty=2)
    return generate.classify(entry, outcome, talk_mode, generate.capacity(), audit.Room())


def test_a_clean_candidate_is_emitted():
    assert _classify(_candidate()).emitted


@pytest.mark.parametrize(
    "mutate, reason",
    [
        (lambda c: c["options"].__setitem__(1, {"text": "x", "kind": "panic", "why_tempting": "y"}), "of kind 'panic'"),
        (lambda c: c["options"].__setitem__(slice(0, 2), [{"text": t, "kind": "ub", "why_tempting": "y"} for t in "uv"]), "of kind 'ub'"),
        (lambda c: c.__setitem__("expect", "ub"), "expect is ub"),
        (lambda c: c["options"][0].__setitem__("why_tempting", ""), "option 1 has no why_tempting"),
        (lambda c: c.__setitem__("expect", "prints 1"), "expect is"),
        (lambda c: c.__setitem__("hint", ""), "no hint"),
        (lambda c: c["options"].pop(), "4 options, expected 5"),
        (lambda c: c["options"][1].__setitem__("text", "1"), "duplicate option text"),
    ],
)
def test_shape_breaks_are_reported(mutate, reason):
    candidate = _candidate()
    mutate(candidate)
    outcome = _classify(candidate)
    assert not outcome.emitted
    assert any(reason in r for r in outcome.reasons), outcome.reasons


def test_a_refusal_is_reported():
    entry = {"custom_id": "c01", "result": {"type": "succeeded", "message": {"content": [], "stop_reason": "refusal", "usage": {"input_tokens": 5, "output_tokens": 0}}}}
    outcome = generate.classify(entry, generate.Outcome("c01", "t-01", "x", 2), False, generate.capacity(), audit.Room())
    assert outcome.reasons == ["refused (stop_reason refusal)"] and outcome.usage["input_tokens"] == 5


def test_a_missing_result_is_reported(tmp_path):
    request = RunRequest(count=4, difficulty=2, topics=TOPICS)
    report = rows(go(tmp_path, replay("clean"), request=request))
    assert report["c04"]["reasons"] == ["no result came back for this request"]


# --------------------------------------------------------------------------- #
# Ruling 7: the shape, and no model claim of output in any answer field
# --------------------------------------------------------------------------- #


def test_emitted_candidates_are_3_1_minus_verified_and_review(tmp_path):
    go(tmp_path, replay("clean"))
    for path in emitted_files(tmp_path):
        data = json.loads(path.read_text(encoding="utf-8"))
        question = bank.question_from_dict(data)  # validates, refuses unknown keys
        assert "verified" not in data and "review" not in data
        assert "correct" not in data and "expect" not in data
        assert len(question.trace.steps) >= generate.MIN_TRACE_STEPS
        assert not any(v.name == "stdout" for step in question.trace.steps for v in step.values)
        assert bank.correct_index(question) is None  # nothing to derive from: no record
        assert all(o.why_tempting for o in question.options)


def test_the_options_are_written_in_the_order_the_model_gave_them(tmp_path):
    """No shuffle, no balancing: bank order means nothing (AC-23, AC-26)."""
    go(tmp_path, replay("clean"))
    fixture = json.loads((FIXTURES / "clean.json").read_text(encoding="utf-8"))
    for i, path in enumerate(emitted_files(tmp_path)):
        model = json.loads(fixture["results"][i]["result"]["message"]["content"][1]["text"])
        assert [o["text"] for o in json.loads(path.read_text(encoding="utf-8"))["options"]] == [o["text"] for o in model["options"]]


def test_the_generator_imports_nothing_that_places_an_answer():
    for path in sorted(SRC.glob("generate*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        imported = set()
        for node in tree.body:
            if isinstance(node, ast.ImportFrom) and node.module:
                imported |= {node.module} | {f"{node.module}.{a.name}" for a in node.names}
            elif isinstance(node, ast.Import):
                imported |= {a.name for a in node.names}
        assert not imported & {"popquiz.slot", "popquiz.schedule", "popquiz.verify"}, path.name


def test_the_audit_lints_pass_over_the_generator():
    sources = audit.pipeline_sources()
    assert "generate.py" in sources
    assert audit.slot_call_violations(sources) == []
    assert audit.ledger_violations(sources) == []


def test_expect_goes_to_the_report_only(tmp_path):
    report = rows(go(tmp_path, replay("clean")))
    assert report["c01"]["expect"] == "ran"
    assert "expect" not in (tmp_path / "t1" / "t1-01.json").read_text(encoding="utf-8")


def test_quoted_spans_are_listed_for_review(tmp_path):
    report = rows(go(tmp_path, replay("clean")))
    assert "quoted_spans" in report["c01"]


# --------------------------------------------------------------------------- #
# AC-4: talk mode
# --------------------------------------------------------------------------- #


def test_ac4_talk_mode_tags_each_candidate_with_a_named_std_or_core_concept(tmp_path):
    talk = Talk(
        title="Moving out: what std::mem does when you are not looking",
        abstract="A tour of std::mem::take, replace and swap, and of the order values are dropped in.",
    )
    client = replay("talk")
    report = go(tmp_path, client, request=RunRequest(count=2, difficulty=3, talk=talk))
    assert talk.title in client.submitted[0]["params"]["messages"][0]["content"]
    assert "concept" in client.submitted[0]["params"]["output_config"]["format"]["schema"]["properties"]
    concepts = [r["concept"] for r in report["candidates"]]
    assert concepts == [{"name": "std::mem::take", "kind": "std"}, {"name": "drop order", "kind": "core_language"}]
    topics = [json.loads(p.read_text(encoding="utf-8"))["topic"] for p in emitted_files(tmp_path)]
    assert topics == ["std::mem::take", "drop order"]
    assert report["honoured"]["topics"]["honoured"] is None


def test_ac4_a_talk_candidate_without_a_real_std_path_is_reported():
    outcome = _classify(_candidate(concept={"name": "moving things", "kind": "std"}), talk_mode=True)
    assert not outcome.emitted and "not a std/core/alloc path" in outcome.reasons[0]
    outcome = _classify(_candidate(), talk_mode=True)
    assert outcome.reasons == ["talk: no named std or core-language concept"]


# --------------------------------------------------------------------------- #
# AC-5 / ruling 10: tokens, cost and wall-clock per candidate
# --------------------------------------------------------------------------- #


def test_ac5_the_report_carries_per_candidate_tokens_cost_and_wall_clock(tmp_path):
    report = go(tmp_path, replay("clean"))
    assert report["numbers"] == "fixture" and "FIXTURE NUMBERS" in report["numbers_note"]
    assert report["prices"] == {"input_per_mtok": 4.0, "output_per_mtok": 20.0, "basis": "list-price estimate", "batch_discount": 0.5}
    first = rows(report)["c01"]
    assert (first["input_tokens"], first["output_tokens"]) == (2100, 1800)
    assert first["cost_usd"] == pytest.approx((2100 * 4 + 1800 * 20) / 1e6 * 0.5)
    assert all("wall_clock_s" in r for r in report["candidates"])
    totals = report["totals"]
    assert totals["input_tokens"] == 2100 + 2050 + 2080
    assert totals["cost_usd"] == pytest.approx(sum(r["cost_usd"] for r in report["candidates"]))


def test_ac5_reported_candidates_still_carry_what_they_cost(tmp_path):
    report = rows(go(tmp_path, replay("invalid-shape")))
    assert report["c02"]["output_tokens"] == 16000 and report["c02"]["cost_usd"] > 0


def test_ac5_wall_clock_runs_from_submit_to_seen_ended(tmp_path):
    ticks = iter([100.0, 107.5])
    report = go(tmp_path, replay("clean"), clock=lambda: next(ticks))
    assert report["totals"]["wall_clock_s"] == 7.5
    assert {r["wall_clock_s"] for r in report["candidates"]} == {7.5}


def test_ac5_cache_tokens_are_costed_if_a_result_reports_them():
    usage = {"input_tokens": 0, "output_tokens": 0, "cache_creation_input_tokens": 1_000_000, "cache_read_input_tokens": 1_000_000}
    assert generate.cost_usd(usage, "claude-opus-5-5") == pytest.approx((4 * 1.25 + 4 * 0.1) * 0.5)


# --------------------------------------------------------------------------- #
# AC-2 / ruling 9: killed mid-run, re-run with no cleanup
# --------------------------------------------------------------------------- #
#
# A real process kill is not available here: `test_runner.py` forbids a test from
# starting a process. So the kill is a KeyboardInterrupt raised at the worst moments
# - halfway through writing a candidate's bytes, after a candidate is written but
# before it is renamed into place, and while the batch's results are being read -
# and the re-run happens in the same process, on the same directory, untouched.


class Killed(KeyboardInterrupt):
    pass


def assert_no_partial_record(run_dir: Path):
    for path in run_dir.rglob("*.json"):
        json.loads(path.read_text(encoding="utf-8"))  # every visible file is whole
        if path.parent == run_dir:
            bank.question_from_dict(json.loads(path.read_text(encoding="utf-8")))


def rerun_completes(tmp_path):
    again = replay("clean")
    report = go(tmp_path, again)
    assert again.submit_calls == 0, "a resumed run never submits a second batch"
    assert report["resumed"] is True
    assert len(emitted_files(tmp_path)) == 3
    assert not list((tmp_path / "t1").rglob("*.partial"))
    assert (tmp_path / "t1" / "_run" / "report.json").is_file()
    return report


def test_ac2_killed_halfway_through_writing_a_candidate(tmp_path, monkeypatch):
    real = generate._write_bytes
    writes = []

    def half_then_die(handle, data):
        writes.append(handle.name)
        if handle.name.endswith(".t1-02.json.partial"):
            handle.write(data[: len(data) // 2])
            raise Killed
        real(handle, data)

    monkeypatch.setattr(generate, "_write_bytes", half_then_die)
    with pytest.raises(Killed):
        go(tmp_path, replay("clean"))
    run_dir = tmp_path / "t1"
    assert [p.name for p in emitted_files(tmp_path)] == ["t1-01.json"]
    assert (run_dir / ".t1-02.json.partial").exists()
    assert not (run_dir / "_run" / "report.json").exists()
    assert_no_partial_record(run_dir)

    monkeypatch.setattr(generate, "_write_bytes", real)
    rerun_completes(tmp_path)


def test_ac2_killed_after_writing_before_the_rename(tmp_path, monkeypatch):
    real = generate._commit

    def die_on_third(partial, path):
        if path.name == "t1-03.json":
            raise Killed
        real(partial, path)

    monkeypatch.setattr(generate, "_commit", die_on_third)
    with pytest.raises(Killed):
        go(tmp_path, replay("clean"))
    assert_no_partial_record(tmp_path / "t1")
    monkeypatch.setattr(generate, "_commit", real)
    rerun_completes(tmp_path)


def test_ac2_killed_while_reading_the_batchs_results(tmp_path):
    class DiesMidResults(ReplayBatchClient):
        def results(self, batch_id):
            for i, entry in enumerate(super().results(batch_id)):
                if i == 1:
                    raise Killed
                yield entry

    with pytest.raises(Killed):
        go(tmp_path, DiesMidResults(FIXTURES / "clean.json"))
    assert emitted_files(tmp_path) == []
    rerun_completes(tmp_path)


def test_ac2_killed_while_the_batch_is_in_flight(tmp_path):
    """Interrupted during the wait (a replay with a delay): the batch id is already
    on disk, so the re-run picks the batch up rather than paying for another."""

    def interrupted(seconds):
        raise Killed

    class StillRunning(ReplayBatchClient):
        def poll(self, batch_id):
            return "in_progress"

    with pytest.raises(Killed):
        go(tmp_path, StillRunning(FIXTURES / "clean.json"), sleep=interrupted)
    state = json.loads((tmp_path / "t1" / "_run" / "state.json").read_text(encoding="utf-8"))
    assert state["batch_id"] == "msgbatch_fixture_clean" and state["status"] == "submitted"
    rerun_completes(tmp_path)


def test_ac2_a_run_that_stopped_while_submitting_refuses_to_submit_again(tmp_path):
    class DiesInSubmit(ReplayBatchClient):
        def submit(self, requests):
            raise Killed

    with pytest.raises(Killed):
        go(tmp_path, DiesInSubmit(FIXTURES / "clean.json"))
    again = replay("clean")
    with pytest.raises(GenerateError, match="a batch may already exist"):
        go(tmp_path, again)
    assert again.submit_calls == 0


def test_ac2_a_rerun_with_a_different_request_is_refused(tmp_path):
    go(tmp_path, replay("clean"))
    with pytest.raises(GenerateError, match="different request"):
        go(tmp_path, replay("clean"), request=RunRequest(count=3, difficulty=3, topics=TOPICS))


def test_ac2_a_fixture_run_id_never_resumes_as_a_paid_run_nor_the_reverse(tmp_path):
    go(tmp_path, replay("clean"))
    paid = replay("clean")
    with pytest.raises(GenerateError, match="different request"):
        go(tmp_path, paid, fixture=False, cap_usd=10.0)
    assert paid.submit_calls == 0

    go(tmp_path, replay("clean"), fixture=False, cap_usd=10.0, run_id="p1")
    with pytest.raises(GenerateError, match="different request"):
        go(tmp_path, replay("clean"), run_id="p1")


def test_candidates_are_never_written_into_bank_questions(tmp_path, monkeypatch):
    questions = tmp_path / "bank" / "questions"
    monkeypatch.setattr(generate, "QUESTIONS_DIR", questions)
    for out in (questions, questions / "sub"):
        with pytest.raises(GenerateError, match="bank/questions"):
            generate.run(
                RunRequest(count=3, difficulty=2, topics=TOPICS),
                client_factory=lambda: replay("clean"),
                out=out,
                run_id="q1",
                fixture=True,
            )
    assert not questions.exists()


def test_ac2_a_finished_run_rerun_changes_nothing(tmp_path):
    first = go(tmp_path, replay("clean"))
    before = {p.name: p.read_bytes() for p in emitted_files(tmp_path)}
    second = rerun_completes(tmp_path)
    assert {p.name: p.read_bytes() for p in emitted_files(tmp_path)} == before
    assert first["candidates"] == second["candidates"]


def test_the_default_destination_is_a_run_directory_under_bank_candidates():
    assert generate.DEFAULT_OUT == audit.DEFAULT_REPO / "bank" / "candidates"
    assert "questions" not in generate.DEFAULT_OUT.parts


@pytest.mark.parametrize("run_id", ["../escape", "Q3", "a/b", ""])
def test_a_run_id_cannot_name_a_path(tmp_path, run_id):
    with pytest.raises(GenerateError):
        go(tmp_path, replay("clean"), run_id=run_id)


# --------------------------------------------------------------------------- #
# Option b: the model drafts the trace; the verifier writes its stdout row
# --------------------------------------------------------------------------- #

TRACE_REQUEST = RunRequest(count=4, difficulty=2, topics=TOPICS)
MODEL_CLAIM = "MODEL-CLAIMED-OUTPUT"  # what c01's model offered as stdout


def _trace_run(tmp_path):
    report = go(tmp_path, replay("trace-drafts"), request=TRACE_REQUEST)
    rows = {row["custom_id"]: row for row in report["candidates"]}
    return report, rows


def _stdout_rows(question):
    return [v for step in question.trace.steps for v in step.values if v.name.strip().lower() == "stdout"]


def test_a_drafted_trace_is_emitted_with_no_stdout_and_the_discard_is_reported(tmp_path):
    """b1, b2: the steps the model drafted are kept; the stdout value it offered (named
    ' Stdout ') is discarded, reported, and written nowhere - not the candidate, not
    the report."""
    report, rows = _trace_run(tmp_path)
    assert rows["c01"]["status"] == "emitted"
    path = Path(rows["c01"]["path"])
    question = bank.question_from_dict(json.loads(path.read_text(encoding="utf-8")))

    assert len(question.trace.steps) == 4
    assert [s.note for s in question.trace.steps][0] == "Seven elements go in."
    assert sum(s.pivot for s in question.trace.steps) == 1
    assert _stdout_rows(question) == []
    assert MODEL_CLAIM not in path.read_text(encoding="utf-8")
    assert MODEL_CLAIM not in json.dumps(report)
    assert rows["c01"]["discarded"] == ["step 4: a stdout value, which only the verifier writes"]


def test_a_does_not_compile_draft_has_no_stdout_row(tmp_path):
    _, rows = _trace_run(tmp_path)
    assert rows["c02"]["status"] == "emitted"
    question = bank.question_from_dict(json.loads(Path(rows["c02"]["path"]).read_text(encoding="utf-8")))
    assert len(question.trace.steps) == 2
    assert _stdout_rows(question) == []
    assert "discarded" not in rows["c02"]


def test_a_trace_too_short_or_off_the_source_is_reported_not_trimmed(tmp_path):
    """SPEC 7.4's two steps, and steps that point at real lines: reported, never padded
    or clipped into shape."""
    _, rows = _trace_run(tmp_path)
    assert rows["c03"]["status"] == "reported"
    assert any("1 step(s); a question needs at least 2" in r for r in rows["c03"]["reasons"])
    assert rows["c04"]["status"] == "reported"
    assert any("step 2: points at a line outside the source's 5" in r for r in rows["c04"]["reasons"])
    assert len(emitted_files(tmp_path)) == 2


def test_the_brief_tells_the_model_the_output_is_not_its_to_write():
    for talk_mode in (False, True):
        prompt = generate.system_prompt(generate.capacity(), talk_mode)
        assert "Never name `stdout` in `values`" in prompt
        assert "never state or quote what the program prints in a note" in prompt
        assert "At least two steps" in prompt


@pytest.mark.parametrize(("custom_id", "program", "expect"), [("c01", "q3", "ran"), ("c02", "q8", "does_not_compile")])
def test_the_verifier_fills_the_drafted_trace_from_what_ran(tmp_path, custom_id, program, expect):
    """b3, b6: a generated candidate, verified on the stub Runner's recordings, gets its
    stdout row from the record - or none, if it did not run. The fixtures reuse the
    bank's q3 and q8 sources verbatim, so the recordings are of the program the
    candidate holds; only the generated id is swapped for the recording's."""
    _, rows = _trace_run(tmp_path)
    drafted = bank.question_from_dict(json.loads(Path(rows[custom_id]["path"]).read_text(encoding="utf-8")))
    candidate = dataclasses.replace(drafted, id=program)
    verdict = verify.verify(candidate, StubRunner(RECORDINGS), expect=expect, pin=sandbox.read_pin())
    assert verdict.accepted, verdict.reason

    after = verify.with_record(candidate, verdict)

    assert bank.question_from_dict(bank.question_to_dict(after)) == after
    if expect == "ran":
        recorded = json.loads((RECORDINGS / f"{program}.json").read_text(encoding="utf-8"))["run:1"]["stdout"]
        assert _stdout_rows(after) == [bank.ValueDelta("stdout", "—", bank.normalized_output(recorded))]
        assert after.trace.steps[-1].values[-1] == _stdout_rows(after)[0]
    else:
        assert _stdout_rows(after) == []
    assert after.trace.steps[:-1] == drafted.trace.steps[:-1]
    assert MODEL_CLAIM not in json.dumps(bank.question_to_dict(after))
