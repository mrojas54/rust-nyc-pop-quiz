# PQ-83: PRD.md: mark the superseded 79.99/0.01/20 category weights

PRD.md still states the version-one answer-category weights 79.99% / 0.01% / 20% as a product constant (lines ~346-363, ~774-777 on main 0ca67a1). CLAUDE.md records that a fixed, published distribution is superseded (PHILOSOPHY.md section 2), and PQ-62 exempted PRD.md as a whole from the distribution scan (DISTRIBUTION_EXEMPT_PATHS, audit.py:925) rather than editing it. PRD.md is a prior artifact, so the edit is the client's call: options are a superseded banner at each passage, or striking the passages. Acceptance: the client approves exact wording; the PRD.md path exemption is narrowed or removed and the distribution scan still reports 0 hits; just test green.

# Plan (delegator, 2026-11-04)

Base `origin/main` @ `0ca67a1`. Survey: `rg` for every number, "weight", "prior", "nonuniform",
"almost always", plus the AC-25 scan (`audit.distribution_hits`) run on `PRD.md` with the path
exemption ignored. The scan flags **8 lines**: 346, 347, 351, 352, 353, 773, 774, 974. Two more
passages carry the vector but escape the scan (no kind word within a line): 363 and 899. One
states the tell in words with no number: 354–356 ("can rationally almost always ignore
**does not compile**"). Drafts were checked by building both variants in scratch and scanning
them: option 2 → 0 hits, no line over 80 columns, none of 79.99 / 0.01 / 26.663 / 0.007 /
0.013 / 0.0001 left; option 1 → the same 8 lines still hit.

## Passages

| # | Lines | What stands there | Scan hit |
|---|---|---|---|
| P1 | 346–348 | last sentence of the sampler bullet: "The version-one weights are 79.99% … 0.01% … 20% …" | 346, 347 |
| P2 | 349–353 | label priors 26.663% / 0.01% / 20%, and position 20% | 351, 352, 353 |
| P3 | 354–356 | "a repeat attendee can rationally almost always ignore **does not compile**" (tell in words) | — |
| P4 | 363–366 | the 79.99/0.01/20 vector as an immutable product constant | — (363 has the vector) |
| P6 | 772–775 | million-draw simulation targets 0.007%–0.013%, 0.25 pts of 20% | 773, 774 |
| P8 | 898–901 | test item "the 79.99/0.01/20 semantic-label priors" | — (899 has the vector) |
| P9 | 974–975 | "The 0.01% compiler-error target is an intentional, learnable answer prior" | 974 |

Left verbatim, on purpose (say so if you want them changed):
- 372–373 "Apart from the deliberately nonuniform semantic-label priors above": no number, no
  category named; under option 2 "above" still points at P2's superseded note.
- 776–779 "each final choice position correct within 0.25 percentage points of 20%": the
  position odds are chance (1 in 5) and AC-23 still requires them; not a category weight, not a
  scan hit. Under option 2 its "the same simulation" still has P6's note as antecedent.

## Option 2 — strike (recommended). Exact replacement text

P1 (346–348; lines 343–345 of the bullet unchanged):
```
  participant-observable state. *Superseded: version one sampled answer
  categories from a fixed, published weight vector; no distribution is
  published now (`PHILOSOPHY.md` §2, `CLAUDE.md` "Standing of `PRD.md`").*
```
P2 (349–353, whole bullet):
```
- *Superseded: version one's weights made the semantic answer labels
  deliberately nonuniform and published each label's prior; no distribution is
  published now (`PHILOSOPHY.md` §2, `CLAUDE.md` "Standing of `PRD.md`").*
```
P3 (354–356, whole bullet):
```
- *Superseded: version one accepted, as a product tradeoff, that its weights
  let a repeat attendee rule out one answer label; no distribution is published
  now (`PHILOSOPHY.md` §2, `CLAUDE.md` "Standing of `PRD.md`").*
```
P4 (363–366, whole bullet):
```
- *Superseded: version one fixed its weight vector as an immutable product
  constant; no distribution is published now (`PHILOSOPHY.md` §2, `CLAUDE.md`
  "Standing of `PRD.md`").*
```
P6 (772–775, whole bullet):
```
- *Superseded: version one set a target share for each answer category in a
  million-draw sampler-only simulation; no distribution is published now
  (`PHILOSOPHY.md` §2, `CLAUDE.md` "Standing of `PRD.md`").*
```
P8 (898–901; the numeric item is dropped from the list, the rest kept):
```
- Enforcement of the answer distribution: per-slot independence, room-size
  independence, uniform secure position shuffling, absence of additional
  category-correlated cues, and the defined empirical tolerances. *Superseded:
  version one also enforced its published semantic-label priors; no
  distribution is published now (`PHILOSOPHY.md` §2, `CLAUDE.md` "Standing of
  `PRD.md`").*
```
P9 (974–975, whole bullet):
```
- *Superseded: version one accepted a learnable answer prior as a tradeoff
  against its repeat-attendee anti-gaming objective; no distribution is
  published now (`PHILOSOPHY.md` §2, `CLAUDE.md` "Standing of `PRD.md`").*
```
No replacement names a category's share or which category was rare.

Code under 2: delete the `"PRD.md"` entry from `DISTRIBUTION_EXEMPT_PATHS`. In
`test_audit.py`, move `"PRD.md"` from `test_skipped_and_exempt_paths_are_not_read` to
`test_a_tell_anywhere_authored_is_caught`, and add it to
`test_every_authored_file_outside_bank_is_in_scope`.

## Option 1 — mark. Exact note text

One note bullet inserted directly above each of the seven bullets (P1–P4, P6, P8, P9), each
passage kept verbatim:
```
- *Superseded: the next item states version one's fixed, published category
  weights; no distribution is published now (`PHILOSOPHY.md` §2, `CLAUDE.md`
  "Standing of `PRD.md`").*
```
The numbers stay public, so the scan still flags **8 lines**. Replacement for the path entry:
8 `LINT_EXEMPTIONS` entries keyed on those exact lines (the existing
`test_the_exempt_line_is_still_what_was_reviewed` re-arms each on any edit), or keep the path
entry with its reason rewritten to "marked superseded, numbers kept at the client's ruling".

## Mutations (after implement)

- M1: put `20% undefined behavior` back into one passage → `test_the_real_participant_facing_files_are_clean` reports `PRD.md:<line>`.
- M2: restore the `"PRD.md"` path entry → `test_a_tell_anywhere_authored_is_caught[PRD.md]` and the in-scope test fail.

## Commits

1. `PRD.md`: the approved wording only. 2. `audit.py` exemption + `test_audit.py`. Then `just test`,
Codex review (≤2 passes), validate, PR, stop at `review`.
