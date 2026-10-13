# PQ-72: PQ-23 follow-ups: re-verify-path tests, quotes_checked docs, review.rs nits

Nits 3–5 from the orchestrator's exact-head review of PR #80 (comment on PQ-23). Nits 1–2 were fixed in PR #81 (26cccc0).
3. Tests: rejected-stays-rejected is covered only on the no-program-change edit path. Add the re-verify path (_verify_into_bank) and the unreviewed case; add an identical-content edit of an affirmed question.
4. review.py: quotes_checked survives edits while the span text survives. Probably intended (AC-73 keys on the span) but undocumented: document it.
5. review.rs:235 passes the id positionally with no `--` (real ids are q<N>); review.rs:321 sets `changed` on "Not recorded" (harmless reload).
Also deferred by the PQ-23 delegator (triage, not required): multi-blocker sentences, newline in --by, predictable editor temp path, panic-unsafe env cleanup, absolute git-status guard, Esc can't cancel re-verify.
