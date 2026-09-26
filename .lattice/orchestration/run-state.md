# Run state — Rust NYC Pop Quiz build (lattice-orchestrator)

Opened 2026-09-20. Phase 0 approved by the client the same day. Phase 0 record:
`phase0-trail.md` in the orchestrator's scratchpad (copied below where it matters).

## Configuration

| Key | Value |
|---|---|
| Autonomy | Moderate |
| N (max concurrent delegators) | 4 |
| PR merge policy | Leave at `review`; the client merges, or says the word and the Orchestrator merges with a merge commit |
| PR base | `origin/main` (PR #7 merged the contract amendments at `7abfce0`) |
| Git remote (verified) | `origin` = `git@github.com:mrojas54/rust-nyc-pop-quiz.git` |
| Terminal pre-merge status | **`review`** (classic preset: no `pr_open`, no `in_validation`) |
| Status vocabulary | `backlog · in_planning · planned · in_progress · review · done · blocked · needs_human · cancelled` |
| WIP limits | `in_progress` 10, `review` 29 (raised from 5 at init, F-4) |
| Ticket fidelity | Verbose |
| plan_review_mode / review_mode | `--mode single`, `LATTICE_SPAWN_BACKEND=headless` |
| Master Validator | On |
| Result Validator | On |
| Auto-close surfaces | On |
| c11 workspace | `workspace:1`, Orchestrator at `pane:1` (`surface:$C11_SURFACE_NUM`), Phase 0 document at `surface:22` |
| Runtime/browser approval scope | Delegators may start local servers and drive the c11 browser against them. Push, PR, deploy, merge, Docker and `flyctl` run outside the sandbox with the client's approval, one command at a time |
| Delivery-receipt path | `.lattice/orchestration/delivery-receipts.jsonl` |
| Run branch | `ai-c11-cc/lattice-build` (holds `.lattice/`; created from `ced5cb2`, fast-forwarded to `7abfce0`) |
| Models | Delegators Opus 5; headless reviews Sonnet 5; Result Validator Opus 5 |
| Held tickets (tag `held`) | T-09 (H-2 DNS; fallback host allowed) · T-10 (H-3 Discord app) · T-16 (H-4 API key) |
| Stop points | End of M1 → HC-0 to the client · before every push, PR, deploy, merge |
| Sandbox carve-out | `.claude/settings.json` re-allows reading `.env.example` (F-5), committed so worktrees inherit it |

## Standing clauses in every boot prompt

Criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Port `slot_for_day` only, never `slot_for_meetup`. `verify.py` never calls Miri; T-15b adds it. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Commit signing goes through 1Password: retry once, then escalate to the client, never bypass. Shared files are serialized: `justfile`, `pyproject.toml`, CI config, `web/shared`, `room/src/phase.rs`, `.env.example`.

## Workspace panes (c11 refs)

| Role | Ref |
|---|---|
| main_view_area | `pane:1` (Orchestrator `surface:8`; Phase 0 doc `surface:22`) |
| control_surface | `surface:23` (Lattice Board browser tab in `pane:1`) |
| delegate_view_area_1..3 | The window cannot split (`pane_too_small`); delegators open as tabs of `pane:1`, soft cap 15. `pane:5` is the client's own terminal and is never used for spawns |
| lattice_dashboard_port | 48731 (log: `$TMPDIR/lattice-dashboard-48731.log`) |

## Install facts pinned after init

- Lattice 0.2.0 has **no** `plan-review`, `code-review`, `claim`, `needs-human` or `review-status` commands. Reviews come from a Sonnet review subagent spawned inside the delegator's session with the Agent tool; verdicts attach via `lattice attach --role review`. Escalation is `lattice status <ID> needs_human` plus a c11 flag.
- `lattice plan <ID>` prints the plan path; `lattice complete <ID> --review "…"` runs the review → done ceremony (Orchestrator only).
- Delegators launch as `claude --model opus --permission-mode auto` in their worktree; the sandbox applies, and pushes, PRs, Docker and c11 calls prompt the client in the delegator's tab.
- Boot prompts are staged in the Orchestrator's scratchpad (`…/scratchpad/prompts/<ID>-<slug>.md`) with the Clause-1 cwd guard on line one.
- Worktrees live at `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/<slug>` on branches `ai-c11-cc/<slug>` from `origin/main`.

## Tickets in scope

| Ticket | Lattice | Title | Milestone | Mode | Status | Branch base |
|---|---|---|---|---|---|---|
| T-01 | PQ-1 | Repo scaffold, justfile, CI | M0 | inline-full | backlog | origin/main |
| T-02 | PQ-2 | Shared web layer: tokens, fonts, well, type model | M0 | inline-full | backlog | origin/main |
| T-03 | PQ-3 | Burst spike on Fly | M0 | sub-agent-full | backlog | origin/main |
| T-04a | PQ-4 | Phase machine and sealed answers module | M1 | sub-agent-full | backlog | origin/main |
| T-04b | PQ-5 | Sessions and the answer store | M1 | inline-full | backlog | origin/main |
| T-04c | PQ-6 | WebSocket transport and reconnect | M1 | inline-full | backlog | origin/main |
| T-05 | PQ-7 | The wall | M1 | inline-full | backlog | origin/main |
| T-06 | PQ-8 | The buzzer | M1 | inline-full | backlog | origin/main |
| T-07 | PQ-9 | The host phone | M1 | inline-full | backlog | origin/main |
| T-08 | PQ-10 | Canary secrecy suite | M1 | inline-full | backlog | origin/main |
| T-09 | PQ-11 | Deploy to Fly with the host stand-in | M1 | fast-track | backlog · **held** | origin/main |
| T-26 | PQ-12 | Static fallback and host sheet | M1 | inline-full | backlog | origin/main |
| T-10 | PQ-13 | Discord OAuth and role check | M2 | inline-full | backlog · **held** | origin/main |
| T-11 | PQ-14 | Room lifecycle and the used ledger | M2 | inline-full | backlog | origin/main |
| T-12 | PQ-15 | Take it home page | M2 | fast-track | backlog | origin/main |
| T-13 | PQ-16 | Accessibility sweep | M2 | fast-track | backlog | origin/main |
| T-25 | PQ-17 | Admin channel for the pipeline | M2 | inline-full | backlog | origin/main |
| T-14 | PQ-18 | Bank format and MVP migration | M3 | inline-full | backlog | origin/main |
| T-15a | PQ-19 | Verification sandbox image | M3 | inline-full | backlog | origin/main |
| T-15b | PQ-20 | Verifier with Miri and the pin | M3 | sub-agent-full | backlog | origin/main |
| T-16 | PQ-21 | Generator via Message Batches | M3 | inline-full | backlog · **held** | origin/main |
| T-17 | PQ-22 | Dedupe | M3 | fast-track | backlog | origin/main |
| T-18 | PQ-23 | Review surface | M3 | inline-full | backlog | origin/main |
| T-19 | PQ-24 | Bank audit and the slot function | M3 | sub-agent-full | backlog | origin/main |
| T-20 | PQ-25 | Schedule and sync | M3 | inline-full | backlog | origin/main |
| T-21 | PQ-26 | Burst and smoke in CI | M4 | inline-full | backlog | origin/main |
| T-22 | PQ-27 | Copy freeze and lints | M4 | inline-full | backlog | origin/main |
| T-23 | PQ-28 | Guardrail audit | M4 | sub-agent-full | backlog | origin/main |
| T-24 | PQ-29 | Organizer runbook | M4 | fast-track | backlog | origin/main |

Dependencies: 89 `depends_on` links on the board, one per BUILDPLAN dependency (T-23 on everything). Longest chain 10.

## Amendments routed upstream to tone-architect (not authored here)

- **F-2.** `EVALUATION.md` HC-0 trigger cites `burst` green, but `burst` is T-21 (M4). Proposed one-line amendment: HC-0's burst evidence is T-03's spike measurement re-run against the deployed skeleton. Client accepted this reading 2026-09-20; the run proceeds on it.
- **F-8.** `SPEC.md` §3.1 lists the review surface as the writer of `review`, but §7.3 has dedupe marking `near_duplicate_of`. Ruled for the run: T-17 writes it, T-18 reads it. Proposed amendment: name dedupe as that field's writer in §3.1.

- **F-15 (found by PQ-18, 2026-09-20).** A does-not-compile question can never be affirmed: `SPEC.md` §7.4 requires a trace of at least two steps before affirm; §3.1 and D-10 make the last step the one whose `values` names `stdout`; §3.2 gives a does-not-compile record no `stdout`. Jointly unsatisfiable for a class the contract requires (q8, AC-11, AC-24, D-22, the minimum viable cut). **Recommended amendment:** for a does-not-compile question the resolving step is the one whose `values` names `error` (the compiler's code or codes, from `compile_error_code`) instead of `stdout`; `work` still shows steps `0..M-2` and `reveal` enters at `M-1`; the two-step minimum stands. Touches §3.1 (`trace`), §5.3, §7.4, D-10's wording, AC-72 and AC-97's tests. **Run ruling meanwhile:** PQ-18 stores q8's trace as drafted; T-18's affirm gate is built to the amended rule once the client says so; until then T-18 is not dispatched past planning on this point.

- **F-16 (found by PQ-2).** `SPEC.md` §15 claims both fonts come from the design system's `assets/fonts/`; only Cascadia Mono is there, Instrument Serif came from a font CDN in the prototype. Run ruling: Instrument Serif vendored from its upstream OFL release with checksum and licence. Amendment: correct §15.
- **F-17 (found by PQ-2).** `prototypes/_shared/tokens.css` §1 has a Google Fonts `@import` that D-14, AC-77 and AC-102 forbid. Run ruling: dropped in the build, vendored `fonts.css` imported instead. Amendment: patch the prototype.
- **Naming trap for T-05 / T-12 briefs:** the prototype's `data.js` calls the third beat `argue`; the contract's field is `takeaway` (§3.1). `argu` is also a Forbidden-row pattern. Consuming tickets use `takeaway` and never let the prototype's key reach a string.
- **Doc bug cleared to PQ-2:** `web/README.md` line 25's test command; the `justfile` was already right.
- **F-18 (found by PQ-18).** `SPEC.md` §3.1 files `why_tempting` under `explains`, one per incorrect option. The answer slot is drawn per date and reorders the options, so a beat keyed by letter dangles and one keyed by position mis-attaches on the first reorder. Run ruling: `why_tempting` lives on each `Option`, next to the text it explains; `explains` keeps `what` and `takeaway`. Amendment: move the field in §3.1 and say so in §4.5 and §13.

## Decision log (append-only)

- 2026-09-20 [autonomy: Moderate] Run branch `ai-c11-cc/lattice-build` created from `ced5cb2` per the client's instruction; not from `main`, not from the empty `ai-c11-cc/build-orchestration`.
- 2026-09-20 [client] `fly auth whoami` = logged in. T-03 released from the hold list.
- 2026-09-20 [client] PR #7 (`ai-c11-cc/contract-amendments` → `main`) opened and merged with a merge commit at `7abfce0`. Every ticket PR bases on `origin/main`.
- 2026-09-20 [client] `.env.example` sandbox carve-out via `sandbox.filesystem.allowRead` in the committed `.claude/settings.json`.
- 2026-09-20 [client] F-2 reading accepted; F-4 WIP raise accepted; config and validation plan approved as drafted.
- 2026-09-20 [autonomy: Moderate] Lattice `opinionated` preset checked: identical statuses, transitions and limits to `classic`; only `display_names` differ. `classic` kept as instructed.
- 2026-09-20 [autonomy: Moderate] Held tickets carry the tag `held` in `backlog` because `backlog → blocked` is not a legal transition on this preset; the dispatch loop never spawns a `held` ticket.
- 2026-09-20 [autonomy: Moderate] T-19's ticket lists AC-23a and AC-23b explicitly (F-7). T-17 owns `near_duplicate_of` (F-8). T-02 lands ported strings in `web/shared/copy` (F-11).
- 2026-09-20 [autonomy: Moderate] Run files committed on `ai-c11-cc/lattice-build` at `e3d9e50` (signed; 1Password signing worked from this session outside the sandbox).
- 2026-09-20 [autonomy: Moderate] PQ-1 at `review`: PR #8 (`ai-c11-cc/repo-scaffold` @ `2ab95e3` on `7abfce0`), `just test` warm 0.28 s, cold 12.21 s. Five deviations flagged by the delegator, all accepted: a `setup` recipe owning the one network step; `.gitignore` untouched (Orchestrator amendment R-7: never ignore `.lattice/` or `HANDOFF.md`); no `rust-toolchain.toml` (slot left for T-15a's pin); validation notes attach as `--role review` because the install accepts no other role; root `README.md` corrected. CI unproven until the PR runs it.
- 2026-09-20 [autonomy: Moderate] **Policy tightening.** The delegator's push and PR went through auto mode; whether the client approved them in the tab is not knowable from here. `.claude/settings.json` now carries `ask` rules for `git push`, `gh pr create`, `gh pr merge`, `fly`, `flyctl` and `docker`, so those always prompt. The file is copied into every worktree at creation and left uncommitted there; it reaches `main` with the orchestration branch.
- 2026-09-20 [autonomy: Moderate] Wave 2 dispatched by press-ahead off `origin/ai-c11-cc/repo-scaffold` @ `2ab95e3`: PQ-2 (shared web layer), PQ-3 (burst spike), PQ-18 (bank format), PQ-19 (sandbox image). Their PRs base on `ai-c11-cc/repo-scaffold`; **retarget each to `main` after #8 merges and before its branch is deleted.** PQ-3's sub-agent-full runs inline in one tab (planner and reviewers as Agent-tool subagents) because the window cannot split.
- 2026-09-20 [autonomy: Moderate] Two cross-language rulings recorded for the validator: the receipt (§7.5) is one Python function in `popquiz.receipt` plus a JavaScript twin in `web/shared` (T-05), both tested against one fixture set at `bank/fixtures/receipts/`; the type-model formula lives in `web/shared/typemodel.js` and T-19 mirrors it in Python against the same worked examples.
- 2026-09-20 [autonomy: Moderate] PQ-3 flagged three ticket-text vs brief divergences. Ruled for all wave-2 tickets by comment: the brief governs (PR base is the scaffold branch until #8 merges; fly/push/PR run in the delegator's tab under the `ask` rules). Third point was a real gap: worktree sandboxes could not write the root board, so every `lattice` write needed a bypass. `.claude/settings.json` now adds `sandbox.filesystem.allowWrite` for the board directory; re-seeded into all five worktrees.
- 2026-09-20 [client: "merge 8"] PR #8 merged to `main` with a merge commit, `0742d35`, after both CI jobs passed on `2ab95e3`. PQ-1 completed on the board. Wave 2 told to rebase onto `origin/main` and base their PRs on `main`; the scaffold branch is left on `origin` until all four have rebased. No stacked PRs existed yet, so nothing needed retargeting.
- 2026-09-20 [autonomy: Moderate] Footgun: never `git switch` in the root checkout while the board has uncommitted changes (git refuses, correctly; the working tree is the database). Update the local `main` ref with `git fetch origin main:main` instead.
- 2026-09-20 [autonomy: Moderate] Phase 1 opened. PQ-1 (T-01) dispatched at surface:24 in worktree `repo-scaffold`, branch `ai-c11-cc/repo-scaffold` from `origin/main` @ `7abfce0`. Wave 2 (T-02, T-03, T-14, T-15a) spawns when PQ-1 reaches `review` (press-ahead), branching off the in-review scaffold branch.

## Run-time footguns

| Symptom | Cause | Mitigation |
|---|---|---|
| `git status` prints `.env.example: Operation not permitted`; `git switch` prints `D .env.example` | Sandbox denies `**/.env.*`, which matches `.env.example` | Carve-out in `.claude/settings.json` (`sandbox.filesystem.allowRead`). Never `git add -A` regardless |
| `c11` socket timeouts (10 s) mid-session | Transient; the app recovered within minutes | Retry once after a minute; do not spawn while it is down |
| `c11 new-pane` refuses: `pane_too_small` | The Orchestrator's window is narrow | Delegate surfaces open as tabs of an existing pane, not new panes, unless the client widens the window |
| A freshly launched delegator sits on "allow reading from …/scratchpad/prompts" | The boot prompt lives outside the delegator's worktree, so auto mode asks once before reading it | Stage each boot prompt inside the worktree at `.claude/boot-prompt.md` (beside the seeded settings file, left uncommitted); if a tab is already waiting, the Orchestrator answers Yes with `send-key enter` |
| `c11 new-surface` times out after ~10 s but the surface exists | Socket contention while several surfaces spawn | Always `c11 tree` before retrying a create; launch into the surfaces that appeared rather than creating more |
- 2026-09-20 [autonomy: Moderate] GitHub auto-deleted `ai-c11-cc/repo-scaffold` on merge (repository setting). `main` @ `0742d35` carries `2ab95e3`; wave 2 rebases onto `origin/main` per the earlier comment. PQ-18 confirmed by merge-base. No orchestrator action.
| Delegators stop mid-implementation and ask a human to write a function ("Learn by Doing", `TODO(human)`) | They inherit the operator's user-level Learning output style | `.claude/settings.json` sets `"outputStyle": "default"` for the project; seeded into every worktree; stalled delegators told by `c11 send` to finish the piece themselves. Add to every future boot prompt: "you are a delegator, not a tutor" |
| Every tab idles at once with "Usage limit reached · continuing at 10pm" | The claude.ai session limit is account-wide; the fleet and the Orchestrator pause together | Nothing to do but wait; on resume, read every screen before assuming a stall, and re-arm the Monitor. Prompts waiting on the client (fly, docker) survive the pause |
| UserPromptSubmit hook error `dyld: Library not loaded …gettext/lib/libintl.8.dylib` in a delegator tab | A user hook depends on a Homebrew lib missing under `/usr/local` | Non-blocking; the client's environment, not the run's. Noted, not fixed |
- 2026-09-20 [autonomy: Moderate] Fleet resumed after the account usage limit reset. PQ-2 and PQ-18 were stalled on Learning-style handoffs to a human; project `outputStyle` set to `default`, both told to finish the work themselves. PQ-3 waits on the client's `fly` approval, PQ-19 on `docker`; client notified.
- 2026-09-20 [needs_human] PQ-18 blocked on commit signing: four attempts, three distinct 1Password errors (socket denied in sandbox; 'failed to fill whole buffer' twice; 'agent returned an error'). Config untouched, no bypass, commit 1 of 4 staged. The client commits from a normal terminal in the bank-format worktree using the add lists and messages printed in tab 31. Signing worked from the Orchestrator's own session twice today, so the difference is the tab's process context or 1Password's approval dialog, not the key.
- 2026-09-20 [needs_human] PQ-2 also blocked on signing after 18:20 (its 123cbc7 signed at 18:20; every commit since fails). Diagnosis from PQ-2: the 1Password SSH agent is unlocked and lists the key; op-ssh-sign waits on a desktop-app approval a non-TTY call cannot answer. Fleet-wide; one client action (approve or unlock in the 1Password app, or commit from a normal tab) clears both PQ-2 and PQ-18. Footgun row added.
| Every delegator's `git commit` fails with `1Password: failed to fill whole buffer` while the agent's `ssh-add -l` lists the key | `op-ssh-sign` asks the 1Password desktop app to authorize each signature; from a c11-spawned tab the approval dialog never gets answered (or times out), so signing hangs then errors. The Orchestrator's own session signs fine while the app is freshly unlocked | The client approves in the 1Password app (or turns on "remember approval" for the git signing item), or commits from a normal terminal in the worktree. Never bypass. Consider batching: delegators stage per commit and post the add list + message so a human can run them in one go |
- 2026-09-20 [client, in tab 29] Signing unblocked: `git -c gpg.ssh.program=ssh-keygen commit` with `SSH_AUTH_SOCK` at the 1Password agent. Same key, good signatures verified on b498ad8 and 4c6e55e. Passed per invocation, nothing persisted. Recipe sent to PQ-18; becomes standing clause 5's fallback in every later boot prompt (never `--no-gpg-sign`).
- 2026-09-20 23:1x [waiting on client] All four wave-2 tabs hold an `ask`-rule prompt: 29 `git push` (shared-web-layer, first push), 30 `fly auth whoami`/`fly orgs list` (spike pre-deploy check), 31 the 1Password agent `SSH_AUTH_SOCK` export for signing, 32 `docker run` (sandbox image probe). Nothing pushed yet. Client notified.
- 2026-09-21 [autonomy: Moderate] PQ-2 at `review`: PR #9 (`ai-c11-cc/shared-web-layer` @ `223a201` on `0742d35`), verified head==remote, CI green. Receipt appended; surface:29 closed. Nothing newly dispatchable: T-19 waits on T-14, T-05/06/07 on T-04a–c.
- 2026-09-21 03:3x [waiting on client] PQ-3 (2 commits) at a `fly` prompt; PQ-18 (5 signed commits) at its push/`gh` prompt; PQ-19 at a `docker` prompt. Nothing blocked on the board; all three are in_progress and waiting in-tab.
- 2026-09-21 [autonomy: Moderate] PQ-18 pushed and PR #10 opened (`ai-c11-cc/bank-format` @ `b4d946c` on `0742d35`), CI green; ticket still `in_progress` pending its DONE comment. PQ-3 and PQ-19 remain at fly/docker prompts. Client asked for status; given.
- 2026-09-21 [autonomy: Moderate] PQ-18 at `review` (PR #10 @ `b4d946c`), DONE verified. Surface:31 closed. Press-ahead: T-19 (PQ-24) and T-17 (PQ-22) dispatchable off `origin/ai-c11-cc/bank-format`; two slots free (PQ-3, PQ-19 live).
- 2026-09-21 [autonomy: Moderate] Wave 3 dispatched by press-ahead off `origin/ai-c11-cc/bank-format` @ `b4d946c` (PR #10 at review): PQ-24 bank audit (surface:34), PQ-22 dedupe (surface:35). Boot prompts staged inside each worktree at `.claude/boot-prompt.md`. Both told: not a tutor; ssh-keygen signing fallback; stdlib only; PQ-24 touches only the bank-audit lines of the justfile because PQ-19 is editing it. Four delegators live (PQ-3, PQ-19, PQ-24, PQ-22).
| A freshly launched delegator stops on its very first command with "Contains brace with quote character (expansion obfuscation)" | The boot prompt's cwd guard `test … \|\| { echo "FATAL"; exit 99; }` trips the permission classifier's obfuscation heuristic, even under `--permission-mode auto` | Guard rewritten as `[ "$(pwd)" = "…" ] \|\| exit 99` with the message in a comment; template fixed for later waves. Tabs already stopped need one Yes from the client |

## ▶ RESUME HERE — Session 226a33ef pausing 2026-09-26 09:xx at the client's word ("after this set, pause and we'll refresh in a new session")

**Read the decision log entries dated 2026-09-26 first** (below the older RESUME note); they supersede it. Then `agents.md` and the last receipts in `delivery-receipts.jsonl`.

**State at pause (final, 09:4x).** `main` = `696b3ce`. No delegator is running; every tab is closed. Five PRs open, every one verified (head == remote, review attached) and at `review` on the board:

| PR | Ticket | Head | Base | CI at last check | Review |
|---|---|---|---|---|---|
| #14 | PQ-22 dedupe | `20f5cc9` | main | green | FAIL round 3, accepted ship-as-is by the client; C1–C7 on PQ-30 |
| #15 | PQ-4 phase machine | `f7faccd` | main | green | PASS |
| #16 | PQ-20 verifier | `ed20c2a` | main | green | PASS-WITH-NITS |
| #17 | PQ-6 transport | `7a67917` | **ai-c11-cc/phase-machine** | `test` green, `test-full` running | PASS-WITH-NITS (staged tree) |
| #18 | PQ-5 sessions | `dd36b44` | **ai-c11-cc/phase-machine** | running | PASS-WITH-NITS |

Held: PQ-11, PQ-13, PQ-21. Follow-ups minted: PQ-30 (dedupe false matches, the client's), PQ-31 (room correct-index twin, after PQ-4 + PQ-20), PQ-32 (wire the session map into the transport, after PQ-5 + PQ-6; PQ-7/8/9 consume the wired room). Amendments routed upstream: F-2, F-8, F-15…F-21.

**Waiting on the client:** merge words for #14, #15, #16, then #17 and #18. **Merge order matters:** before merging #15, `gh pr edit 17 --base main && gh pr edit 18 --base main` — GitHub auto-deletes the head branch on merge and a PR whose base vanished cannot be reopened. After #15 merges, #17 and #18 show only their own diffs against main; both merge cleanly on their own but touch `routes.rs` and `lib.rs` additively — merge one, then re-check the other's mergeability (union resolution if GitHub reports a conflict). F-15 ruling (does-not-compile trace) gates PQ-23; F-20 (normalized match → review queue) is recommended, not adopted.

**Templates are now durable** at `.lattice/orchestration/templates/` (`header.py`, `gen-wave5.py`, `gen-wave6.py`); the scratchpad copies are disposable.

**Next dispatchable (in order):** PQ-32 wiring (after #17 and #18 merge; fast-track); PQ-31 (after #15 and #16 merge; fast-track); then PQ-7 wall, PQ-8 buzzer, PQ-9 host off `main` once PQ-32 is at `review` (press-ahead off its branch is fine); PQ-23 review surface after the F-15 ruling; PQ-10 canary after PQ-7/8/9; PQ-12 static fallback after PQ-7. Fix `header.py` first: serial attach-then-status (footgun below), `git commit` with the sandbox bypass, receipts via `git -C <worktree>`. Templates: `header.py`, `gen-wave5.py`, `gen-wave6.py` in session 226a33ef's scratchpad (`/private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz/226a33ef-…/scratchpad/prompts/`); copy `header.py` somewhere durable first — scratchpads are reaped. Add to every new boot prompt: `git commit` runs with the sandbox bypass; the first permission prompt is answered with option 2 (switch to auto mode); receipts use `git -C <worktree>` values, never `$(pwd)` inside the `cd "$LATTICE_ROOT"` subshell.

**To resume:** `/lattice-orchestrator` with "resume the run; read .lattice/orchestration/run-state.md and agents.md first". Surface refs change when c11 restarts: `c11 tree` before touching any tab.

## ▶ (older) RESUME note — Run paused by the client 2026-09-22 01:2x (weekly usage: 75% gone)

The client paused the fleet at 23:48 on 2026-09-21 (in the dedupe tab) and at 01:2x
told the Orchestrator to save state and stop. The dispatch loop is stopped. The
delegator tabs are left open and idle; nothing runs.

**State at pause (updated 01:5x).** Done: PQ-1 (#8, `0742d35`), PQ-2 (#9, `36250b3`),
PQ-18 (#10, `238146e`), PQ-24 (#13, `d88fab4`), PQ-3 (#11, `77c4a32`), PQ-19 (#12,
`696b3ce`). `main` = `696b3ce`. No open PRs. M0 complete. Idle delegators, each with its
state on its ticket:
- PQ-24 bank audit: **done** since 01:45 (PR #13 merged at `d88fab4`); tab 34 closed.
- PQ-22 dedupe, tab 35, `in_progress`, PAUSED comment on the ticket: HEAD `f6a8971`, 3
  signed commits unpushed, review-round-1 fix written and uncommitted, 226 tests green.
  Resume steps in that comment; my ruling clears `test_migration.py:90` to it.
- PQ-20 verifier, tab 39, `in_planning`, READY posted, holding a `docker image inspect`
  prompt. Yes there lets it continue planning; it reads its handover notes on the ticket.
Held: PQ-11 (T-09), PQ-13 (T-10), PQ-21 (T-16). Amendments routed upstream: F-2, F-8,
F-15, F-16, F-17, F-18 (see `sequence/run-state.md`).

**Waiting on the client:** (1) Yes in tab 39 (PQ-20's Docker check). (2) The D-A go/no-go on PQ-3's numbers (AC-54 92 ms, AC-53 63 ms, AC-41 117 ms,
all far under the limits): PQ-4 (T-04a, phase machine) and with it all of M1 dispatch on
that word. (3) Fly: the org has no card, so machines stop after 5 min; HC-0's re-run
needs a card or the same windowing. The spike app `rustnyc-popquiz-spike` is scaled to
zero, not destroyed.

**To resume** in a fresh session: `/lattice-orchestrator` with "resume the run; read
.lattice/orchestration/run-state.md and agents.md first". Phase 0 collapses to
surface-and-confirm. Answer the two prompts (or the client does), tell tab 35 "resume"
by `c11 send`, then re-enter the dispatch loop (`/loop`, self-paced). Verify every DONE
comment against `gh pr view` and `git ls-remote` before appending a receipt.

**Next dispatchable:** PQ-4 on the client's D-A go (off `origin/ai-c11-cc/burst-spike`
until #11 merges, then `main`); PQ-23 (T-18) after PQ-20 reaches review **and** the F-15
ruling; PQ-5/PQ-6 after PQ-4. Boot prompts: `gen-wave4.py` in this session's scratchpad
(`/private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz/f8540abf-…/scratchpad/prompts/`)
is the current template; it imports the HEADER from wave 2's generator and rewrites the
stacking clauses for a two-parent branch. Stage in `<worktree>/.claude/boot-prompt.md`,
seed `.claude/settings.json`, launch one surface at a time, then expect one permission
prompt on the guard line (footgun below) and answer it with `send-key enter`.
- 2026-09-21 [autonomy: Moderate] Client answered tabs 34 and 35: PQ-24 in_progress, PQ-22 in_planning. Orchestrator loop paused for context budget; resume note written above.
- 2026-09-21 [client: "merge 9 and then merge 10"] PR #9 merged at `36250b3`, PR #10 at `238146e`, both with merge commits after CI green on their exact heads. PQ-2 and PQ-18 completed. Four live delegators told to rebase onto `origin/main`. `main` local ref updated by fetch.
- 2026-09-21 23:47 [autonomy: Moderate] Session `f8540abf` resumed the run from the RESUME note; preflight green (lattice 0.2.0, origin verified, c11). Phase 0 collapsed to surface-and-confirm; nothing re-asked.
- 2026-09-21 23:50 [autonomy: Moderate] PQ-3 DONE verified: PR #11 @ `ea858ec` on `0742d35`, head == `git ls-remote`, both CI jobs SUCCESS, MERGEABLE/CLEAN. Receipt appended. Surface:30 closed.
- 2026-09-21 23:52 [autonomy: Moderate] PQ-19 DONE verified: PR #12 @ `ff99bd2` on `0742d35`, head == remote, both CI jobs SUCCESS (first x86_64 image run), MERGEABLE/CLEAN. Receipt appended. Surface:32 closed.
- 2026-09-21 23:52 [autonomy: Moderate] Neither #11 nor #12 is rebased onto today's `main` (`238146e`); both merge cleanly (dry-run `git merge-tree`), so no rebase and no force-push was asked of anyone. The no-force-push rule stands: an in-review branch is never rewritten after its first push.
- 2026-09-21 23:53 [autonomy: Moderate] Wave 4, press-ahead: PQ-20 (T-15b verifier) dispatched at surface:39 in worktree `verifier`, branch `ai-c11-cc/verifier` off `origin/ai-c11-cc/sandbox-image` @ `ff99bd2`. It needs #12's files *and* main's bank format, so its first commit is a plain merge of `origin/main` @ `238146e` (no conflicts, dry-run from here); the brief forbids `git rebase` on that branch and opens the PR against `main` with a "based on #12" line. `pipeline/src/popquiz/sandbox.py` cleared for the Miri flag set only (PQ-19's handover notes recorded on PQ-20).
- 2026-09-21 23:54 [autonomy: Moderate] PQ-4 (T-04a) is board-dispatchable (its only dependency, PQ-3, is at review) but held for the client's D-A go/no-go, as the RESUME note and PQ-3's own comment require; surfaced to the client this tick.
| A freshly launched delegator stops on its first commands with "Contains shell syntax (string) that cannot be statically analyzed", offering "2. Yes, and switch to auto mode" | `claude --model opus --permission-mode auto` did **not** start PQ-20 in auto mode (verified 2026-09-21 23:57: plain Yes answered one prompt and the next command prompted again with the same offer). Auto mode's own prompts read differently ("Ask rule … overrides auto mode") | At the first prompt choose option 2 (`send-key down`, `send-key enter`); the status line then shows `⏵⏵ auto mode on` and the run continues. About 40 s per spawn. Next spawn: try `--permission-mode auto` again and check the status line before anything else |
- 2026-09-22 00:17 [client, in tab 35 at 23:48] **Run paused by the client for the weekly quota** (7-day usage 58%). PQ-22 saved state to a PAUSED comment (HEAD `f6a8971`, 3 signed commits unpushed, review-round-1 fix uncommitted). The Orchestrator learned of the pause on this tick; PQ-20 had been spawned at 23:53 without knowing it. No further spawns until the client says resume. PQ-4 stays held for the D-A go regardless.
- 2026-09-22 00:17 [autonomy: Moderate] Fleet is idle on two ask-rule prompts that are the client's: tab 34 (PQ-24, `gh pr create`), tab 39 (PQ-20, `docker image inspect` on the pinned image tag). Neither consumes quota while waiting. Tab 35 (PQ-22) paused. Nothing on the board is needs_human or blocked.
- 2026-09-22 00:17 [autonomy: Moderate] Ruling on PQ-22's FINDING: `pipeline/tests/test_migration.py:90` asserts `bank/history.json` equals the migrator's empty shape, which AC-17 contradicts once dedupe grows the file. T-14 is closed, so that one assertion is cleared to PQ-22 to narrow to the shape. Recorded as a deviation for the validator; no contract amendment needed (SPEC §3.3 already says history grows).
- 2026-09-22 00:17 [autonomy: Moderate] Loop cadence lengthened to 30 min while the client's pause holds; a tick still banners the two prompts and verifies any DONE that lands.
- 2026-09-22 01:2x [client: "save run state. 75% of usage is gone for the week"] Dispatch loop stopped; run-state, agents.md and receipts committed on `ai-c11-cc/lattice-build`. Two quiet ticks (00:49, 01:20) preceded this: no board, PR or tab change since 00:17. Delegator tabs 34, 35, 39 left open and idle. Not pushed.
- 2026-09-22 01:45 [client: "merge pr 13"] The client answered tab 34 after the pause; PQ-24 opened PR #13 (`ai-c11-cc/bank-audit` @ `45cd8d0` on `238146e`) and posted DONE with eight deviations. Verified head == remote, both CI jobs SUCCESS on the exact head, MERGEABLE/CLEAN. Merged with a merge commit at `d88fab4`. PQ-24 completed; receipts appended; surface:34 closed; PQ-20 told to merge `origin/main`, PQ-22 told to rebase on resume (its branch was never pushed). Local `main` ref updated by fetch. Loop stays stopped; the client's pause holds.
- 2026-09-26 03:4x [autonomy: Moderate] Session `226a33ef` resumed the run on the client's "continue work" (2026-09-25 15:0x) and "retry questions" (2026-09-26 03:3x). Preflight green (lattice 0.2.0, origin verified, c11 up). c11 had restarted: the run is now `workspace:1`, Orchestrator `surface:8` in `pane:1`; `pane:2` is the client's own terminal (`surface:10`), never used for spawns. Every delegator tab from before the pause is gone; both paused delegators (PQ-20, PQ-22) are respawned fresh in their existing worktrees from a `.claude/resume-prompt.md` that layers on the original boot prompt. The wave-2 scratchpad (the `HEADER` template) was reaped with its session; the template now lives at this session's `scratchpad/prompts/header.py` and `gen-wave5.py`.
- 2026-09-26 03:4x [client: "Go"] **D-A go** given on PQ-3's numbers (AC-54 92 ms, AC-53 63 ms, AC-41 117 ms, AC-52 exact). M1 opens: PQ-4 (T-04a phase machine) dispatched at `surface:18`, worktree `phase-machine`, branch `ai-c11-cc/phase-machine` off `origin/main` @ `696b3ce`, sub-agent-full run inline, actor `agent:delegator-pq4`. PQ-5 and PQ-6 press ahead off that branch when it reaches `review`; PQ-7/8/9 need PQ-2 (done) and PQ-4/5/6.
- 2026-09-26 03:4x [client: "Fresh round 3"] PQ-22 dedupe: the client authorized one more review cycle, scoped to the open Critical (function-parameter, closure and pattern scoping in the rename map), the attached never-equal regression, and the `test_migration.py:90` narrowing. Fresh delegator `agent:delegator-pq22r3` at `surface:19` in the existing worktree at `285b22d` (already pushed, fast-forward only from here). Round 3 is the last; a new class of FAIL stops at `needs_human` again.
- 2026-09-26 03:4x [autonomy: Moderate] PQ-20 verifier respawned at `surface:17` in worktree `verifier` (HEAD `9a758cd`, still `in_planning`). Rulings posted on the ticket: (1) `toolchain` mode in `sandbox.MODES` cleared; (2) dropping the `verify:T-15b` PENDING token cleared now that PQ-24 merged; (3) **F-19** — `bank.correct_index` derives only output and does-not-compile answers, so an accepted `panic` or `ub` question gets `None` or a distractor; run ruling clears PQ-20 to map those by option kind (`panic` ↔ non-zero exit, `ub` ↔ Miri UB with both borrow models agreeing), never by text; routed upstream as an amendment to §3.1. Its parent #12 merged, so its first act is `git branch --unset-upstream && git merge origin/main` (dry-run clean; the sandbox refused the unset from this seat: `.git/config` is not writable inside it).
- 2026-09-26 03:4x [autonomy: Moderate] Verified before dispatch: `origin/main` = `696b3ce`; remote branches are `main` and `ai-c11-cc/dedupe` @ `285b22d` only; PRs #6–#13 all MERGED, none open. Network from the sandbox is blocked for `gh` and `git fetch` (TLS through the proxy fails, ssh proxy refuses); both run with the bypass, one command at a time.
- 2026-09-26 04:0x [needs_human] **PQ-22 dedupe round 3 FAIL (final).** HEAD `20f5cc9`, four signed commits over `285b22d`, not pushed, no PR. The round-2 Critical is fixed (declared names carry token-range scopes; 18 rustc-probed twins pass; `test_migration.py:90` narrowed). The reviewer (Opus, 45 A/B pairs, 12,300 fuzz mutations) found seven open false-match classes: four of the same class (ranges drawn too wide: closures into if-let, items into nested mods, macro_rules, comma-less arms) and three new (path tails, lifetime vs identifier, free fn vs method). Orchestrator reading: the divergence pattern from the playbook — the acceptance language is the defect. AC-15 / SPEC §7.3 say a normalized duplicate is **rejected**; a token-level renamer (the round-1 deviation from "normalized AST") cannot decide that soundly, and each false match silently drops a real question. **F-20** routed upstream. Recommendation to the client: demote the normalized-duplicate verdict from reject to the review queue, marked *normalized duplicate of q* (exact hashes still reject; the organizer confirms). Alternatives: ship as is with the classes recorded; or a real Rust parser (a dependency, larger). Flag raised on `surface:19`; delegator told to hold and stay at `needs_human`.
- 2026-09-26 04:1x [client, in tab 19] **PQ-22: ship as is, with a follow-up ticket.** The client had the delegator push (`origin/ai-c11-cc/dedupe` = `20f5cc9`, fast-forward) and mint **PQ-30** "Dedupe: close the remaining false normalized duplicates" (bug, high, backlog) holding C1–C7 with their A/B pairs. Orchestrator: PQ-22 proceeds to PR → `review` with the seven classes and PQ-30 named as deviations; PQ-30 depends on PQ-22. F-20 (normalized match → review queue instead of reject) stays routed upstream as the amendment that would make the class moot; not adopted by this ruling.
- 2026-09-26 04:3x [autonomy: Moderate] PQ-22 at `review`: PR #14 (`ai-c11-cc/dedupe` @ `20f5cc9` on `main`; merge base `d88fab4`, GitHub MERGEABLE), head == `git ls-remote`, `just test` SUCCESS, `just test-full` in progress at receipt time. Review evidence is a FAIL (round 3 on `27b93fd`) accepted by the client as ship-as-is; recorded in the receipt so the validator sees it. Receipt appended; surface:19 closed. PQ-22 waits for the client's merge word. PQ-30 (depends on PQ-22) is not dispatched: it is the client's new ticket, outside the BUILDPLAN, and the run's contract check would need F-20 ruled first.
- 2026-09-26 04:4x [autonomy: Moderate] PQ-4 at `review`: PR #15 (`ai-c11-cc/phase-machine` @ `f7faccd` on `696b3ce`), head == remote, `just test` SUCCESS, `test-full` in progress, MERGEABLE; review PASS on `16471a6`, `f7faccd` a test-only Minor fix. Receipt appended; surface:18 closed. Rulings: deviation (1) kept as an exact mirror of `bank.py` for now — **PQ-31** minted (fast-track, depends on PQ-4 and PQ-20) to move the Rust twin to the F-19 by-kind rule once both land; deviation (2) recorded on PQ-25 (T-20's push arranges options into wall order); deviation (7) on PQ-27 (server diagnostics vs the copy lint).
- 2026-09-26 04:4x [autonomy: Moderate] **Wave 6, press-ahead off `origin/ai-c11-cc/phase-machine` @ `f7faccd` (PR #15):** PQ-5 sessions (surface:20, worktree `sessions`, branch `ai-c11-cc/sessions`) and PQ-6 transport (surface:21, worktree `transport`, branch `ai-c11-cc/transport`), both inline-full, in parallel on the same parent with a shared-file split (PQ-5 owns `rooms.rs`/`sessions.rs`/the `view.rs` per-session field; PQ-6 owns `ws.rs` and `Cargo.toml`; `routes.rs` additive blocks; whichever merges second merges main and wires the other's seam). **Retarget both child PRs to `main` before #15 merges** — GitHub auto-deletes the head branch on merge and a PR whose base vanishes cannot be reopened. Template: `gen-wave6.py` in this session's scratchpad.
- 2026-09-26 04:4x [waiting on client] PQ-20 (verifier) holds a `gh pr create` ask-rule prompt in surface:17; its push already landed (`origin/ai-c11-cc/verifier` exists). PR #14 (dedupe) has both CI jobs green and waits for the merge word; PR #15 waits for `test-full`.
- 2026-09-26 04:5x [autonomy: Moderate] PQ-5 went `needs_human` on signing: `Could not connect to socket (sandboxed)` then the ssh-keygen fallback's `communication with agent failed`. Diagnosis: the commit ran inside the Bash sandbox, which denies the 1Password agent socket; sibling tabs signed at 04:24 and 04:27. Told to re-run the commit with the sandbox bypass and resume. Footgun row added; every later boot prompt says `git commit` needs the sandbox bypass.
| A delegator's `git commit` fails with `1Password: Could not connect to socket` and the ssh-keygen fallback with `communication with agent failed`, while `ssh-add -l` lists the key | The commit ran inside the Bash sandbox, which denies unix sockets, including the 1Password agent's | Run `git commit` with the sandbox bypass on that one call (the permission prompt is expected). Boot prompts say so from wave 7 on |
- 2026-09-26 05:0x [needs_human, fleet-wide] **1Password signing stalled.** PQ-5's retry with the bypass still failed (`failed to fill whole buffer`; ssh-keygen fallback `communication with agent failed`). A signing probe from the Orchestrator's seat through the 1Password agent socket hung 25 s with no signature: the app is locked or holding an unanswered approval. The earlier diagnosis (sandbox) covered only PQ-5's first attempt. Client action: unlock 1Password / answer the git signing approval (or turn on "remember approval" for the key). PQ-6 and PQ-20 warned to stage and wait, never loop or bypass. Flag on the Orchestrator tab and PQ-5's tab; push notification sent.
- 2026-09-26 09:09 [client: "retry"] 1Password signing back (probe signed in 1 s). PQ-5 and PQ-6 told to retry their staged commits and continue to PR; flags lowered. Board committed at `94ed71f`. Client then said: **"after this set, pause and we'll refresh in a new session"** — the loop closes once PQ-20, PQ-5 and PQ-6 are at `review`.
- 2026-09-26 09:2x [autonomy: Moderate] PQ-20 at `review`: PR #16 (`ai-c11-cc/verifier` @ `ed20c2a` on `main`), head == remote, both CI jobs SUCCESS, MERGEABLE. `just test` 2.9 s, 459 tests. Eight deviations accepted, including (7) `bank.correct_index` now follows F-19 (dnc → UB under both models → non-zero exit = panic → output text), so **PQ-31** is live once #15 and #16 merge. **F-21 (found by PQ-20):** `check_provenance` cannot detect a self-consistent edit inside a record (stdout and option text changed together); SPEC §3.2 binds no field to the run. Proposal for tone-architect: a verifier-written digest over the record. Routed upstream. Note (c): a UB question must print only before its UB — a constraint for T-16's generator brief. Note (d): the justfile's reserved-name comment is stale for `verify` and `bank-audit` (cosmetic; for whoever next holds the justfile). Surface:17 closed.
- 2026-09-26 09:3x [autonomy: Moderate] PQ-6 at `review`: PR #17 (`ai-c11-cc/transport` @ `7a67917` **stacked on #15**, base `f7faccd`), head == remote, `just test` SUCCESS, `test-full` in progress, MERGEABLE; review PASS-WITH-NITS on the staged tree with doc-only fixes after (accepted: signing was stalled when it reviewed). Receipt appended; surface:21 closed. **PQ-32** minted (wire the session map into the transport; depends on PQ-5 and PQ-6) so neither branch wires the other's seam. PQ-6's deviation (3) (buzzer attach frame carries its own saved answer) goes into PQ-10's canary brief; (4) (`main.rs` must call `ws::serve`) into PQ-11's. PQ-5: four signed commits, at its `git push` prompt (client's).
- 2026-09-26 09:4x [autonomy: Moderate] PQ-5 at `review`: PR #18 (`ai-c11-cc/sessions` @ `dd36b44` **stacked on #15**, base `f7faccd`), head == remote, MERGEABLE, review PASS-WITH-NITS on the head, CI in progress at receipt time (re-check before merge). Receipt appended; surface:20 closed. **The set is complete: M1's PQ-4, PQ-5, PQ-6 and M3's PQ-20, PQ-22 are all at `review`.** Loop stopped at the client's word; resume in a fresh session from the RESUME note at the top.
| `lattice attach … && lattice status … review` run as parallel calls in one batch leaves the ticket at `in_progress` | The two writes race on this install and the later event wins | Delegators run attach, then status, serially, and verify with `lattice show --json`; the boot-prompt line "as parallel calls in one batch" is wrong for Lattice 0.2.0 — fix it in `header.py` before the next wave |
- 2026-09-22 01:5x [client: "merge all open prs if theyve been reviewed"] Both open PRs had a code-review verdict attached on their ticket (PQ-3: PASS-WITH-NITS on `c693574`; PQ-19: PASS-WITH-NITS on `f6fdf21`), plus a Validation note each. Re-verified CI SUCCESS on the exact heads and MERGEABLE/CLEAN after #13 moved main. Merged in order with merge commits: #11 → `77c4a32`, then #12 re-checked against the new main and merged → `696b3ce`. Both branches auto-deleted by GitHub. PQ-3 and PQ-19 completed; receipts appended. PQ-20 told its parent merged and to unset the dead upstream then merge `origin/main`; PQ-22 told main moved. `main` = `696b3ce`. M0 is complete; M3 has PQ-18, PQ-19, PQ-24 done. The client's pause and the stopped loop still hold; PQ-4 still waits for the D-A go.
