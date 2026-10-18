[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/room-affirmed" ] || exit 99   # FATAL: wrong cwd — HALT, do not cd

# Resume — PQ-77 delegator, fresh session (stdout fix → test-full → PR phase)

You are the PQ-77 delegator, resumed at your checkpoint (16:16Z 10-18). HEAD `8e4e2d6`, rebased on `origin/main`
`7573efa`, green, not pushed. Codex review is at its cap (2 passes); mutations M1–M8 shown; local validation attached.

1. Read `.claude/boot-prompt.md` in full (your brief). Skip §0's READY; post `RESUMED PQ-77 CWD <worktree> HEAD <sha> MODE active`
   (comment and mail). Re-run §0's guards and identity block.
2. Read `.claude/checkpoint-pq77.md` — its **Client ruling 16:16Z ("a + canary change")** section is binding and is your
   first task. Then `lattice comments PQ-77`.
3. `lattice status PQ-77 in_progress` (it is `needs_human`); lower the panel flag.
4. Do the ruling's work, then `just test-full` once, then push/PR/CI/attach/`review`/DONE per boot prompt §3.
   If `origin/main` moved, rebase before the push (not pushed yet) and re-run `just test`.

Session cap still applies (~60–80 tool calls): checkpoint and stop if you near it before the PR is open.

## Orchestrator amendment, 16:2xZ 10-18 — overrides the checkpoint's "No third Codex pass" (BINDING)

The client, answering the Orchestrator directly: **"fix it here, one more review."** After the stdout fix + canary change is committed and `just test` is green (mutation shown), run **one more Codex review — pass 3, the last** — scoped to that commit and its effect on the canary: `codex exec -s read-only "<prompt>"` from this worktree with the bypass, the prompt naming the commit, ruling, the dropped `PLANT_MIDDLE_STDOUT` plant and the four repointed assertions, asking whether the canary still proves G-3's walk filter as defence in depth and whether room and Python now agree on stdout rows. Fix only Critical/Major from it; there is no pass 4 — anything left after pass 3 goes in DONE as open, and you stop for the Orchestrator. Attach the verdict `--role review` naming the reviewed HEAD. DONE says "pass 3 by the client's ruling" instead of "tests only".
