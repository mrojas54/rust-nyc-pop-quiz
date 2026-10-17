# Rust NYC Pop Quiz — organizer guide

Prepare the question bank, schedule one question, then run the rehearsal.
The meetup segment is one question, scheduled last, lasting 3–5 minutes.

## Tooling status

Checked against `main` at `df285ae` on 2026-10-17. This is a source check: it
says what the commands do, not which version is deployed or which questions
are ready.

- Generation (`just generate`), verification (`just verify`), organizer review
  (`popquiz.review` and the organizer TUI) and scheduling (`just schedule`,
  `just sync`) all exist. `pipeline/README.md` has each command;
  `docs/RUNBOOK.md` is the night itself, from scheduling to sync, and this
  guide follows it.
- A record in the local bank is not scheduled until `just schedule` pushes it,
  and a restart or deploy of the room forgets every scheduled question.

Run the commands from the repository root of an up-to-date `main`, after
`just setup`.

## 1. Prepare the private bank

### Generate candidates

- [ ] Generate a batch offline. The room app does not generate questions.
- [ ] For each candidate, prepare the Rust source, five options, private hint,
      explanation, and stepped trace in the bank record format.
- [ ] Keep candidates and the bank private.

`just generate` drafts candidates into `bank/candidates/` and never into
`bank/questions/`; `pipeline/README.md`, *Generating*, has the backends, the
fixture replay and the spend cap. The first real run is the client's (HC-2).
The model never writes an answer: `just verify` writes what the program
printed.

### Verify

- [ ] Run each candidate through the pinned verifier and sandbox.
- [ ] Keep only accepted candidates. Read rejection or refusal messages.
- [ ] Let the verifier write the verification record. Never edit the observed
      output or add a hand-written correct-answer field.

From the repository root, with the sandbox image built (`just sandbox-build`,
which needs Docker):

```sh
just verify bank/candidates/<id>.json --expect ran
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
- [ ] Record acceptance and the organizer's affirmation through
      `popquiz.review` (`accept`, then `affirm --by NAME`), usually from the
      organizer TUI's review screen (`v`). Never by hand: the affirmation is
      bound to the text by a digest, and any later edit clears it.
- [ ] Commit the accepted record under `bank/questions/<id>.json`.

`python -m popquiz.review <command> <id> --bank bank` is the only writer of a
review decision; `pipeline/README.md`, *Reviewing*, lists the commands. A
drafted explanation or a file's presence in the bank is not approval:
`just schedule` refuses a question that is not accepted and affirmed, and names
what is missing.

Keep answer-position selection unchanged: it is drawn from the meetup date,
independently of history. Do not balance letters or publish category quotas.

## 2. Choose and schedule one question

- [ ] Pick one from the reserve. From `pipeline/`,
      `python -m popquiz.schedule reserve` prints it and writes nothing
      (`docs/RUNBOOK.md`, *The reserve*).
- [ ] Check actual previous use, including rehearsals and prototype runs.
      Do not infer that a question is unused from an empty server ledger.
- [ ] Reserve a different unused question for the meetup if rehearsal attendees
      will be there. Do not assume `q3` is available.
- [ ] Deploy first if the room is behind `main` (`docs/RUNBOOK.md`, *Deploy
      before the rehearsal*). A deploy or restart forgets every scheduled
      question, so scheduling comes last.
- [ ] Schedule it, from the repository root:

```sh
POPQUIZ_ADMIN_TOKEN="$(op read 'op://<vault>/<item>/<field>')" \
  just schedule <id> --date <YYYY-MM-DD> --room https://rustnyc-popquiz.fly.dev --out ~/popquiz-<YYYY-MM-DD>
```

`just schedule` is the only way to push a question. It prints the reserve,
refuses a question that is not accepted, affirmed and unused, arranges the
options for that date (the correct one's position comes from the date and
nothing else, AC-23), pushes the arranged record to the room, and writes the
static fallback `<id>.html` and the host sheet beside it. Never push a bank
record by hand: the room renders the order it is given, and the bank file holds
the generator's order. `--date` decides where the answer sits; it does not set
a calendar time.

The token comes out of 1Password into the environment of that one command. It
goes in no file, no URL, no page and no note. `--out` must be outside the
repository: both files hold the answer.

A successful push prints `scheduled: new` (or `scheduled: replaced` for a
re-push). Check it before proceeding. A scheduled question still counts in the
reserve: it is used when its room is released, and `just sync` brings that
into the bank.

## 3. Create the room

Do this in the last ten to fifteen minutes before the run, not earlier: a room
nobody acts on closes after 30 minutes.

- [ ] Open the host page with the scheduled ID:

  `https://rustnyc-popquiz.fly.dev/host?question=<id>`

- [ ] Replace `<id>` with the actual ID. Do not leave `question=` empty.
- [ ] Select **Sign in with Discord** and complete sign-in.
- [ ] Select **Create a room**. Discord checks your organizer access.
- [ ] Save the room's host-screen address privately. It is the resume link and
      it contains host access.
- [ ] On the projector laptop, open `/wall/<room_id>` on the same host, with
      the room id from the host screen's address. The helpers' phones join
      from the link on the wall's join strip.

## 4. Before rehearsal

- [ ] Tell helpers which question they will see and that it will be spent.
- [ ] Keep lights up.
- [ ] Give the observer sheet to someone else. The host does not take notes.
- [ ] At the venue, measure projected width, height, and distance to the last
      occupied row. Elsewhere, mark measurements skipped; do not estimate.
- [ ] Confirm the code fits and can be read from the back.
- [ ] If the room cannot be reached, run the static fallback `<id>.html` that
      `just schedule` wrote. Record which surface the run used, the room or the
      fallback.

## 5. Run the segment

Use the host phone's controls in this order. Each phase is a block of
`docs/RUNBOOK.md`, *The host script*, which also has what to say.

- [ ] **1 · idle:** the wall shows *Time for a pop quiz.* and the join strip;
      the host phone shows *Start* and `Joined: ‹n›`. Give it thirty seconds.
- [ ] **2 · live:** tap **Start**. Code and options appear. The observer
      starts the segment clock and measures time to the first visible opinion.
- [ ] **3 · closed:** tap **Close answers**. The wall says answers are closed
      and voting locks.
- [ ] **4 · the split:** tap **Show the room its split**. Counts appear before
      the answer. A participant's choice gets a count, never a cross.
- [ ] **5 · work:** tap **Trace**. Step with `←` and `→`, reading each step's
      words off the host phone. The wall shows steps without the answer,
      receipt, or syntax colour, and stops one step short of the line that
      prints. Do not ask participants to explain, compare, or volunteer.
- [ ] **6 · reveal:** tap **Reveal**. Follow **Read it aloud**: *What happens*,
      *Why ‹n› of us said ‹X›*, and *What to remember*, as written. Check the
      correct-option tick, the popular wrong-answer count, and the receipt.
- [ ] **7 · released:** tap **End Pop Quiz**. Check the take-it-home link and
      QR. The observer stops the clock. Target: no more than five minutes from
      code appearing to release.

An offered contribution can be taken. Nobody is obliged to speak.

## 6. If something fails

| What you see | What to do |
|---|---|
| `400` from a link ending in `question=` | Open the host URL with a nonempty question ID. |
| “No question is scheduled with that id.” | The push did not land, or the machine restarted since: scheduled questions live in memory. Run `just schedule` again with the same id and date, and check it prints `scheduled: new`. |
| `just schedule` says `the room refused the admin token (401)` | The value in 1Password is not the room's Fly secret (`room/README.md`, *Pipeline channel*). Discord credentials do not authorize this channel. |
| Discord denies room creation | Check membership and the configured organizer role. |
| “That question has already been run. Pick another.” | Choose a different approved, unused question. Do not rename a used question to bypass the refusal. |
| “That question is open in another room. Pick another.” | A room on this machine still holds it. Open that room's saved host-screen address, or wait for it to go quiet (`docs/RUNBOOK.md`, *If something fails*). |
| Host phone lost | Open the saved host-screen address on another phone. |
| Code clips or scrolls on the wall | The wall shows a visible edge on the side that lost content. Have the observer record the affected lines or edge and the room measurements; `just sync` reports the fit afterwards. Do not record fit as passing. |
| Answer or receipt appears during the walk-through | Record the defect and the phase where it appeared. |
| Host improvises, apologises, or asks for a contribution | Have the observer record the exact words. |

**Run it again** does not establish that a question is reusable. The used-question
rule still applies.

## 7. Afterward

- [ ] Ask helpers whether they could read the code from the back.
- [ ] Ask whether seeing the split before the answer helped or felt like delay.
- [ ] Capture anything awkward and any contribution offered without prompting.
- [ ] Save the filled sheet at `mvp/<YYYY-MM-DD>/practice-run.md`.
- [ ] Sync the same night, before anything restarts the room
      (`docs/RUNBOOK.md`, *After*):

  ```sh
  POPQUIZ_ADMIN_TOKEN="$(op read 'op://<vault>/<item>/<field>')" \
    just sync --room https://rustnyc-popquiz.fly.dev
  ```

  Check its `used:` line names the question that ran, and commit the changed
  record under `bank/questions/`. `used` is written only from the room's
  ledger (AC-92), never by hand.
- [ ] If the run used the fallback instead of the room, there is no ledger
      entry. Write on the sheet that the helpers saw the question, and do not
      pick it for the meetup (*Choosing a question*, `docs/RUNBOOK.md`).
- [ ] Write the room measurements, or that they were skipped, on the sheet.
      The wall's own numbers are code (`web/shared/typemodel.js`) and change
      through a pull request, not on the night;
      `just bank-audit --screen-width-ft <W> --screen-height-ft <H> --back-row-ft <D>`
      checks the bank against the room you measured.
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

- `docs/RUNBOOK.md`: the night on the built room, from scheduling to sync; the
  host script with what to say; the failure table.
- `pipeline/README.md`: every pipeline command, generating to syncing.
- `PHILOSOPHY.md`: segment rules, privacy, and answer-position selection.
- `SPEC.md` §§7–8: pipeline, review, authorization, and scheduling.
- `bank/README.md`: record format and verification provenance.
- `mvp/PRACTICE-RUN.md`: observer capture form.
- `sequence/T-20-drive-guide.md`: historical prototype drive; its controls differ
  from the built app.

The guide does not establish that a candidate, schedule, deployment, or rehearsal
has passed. Keep the response, verification record, and observer sheet for each
completed step.
