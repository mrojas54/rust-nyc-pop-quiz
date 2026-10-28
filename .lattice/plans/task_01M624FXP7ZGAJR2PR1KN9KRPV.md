# PQ-80: Organizer schedules the question from the host page (browser scheduling)

Michelle wants this eventually (2026-10-28): an organizer schedules the nights question from the host page while signed in, instead of `just schedule` with the admin token.

Source: Codex branch codex/app-question-scheduling, started 2026-10-11 on base 046779c (PR #67). One commit, ff574da "test: define organizer scheduling flow (RED)", plus uncommitted work across 26 files: GET /organizer/questions (approved, unused; id and topic only), POST /organizer/schedule (organizer auth, club-scoped, 409 on bad dates, unapproved ids and a question already held by a room), a host-page picker and date form, copy keys, and a SPEC.md "Organizer scheduling copy" section. The whole thing is saved as the attached patch.

Before building:
- Contract: D-20 says the laptop reaches the room through admin routes and one bearer token the client holds. This adds a second scheduling path that skips that token. Amend SPEC/BUILDPLAN first.
- The patch applies cleanly to 046779c but not to main: PQ-48 rekeyed scheduling by (club, question_id), and the room files have moved since. Port it rather than apply it.
- Untested: nobody has built the uncommitted half or run its tests.
