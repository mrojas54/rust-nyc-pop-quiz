# Rust NYC Pop Quiz — organizer guide

Prepare the question bank, schedule one question, then run the rehearsal.
The meetup segment is one question, scheduled last, lasting 3–5 minutes.

## Tooling status

Checked against local `origin/main` at `a599e80` on 2026-10-02. This is a
source check, not confirmation of the deployed version or available questions.
The current working branch is older than that snapshot.

- The verifier has a command-line entry point on that snapshot.
- Generation, organizer review, and scheduling commands are unfinished.
- Scheduling is available through the authenticated server admin endpoint.
- A record in the local bank is not automatically scheduled on the server.

Do not run the commands below from the older working branch. Use a checkout
containing the inspected implementation and its dependency and sandbox setup.

## 1. Prepare the private bank

### Generate candidates

- [ ] Generate a batch offline. The room app does not generate questions.
- [ ] For each candidate, prepare the Rust source, five options, private hint,
      explanation, and stepped trace in the bank record format.
- [ ] Keep candidates and the bank private.

The generation module is unfinished in the inspected snapshot. There is no
working generation command to give here. Candidate preparation needs an
authoring process before the remaining steps can be completed.

### Verify

- [ ] Run each candidate through the pinned verifier and sandbox.
- [ ] Keep only accepted candidates. Read rejection or refusal messages.
- [ ] Let the verifier write the verification record. Never edit the observed
      output or add a hand-written correct-answer field.

From the `pipeline/` directory of a checkout containing the verifier:

```sh
uv run python -m popquiz.verify /absolute/path/to/candidate.json --expect ran
```

Use `--expect does_not_compile` or `--expect ub` for candidates declared that
way. The declaration does not decide the answer; the machine checks it.

The verifier checks the toolchain pin, compilation, five native runs, and
Miri where applicable. Compile-failure questions record compiler error codes;
they do not run. Exit 0 means accepted and the candidate file was updated;
exit 1 means rejected; exit 2 means the verifier refused or could not read the
input. Do not advance after either failure.

Miri checks executed paths. Repeated output is evidence of determinism, not a
guarantee. The built pipeline stores verification in each question's `verified`
record; the older MVP uses a separate `verified.json`.

### Check duplicates and review

- [ ] Check exact and normalized duplicates against the bank and history.
      Review near-duplicate candidates.
- [ ] Check difficulty, source legibility, projector fit, and option length.
- [ ] Read the explanation and trace. Check quoted output against verification.
- [ ] Include a reason each incorrect option is tempting.
- [ ] Keep at least two trace steps so the pre-reveal walk-through has content.
- [ ] Re-verify after any source or option change.
- [ ] Record acceptance and explicit organizer affirmation, including who and
      when, in the bank record.
- [ ] Save approved records under `bank/questions/<id>.json`.

The organizer review interface is unfinished in the inspected snapshot.
Approval must be recorded in the supported record format; a drafted explanation
or a file's presence in the bank is not approval.

Keep answer-position selection unchanged: it is drawn from the meetup date,
independently of history. Do not balance letters or publish category quotas.

## 2. Choose and schedule one question

- [ ] Choose an approved question that fits the room.
- [ ] Check actual previous use, including rehearsals and prototype runs.
      Do not infer that a question is unused from an empty server ledger.
- [ ] Reserve a different unused question for the meetup if rehearsal attendees
      will be there. Do not assume `q3` is available.
- [ ] Upload the complete approved bank record to the server.

Scheduling means uploading the record so the server can create a room for its
ID. It does not set a calendar time. Discord login does not upload the record.

The upload requires the organizer's `POPQUIZ_ADMIN_TOKEN`, matching the server's
secret. Keep it in local environment configuration; do not put it in a browser
URL, the guide, or the repository.

```http
PUT https://rustnyc-popquiz.fly.dev/admin/questions/<id>
Authorization: Bearer <POPQUIZ_ADMIN_TOKEN>
Content-Type: application/json

<complete approved question JSON>
```

The path ID must match the record ID. A successful response is `201` for a new
schedule or `200` for a replacement, with `scheduled: "new"` or
`scheduled: "replaced"`. Check the response before proceeding.

The authenticated `GET /admin/used` endpoint returns the server's used ledger.
Compare it with organizer records before choosing a question. Scheduling can
refuse a used question or one already held by a room.

## 3. Create the room

- [ ] Open the host page with the uploaded ID:

  `https://rustnyc-popquiz.fly.dev/host?question=<id>`

- [ ] Replace `<id>` with the actual ID. Do not leave `question=` empty.
- [ ] Select **Sign in with Discord** and complete sign-in.
- [ ] Select **Create a room**. Discord checks your organizer access.
- [ ] Save the room's host resume link privately. It contains host access.
- [ ] Open the room's wall on the projector. Use the join link shown there for
      the helpers' phones.

## 4. Before rehearsal

- [ ] Tell helpers which question they will see and that it will be spent.
- [ ] Keep lights up.
- [ ] Give the observer sheet to someone else. The host does not take notes.
- [ ] At the venue, measure projected width, height, and distance to the last
      occupied row. Elsewhere, mark measurements skipped; do not estimate.
- [ ] Confirm the code fits and can be read from the back.
- [ ] If using the prototype, keep its **PROTOTYPE** badge visible. Record
      whether this run used the built app or the prototype.

## 5. Run the segment

Use the built app's host phone controls in this order.

- [ ] **1 · Title:** check the title and join screen.
- [ ] **2 · Question live:** tap **Put it on the screen**. Code and options
      appear. The observer starts the segment clock and measures time to the
      first visible opinion.
- [ ] **3 · Answers closed:** tap **Close answers**. The wall says answers are
      closed and voting locks.
- [ ] **4 · The split:** tap **Show the room its split**. Counts appear before
      the answer. A participant's choice gets a count, never a cross.
- [ ] **5 · Work through it:** tap **Let's walk it**. Use the forward and back
      trace controls. The wall shows steps without the answer, receipt, or
      syntax colour. Do not ask participants to explain, compare, or volunteer.
- [ ] **6 · Reveal:** tap **Reveal**. Follow **Read it aloud**: what happened,
      why the other reading was tempting, and what to remember. Check the
      correct-option tick, the popular wrong-answer count, and the receipt.
- [ ] **7 · Released:** tap **Release the room**. Check the take-home link and
      QR. The observer stops the clock. Target: no more than five minutes from
      code appearing to release.

An offered contribution can be taken. Nobody is obliged to speak.

## 6. If something fails

| What you see | What to do |
|---|---|
| `400` from a link ending in `question=` | Open the host URL with a nonempty question ID. |
| “No question is scheduled with that id.” | Confirm the ID and successful upload. In the inspected implementation, scheduled records are held in memory; a server restart can require another upload. Check used status before uploading again. |
| Unauthorized admin upload | Check that local `POPQUIZ_ADMIN_TOKEN` matches the server configuration. Discord credentials do not authorize this endpoint. |
| Discord denies room creation | Check membership and the configured organizer role. |
| “That question has already been run. Pick another.” | Choose a different approved, unused question. Do not rename a used question to bypass the refusal. |
| Host phone lost | Open the saved host resume link on another phone. |
| Code clips or scrolls on the wall | Have the observer record the affected lines or edge and the room measurements. Do not record fit as passing. |
| Answer or receipt appears during the walk-through | Record the defect and the phase where it appeared. |
| Host improvises, apologises, or asks for a contribution | Have the observer record the exact words. |

**Run it again** does not establish that a question is reusable. The used-question
rule still applies.

## 7. Afterward

- [ ] Ask helpers whether they could read the code from the back.
- [ ] Ask whether seeing the split before the answer helped or felt like delay.
- [ ] Capture anything awkward and any contribution offered without prompting.
- [ ] Save the filled sheet at `mvp/<YYYY-MM-DD>/practice-run.md`.
- [ ] Record the question as spent if attendees saw its source and answer,
      including when a prototype run did not update the server ledger.
- [ ] Check the server used ledger after release against what actually ran.
- [ ] Record room measurements, or that they were skipped, in
      `sequence/run-state.md` and the room configuration records.
- [ ] Resolve findings before the meetup; use a fresh question there.

## Observer sheet

Give the observer a copy of `mvp/PRACTICE-RUN.md` for the run. Capture:

- Total duration: code appears to room released; target ≤5 minutes.
- First visible opinion: target ≤30 seconds. With two regulars, this is a lower
  bound for a wider audience.
- Projected width, height, and distance to the last occupied row, or skipped.
- Source fit: fit, clipped, or scrolled.
- Host improvisations and exact words asking for a contribution.
- Whether the walk-through had content and the answer stayed hidden.
- Back-row legibility and feedback about the split before the answer.

Two regulars cannot establish crowd participation, beginner understanding,
difficulty for newcomers, or how anonymous wrongness feels in the full room.

## Technical references

- `PHILOSOPHY.md`: segment rules, privacy, and answer-position selection.
- `SPEC.md` §§7–8: pipeline, review, authorization, and scheduling.
- `bank/README.md`: record format and verification provenance.
- `mvp/PRACTICE-RUN.md`: observer capture form.
- `sequence/T-20-drive-guide.md`: historical prototype drive; its controls differ
  from the built app.

The guide does not establish that a candidate, schedule, deployment, or rehearsal
has passed. Keep the response, verification record, and observer sheet for each
completed step.
