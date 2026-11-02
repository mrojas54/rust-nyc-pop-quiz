# PQ-53: The provenance check is not on the build path, so an uncommitted or local bank schedules without it

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-4** (G-2, Major, new): The provenance check is not on the build path, so an uncommitted or local bank schedules without it
- Evidence: check_provenance (verify.py:577) runs in CI over committed files only (test_verify.py:425-428); schedule.refusal (schedule.py:127) and fallback.bake (fallback.py:105) skip it; the replay covers MIGRATED only (migrate_mvp.py:76); M2a survived
- What would close it: Call check_provenance in refusal and bake, and replay every bank question that has a recording

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-01)

Re-verified on e073ec7: `check_provenance` (verify.py:632) has no caller in `pipeline/src`.

## Routes to a scheduled record or a fallback

| Route | Through | Checked after this change |
|---|---|---|
| `just schedule` (with `--room` or `--no-push`) | `schedule._schedule` -> `refusal`, then `build_html`/`host_sheet`/`write_fallback` -> `bake` | yes, both |
| Organizer TUI (`prototypes/organizer-tui/src/scheduling.rs`) | subprocess of `popquiz.schedule schedule` | yes, same path |
| `fallback.build_html`, `host_sheet`, `write_fallback` | each calls `bake`; `fallback` has no command of its own | yes |
| `just demo` | curl PUTs the stored file straight to the room | **no**: neither function. `justfile` is not cleared. Asked by comment and mail; without clearance it goes to DONE as not-done |
| Hand curl to `/admin/questions/{id}`; `deployed-burst.yml` (q3 under harness ids) | the room's admin channel | **no**: the room's `answers::load_arranged` reads `verified_at` but does not refuse on it. Room code is not cleared. DONE as not-done |

Loading: `load_question` and `load_bank` both go through `bank.question_from_dict`, which refuses unknown fields, `correct` among them, so every `Question` reaching `refusal` or `bake` from a file has already passed that. The `correct` branch stays in `check_provenance` for its more specific message.

## The change

1. `verify.py`: move the checks a loaded record can still fail out of `check_provenance` into `check_question(question)`. `check_provenance` keeps the raw-file part (`correct`, the load) and calls it. No body is copied and no text changes.
2. Import cycle: none. `verify` imports `bank`, `sandbox`, `runner`; `runner` imports `sandbox`; none reaches `schedule`, `fallback` or `audit`. Checked again by importing each first in a fresh interpreter.
3. `schedule.refusal`: the check is the last refusal, after the reserve check, and returns the `ProvenanceError` text.
4. `fallback.bake`: the check is the last refusal, raised through `_refuse`. `_refuse` prefixes the id and the text already starts with it, so the leading id is removed once. The wording is otherwise the existing text. In the CLI an organizer sees `refusal`'s line, since it runs first.
5. Real questions: `refusal` and `bake` outcomes for q3, q4, q7, q8 captured before the change (bake digests, stored order and three dates); compared after. Any change halts.

## The replay

A recording is `pipeline/tests/fixtures/verify/recordings/<program>.json`, made by `record.py` on the pinned image. Recordings for bank ids today: q3, q4, q7, q8, exactly `MIGRATED`, so coverage is complete but by coincidence of a constant. `test_the_correct_option_is_the_machines_output` will parametrize over bank ids that have a recording, derived from the two directories, and a guard test will fail if a `MIGRATED` question loses its recording or the helper leaves a recorded question out. The replay stays a test: it is not on the build path, because a laptop has no pinned image and the record has no field binding it to a run.

## Tests and mutations

- `test_verify`: `check_question` refuses what `check_provenance` refuses on a loaded question, with the same text. Mutation: remove a check from `check_question`.
- `test_schedule`: M2a through the CLI (non-legacy, no `verified_at`/`verifier_version`, affirmed, copied bank) refused with nothing sent or written; a self-contradicting record refused; a used or unaffirmed question with a bad record still reads as it does today. Mutation: remove the call from `refusal`.
- `test_fallback`: the same records refused by `bake` and writing neither file. Mutation: remove the call from `bake`.
- `test_migration`: the derived replay. Mutation: helper returns a constant list.
- Audit's M2a: rerun by hand against a copied bank with the real CLI at validation.

## Stated limit

A self-consistent hand edit inside a record still passes, on every route. No digest field is added. Docs and the PR say so.

## Found, not fixed

`audit.in_reserve` does not apply the check, so the reserve line can count a record `schedule` refuses. `audit.py` is not cleared and a changed count changes what the organizer reads. Goes to DONE.
