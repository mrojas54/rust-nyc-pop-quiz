"""The generator's second backend: headless Claude Code (`claude -p`) on the client's
subscription (Lattice PQ-69, the client's ruling of 2026-10-11).

**This client has never made a real call.** Every test runs it against a fake
`claude` executable that replays fixtures. The output shape it reads is the one the
Claude Code documentation describes for `--output-format json` (the headless and
Agent SDK pages: `type`, `subtype`, `is_error`, `result`, `structured_output`,
`duration_ms`, `total_cost_usd`, `usage`); it is documented, not observed, until
the client's first run (HC-2).

It sits behind the same seam as the Message Batches client (`generate.BatchClient`):
`submit` keeps the requests `generate.build_requests` made - the one brief builder
for both backends - and starts nothing; `results` runs **one `claude` process per
request**, in order, and turns each answer into the batch result shape, so the
same `generate.classify` reads it. Nothing here chooses, balances or records an
answer position, and nothing the model says about a program's output reaches an
answer field: that is `classify`'s rule, unchanged.

**Billing.** With `ANTHROPIC_API_KEY` in its environment, `claude -p` bills the API
instead of the plan. So the child's environment is a copy of this process's with
every billing and provider switch removed by name (`child_env`), and no value is
read, logged or printed. `--bare` is never passed: its auth is the API key only.
Settings, CLAUDE.md, hooks, MCP servers, plugins, skills and tools are all switched
off by flag (`argv`), and the child runs in a fresh empty directory.

**The cap and the stop.** The run refuses before this client is built unless
`--max-candidates` is set and covers the request (`generate.check_candidates`). A
child that exits non-zero, says `is_error`, times out or prints something that is
not the documented JSON stops the run at that candidate: no retry, no fallback to
the other backend. Its usage-limit or rate-limit text is quoted in `stopped`. One
`is_error` is a candidate's own failure rather than a stop: the CLI giving up on
a schema-valid answer (`error_max_structured_output_retries`).

**Re-runnable (AC-2).** Each finished child's result is written atomically to
`<run>/_run/cli/<custom_id>.json`; a rerun with the same `--run-id` serves those
and runs only the rest. A child that is running when the run is interrupted is
killed with its process group. A SIGKILL of the generator itself cannot reach the
child: that one candidate finishes on plan usage and the rerun asks for it again.
"""

from __future__ import annotations

import json
import os
import shutil
import signal
import subprocess
import tempfile
import time
from collections.abc import Iterable, Iterator, Mapping, Sequence
from pathlib import Path
from typing import Any

from popquiz.generate import API_EQUIVALENT, CLAUDE_CLI, GenerateError, write_atomic

#: Seconds one candidate may run before its process group is killed and the run
#: stops. Generous: one structured answer at high effort, with no tools.
DEFAULT_TIMEOUT_S = 900.0

#: The one `is_error` subtype that is a candidate's failure, not a reason to stop.
PER_CANDIDATE_ERRORS = frozenset({"error_max_structured_output_retries"})

#: Every variable whose presence moves a `claude` child off the subscription: an API
#: key or token, a different endpoint, headers, or a third-party provider. Matched
#: by name and prefix; values are never read.
BILLING_PREFIXES = ("ANTHROPIC_", "CLAUDE_CODE_USE_")
BILLING_NAMES = frozenset({"AWS_BEARER_TOKEN_BEDROCK"})

REQUESTS_FILE = "requests.json"

NO_CLAUDE = "Refusing to start: no claude executable was found; install Claude Code or pass --claude-bin."


def not_executable(path: str) -> str:
    return f"Refusing to start: {path} is not an executable file."


def is_billing_switch(name: str) -> bool:
    return name.startswith(BILLING_PREFIXES) or name in BILLING_NAMES


def child_env(parent: Mapping[str, str]) -> dict[str, str]:
    """A copy of `parent` without any billing or provider switch (PQ-69 ruling 3).

    Kept: everything else, including `CLAUDE_CODE_OAUTH_TOKEN`, which is the
    subscription's own token, and `HOME`, through which the CLI finds its login.
    """
    return {name: value for name, value in parent.items() if not is_billing_switch(name)}


def argv(binary: str, params: Mapping[str, Any]) -> list[str]:
    """The child's command line, from one request's parameters. Never a shell string.

    Flags, as `claude --help` (2.1.296) describes them: `--tools ""` disables every
    tool; `--setting-sources ""` loads no user, project or local settings;
    `--strict-mcp-config` with no `--mcp-config` loads no MCP server; `--safe-mode`
    disables CLAUDE.md, skills, plugins, hooks, MCP and custom agents while auth and
    model selection work normally; `--disable-slash-commands` turns skills off;
    `--no-session-persistence` keeps no transcript. `--system-prompt` replaces the
    coding-agent prompt with the generator's brief.
    """
    config = params["output_config"]
    return [
        binary,
        "-p",
        "--model", params["model"],
        "--output-format", "json",
        "--json-schema", json.dumps(config["format"]["schema"], separators=(",", ":")),
        "--system-prompt", params["system"],
        "--effort", config["effort"],
        "--tools", "",
        "--setting-sources", "",
        "--strict-mcp-config",
        "--safe-mode",
        "--disable-slash-commands",
        "--no-session-persistence",
    ]


def prompt(params: Mapping[str, Any]) -> str:
    """The user message, which goes to the child on stdin."""
    return params["messages"][0]["content"]


def _one_line(text: str, limit: int = 240) -> str:
    line = " ".join(str(text).split())
    return line if len(line) <= limit else line[: limit - 1] + "…"


class Stop(Exception):
    """The run stops at this candidate. The message says why, in one line."""


class ClaudeCliClient:
    """One `claude -p` process per candidate, behind `generate.BatchClient`."""

    backend = CLAUDE_CLI
    numbers = API_EQUIVALENT

    def __init__(
        self,
        binary: str | None,
        cache_dir: Path,
        timeout_s: float = DEFAULT_TIMEOUT_S,
        environ: Mapping[str, str] | None = None,
    ) -> None:
        resolved = binary if binary is not None else shutil.which("claude")
        if resolved is None:
            raise GenerateError(NO_CLAUDE)
        if not (os.path.isfile(resolved) and os.access(resolved, os.X_OK)):
            raise GenerateError(not_executable(resolved))
        self.binary = os.path.abspath(resolved)
        self.cache_dir = Path(cache_dir)
        self.timeout_s = timeout_s
        self._environ = os.environ if environ is None else environ
        self.stopped: str | None = None
        #: The custom ids a child was started for, in order: a test reads it to
        #: prove a rerun served finished candidates without starting them again.
        self.spawned: list[str] = []

    # -- the seam ---------------------------------------------------------- #

    def submit(self, requests: Sequence[Mapping[str, Any]]) -> str:
        """Keep the requests, for this run and any rerun of it. Starts nothing."""
        write_atomic(self.cache_dir / REQUESTS_FILE, list(requests))
        return CLAUDE_CLI

    def poll(self, batch_id: str) -> str:
        return "ended"

    def results(self, batch_id: str) -> Iterable[Mapping[str, Any]]:
        return self._results()

    def _results(self) -> Iterator[Mapping[str, Any]]:
        requests = json.loads((self.cache_dir / REQUESTS_FILE).read_text(encoding="utf-8"))
        for request in requests:
            cid = request["custom_id"]
            cached = self.cache_dir / f"{cid}.json"
            if cached.is_file():
                yield json.loads(cached.read_text(encoding="utf-8"))
                continue
            try:
                entry, keep = self._run_one(cid, request["params"])
            except Stop as stop:
                self.stopped = (
                    f"Stopped at {cid}: {stop}. Candidates finished before it are written; "
                    "rerun with the same --run-id to pick up the rest."
                )
                yield _errored(cid, "stopped", str(stop), {})
                return
            if keep:
                write_atomic(cached, entry)
            yield entry

    # -- one candidate ------------------------------------------------------ #

    def _run_one(self, cid: str, params: Mapping[str, Any]) -> tuple[dict[str, Any], bool]:
        """One child, start to finish. Returns the entry and whether it is finished
        (cached for a rerun). Raises `Stop` for anything that ends the run."""
        self.spawned.append(cid)
        started = time.monotonic()
        with tempfile.TemporaryDirectory(prefix="popquiz-claude-") as cwd:
            child = subprocess.Popen(
                argv(self.binary, params),
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                cwd=cwd,
                env=child_env(self._environ),
                text=True,
                start_new_session=True,
            )
            try:
                out, err = child.communicate(prompt(params), timeout=self.timeout_s)
            except subprocess.TimeoutExpired:
                _kill(child)
                raise Stop(f"claude ran past the {self.timeout_s:g} s timeout and was killed") from None
            except BaseException:
                _kill(child)
                raise
        elapsed = time.monotonic() - started

        try:
            data = json.loads(out)
        except json.JSONDecodeError:
            data = None
        if child.returncode != 0:
            said = _said(data) or _one_line(err) or "no output"
            raise Stop(f"claude exited {child.returncode}: {said}")
        if not isinstance(data, dict) or data.get("type") != "result":
            raise Stop("claude printed something other than its JSON result")

        measured = {
            "wall_clock_s": _seconds(data.get("duration_ms"), elapsed),
            "api_equivalent_usd": data["total_cost_usd"] if _is_number(data.get("total_cost_usd")) else None,
        }
        if data.get("is_error") is True:
            subtype = str(data.get("subtype") or "error")
            if subtype in PER_CANDIDATE_ERRORS:
                return _errored(cid, subtype, _said(data) or subtype, measured), False
            raise Stop(f"claude reported {subtype}: {_said(data) or 'no message'}")
        structured = data.get("structured_output")
        if structured is None:
            raise Stop("claude's result has no structured_output")
        usage = data.get("usage") if isinstance(data.get("usage"), dict) else {}
        message = {
            # Only the schema-validated object is read; the `result` prose never is.
            "content": [{"type": "text", "text": json.dumps(structured, ensure_ascii=False)}],
            "stop_reason": data.get("stop_reason") or "end_turn",
            "usage": usage,
        }
        entry = {"custom_id": cid, "result": {"type": "succeeded", "message": message}, "measured": measured}
        return entry, True


def _errored(cid: str, kind: str, message: str, measured: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "custom_id": cid,
        "result": {"type": "errored", "error": {"error": {"type": kind, "message": _one_line(message)}}},
        "measured": dict(measured),
    }


def _said(data: Any) -> str:
    """What a result says about itself: `result`, or the error subtypes' `errors`."""
    if not isinstance(data, dict):
        return ""
    if isinstance(data.get("result"), str) and data["result"].strip():
        return _one_line(data["result"])
    errors = data.get("errors")
    if isinstance(errors, list) and errors:
        return _one_line("; ".join(str(e) for e in errors))
    return ""


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _seconds(duration_ms: Any, fallback_s: float) -> float:
    return round(duration_ms / 1000 if _is_number(duration_ms) else fallback_s, 3)


def _kill(child: subprocess.Popen[str]) -> None:
    """Kill the child's whole process group, then reap it."""
    try:
        os.killpg(child.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    child.communicate()
