VALIDATION PQ-28 — audit tree origin/main @ 0c7d60a; branch ai-c11-cc/guardrail-audit.
Sandbox note: inside the Bash sandbox `just test` fails only on environment (uv cache "Operation not permitted"; Discord mock local bind). All runs below were with the sandbox off.

Untouched tree:
- just setup: EXIT 0 (venv created, crates fetched)
- just test: EXIT 0. room 236 passed / 4 ignored; pipeline 710 passed, 1 warning; web 286 passed / 0 failed. 17.7 s warm (`just test  14.78s user 3.47s system 103% cpu 17.683 total`).
- just canary: EXIT 0, 12 passed.
- just secret-scan: EXIT 0, 2 passed.
- just bank-audit: EXIT 0, "bank-audit: passed", known WARN quoted: "WARN tell: option position [AC-26]: worst rule 'index E': 4 hit(s) vs 0.80 by chance = 5.00x over 4 question(s), tail p=0.0016".
- Not run: just test-full (Docker/sandbox image), smoke/burst against the deployed room (out of scope).

Mutations: 49 tried, 34 caught, 15 survived; each applied by exact-string replace, the named test run, reverted with git restore, tree checked clean (git diff --quiet && git status --short) before the next. Data experiments used a copied bank in the session scratchpad, never the tree. Full table in the report's last section.
SURVIVED: M1c (arrange avoids last meetup's letter), M2a (no verifier provenance schedules), M2c (stdout hand edit, copied bank), M3e/M3e2 (RevealWitness forge / derive Default), M5b3 (forbidden phrase in routes.rs reason), M5d2 (forbidden phrase in q4 explains.what), M6h (fallback.py phase literal), M7a/M7b (Miri line with no miri), M9a2 (route in lib.rs), M10d (fit after release), M11b (distribution sentence in RUNBOOK), M12b (hand-typed affirmation), M12d (standalone fallback, answer at E).
Probe P1: two rooms on q3, release A, /last serves "correct": true on E while B is live. Throwaway test, reverted.
Harness incident: M6a2 (reveal -> work) made phase_table loop forever; killed after ~6.5 h, runner restored phase.rs; a 900 s per-mutation timeout was added after.
Planned, not run: M3g, M4c (catching tests read, not proven).

Exit check: git diff origin/main --stat = docs/audit/2026-10-guardrail-audit.md, docs/audit/README.md (both new). Commits 0ec43c1, 6a9fd7f, 17cf0f8, each with a Good signature.