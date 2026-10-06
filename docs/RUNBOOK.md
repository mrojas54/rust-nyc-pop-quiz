# Running the Pop Quiz

How an organizer schedules one question, hosts it on the built room and spends
it afterwards. This describes the room on `main` at `de1da5f`. The hand-run
projector deck has its own run-of-show in `mvp/README.md`; this is the
successor to it for the built room.

## What this is

One question, three to five minutes, at the very end of the night. After the
last talk, with nothing after it. *End Pop Quiz* releases the room, and the
room walks to the bar still talking about it (AC-89, AC-90).

Why one question and why last: `mvp/README.md`, *The shape*. In short, the
working-through never has to be cut off, one question a night makes a batch
last most of a year, and nobody leaves during a three-minute closer.

The room is the wall on the projector, a buzzer on every phone and the host
phone in your hand. Every phase change is a tap on the host phone. No phase
advances on its own.

## Before the night

### The reserve

The reserve is every question that is accepted, affirmed and unused (`SPEC.md`
§3.3). `just schedule` and `just sync` print it before they do anything else.
A snapshot, taken on `de1da5f`; the numbers move with every review and every
night:

```
reserve: 0 ready (accepted, affirmed, unused) - 0 meetups of runway at one question a meetup
trend: +0 accepted awaiting affirmation, +4 not yet reviewed, -0 used in the last 90 days
warning: 0 ready is below the threshold of 2 - generate and review now; the lead time is 2 meetups
```

The warning fires below two meetups of supply. Generating (T-16) and reviewing
(T-18) have no commands yet; `pipeline/README.md` lists each module and the
ticket that builds it.

### Choosing a question

Pick one from the reserve. Check that nobody who will be in the room has seen
it, including at a rehearsal. Pick it for the room, not for the letter: the
answer's position is drawn from the meetup's date and nothing else, and
`just schedule` does the drawing. Uneven letters and repeats are correct. Never
pick a question to even them out (`PHILOSOPHY.md` §2).

### Affirming a question — pending the owner's ruling (F-52)

This section is a placeholder for the owner's ruling on F-52, which is still
open. The ruling replaces it.

`just schedule` refuses any question whose review block lacks `affirmed_by`
or `affirmed_at` (AC-72, G-12). The contract names the review surface as the
writer of those two fields (`SPEC.md` §3.1, §7.4). That surface is T-18, and it
is not built. Every committed question is unaffirmed, so no committed question
can be scheduled today. Run against `q3`, the refusal is:

```
popquiz.schedule: q3 is not affirmed - it has no affirmed_by and no affirmed_at, and it has not been accepted at review - and an unaffirmed question cannot be scheduled (AC-72)
```

It exits 1 and writes nothing. How the first question gets affirmed is the
owner's open decision. Until it is made, there is no step here.

### Deploy before the rehearsal

The deployed room is behind `main`. Fly app `rustnyc-popquiz` runs `1263add`
(2026-09-29), seventeen merges before `de1da5f`. Deploying is the client's
step; `room/README.md`, *Deploying*, has the whole of it. The order matters:

1. **If a night ran since the last sync, `just sync` first.** A deploy restarts
   the machine, and the used ledger lives in memory.
2. Deploy from an up-to-date `main`, from the repository root:
   `fly deploy --ha=false --remote-only`. One machine, never two.
3. Pre-flight: `just smoke https://rustnyc-popquiz.fly.dev --question smoke-q3`,
   with `POPQUIZ_ORGANIZER_SESSION` in the environment (`room/README.md`,
   *Smoke*, says where it comes from). It runs all seven
   phases on `smoke-q3`, a harness id, so no bank question is spent.
   `smoke-q3` is not a bank record, so `just schedule` cannot push it: push it
   with the loop in `room/README.md`, *Burst*, under *Scheduling*, first.
4. `fly apps restart rustnyc-popquiz` after every smoke run. **Never during a
   night.**
5. Then `just schedule`. A restart or deploy forgets every scheduled question,
   so scheduling comes last.

The room runs on a paid Fly org, a card with a spending cap. The machine runs
until something stops or redeploys it. A restart or a deploy loses every room,
every scheduled question and the used ledger, which lives in memory. `just sync`
first pulls that ledger into the bank; the ledger is the only part that can be
saved.

### Scheduling

From the repository root, the day of the meetup, after any deploy (*Deploy
before the rehearsal*, above):

```
POPQUIZ_ADMIN_TOKEN="$(op read 'op://<vault>/<item>/<field>')" \
  just schedule <id> --date <YYYY-MM-DD> --room https://rustnyc-popquiz.fly.dev --out ~/popquiz-<YYYY-MM-DD>
```

- **The token** comes out of 1Password into the environment of that one
  command. It goes in no file, no URL, no page and no note (`SPEC.md` §8.3).
- **`--date`** is the meetup's date. It decides where the answer sits.
- **`--out`** is a directory outside the repository, written as an absolute
  path. The two files it gets hold the answer, and `just schedule` refuses a
  directory inside the repository.

It prints the reserve, then the question and date, then `scheduled: new` (or
`scheduled: replaced` for a re-push), then the two files it wrote:

- `<id>.html` — the static fallback. The wall's seven views with this question
  baked in, no network. `Space` next phase, `←` `→` step the trace, `Esc` back a
  phase (AC-102).
- `<id>.host-sheet.txt` — the host sheet. The three beats and each step's words,
  phase by phase, for when there is no host phone. Print it.

Both stay on your laptop. Venue wifi is assumed hostile; these two files are
the night when the room cannot be reached. For a night you already know the
room is down, `--no-push` in place of `--room` writes the two files and sends
nothing, with no token needed.

The flags come from `just schedule --help` and `just sync --help`.

## At the venue

Take three measurements once, ever (AC-100):

- the projected image's width, paced heel to toe;
- its height (a 4:3 projector or a TV breaks the 16:9 guess);
- the distance to the last row anyone actually sits in.

Until someone takes them, the wall runs on a labelled guess: 15 ft wide, 16:9,
back row at 20 ft (`EVALUATION.md`, *What the operator supplies*). Write them
on the practice-run sheet. To see whether the bank fits the room you measured:

```
just bank-audit --screen-width-ft <W> --screen-height-ft <H> --back-row-ft <D>
```

It prints each check and writes its report to `bank/audit/<YYYY-MM-DD>.json`.

The wall's own numbers are code, in `web/shared/typemodel.js`, and change
through a pull request, not on the night.

Then: projector on, lights up, the fallback `<id>.html` open in a laptop tab
and the host sheet printed.

## Creating the room

Do this in the last ten to fifteen minutes of the last talk, with the wall not
yet on the projector. Not earlier: a room nobody acts on closes after 30
minutes in `idle` (`SPEC.md` §4.6).

1. On the host phone, open `https://rustnyc-popquiz.fly.dev/host?question=<id>`.
2. Tap *Sign in with Discord*. Discord asks once for your identity and server
   membership.
3. Tap *Create a room*. The room asks Discord whether your account holds the
   organizer role, then moves the phone to the room's host screen.
4. **Save that address privately.** The host screen's address is the resume
   link: open it on another phone and that phone controls the same room
   (AC-50). It is host access; never put it on the screen.
5. On the projector laptop, open `/wall/<room_id>` on the same host, with the
   room id from the host screen's address. The wall shows the title card and
   the join strip, `join @ ‹link›` · `Joined: ‹n›`.

Put the wall on the projector when the last talk ends.

## The host script

The Pop Quiz is one question, and it is scheduled last: after the last talk, with nothing after it, and the wrap-up releases the room.

Each block is a phase in the room's order. The action is the button on the
host phone, exactly as it reads. Five minutes end to end, *Start* to *End Pop
Quiz* (AC-89); only the walk-through is worth spending extra time on.

### idle — before the question

The wall: **Time for a pop quiz.**, the join strip, `Joined: ‹n›`. Each phone
that joins shows *You're in.* The host phone shows *Start*, the code and
`Joined: ‹n›`.

> **Say:** Time for a pop quiz. Join on your phone — the link is on the screen.

Give it thirty seconds, or until `Joined:` stops climbing.

### live — question live

**Tap *Start*.** The wall shows the source in colour, the five options beneath
it, and `Joined: ‹n›` · `Answered: ‹n›`. No timer. Every phone shows the letters
A to E.

Say nothing for thirty seconds. Let them read. The silence is the format; talk
over it and half the room never forms an opinion.

> **Say:** Vote on your phone. You can change it until I close answers.

Watch `Answered:` on the host phone. When it stops climbing, close. About a
minute in all.

**The hint.** Every phone has *Show me a hint* from here on. Taking it tells
nobody: not you, not the wall, not the totals (`SPEC.md` §4.2). There is no
hint button on the host phone. If `Answered:` is low at forty-five seconds:

> **Say:** There's a hint on your phone if you want one. Nobody sees who takes it.

### closed — answers closed

**Tap *Close answers*.** The wall says **answers are closed**. Each phone
keeps its last saved answer. Go straight on.

### split — the split

**Tap *Show the room its split*.** The wall shows five bars, `n · p%` each, and
how many in the room answered. No answer yet. Each phone shows the same bars
with its own letter marked.

Say nothing. Give the room five seconds to look at how it read the program.

### work — walking it through

**Tap *Trace*.** The wall drops the colour and steps the trace, *Step ‹N› of
‹M›*. The host phone shows each step's words, with `←` and `→` to step. The wall
stops one step short of the line that prints; the answer stays hidden.

> **Say:** Let's walk it. Nobody has to say anything.

Read each step's words off the host phone, then `→`. Read the program, not the
answer. If someone offers a reading, take it. If nobody does, keep stepping.
This is the segment; everything before it is setup. Thirty seconds to two
minutes.

### reveal — the answer

**Tap *Reveal*.** The wall marks the correct option with a ✓, names the
option most of the room chose instead with its count, shows *How we know*, and
the trace lands on the step that prints. Each phone shows **✓ It was ‹Y›.**

The host phone shows **Read it aloud** and three beats: *What happens*, *Why
‹n› of us said ‹X›*, *What to remember*. Read them aloud as written. They are
written to be read, not paraphrased. When nobody chose another option, the
middle beat is headed *Why nobody said anything else* and has no text: read
*What happens*, then *What to remember*.

> **Say:** (the three beats, verbatim)

About a minute.

### released — released

**Tap *End Pop Quiz*.** The wall shows **Let's go to the bar.**, the
take-it-home link and a QR code. Nothing else.

> **Say:** Let's go to the bar.

You are done. The host phone offers *Run it again*; leave it alone (*Do not*,
below).

## Do not

- **Do not run two questions because the first went well.** The second is always
  worse, and it burns a question that would have carried another meetup. Do not
  tap *Run it again* at a meetup; it makes a new room for a new question.
- **Do not announce the segment at the start of the night.** Announced, it
  becomes a thing people prepare for.
- **Do not read the answer off the laptop or the host sheet before the reveal.**
  You play too (`PHILOSOPHY.md` §7). The host phone shows no answer before
  *Reveal*, and the two files on your laptop hold it. Two things to know, in the
  contract's words:

  > Option text is public — the correct answer is always one of the five visible options.

  > A host who reads Rust can work out the answer from the source; the host's not being shown it keeps the host honest, it is not a security guarantee.

- **Do not ask anyone to explain their pick, compare answers or volunteer.** An
  offered reading can be taken; none is asked for (AC-98).
- **Do not restart or redeploy the room before `just sync`.** The used ledger is
  in memory until sync pulls it.
- **Do not re-push or rename a question to get past a refusal.** A question runs
  once, ever.

## After

**Sync before anything restarts the room.** The same night, or the next morning
at the latest:

```
POPQUIZ_ADMIN_TOKEN="$(op read 'op://<vault>/<item>/<field>')" \
  just sync --room https://rustnyc-popquiz.fly.dev
```

It prints the reserve, then one line per synced record, `used: <id> on <date>,
room <room_id>`, and a `fit:` line for any room whose wall reported something
other than *fits*, or nothing at all. Ids from test runs (`smoke-q3`,
`burst-q3`) are listed as skipped. Running it twice changes nothing.

`used` is written when the room is released, and only from the room's ledger
(AC-92). The ledger is the record of which question each meetup used, not your
memory and not a built file. Commit the changed record under `bank/questions/`.

The night's failed-request rate (AC-55) is in Fly's logs. Read it before
anything restarts the machine; the commands are in `room/README.md`, *After a
meetup*.

File the sheets:

- the meetup's field notes at `mvp/<YYYY-MM-DD>/field-notes.md` (HC-4);
- a rehearsal's sheet at `mvp/<YYYY-MM-DD>/practice-run.md` (HC-1).

## If something fails

| What you see | What to do |
|---|---|
| `just schedule` prints `… is not affirmed …` | No question is affirmed yet. See *Affirming a question — pending the owner's ruling (F-52)*. Do not edit the record. |
| `POPQUIZ_ADMIN_TOKEN is not set in the environment` | Run the command with the `op read` prefix shown above. |
| `the room refused the admin token (401)` | The value in 1Password is not the room's Fly secret. `room/README.md`, *Pipeline channel*, has the rotation. |
| `… is inside the repository …` | Give `--out` a directory outside it, such as `~/popquiz-<YYYY-MM-DD>`. |
| `… has already been run; a question is never run twice.` | That machine released this question. Pick another. |
| `A room is running … it can be replaced once that room is gone.` | A room already holds the question. Do not re-push during a night. |
| *Sign in with Discord* answers `400` | The question id in the link is empty or not an id. An id is 1 to 64 letters, digits, `-` or `_`. Open `/host?question=<id>` with the real id. |
| Coming back from Discord answers `400` | The sign-in was replayed, expired or lost its cookie. Open `/host?question=<id>` again and tap *Sign in with Discord* once. |
| `No question is scheduled with that id.` | The push did not land, or the machine restarted since. Run `just schedule` again with the same id and date, and check it prints `scheduled: new`. |
| `That question has already been run. Pick another.` | This machine has released that question. Pick another. |
| `That question is open in another room. Pick another.` | Another room on this machine has that question and has not been released. Open that room's saved host-screen address instead, or pick another question. A room that went quiet stops holding it. |
| `That Discord account isn't in the Rust NYC server.` | Sign in with the account that is. |
| `That Discord account doesn't have the organizer role.` | Sign in with an account that holds the organizer role. |
| *Create a room* fails with `503` | Discord did not answer. Try again in a minute. A room already open does not need Discord. |
| The host phone dies or is lost | Open the saved host-screen address on another phone. It controls the same room. |
| Phones show *That room went quiet and closed.* | Nobody acted on the room for 30 minutes (20 after *Start*). Create a new room from `/host?question=<id>`; the question is still unused. |
| The room cannot be reached | Open `<id>.html` on the laptop and step it with `Space`, `←` `→` and `Esc`. Read from the host sheet. Votes are hands. At the reveal the sheet lists every incorrect option by letter with why it tempts; read the one for the option most hands chose. |
| Code is clipped on the wall | The wall shows a visible edge on the side that lost content. Keep going and write it on the sheet; `just sync` reports the fit afterwards. |
| The answer, a ✓ or *How we know* shows before *Reveal* | A defect. Write down the phase it appeared in. |
| The host apologises, improvises or asks for a contribution | The observer writes down the exact words (AC-51, AC-98). |
| `just sync` exits 1 with `popquiz.schedule: refused …` | Nothing was written to that record. Leave it as it is and raise it; the line says which rule it hit. |

## The instruments

- `mvp/PRACTICE-RUN.md` — the rehearsal with two regulars (HC-1). Copy it, fill
  it in place.
- `mvp/FIELD-NOTES-TEMPLATE.md` — the meetup (HC-4). Someone other than the host
  holds it.
- `mvp/README.md` — the hand-run deck and its own run-of-show.
