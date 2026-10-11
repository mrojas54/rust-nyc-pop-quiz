"""A fake `claude` executable for the claude-cli backend's tests (PQ-69 ruling 7).

It never reaches a network or a model. The test writes a two-line `claude` wrapper
that runs this file with the test's own interpreter, and puts only that wrapper's
directory on `PATH`. It answers from the script named by `FAKE_CLAUDE_SCRIPT`,
whose first response with a `when` that occurs in the prompt wins:

    {"responses": [{"when": "Topic: ownership", "exit": 0, "stdout": {...},
                    "stderr": "", "sleep_s": 0}]}

`stdout` is printed as JSON, or as given when it is a string. `sleep_s` delays the
answer, so a run can be killed while a candidate is in flight.

Each call is recorded to `FAKE_CLAUDE_LOG/<n>-<pid>.json`: its argv, cwd and the
entries in it, the prompt on stdin, its process group, and the **names** of the
environment variables it saw. Never a value: the billing test proves a key's name
is absent, and recording its value would only put a secret on disk.
"""

import json
import os
import sys
import time


def main() -> int:
    prompt = sys.stdin.read()
    log_dir = os.environ.get("FAKE_CLAUDE_LOG")
    if log_dir:
        n = len(os.listdir(log_dir))
        record = {
            "argv": sys.argv[1:],
            "cwd": os.getcwd(),
            "cwd_entries": sorted(os.listdir(".")),
            "env_names": sorted(os.environ),
            "prompt": prompt,
            "pid": os.getpid(),
            "pgid": os.getpgid(0),
        }
        with open(os.path.join(log_dir, f"{n:03d}-{os.getpid()}.json"), "w", encoding="utf-8") as handle:
            json.dump(record, handle)
    with open(os.environ["FAKE_CLAUDE_SCRIPT"], encoding="utf-8") as handle:
        script = json.load(handle)
    response = next((r for r in script["responses"] if r.get("when", "") in prompt), None)
    if response is None:
        print("fake claude: no response in the script matches this prompt", file=sys.stderr)
        return 97
    if response.get("sleep_s"):
        time.sleep(response["sleep_s"])
    out = response.get("stdout", "")
    sys.stdout.write(out if isinstance(out, str) else json.dumps(out))
    sys.stderr.write(response.get("stderr", ""))
    return int(response.get("exit", 0))


if __name__ == "__main__":
    sys.exit(main())
