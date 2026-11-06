# PQ-80: Organizer schedules the question from the host page (browser scheduling)

Michelle wants this eventually (2026-10-28): an organizer schedules the nights question from the host page while signed in, instead of `just schedule` with the admin token.

Source: Codex branch codex/app-question-scheduling, started 2026-10-11 on base 046779c (PR #67). One commit, ff574da "test: define organizer scheduling flow (RED)", plus uncommitted work across 26 files: GET /organizer/questions (approved, unused; id and topic only), POST /organizer/schedule (organizer auth, club-scoped, 409 on bad dates, unapproved ids and a question already held by a room), a host-page picker and date form, copy keys, and a SPEC.md "Organizer scheduling copy" section. The whole thing is saved as the attached patch.

Before building:
- Contract: D-20 says the laptop reaches the room through admin routes and one bearer token the client holds. This adds a second scheduling path that skips that token. Amend SPEC/BUILDPLAN first.
- The patch applies cleanly to 046779c but not to main: PQ-48 rekeyed scheduling by (club, question_id), and the room files have moved since. Port it rather than apply it.
- Untested: nobody has built the uncommitted half or run its tests.

# Plan (delegator, 2026-11-06)

Base `origin/main` @ `c810d0dc44955d69f3679b35f67523ec91c8423e`. Worktree
`~/rust-nyc-pop-quiz-worktrees/pq-80-browser-scheduling`, branch
`ai-c11-cc/pq-80-browser-scheduling`.

**Client steer, 2026-11-06 (panel, "continue b"):** design option **(b)**. The
laptop pushes the questions over the existing admin channel, and the phone picks
one. That steer chooses the option. It is not the `RESUME`: the contract wording
below still waits for her answer to the QUESTION.

## What (b) means, in one paragraph

Nothing new carries an answer. `popquiz schedule <id> --date <meetup> --club <c>
--room <url>` already pushes one arranged record per run, and the room already
holds any number per club (`questions: (club, id) → Scheduled`, D-26/PQ-48). The
organizer runs it once per question on a shortlist, ahead of the night, and each
run writes that question's static fallback and host sheet as today. On the night
the signed-in host page lists what is scheduled for that club (id and topic),
the organizer picks one, and *Create a room* creates the room on it through the
existing `POST /rooms` with the existing refusals. One new read-only route, no new
write route, no date form, no arrangement in the room, no catalog in the image.

**What the organizer gives up:** the laptop is still needed, before the night,
for the push. **What they gain:** the choice moves to the phone and can be made
in the room. If the room restarts (any merge to `main` is a deploy), the pushed
records are gone, as they are today. The picker then says nothing is scheduled.
The static fallbacks on the laptop still work.

## 1. The patch, file by file (keep / port / drop)

| Patch file (2026-10-11 layout) | Today | Verdict |
|---|---|---|
| `.dockerignore`, `Dockerfile` (bank into the image) | — | **drop**: no bank in the image under (b) |
| `SPEC.md` *Organizer scheduling copy* | §8.5 + a §11 row | **drop**; replaced by the approved wording below |
| `room/Cargo.toml`, `Cargo.lock` (`blake2`) | `popquiz/Cargo.toml` | **drop**: no new dependency (checked: nothing in (b) needs one) |
| `room/src/answers.rs` (`load_record`, `pub mod catalog`) | `quiz_session/models/answers.rs` | **drop**; the only change there is §4's topic onto the public half at the split |
| `room/src/answers/catalog.rs`, `room/build.rs` | — | **drop** (option a) |
| `room/src/slot.rs`, `bank/fixtures/scheduling-order.json`, `pipeline/tests/test_browser_schedule_order.py` | — | **drop** (option c; `pipeline/**` is PQ-58's) |
| `room/src/copy.rs`, `web/shared/copy.js` (16 `host_schedule_*`) | `popquiz/src/copy.rs`, `web/shared/copy.js` | **port** as 6 `host_pick_*` keys (§5) |
| `room/src/discord.rs` `begin` accepts an empty question | `quiz_session/host/discord/mod.rs` | **port**, so `/host` with no `?question=` can sign in |
| `room/src/rooms.rs` `organizer_questions` | new `quiz_session/host/choices.rs` | **port**, re-sourced from the scheduled map, not a catalog |
| `room/src/rooms.rs` `schedule_from_bank`, the used check in `schedule_for` | — | **drop**: main keeps the used refusal per club in `create_for` on purpose |
| `room/src/routes.rs` `GET /organizer/questions` | `server/routes/host.rs` | **port** (no-store, no-referrer) |
| `room/src/routes.rs` `POST /organizer/schedule` | — | **drop**: under (b) there is no browser write |
| `room/tests/auth.rs` (empty question signs in) | `popquiz/tests/auth.rs` | **port** |
| `room/tests/canary_scan/mod.rs` (two `UNSCANNED` rows) | same | **port as `DRIVEN`**: the walk scans the new route with the plants |
| `room/tests/organizer_schedule.rs` (ff574da + WIP) | `popquiz/tests/organizer_questions.rs` | **port** the auth, list-shape, no-admin-token, club and released cases; **drop** the date, catalog and order cases |
| `room/tests/scheduling_preview.rs` | — | **drop** |
| `web/host/host.js`, `host.css` | same | **port** loading, 401, reload and escaping; **drop** the date form, schedule submit, edit and *existing* states |
| `web/test/host.test.js`, `a11y.test.js` | same | **port** |
| `web/test/host-scheduling.test.js` | `web/test/host-pick.test.js` | **port** without the schedule and date cases |

## 2. The six design decisions

1. **Where the questions come from: (b), the client's pick.** (a) bakes every
   affirmed record into the binary. That puts an answer into the room by a route
   other than the admin channel, which contradicts AC-101 as written ("accepts a
   scheduled question — answer included — … only over an authenticated admin
   channel"). It also makes the room a second arranger, and a new question then
   needs a deploy. (c) keeps the answer on the admin channel, but the room
   arranges, in a Rust twin of `slot_for_day` that must stay bit-identical to the
   Python draw. That is a second writer in the area that has regressed four
   times (PHILOSOPHY §2). (b) touches neither the answer nor the arrangement.
   Every candidate pushed for a date has its answer at `slot_for_day(date)`, so
   the pick cannot move it (AC-23).
2. **Auth and scope.** The route takes the organizer session (`Authorization:
   Bearer`, from the fragment the Discord callback lands on) and runs the same
   `HostAuth::authorize_create(bearer, club)` as `POST /rooms`: that club's role
   ID, AC-65/AC-103. Today the session is not bound to a club, so the club is
   named in the request and authorized by that club's role, exactly as *Create a
   room* does. Binding the session to one club would change the Discord module
   and `POST /rooms` too, so it is out of scope (deviation D1). The admin check,
   `server/admin.rs` and `admin_routes` are untouched. The canary's
   `admin_probe` already proves that an organizer bearer opens nothing under
   `/admin`, and the route-table test (AC-101) picks up the new route from
   `server/routes/` and fails until it is `DRIVEN`.
3. **The host page.** When signed in, `/host[?question=…][&club=…]` loads the list,
   then shows a `<fieldset>` with the legend **Question** and one radio per
   question (`‹id› · ‹topic›`), with the link's question preselected when it is
   listed (or the only one, when only one is listed). Below that is the existing
   **Create a room** primary, disabled until a radio is chosen. Radios rather than
   a `<select>`: a shortlist of a few is read whole on a phone, each row is a 44
   px target, and the checked state is the native mark (not colour, AC-40).
   Arrow keys move inside the group and Tab reaches *Create a room*. Long topics
   wrap and never scroll sideways (AC-33). The empty and failed states have a
   **Reload** button. A 401 returns to *Sign in with Discord*, as create does.
   Phone width is designed first. `web/test/a11y.test.js` gets the new screens
   (pick, pick chosen, empty, failed), and `just a11y` runs.
4. **State.** No new state. The listing reads the existing `questions` map:
   that club's records that are affirmed (`Scheduled::refusal` is `None`), not
   used by the club (`used.contains`), and not held by a live room
   (`Room::holds`, the same test `create_for` uses), sorted by id. It returns
   `{"questions":[{"id","topic"}]}` and nothing else. The topic is not answer
   material. It is copied onto the public half (`PublicQuestion.topic`) when
   `answers::load` splits the record, so the listing reads only the public half,
   which adds no new reader of the vault. It never returns options, source, hint,
   explanation, receipt, `bank_order`, the slot or a date.
5. **`just schedule` stays the one way in.** There is one scheduling path, the
   admin `PUT`. Parity test `picker_lists_exactly_what_the_admin_channel_scheduled`
   (in `organizer_questions.rs`): push q3 for nyc and another record for la over
   `PUT /admin/…`. The nyc list then holds exactly q3. The room created from the
   pick equals the room created from `/host?question=q3`, with the same host
   projection and the same refusals. Releasing it removes q3 from the list.
6. **Tests, in order.** (i) The RED port: `popquiz/tests/organizer_questions.rs`
   (no session or an unknown one is 401; another club's role is 403; the list
   holds exactly `id`/`topic`; create without the admin token; club scoping;
   released and held questions drop out; `no-store`/`no-referrer`), plus
   `web/test/host-pick.test.js`. Both fail first. (ii) The route is `DRIVEN` in
   `canary_scan`, so the plants are scanned in the list response. (iii) In
   `auth.rs`, an empty question signs in, and the bad ids are still 400. (iv) The
   copy twin tests come for free through `copy::ALL` and `COPY_ROWS`, and the
   reason lint passes unchanged because no new reason string is added: every
   refusal is an existing `copy::` constant. (v) The a11y screens are added.
   (vi) The burst/smoke harness and `tests/boundary.rs` are untouched; boundary
   must still pass with the `topic` field.

## 3. The 409 table (all existing, via `POST /rooms`)

| Case | Refusal | Source |
|---|---|---|
| Not scheduled for this club | `copy::ROOM_NOTHING_SCHEDULED` | `create_for` |
| Already run by this club | `copy::ROOM_QUESTION_ALREADY_RUN` | `create_for` (AC-105) |
| A live room holds it | `copy::ROOM_QUESTION_OPEN_ELSEWHERE` | `create_for` (GAP-8) |
| Unaffirmed or stale | `Scheduled::refusal` | `create_for` (AC-72, G-12) |
| No session / another club's role | 401 / 403 `wrong_role` | `authorize_create` |

The listing hides the first four, so a refusal on create means the state changed
between the load and the press. The page shows the reason and reloads the list.
The date cases (not a meetup date, past) are the laptop's, as today:
`popquiz schedule --date` takes the date.

## 4. Contract amendment drafts (for the QUESTION, verbatim)

**SPEC §8.5 (new, after §8.3; §8.3 untouched):**

> ### 8.5 Choosing the question on the host phone (PQ-80)
>
> A signed-in organizer can pick the night's question on the host phone instead
> of naming it in the host link. `GET /organizer/questions?club=‹club›` takes the
> organizer session and the same check as *Create a room* (§8: that club's role,
> AC-103). It answers with the questions scheduled for that club that the club can
> still run (affirmed, not used by the club, held by no room) as `id` and `topic`
> only, with `Cache-Control: no-store`. The organizer picks one and creates the
> room on it through `POST /rooms` as before, with the same refusals. The host
> phone schedules nothing: a question still reaches the room only as §8.3
> describes, arranged for the club's meetup date by `popquiz schedule` on the
> laptop, and the organizer may push several that way before the night to choose
> between. Listing and picking use nothing (G-10). The listing reads the
> question's public half alone; no answer, option order or arrangement leaves the
> sealed module (AC-61). D-20 stands: the admin channel is still two routes and
> one token, and the organizer session does not open it.

**SPEC §11, one new row (after *Host, actions*):**

> | Host, choosing the question (§8.5) | legend **Question** · each choice `‹id› · ‹topic›` · *Loading questions…* · *Nothing is scheduled for this club. Schedule one on the laptop, then reload.* · **Reload** · *Couldn't load the questions.* |

**BUILDPLAN, a dated note under D-E and after the T-25 row:**

> *2026-11-06 (PQ-80):* the host phone lists the questions scheduled for the
> organizer's club and creates the room on the one picked (`SPEC.md` §8.5). This
> is not a third admin route or a second way in. The answer still enters only
> through `PUT /admin/…` with the admin token, and the listing takes the
> organizer session, which never opens `/admin`. The server may hold several
> scheduled questions per club; the organizer chooses among them on the night.

**BUILDPLAN, a row in M4 after T-24:**

> | **T-27** | Choosing the question on the host phone (PQ-80): `GET /organizer/questions` (organizer session, the club's role, `id` and `topic` only), the host page's picker, and the runbook's shortlist push | T-07, T-10, T-25 | `organizer_questions.rs`, `host-pick.test.js`, the canary walk, `just a11y` |

**New criterion: recommend none.** AC-101 already confines the answer to the
admin channel, and (b) keeps it there. AC-72 and G-12 confine scheduling to
affirmed questions. AC-92 and G-10 cover use at release. AC-103 covers the club's
role, and AC-61 covers the sealed module. If she wants the phone's choice under
its own ID, the draft is:
**AC-106** *A signed-in organizer can choose the night's question on the host
phone from the questions scheduled for their club, shown by id and topic only.
Choosing uses nothing, and the question still reaches the room only over the
admin channel (AC-101).* (USER_STORIES row + EVALUATION line; tone-architect's
call.)

## 5. Copy keys (new; no existing sentence changes)

| Key (`copy.rs` / `copy.js`) | Text |
|---|---|
| `host_pick_legend` / `HOST_PICK_LEGEND` | Question |
| `host_pick_choice` / `HOST_PICK_CHOICE` | ‹id› · ‹topic› |
| `host_pick_loading` / `HOST_PICK_LOADING` | Loading questions… |
| `host_pick_empty` / `HOST_PICK_EMPTY` | Nothing is scheduled for this club. Schedule one on the laptop, then reload. |
| `host_pick_reload` / `HOST_PICK_RELOAD` | Reload |
| `host_pick_load_failed` / `HOST_PICK_LOAD_FAILED` | Couldn't load the questions. |

`COPY_ROWS` gets the row label "Host, choosing the question (§8.5)". No room
reason string is added.

## 6. Milestones (each `just test` green, then a checkpoint)

0. The approved contract wording, verbatim, as its own commit (SPEC §8.5/§11,
   BUILDPLAN, plus AC-106 only if she asks for it).
1. RED: `organizer_questions.rs` and `host-pick.test.js` fail for the missing
   route and screen. (This one is committed red-then-green with M2/M3; the
   failing run is shown in the PR.)
2. Room: `PublicQuestion.topic`; `host/choices.rs` (`AppState::choices_for`);
   `GET /organizer/questions` in `routes/host.rs`; the `DRIVEN` row and the
   scan; `begin` accepts an empty question; the `auth.rs` case.
3. Copy keys, then the host page (`host.js` `renderPick`, `host.css`), the a11y
   screens, and `host.test.js`.
4. Docs: `docs/RUNBOOK.md` (push the shortlist, pick on the phone, and a restart
   forgets the shortlist), `popquiz/README.md` route table, `web/README.md`.
5. Codex code review (at most two passes), validation (`just test`, `just a11y`,
   a c11 browser run against a local room), then the PR.

## Deviations (proposed)

- **D1.** The club is named in the request and authorized by that club's role,
  as `POST /rooms` does. The session is not bound to a club, against the boot's
  "club from the session". Binding it would change the Discord module and the
  create route as well.
- **D2.** The RED test from ff574da is ported in part. Its schedule, date and
  catalog cases test option (a)/(c) machinery that (b) does not build.

## Plan-review resolutions (AUTHORITATIVE)

Codex plan review, 2026-11-06 (codex-cli 0.167.0, prompt only, against
c810d0d): **FAIL**, two Majors, four Minors, one NIT. It agrees (b) is the
safest option for the sealed answer. Each finding below was checked against the
code before it was accepted. Where this block and §§1–6 disagree, this block wins.

**R1 (Major, accepted): the shortlist must be tied to its meetup date.**
Checked: the admin `PUT` body carries no date, and `Scheduled`
(`answers.rs:660`) keeps none. Shortlist records nobody picked therefore stay
selectable at the next meetup, with the old date's answer slot, until the room
restarts. §2.1's "every candidate shares the date's slot" holds only on the
night they were pushed for. Two fixes; **recommend R1-a**:
- **R1-a (date travels with the record).** `popquiz schedule` adds
  `"meetup_date": "YYYY-MM-DD"` (the `--date` it arranged for) to the `PUT`
  body. The room stores it beside the record as plain metadata: it is not the
  slot, and the room arranges nothing. A record lists, and creates, only when
  its date equals the club's meetup date today (`used::meetup_date_in(now,
  zone)`, the one the release ledger already uses). `create_for` gains one
  refusal, `ROOM_QUESTION_OTHER_DATE` (§5 below). This also closes the same
  hole on today's `/host?question=` path. *Cost:* a small hunk in
  `pipeline/src/popquiz/schedule.py` `push` plus its test (`pipeline/**` is
  PQ-58's this wave, so it needs clearance), and `server/admin.rs` refuses a
  body without the date with 400. The laptop and the room ship in one PR, so
  there is no mixed version.
- **R1-b (no pipeline change).** At release, the room drops that club's other
  scheduled records. Dropping them does not use them (G-10). This misses rooms
  that expire unreleased, and it forbids pushing more than one meetup ahead.

Also corrected (§3): `--date` checks only YYYY-MM-DD syntax, not "past". Under
R1-a, the room's today-equality covers both a past date and a future one.

**R2 (Major, accepted): the full auth matrix.** §3 gains rows:
`403 wrong_server` (not in the guild, `HOST_DENIED_WRONG_SERVER`); an unknown or
malformed `club` returns `403 wrong_role` (`HOST_DENIED_WRONG_ROLE`, via
`club_of`, the same answer as a missing role), never an empty list or a 404; an
expired or revoked session returns 401; Discord unreachable returns 503, which is
not a denial. Each row gets a route test, and the 401 and 403 rows get a host-page
test. D1 stands and is stated in §8.5: the request names the club, and that
club's role authorizes it.

**R3 (Minor, accepted): one eligibility predicate.** A read-only
`eligibility(questions, rooms, used, club, id, now) -> Result<(), refusal>` in
`quiz_session/host/` serves both the listing and `create_for`. Create still
calls it under its existing lock order (`questions`, then `rooms`) and stays
authoritative. The listing takes the same locks in the same order and only
reads. Tests: an unaffirmed or stale record, an expired room (listed again), a
wrong date (R1-a), and another organizer creating or releasing between the GET
and the POST (409 with the existing reason, kept on screen across the reload).
Any new file that returns a refusal joins `reason_copy.rs`'s scanned sources.

**R4 (Minor, accepted): the bounded sealed-data promise.** In §8.5, the
sentence "no answer, option order or arrangement leaves the sealed module
(AC-61)" is replaced by: "The listing returns `id` and `topic` only. It adds no
reader of the sealed module and arranges nothing. The room's existing
projections and the reveal are unchanged."

**R5 (Minor, accepted): the empty state is about readiness, not scheduling.**
Under R1-a, held, used and other-date records all drop out, so the empty list
does not mean "nothing scheduled". New `host_pick_empty`: *No question is ready
for this club today.* (the Reload button stays). Tests cover a held-only list and
a used-only list.

**R6 (Minor, accepted): RED is its own commit.** M1 is exempt from the
"green" rule. The failing tests are committed alone, with the failing run's
output in the commit body, then GREEN, VERIFY and DOCS follow as phase commits.

**R7 (NIT, accepted):** T-27 is written in M4's five columns (# · Ticket ·
Criteria · Depends on · Notes), as revised below.

### Revised drafts (supersede §4 and §5 where they differ)

§8.5, as revised (R1-a assumed; under R1-b the sentence beginning "The record
carries" becomes "When the club releases a room, the room forgets the club's
other scheduled questions."):

> ### 8.5 Choosing the question on the host phone (PQ-80)
>
> A signed-in organizer can pick the night's question on the host phone instead
> of naming it in the host link. `GET /organizer/questions?club=‹club›` takes the
> organizer session and the same check as *Create a room*: the request names the
> club, and that club's role authorizes it (§8, AC-103). An unknown club is
> refused as a missing role is. It answers, with `Cache-Control: no-store`, with
> the questions scheduled for that club that the club can run today: affirmed,
> arranged for today's meetup date, not used by the club, and held by no room.
> Each is given as `id` and `topic` only. The organizer picks one and creates the
> room on it through `POST /rooms`, which applies the same checks again and
> refuses with the existing reasons if anything changed in between. The host phone
> schedules nothing: a question reaches the room only as §8.3 describes, arranged
> for the club's meetup date by `popquiz schedule` on the laptop, and the
> organizer may push several that way before the night. The record carries the
> date it was arranged for, and the room offers or creates on it only on that
> date. Listing and picking use nothing (G-10). The listing returns `id` and
> `topic` only; it adds no reader of the sealed module and arranges nothing. The
> room's existing projections and the reveal are unchanged. D-20 stands: the
> admin channel is still two routes and one token, and the organizer session
> does not open it.

§11 row: as §4, with *No question is ready for this club today.* in place of
the old empty sentence, and the new room reason
`ROOM_QUESTION_OTHER_DATE`: *That question was scheduled for another date.*
(R1-a only).

T-27, M4's five columns:

> | **T-27** | Choosing the question on the host phone (PQ-80): `GET /organizer/questions` (organizer session, the club's role, `id` and `topic` only), the host page's picker, the scheduled record's meetup date, and the runbook's shortlist push | AC-101, AC-103, AC-72, AC-92, AC-61 | T-07, T-10, T-25 | `organizer_questions.rs`, `host-pick.test.js`, the canary walk, `just a11y` |

Copy keys, final (7 under R1-a, 6 under R1-b): `host_pick_legend` "Question" ·
`host_pick_choice` "‹id› · ‹topic›" · `host_pick_loading` "Loading questions…" ·
`host_pick_empty` "No question is ready for this club today." ·
`host_pick_reload` "Reload" · `host_pick_load_failed` "Couldn't load the
questions." · (R1-a) `ROOM_QUESTION_OTHER_DATE` "That question was scheduled for
another date."

*2026-11-06, orchestrator (SECTION CLASH): PQ-61 owns SPEC §8.4 (the deployed burst); this section is renumbered §8.5, words unchanged.*

## Reset 2026-11-06 by agent:delegator-pq80
