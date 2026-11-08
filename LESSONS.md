# Lessons

What the build run (2026-09-20 to 11-08, PQ-1 to PQ-89) learned the hard way,
written as rules. The evidence and the full audit are in
`.lattice/orchestration/closeout-audit.md` on the `ai-c11-cc/lattice-build`
branch. A lesson here is timeless: the failure, why it matters, the rule.

## Guardrail scanners parse; they do not pattern-match

**Failure.** A guardrail test that polices source syntax with a regex or a
hand-written tokenizer never converges under review. PQ-47's import scanner
failed Codex review passes 2 through 9, each pass finding a new
compiler-accepted bypass (a U+200E mark between `use` and `openai`). PQ-85
failed passes 2 to 5. PQ-61's YAML line scanner failed three passes.

**Why.** A regex can't match rustc's or YAML's grammar, so every fix exposes
the next bypass, and the two-pass review cap fires on a problem that more
passes can't solve.

**Rule.** Parse with the real parser. `syn` is already a dev-dependency of
`popquiz/` (`visit` feature), and `serde_yaml_ng` covers workflow YAML.
Otherwise, scope the check to the shapes the tree actually contains and refuse
everything else with a documented message. The plan says which before
implementation starts. If a scanner ticket reaches the review cap, the next
step is the parser, not pass 3.

## A brief clears the guard tests that read the files it touches

**Failure.** Sixteen tickets stopped for a clearance QUESTION because a guard
test outside their cleared files failed on a correct change.

**Why.** Each one cost a ruling round trip, and each was predictable from the
files in scope.

**Rule.** When a ticket touches the file in the left column, its brief clears
the test in the right column for one-line registrations:

| Touches | Also clear |
|---|---|
| room routes (`room::router()`) | `popquiz/tests/canary.rs`, `canary_full.rs` (the routes and screens they walk) |
| `room::config` env vars | `popquiz/tests/smoke_config.rs` (reads `DISCORD_VARS`) |
| the bank record fields, `bank/schema/question.schema.json` | `pipeline/tests/test_bank.py` |
| any pipeline code that starts a process | `pipeline/tests/test_runner.py` (`MAY_START_A_PROCESS`) |
| `pipeline/src` writers of the used record | `popquiz/tests/used.rs` (scans `pipeline/src` for `Used(`) |
| `.github/workflows/deployed-burst.yml` | `popquiz/tests/workflow_secrets.rs` and its pinned baseline |
| `CLAUDE.md` | `AGENTS.md` (`cp`), held by `web/test/agents-sync.test.js` |

Add a row when a new guard test reads another file.

## An amendment changes every contract file that carries the text

**Failure.** A ruling amended one contract file and left its sibling
contradicting it. SPEC §4 changed and AC-39 kept "letter and count" (F-49);
SPEC §11's host strings changed and EVALUATION's AC-45 row kept the old ones
(F-53). The Result Validator failed rows on the stale text.

**Why.** The terminal audit judges the text, not the decision log.

**Rule.** Before an amendment PR opens, grep `SPEC.md`, `EVALUATION.md`,
`sequence/USER_STORIES.md`, `web/shared/copy.js` and `popquiz/src/copy.rs` for
the old string or rule, and change every hit in the same PR. Copy is authored
in SPEC §11 first (PR #141 before #140's copy change).
