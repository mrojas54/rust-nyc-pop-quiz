"""The claude-cli backend (PQ-69) against a fake `claude`. No model, no key, no network.

Every test runs `hermetic` (`tests/conftest.py`): `PATH=/usr/bin:/bin`, every
billing variable removed, no network. A test that wants the CLI puts a fake on
`PATH` itself, in its own `tmp_path` (`fixtures/generate/claude/fake_claude.py`).
The real `claude` is out of reach from every test in every directory: the root
conftest's guard fails a test that builds a client on it or starts it. The fake's
answers are the documented `--output-format json` shape, not a recording; see the
fixtures' README.
"""

from __future__ import annotations

import ast
import io
import json
import os
import signal
import stat
import subprocess
import sys
from pathlib import Path

import pytest

from popquiz import generate, generate_cli
from popquiz.generate import Factory, GenerateError, RunRequest
from popquiz.generate_cli import ClaudeCliClient

HERE = Path(__file__).parent
FAKE = HERE / "fixtures" / "generate" / "claude"
SRC = HERE.parent / "src" / "popquiz"
TOPICS = ("ownership", "iterators", "integers")
SENTINEL = "sk-test-not-a-key-0000"  # never a real key; the test looks for it in every file
SAFE_PATH = "/usr/bin:/bin"  # what `hermetic` sets; the fake goes in front of it

pytestmark = pytest.mark.usefixtures("hermetic")


class Fake:
    """A fake `claude` on PATH, its script, and what it recorded."""

    def __init__(self, tmp_path: Path, monkeypatch) -> None:
        self.bin = tmp_path / "fakebin"
        self.log = tmp_path / "fakelog"
        self.bin.mkdir()
        self.log.mkdir()
        self.exe = self.bin / "claude"
        self.exe.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{FAKE / "fake_claude.py"}" "$@"\n', encoding="utf-8")
        self.exe.chmod(self.exe.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        self._mp = monkeypatch
        monkeypatch.setenv("PATH", f"{self.bin}:{SAFE_PATH}")
        monkeypatch.setenv("FAKE_CLAUDE_LOG", str(self.log))

    def script(self, name: str) -> None:
        self._mp.setenv("FAKE_CLAUDE_SCRIPT", str(FAKE / f"{name}.json"))

    def calls(self) -> list[dict]:
        return [json.loads(p.read_text(encoding="utf-8")) for p in sorted(self.log.glob("*.json"))]


@pytest.fixture
def fake(tmp_path, monkeypatch) -> Fake:
    return Fake(tmp_path, monkeypatch)


def cli_run(tmp_path, *, run_id="t1", request=None, max_candidates=3, built=None, timeout_s=60.0, **kwargs):
    request = request or RunRequest(count=3, difficulty=2, topics=TOPICS)
    out = tmp_path / "out"
    clients = built if built is not None else []

    def build():
        client = ClaudeCliClient(None, out / run_id / generate.RUN_SUBDIR / generate.CLI_SUBDIR, timeout_s=timeout_s)
        clients.append(client)
        return client

    report = generate.run(
        request, client_factory=Factory(ClaudeCliClient, build), out=out, run_id=run_id,
        max_candidates=max_candidates, **kwargs,
    )
    return report, clients[-1]


def rows(report):
    return {r["custom_id"]: r for r in report["candidates"]}


def emitted(tmp_path, run_id="t1"):
    return sorted(p.name for p in (tmp_path / "out" / run_id).glob("*.json"))


# --------------------------------------------------------------------------- #
# Ruling 7: hermetic - the real binary is out of reach
# --------------------------------------------------------------------------- #


def test_without_a_fake_the_client_finds_no_claude_and_refuses(tmp_path):
    with pytest.raises(GenerateError) as err:
        ClaudeCliClient(None, tmp_path)
    assert str(err.value) == generate_cli.NO_CLAUDE


GUARD = "the real claude guard"


def test_the_guard_refuses_a_client_given_the_real_claude(tmp_path, claude_decoy):
    with pytest.raises(pytest.fail.Exception, match=GUARD):
        ClaudeCliClient(str(claude_decoy.exe), tmp_path)


def test_the_guard_refuses_a_client_that_finds_the_real_claude_on_path(tmp_path, claude_decoy, monkeypatch):
    monkeypatch.setenv("PATH", f"{claude_decoy.dir}:{SAFE_PATH}")
    with pytest.raises(pytest.fail.Exception, match=GUARD):
        ClaudeCliClient(None, tmp_path)


def test_the_guard_stops_a_generator_run_on_the_real_claude(tmp_path, claude_decoy):
    with pytest.raises(pytest.fail.Exception, match=GUARD):
        generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3",
                               "--claude-bin", str(claude_decoy.exe)))
    assert not claude_decoy.ran()


@pytest.mark.parametrize("found_by", ["--claude-bin", "PATH"])
def test_the_guard_refuses_to_start_a_generator_process_on_the_real_claude(tmp_path, claude_decoy, found_by):
    command = [sys.executable, "-m", "popquiz.generate",
               *argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3")]
    env = {"PATH": SAFE_PATH}
    if found_by == "--claude-bin":
        command += ["--claude-bin", str(claude_decoy.exe)]
    else:
        env["PATH"] = f"{claude_decoy.dir}:{SAFE_PATH}"
    with pytest.raises(pytest.fail.Exception, match=GUARD):
        subprocess.run(command, env=env, capture_output=True, check=False)
    assert not claude_decoy.ran()


def test_the_guard_refuses_to_start_the_real_claude_itself(claude_decoy):
    with pytest.raises(pytest.fail.Exception, match=GUARD):
        subprocess.run([str(claude_decoy.exe), "-p"], check=False)
    assert not claude_decoy.ran()


def test_an_explicit_binary_that_is_not_executable_refuses(tmp_path):
    path = tmp_path / "claude"
    path.write_text("not a program", encoding="utf-8")
    with pytest.raises(GenerateError, match="is not an executable file"):
        ClaudeCliClient(str(path), tmp_path)


def test_importing_the_client_starts_nothing():
    """The module is allowed to start processes (`test_runner.MAY_START_A_PROCESS`),
    so prove what `just test` needs: at import it only defines things."""
    tree = ast.parse((SRC / "generate_cli.py").read_text(encoding="utf-8"))
    allowed = (ast.Import, ast.ImportFrom, ast.Assign, ast.AnnAssign, ast.FunctionDef, ast.ClassDef)
    for node in tree.body:
        is_docstring = isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant)
        assert isinstance(node, allowed) or is_docstring, ast.dump(node)[:80]
    for node in tree.body:
        if isinstance(node, (ast.Assign, ast.AnnAssign)):
            calls = [n for n in ast.walk(node) if isinstance(n, ast.Call)]
            assert all(isinstance(c.func, ast.Name) and c.func.id == "frozenset" for c in calls), ast.dump(node)[:80]


def test_the_child_is_started_from_an_argument_list_never_a_shell():
    tree = ast.parse((SRC / "generate_cli.py").read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            assert not any(k.arg == "shell" for k in node.keywords)
    source = (SRC / "generate_cli.py").read_text(encoding="utf-8")
    assert "os.system" not in source and "shell=" not in source


# --------------------------------------------------------------------------- #
# Rulings 2, 3, 6: what the child is given
# --------------------------------------------------------------------------- #

FLAGS = ("--tools", "--setting-sources", "--strict-mcp-config", "--safe-mode", "--disable-slash-commands", "--no-session-persistence")


def test_the_child_gets_the_brief_the_schema_and_no_tools_settings_or_key(tmp_path, fake, monkeypatch):
    """The recorder fixture: the parent holds a key and every other billing switch;
    the child sees none of them, by name, and only what the run asked for."""
    for name in ("ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_BASE_URL", "ANTHROPIC_CUSTOM_HEADERS",
                 "ANTHROPIC_MODEL", "CLAUDE_CODE_USE_BEDROCK", "CLAUDE_CODE_USE_VERTEX", "CLAUDE_CODE_USE_FOUNDRY",
                 "AWS_BEARER_TOKEN_BEDROCK"):
        monkeypatch.setenv(name, SENTINEL)
    monkeypatch.setenv("CLAUDE_CODE_OAUTH_TOKEN", "subscription-token-placeholder")
    monkeypatch.setenv("POPQUIZ_UNRELATED", "1")
    fake.script("cli-records-argv-and-env")
    report, client = cli_run(tmp_path)
    assert report["totals"]["emitted"] == 3

    calls = fake.calls()
    assert len(calls) == 3
    request = RunRequest(count=3, difficulty=2, topics=TOPICS)
    reqs = generate.build_requests(request, generate.capacity(), generate.DEFAULT_MODEL, generate.DEFAULT_MAX_TOKENS)
    for call, req in zip(calls, reqs, strict=True):
        names = set(call["env_names"])
        assert not {n for n in names if generate_cli.is_billing_switch(n)}, names
        assert {"CLAUDE_CODE_OAUTH_TOKEN", "POPQUIZ_UNRELATED", "HOME"} <= names
        args = call["argv"]
        assert "--bare" not in args and "-p" in args
        flag = {args[i]: args[i + 1] for i in range(len(args) - 1)}
        assert flag["--model"] == generate.DEFAULT_MODEL
        assert flag["--output-format"] == "json"
        assert flag["--effort"] == generate.EFFORT
        assert json.loads(flag["--json-schema"]) == generate.output_schema(False)
        # Ruling 6: one brief builder - the child's system prompt is the batch request's.
        assert flag["--system-prompt"] == req["params"]["system"]
        assert call["prompt"] == req["params"]["messages"][0]["content"]
        assert flag["--tools"] == "" and flag["--setting-sources"] == ""
        assert all(f in args for f in FLAGS)
        assert call["cwd_entries"] == [] and Path(call["cwd"]).name.startswith("popquiz-claude-")
        assert call["pgid"] == call["pid"], "the child leads its own process group"
    assert len({c["cwd"] for c in calls}) == 3, "a fresh directory per candidate"
    assert client.spawned == ["c01", "c02", "c03"]

    # The key's value is nowhere: not in what the fake recorded, the run, or the report.
    for path in [*fake.log.rglob("*"), *(tmp_path / "out").rglob("*")]:
        if path.is_file():
            assert SENTINEL not in path.read_text(encoding="utf-8"), path


def test_child_env_removes_every_billing_switch_and_keeps_the_rest():
    parent = {"ANTHROPIC_API_KEY": "x", "ANTHROPIC_FOUNDRY_API_KEY": "x", "CLAUDE_CODE_USE_VERTEX": "1",
              "AWS_BEARER_TOKEN_BEDROCK": "x", "CLAUDE_CODE_OAUTH_TOKEN": "t", "HOME": "/h", "PATH": "/p"}
    assert generate_cli.child_env(parent) == {"CLAUDE_CODE_OAUTH_TOKEN": "t", "HOME": "/h", "PATH": "/p"}


# --------------------------------------------------------------------------- #
# Ruling 4: the candidate cap, and the stop
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize(
    ("kwargs", "sentence"),
    [
        ({"max_candidates": None}, generate.NO_CANDIDATE_CAP),
        ({"max_candidates": 2}, generate.over_candidate_cap(3, 2)),
        ({"max_candidates": 3, "cap_usd": 5.0}, generate.CLI_NO_USD),
    ],
)
def test_a_claude_cli_run_without_a_covering_cap_refuses_before_anything(tmp_path, fake, kwargs, sentence):
    fake.script("cli-clean")
    built = []
    with pytest.raises(GenerateError) as err:
        cli_run(tmp_path, built=built, **kwargs)
    assert str(err.value) == sentence
    assert built == [] and fake.calls() == []
    assert not (tmp_path / "out" / "t1").exists()


def test_the_cap_is_checked_again_on_a_resume(tmp_path, fake):
    """A resumed claude-cli run starts processes, so it needs its cap too."""
    fake.script("cli-usage-limit")
    cli_run(tmp_path)
    before = len(fake.calls())
    with pytest.raises(GenerateError) as err:
        cli_run(tmp_path, max_candidates=None)
    assert str(err.value) == generate.NO_CANDIDATE_CAP
    assert len(fake.calls()) == before


def test_a_usage_limit_stops_the_run_and_a_rerun_picks_up_the_rest(tmp_path, fake):
    fake.script("cli-usage-limit")
    report, client = cli_run(tmp_path)
    r = rows(report)
    assert report["stopped"].startswith("Stopped at c02: claude exited 1: You've hit your session limit")
    assert "rerun with the same --run-id" in report["stopped"]
    assert r["c01"]["status"] == "emitted"
    assert r["c02"]["reasons"][0].startswith("errored: stopped: claude exited 1: You've hit your session limit")
    assert r["c03"]["reasons"] == [generate.NOT_RUN]
    assert client.spawned == ["c01", "c02"], "never retried, and nothing after the stop"
    assert emitted(tmp_path) == ["t1-01.json"]
    printed = io.StringIO()
    generate.print_report(report, printed)
    assert printed.getvalue().startswith("Stopped at c02")
    assert generate.main.__doc__ and "stopped early" in generate.main.__doc__

    # The limit lifts; the same run id picks up c02 and c03, and c01 is served
    # from what the first run kept rather than asked for again.
    fake.script("cli-clean")
    again, client = cli_run(tmp_path)
    assert again["resumed"] is True and "stopped" not in again
    assert client.spawned == ["c02", "c03"]
    assert emitted(tmp_path) == ["t1-01.json", "t1-02.json", "t1-03.json"]
    assert not list((tmp_path / "out" / "t1").rglob("*.partial"))


def test_a_non_zero_exit_stops_the_run_at_that_candidate(tmp_path, fake):
    fake.script("cli-nonzero-exit")
    report, client = cli_run(tmp_path)
    assert report["stopped"].startswith("Stopped at c01: claude exited 2: Error: something went wrong")
    assert [r["status"] for r in report["candidates"]] == ["reported"] * 3
    assert client.spawned == ["c01"]
    assert emitted(tmp_path) == []


def test_a_candidate_that_fails_validation_is_reported_and_the_run_goes_on(tmp_path, fake):
    fake.script("cli-invalid")
    report, client = cli_run(tmp_path)
    r = rows(report)
    assert "stopped" not in report
    assert r["c01"]["status"] == "emitted"
    assert any("4 options, expected 5" in reason for reason in r["c02"]["reasons"])
    assert r["c03"]["reasons"] == [
        "errored: error_max_structured_output_retries: Failed to provide valid structured output after 3 attempts"
    ]
    assert client.spawned == ["c01", "c02", "c03"]
    # A rerun asks again only for the candidate the CLI gave up on; c02's answer
    # came back whole and is reported, not resubmitted (as with a batch).
    _, client = cli_run(tmp_path)
    assert client.spawned == ["c03"]


def test_a_child_past_its_timeout_is_killed_with_its_group_and_the_run_stops(tmp_path, fake, monkeypatch):
    script = tmp_path / "slow.json"
    clean = json.loads((FAKE / "cli-clean.json").read_text(encoding="utf-8"))
    clean["responses"][0]["sleep_s"] = 30
    script.write_text(json.dumps(clean), encoding="utf-8")
    monkeypatch.setenv("FAKE_CLAUDE_SCRIPT", str(script))
    report, client = cli_run(tmp_path, timeout_s=1.0)
    assert report["stopped"].startswith("Stopped at c01: claude ran past the 1 s timeout and was killed")
    (call,) = fake.calls()
    with pytest.raises(ProcessLookupError):
        os.killpg(call["pgid"], 0)


def test_while_a_child_runs_sigterm_is_handled_an_ignored_sighup_is_not_and_both_are_restored(tmp_path, fake, monkeypatch):
    """PQ-70 item 2, in process: the handler is in place when the child starts, an
    ignored SIGHUP (`nohup`) stays ignored, and the run leaves both as it found them.
    The real signal is sent in `tests/sandbox/generate/test_kill_rerun.py`."""
    seen = []

    class Spy(subprocess.Popen):
        def __init__(self, *args, **kwargs):
            seen.append((signal.getsignal(signal.SIGTERM), signal.getsignal(signal.SIGHUP)))
            super().__init__(*args, **kwargs)

    monkeypatch.setattr(subprocess, "Popen", Spy)
    fake.script("cli-clean")
    assert signal.getsignal(signal.SIGTERM) == signal.SIG_DFL
    previous = signal.signal(signal.SIGHUP, signal.SIG_IGN)
    try:
        cli_run(tmp_path)
        after = (signal.getsignal(signal.SIGTERM), signal.getsignal(signal.SIGHUP))
    finally:
        signal.signal(signal.SIGHUP, previous)
    assert after == (signal.SIG_DFL, signal.SIG_IGN)
    assert len(seen) == 3
    for term, hup in seen:
        assert type(getattr(term, "__self__", None)).__name__ == "_KillChildOnSignal"
        assert hup == signal.SIG_IGN


def executable(path: Path, text: str) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)
    return path


def cached(tmp_path, run_id="m1"):
    return sorted(p.name for p in (tmp_path / "out" / run_id / generate.RUN_SUBDIR / generate.CLI_SUBDIR).glob("c*.json"))


def test_a_claude_that_cannot_be_started_stops_cleanly_and_a_rerun_resumes(tmp_path, fake, capsys):
    """PQ-70 item 3: `Popen` raising `OSError` (here the binary is gone between
    resolve and spawn) is a one-sentence stop, not a traceback."""
    fake.script("cli-clean")
    vanishing = executable(
        tmp_path / "vanishing" / "claude",
        f'#!/bin/sh\n"{sys.executable}" "{FAKE / "fake_claude.py"}" "$@"\nrc=$?\nrm -f "$0"\nexit $rc\n',
    )
    code = generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3", "--claude-bin", str(vanishing)))
    err = capsys.readouterr().err
    assert code == 2
    assert err.startswith(f"generate: Stopped at c02: could not start {vanishing} (No such file or directory)."), err
    assert "rerun with the same --run-id" in err and "Traceback" not in err
    assert cached(tmp_path) == ["c01.json"]

    assert generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3")) == 0
    assert len(fake.calls()) == 3, "c01 once, then only c02 and c03"
    assert cached(tmp_path) == ["c01.json", "c02.json", "c03.json"]


def test_output_that_is_not_utf8_stops_cleanly_and_a_rerun_resumes(tmp_path, fake, capsys):
    """PQ-70 item 3: a child whose stdout is not UTF-8 is a one-sentence stop, and
    the finished child is not read or killed a second time."""
    fake.script("cli-clean")
    garble = tmp_path / "garble.py"
    garble.write_text(
        "import io, runpy, sys\n"
        "prompt = sys.stdin.read()\n"
        "if 'Topic: iterators' in prompt:\n"
        "    sys.stdout.buffer.write(b'\\xff\\xfe not utf-8')\n"
        "    sys.exit(0)\n"
        "sys.stdin = io.StringIO(prompt)\n"
        f"runpy.run_path({str(FAKE / 'fake_claude.py')!r}, run_name='__main__')\n",
        encoding="utf-8",
    )
    garbling = executable(tmp_path / "garbling" / "claude", f'#!/bin/sh\nexec "{sys.executable}" "{garble}" "$@"\n')
    code = generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3", "--claude-bin", str(garbling)))
    err = capsys.readouterr().err
    assert code == 2
    assert err.startswith(f"generate: Stopped at c02: {garbling} printed output that is not UTF-8."), err
    assert "rerun with the same --run-id" in err and "Traceback" not in err
    assert cached(tmp_path) == ["c01.json"]

    assert generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3")) == 0
    assert len(fake.calls()) == 3, "c01 once, then only c02 and c03"


# --------------------------------------------------------------------------- #
# Ruling 5 / AC-5: measured, labelled API-equivalent, never spend
# --------------------------------------------------------------------------- #


def test_the_report_carries_the_clis_own_figures_labelled_not_billed(tmp_path, fake):
    fake.script("cli-clean")
    report, client = cli_run(tmp_path)
    assert report["backend"] == "claude-cli" and report["numbers"] == "api-equivalent"
    assert report["numbers_note"].startswith("API-EQUIVALENT, NOT BILLED")
    assert report["claude_bin"] == str(fake.exe)
    assert "list price" not in json.dumps(report["prices"]) and "batch_discount" not in report["prices"]
    assert report["worst_case_usd"] is None
    first = rows(report)["c01"]
    assert (first["input_tokens"], first["output_tokens"]) == (2100, 1800)
    assert first["api_equivalent_usd"] == 0.0441 and first["wall_clock_s"] == 40.0
    assert all("cost_usd" not in r for r in report["candidates"])
    assert "cost_usd" not in report["totals"], "API-equivalent dollars are never summed as spend"
    assert report["totals"]["wall_clock_s"] == 40.0 + 41.0 + 42.0
    printed = io.StringIO()
    generate.print_report(report, printed)
    text = printed.getvalue()
    assert "API-equivalent $0.0441, not billed" in text
    assert "subscription usage" in text and "batch discount" not in text


# --------------------------------------------------------------------------- #
# Ruling 1: the command line, and no fallback either way
# --------------------------------------------------------------------------- #


def argv_for(tmp_path, *extra):
    return ["--count", "3", *sum((["--topic", t] for t in TOPICS), []), "--difficulty", "2",
            "--out", str(tmp_path / "out"), "--run-id", "m1", *extra]


def test_the_cli_runs_the_claude_cli_backend_end_to_end(tmp_path, fake, capsys):
    fake.script("cli-clean")
    code = generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3"))
    assert code == 0
    out = capsys.readouterr().out
    assert "API-EQUIVALENT, NOT BILLED" in out
    assert len(list((tmp_path / "out" / "m1").glob("*.json"))) == 3


def test_batches_stays_the_default_backend(tmp_path, fake, monkeypatch):
    def boom(*args, **kwargs):
        raise AssertionError("the claude-cli client was built for a default run")

    monkeypatch.setattr(generate_cli, "ClaudeCliClient", boom)
    code = generate.main(argv_for(tmp_path, "--replay", str(HERE / "fixtures" / "generate" / "clean.json")))
    assert code == 0 and fake.calls() == []


def test_a_claude_cli_run_that_cannot_start_never_falls_back_to_batches(tmp_path, monkeypatch, capsys):
    def boom(*args, **kwargs):
        raise AssertionError("AnthropicBatchClient was built for a claude-cli run")

    monkeypatch.setattr(generate, "AnthropicBatchClient", boom)
    monkeypatch.setenv("ANTHROPIC_API_KEY", SENTINEL)  # even with a key to fall back on
    code = generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3"))
    assert code == 2
    assert generate_cli.NO_CLAUDE in capsys.readouterr().err


def test_a_batches_run_that_cannot_start_never_falls_back_to_claude_cli(tmp_path, fake, monkeypatch, capsys):
    def boom(*args, **kwargs):
        raise AssertionError("the claude-cli client was built for a batches run")

    monkeypatch.setattr(generate_cli, "ClaudeCliClient", boom)
    code = generate.main(argv_for(tmp_path, "--cap-usd", "5"))
    assert code == 2
    assert generate.NO_KEY in capsys.readouterr().err
    assert fake.calls() == []


def test_a_stopped_claude_cli_run_never_falls_back_to_batches(tmp_path, fake, monkeypatch):
    def boom(*args, **kwargs):
        raise AssertionError("AnthropicBatchClient was built after a claude-cli stop")

    monkeypatch.setattr(generate, "AnthropicBatchClient", boom)
    fake.script("cli-usage-limit")
    code = generate.main(argv_for(tmp_path, "--backend", "claude-cli", "--max-candidates", "3"))
    assert code == 1


@pytest.mark.parametrize(
    ("extra", "sentence"),
    [
        (("--backend", "claude-cli", "--max-candidates", "3", "--replay", "x.json"), generate.CLI_NO_REPLAY),
        (("--claude-bin", "/x/claude", "--cap-usd", "5"), generate.BATCHES_NO_CLAUDE_BIN),
        (("--backend", "claude-cli"), generate.NO_CANDIDATE_CAP),
    ],
)
def test_each_backend_refuses_the_others_flags(tmp_path, fake, capsys, extra, sentence):
    fake.script("cli-clean")
    assert generate.main(argv_for(tmp_path, *extra)) == 2
    assert sentence in capsys.readouterr().err
    assert fake.calls() == []
