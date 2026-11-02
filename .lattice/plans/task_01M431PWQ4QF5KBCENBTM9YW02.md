# Plan (delegator, 2026-11-02)

Base `622e0fd`. One commit per item. Re-found against this tree.

| Item | State found | Action |
|---|---|---|
| 1 wall clause | closed 10-18 | confirm, DONE |
| 2 stale docs | `measure.html:24` stale; bank computed: q4 5 × 78 widest, q7 6 × 67; `popquiz/README.md` no longer says it | fix measure.html; README already closed; table at :701 stays |
| 3 fit fetch | closed 10-18 | confirm, DONE |
| 4 fallback live region | open; `boot()` in `web/wall/fallback/static.js` never clears on Esc to idle | clear on entering idle plus a test. **`static.js` is not a cleared file: asked** |
| 5 dim rules | three rules; only `prototypes/` sets `data-room`, `wall.css:10` says not ported | delete; add AC-80 absent-dark-mode test |
| 6 pencil | host has no provenance in text (home has `home_machine_only`) | **halt with options, per the ticket** |
| 7 token | no code reads it | remove `.env.example:39-40`, `fly.toml:27` comment; none in justfile; SPEC:553 is a contract note |
| 8 spacing | `migrate_mvp.py:561` | fix |
| schedule.py wording | three open; `fallback.py` message gone, its "Both or neither" docstring overstates the same way | reword; update `test_schedule.py:953` |
| pipeline/README | `:54` is the over-long line (483) | rewrap if the table row allows; else record why |
| popquiz/README trial | already paid-org at :1034; no trial references | already closed |
| just reserve | absent | add recipe; name in runbook |
| RUNBOOK | all three open (:430, :272, Option+Tab absent) | fix |
| NotYetOpen | still never emitted? | confirm, ask in DONE, no edit |

Checks: `just test`, `just a11y`, `just canary`; Codex review; PR; stop at `review`.
