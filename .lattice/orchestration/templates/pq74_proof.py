#!/usr/bin/python3
"""Moves are moves: compare room/src (minus cli/) at BASE and HEAD.

Run from the repo root: python3 pq74_proof.py <base-sha> [head-sha]
Exit 0 only if the lock order per function, the test names and the fn set match.
Lines that differ are printed for a reader: they must be `use`/`mod` or path docs.
"""
import collections
import re
import subprocess
import sys

base = sys.argv[1]
head = sys.argv[2] if len(sys.argv) > 2 else "HEAD"


def git(*a):
    return subprocess.run(["git", *a], check=True, capture_output=True, text=True).stdout


def tree(rev):
    names = git("ls-tree", "-r", "--name-only", rev, "room/src").split()
    return {
        n: git("show", f"{rev}:{n}")
        for n in names
        if n.endswith(".rs") and not n.startswith("room/src/cli/")
    }


VIS = re.compile(r"pub\((super|in crate::quiz_session)\) ")
SKIP = re.compile(r"^\s*(pub(\([^)]*\))? )?(use |mod )")


def lines(files):
    out = collections.Counter()
    for text in files.values():
        in_use = False
        for raw in text.splitlines():
            line = raw.strip()
            if not line:
                continue
            if in_use:  # a multi-line `use`
                in_use = not line.endswith(";")
                continue
            if SKIP.match(raw):
                in_use = not line.endswith(";")
                continue
            out[VIS.sub("", line)] += 1
    return out


FN = re.compile(r"^\s*(?:pub(?:\([^)]*\))? )?(?:async )?fn (\w+)")
LOCK = re.compile(r"lock\(&self\.(\w+)\)")


def locks(files):
    """(type-agnostic) fn name -> the sorted list of its lock sequences."""
    seqs = collections.defaultdict(list)
    for text in files.values():
        name, seq = None, []
        for raw in text.splitlines():
            m = FN.match(raw)
            if m:
                if name and seq:
                    seqs[name].append(tuple(seq))
                name, seq = m.group(1), []
            if name and not raw.strip().startswith("//"):
                seq += LOCK.findall(raw)
        if name and seq:
            seqs[name].append(tuple(seq))
    return {k: sorted(v) for k, v in seqs.items()}


def fns(files):
    out = collections.Counter()
    for text in files.values():
        for raw in text.splitlines():
            m = FN.match(raw)
            if m:
                out[m.group(1)] += 1
    return out


def tests(files):
    out = collections.Counter()
    for text in files.values():
        rows = text.splitlines()
        for i, raw in enumerate(rows):
            if raw.strip() in ("#[test]", "#[tokio::test]"):
                out[rows[i + 1].strip()] += 1
    return out


b, h = tree(base), tree(head)
print(f"files: base {len(b)}, head {len(h)}")
bl, hl = lines(b), lines(h)
only_base, only_head = bl - hl, hl - bl
print(f"\n--- lines only at base ({sum(only_base.values())})")
for l, n in sorted(only_base.items()):
    print(f"  {n}x {l[:150]}")
print(f"\n--- lines only at head ({sum(only_head.values())})")
for l, n in sorted(only_head.items()):
    print(f"  {n}x {l[:150]}")

ok = True
bk, hk = locks(b), locks(h)
print(f"\n--- lock sequences: {sum(len(v) for v in bk.values())} at base, {sum(len(v) for v in hk.values())} at head")
if bk != hk:
    ok = False
    for k in sorted(set(bk) | set(hk)):
        if bk.get(k) != hk.get(k):
            print(f"  DIFF {k}: base {bk.get(k)} head {hk.get(k)}")
print("  equal" if bk == hk else "  DIFFERENT")

if fns(b) != fns(h):
    ok = False
    print("\n--- fn set differs:", (fns(b) - fns(h)), (fns(h) - fns(b)))
else:
    print(f"\n--- fn set equal: {sum(fns(h).values())} functions")
if tests(b) != tests(h):
    ok = False
    print("\n--- inline tests differ:", tests(b) - tests(h), tests(h) - tests(b))
else:
    print(f"--- inline tests equal: {sum(tests(h).values())}")
sys.exit(0 if ok else 1)
