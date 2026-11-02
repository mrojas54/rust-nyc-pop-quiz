# PQ-50 checkpoint (after plan)

Plan: /Users/michellerojas/rust-nyc-pop-quiz/.lattice/plans/task_01M431PWQ4QF5KBCENBTM9YW02.md
Branch ai-c11-cc/pq-50-validation-small-items, base 622e0fd.

Open questions posted (comment and mail): item 4 needs web/wall/fallback/static.js cleared; item 6 options A/B/C for Michelle.
Next: one commit per remaining item, just test, Codex review, validate, PR, stop at review.

## After implement + just test (HEAD 0dfe981)
Item 4 cleared by RULING (static.js only). Nine commits on the branch, not pushed. just test exit 0.
Item 6 still waits on Michelle. Ruling: if unanswered when the rest is ready, stop before the PR and say so; no polling.
Next: Codex review, a11y, canary, then stop. Mutations so far: drop clear -> "Esc back to idle empties"; restore dim rule -> "ac80".

## Before Codex pass 1 (HEAD 54f58ba)
Item 6 answered: RULING option A, done. just test green: cargo 345, node 305, pytest 1149 + 1 skipped. Next: Codex pass 1, then a11y, canary, push, PR, stop at review.

## PR open (HEAD 54f58ba2319f4372f3e56d006712974acb952dbc)
PR https://github.com/mrojas54/rust-nyc-pop-quiz/pull/118, CI green, ticket in review, DONE posted. Remaining: Michelle's merge word and the NotYetOpen question.
