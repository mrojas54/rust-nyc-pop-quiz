#!/usr/bin/env python3
"""Walk the review gate on a temp copy of the fixtures, for an asciinema recording.

Runs the organizer TUI in its own pty and forwards its screen to stdout, sending
keys with pauses; between TUI sessions it prints what the review CLI reports.
Never touches bank/questions/. Record it from the repository root, after building
organizer-preview and `just sandbox-build` (step 3 runs the pinned verifier):

    asciinema rec --headless --window-size 110x36 --idle-time-limit 3 \\
      -c "python3 prototypes/organizer-tui/recordings/review-gate-drive.py" \\
      prototypes/organizer-tui/recordings/review-gate.cast
"""
import fcntl, json, os, pathlib, pty, select, shutil, struct, subprocess, sys, tempfile, termios, time

WT = str(pathlib.Path(__file__).resolve().parents[3])  # the repository root
TUI = f"{WT}/prototypes/organizer-tui/target/debug/organizer-preview"
PY = f"{WT}/pipeline/.venv/bin/python"
HERE = os.path.dirname(os.path.abspath(__file__))
ENTER, ESC = "\r", "\x1b"

tmp = tempfile.mkdtemp(prefix="pq23-validate-", dir="/tmp")
BANK = os.path.join(tmp, "bank")
shutil.copytree(f"{WT}/pipeline/tests/fixtures/review/questions", os.path.join(BANK, "questions"))

def say(text):
    sys.stdout.write("\x1b[2J\x1b[H" + text + "\r\n")
    sys.stdout.flush()
    time.sleep(2.5)

def cli(*args, by=None):
    env = {k: v for k, v in os.environ.items() if k != "POPQUIZ_ORGANIZER"}
    cmd = [PY, "-m", "popquiz.review", *args, "--bank", BANK]
    if by:
        cmd += ["--by", by]
    shown = " ".join(["popquiz.review", *args[:2]] + (["--by", by] if by else []))
    r = subprocess.run(cmd, cwd=f"{WT}/pipeline", env=env, capture_output=True, text=True)
    return shown, r

def state(qid):
    _, r = cli("show", qid)
    d = json.loads(r.stdout)
    return (f"{qid}: status={d['review_status'] if 'review_status' in d else d.get('status')} "
            f"affirmation={d['affirmation']} affirmed_by={d.get('affirmed_by')}")

def tui(keys, organizer=None, qid="q4"):
    env = {k: v for k, v in os.environ.items() if k not in ("POPQUIZ_ORGANIZER", "VISUAL", "EDITOR")}
    env.update(TERM="xterm-256color", VISUAL=os.path.join(HERE, "review-gate-editor.py"))
    if organizer:
        env["POPQUIZ_ORGANIZER"] = organizer
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(WT)
        os.execve(TUI, [TUI, "--bank", BANK, "--question", qid], env)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 36, 110, 0, 0))
    def pump(secs):
        end = time.time() + secs
        while time.time() < end:
            r, _, _ = select.select([fd], [], [], 0.05)
            if r:
                try:
                    data = os.read(fd, 65536)
                except OSError:
                    return False
                if not data:
                    return False
                # A real terminal answers the cursor-position query (DSR) that the
                # TUI sends when it comes back from the editor; answer it the same way.
                if b"\x1b[6n" in data:
                    os.write(fd, b"\x1b[1;1R")
                os.write(sys.stdout.fileno(), data.replace(b"\x1b[6n", b""))
        return True
    pump(1.5)
    for key, wait in keys:
        os.write(fd, key.encode())
        if not pump(wait):
            break
    pump(0.5)
    try:
        os.kill(pid, 15)
    except ProcessLookupError:
        pass
    os.waitpid(pid, 0)

say("PQ-23 validation: the review gate on a temp copy of the fixtures\r\n"
    f"bank: {BANK}\r\nNothing here touches bank/questions/.\r\n\r\n" + state("q4") + "\r\n" + state("q3"))

say("1. Accept q4, then try to affirm with no organizer name set.")
tui([("v", 1.0), (ENTER, 2.0), ("a", 1.0), ("y", 2.5), (ENTER, 2.0),
     ("A", 1.5), ("y", 2.5), (ENTER, 2.0), (ESC, 1.0), ("q", 0.5)])
say("After step 1:\r\n" + state("q4"))

say("2. Affirm q4 as POPQUIZ_ORGANIZER=fixture-organizer, then edit its explanation.")
tui([("v", 1.0), (ENTER, 2.0), ("A", 1.5), ("y", 2.5), (ENTER, 2.0),
     ("e", 2.5), ("y", 3.0), (ENTER, 2.5), ("k", 1.0), (ESC, 1.0), ("q", 0.5)],
    organizer="fixture-organizer")
say("After step 2 (the edit cleared the affirmation):\r\n" + state("q4"))

shown, r = cli("reverify", "q4")
say(f"3. Re-verify q4 with the pinned verifier:\r\n$ {shown}\r\n"
    + (r.stdout + r.stderr).strip().replace("\n", "\r\n") + f"\r\nexit {r.returncode}\r\n\r\n" + state("q4"))

say("4. Affirm q4 again in the TUI.")
tui([("v", 1.0), (ENTER, 2.0), ("A", 1.5), ("y", 2.5), (ENTER, 2.5), (ESC, 1.0), ("q", 0.5)],
    organizer="fixture-organizer")
say("After step 4:\r\n" + state("q4"))

say("5. Fixture q3 (digest-less hand affirmation, like the bank's q3) reads unbound.")
tui([("v", 1.0), (ENTER, 3.0), (ESC, 1.0), ("q", 0.5)], qid="q3")
shown, r = cli("affirm", "q3", by="fixture-organizer")
say(f"$ {shown}\r\n" + (r.stdout + r.stderr).strip().replace("\n", "\r\n") + f"\r\nexit {r.returncode}\r\n\r\n" + state("q3"))
shutil.rmtree(tmp)
say("Done. Temp bank removed.")
