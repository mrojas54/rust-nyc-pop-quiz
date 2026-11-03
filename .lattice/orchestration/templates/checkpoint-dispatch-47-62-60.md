# Checkpoint: dispatch PQ-47, PQ-62, PQ-60 (written 2026-11-03)

Fresh Orchestrator session. Seat `panel:4`, mailbox `pq-orchestrator`, main checkout
`/Users/michellerojas/rust-nyc-pop-quiz` on `ai-c11-cc/lattice-build`. Load
`lattice-orchestrator` and its `references/orchestrator.md`; read `run-state.md`'s
`0x:xxZ (11-03)` line.

## Authorized

Michelle released PQ-47, PQ-62, PQ-60 by naming them (2026-11-03). Routing: Opus builds,
Codex reviews, fast-track, autonomy Moderate, one worktree each, stop at `review`.
Merges need her word. Board comments `RELEASED …` are on all three.

## Done

- Re-verified open at `origin/main` = `1fbc6bdb6852c98a5139c29c18865b0df71d5c6a`.
- Boots staged: `templates/boot-PQ-47.md`, `boot-PQ-62.md`, `boot-PQ-60.md` (PQ-57's
  standing clauses; per-ticket rulings, mutations, cleared files).

## Next (dispatch)

For each ticket, slug from its boot's **Base** line
(`pq-47-room-dep-denylist`, `pq-62-distribution-scan`, `pq-60-reason-copy`):

1. If `origin/main` moved past `1fbc6bd`, update the boot's Base sha (and re-verify).
2. `git worktree add -b ai-c11-cc/<slug> ~/rust-nyc-pop-quiz-worktrees/<slug> 1fbc6bd…`
   (sandbox off), then `git branch --unset-upstream`.
3. Seed: `templates/delegator-settings-seed.json` → `<wt>/.claude/settings.json`; the boot →
   `<wt>/.claude/boot-prompt.md`. Leave both uncommitted.
4. Launch a panel per ticket as last batch did (run-state `11-02: PQ-53 and PQ-56 …
   DISPATCHED`, area `area:11`), Opus, prompt `Read .claude/boot-prompt.md`.
   `--permission-mode auto` fails under `c11 send`: each panel waits at its first prompt
   for Michelle.
5. Expect `READY PQ-<n>` on the board and in the mailbox from each.
6. Run the loop on `/loop`, ≤3300 s between quiet wakes. PQ-62 halts once by design
   (the client picks which files may carry tells): relay her answer as `RESUME PQ-62 …`.
7. Land order when she merges: PQ-60 before PQ-62.

## Stop when

All three at `review` with PRs open and CI green; then report and wait.

## Also owed to Michelle (not part of this unit)

`just schedule` needs her question id, meetup date and club(s); `JoinRefusal::NotYetOpen`.
