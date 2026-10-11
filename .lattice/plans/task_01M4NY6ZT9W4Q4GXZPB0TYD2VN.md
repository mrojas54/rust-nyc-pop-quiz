# PQ-68: Generator follow-ups from the PR #61 review

Follow-ups from the PQ-21 exact-head review of PR #61 (merged at 170cfd4). Held until the client releases it.

Minor:
1. The stdout filter matches by name only (pipeline generate.py:651). Run any value 'now' strings through the AC-73 quoted-output comparison too, so a model-claimed output under another name cannot reach a trace.
2. verify.py:529-547 leaves stdout rows on non-last trace steps untouched. Strip them or refuse them, or have validate_question reject them.
3. AC-2's kill-and-rerun is an in-process KeyboardInterrupt. Add a real subprocess kill -9 case under test-full and assert the room is unaffected.

NIT:
4. run() takes fixture and client_factory separately (generate.py:964-1020); derive fixture from the client type.
5. The single-line option rule is hard-coded (generate.py:324); read cap.option_lines.
6. Add 'ANTHROPIC_API_KEY=' to .env.example. The sandbox blocks .env.* in worktrees, so the client adds this line or allows the edit.

Upstream, not in this ticket: BUILDPLAN D-D's model line should read claude-opus-5-5; EVALUATION's AC-2 row versus the in-process kill.

Guardrails carried from PQ-21: no API call and no key read; never write down what a program prints (AC-7); never overstate verification (AC-43).
