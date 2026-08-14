# T-12 — the drive-through

The one open touchpoint in Stage 2. Nothing converges without it: no `DESIGN.md`,
no `tone-architect`, no build. Written 2026-08-14 so it survives the session.

**About 35 minutes.** All six surfaces are open as browser tabs in the
right-hand pane — **`Takes index`**, **`B · Phone-first`**, **`C ·
Projector-first`**, **`A · Hands`**, **`Review surface`**, **`Discord seam`**.
This checklist lives in the left pane so a take and its instructions are visible
at the same time. The room code everywhere is `KMT4XW`.

**The pick is the start of the design, not the end.** Whatever you choose, we
iterate it together until you love it — many rounds is normal, because this is
the cheapest the design will ever be to change. So drive for *"which of these is
worth arguing about for a week"*, not *"which is finished"*.

---

## Order: B, then C, then A

B and C fork on the same axis — both have phones, they disagree about where the
**code** lives. Driving them back to back is the tight comparison. A asks a
different question entirely (do phones exist at all), so it goes last.

If you want to defend against order effects: whichever one you end up liking
least, drive it once more at the end. Half the time that changes the answer.

---

## 1 · Direction B — Phone-first  (~10 min)

*Core assumption: commitment is private, and anonymity is what makes people
willing to be wrong in front of colleagues.*

- [ ] **`1 · join`** — type `KMT4XW`, press **Join**. This is every attendee's
      first ten seconds of the segment.
- [ ] **`3 · question live`** — the real test. Read nine lines of Rust on a
      375px phone. **Can you form an opinion without pinch-zooming?** That is
      AC-32, and it is `felt` — nobody can answer it but you.
- [ ] **Time yourself.** AC-91 budgets **30 seconds** from question-live to
      having an opinion. Count it.
- [ ] **Pull the hint.** T-13 made it private: available on your own device the
      whole time the question is open, and taking it tells nobody — not the
      host, not the room, not any total. Check that nothing on screen betrays
      you for taking it.
- [ ] **`4 · answers closed` → `5 · where the room landed`.** Stop here. This is
      **AC-93** and the whole of `PHILOSOPHY.md` §9: you find out you are not
      alone *before* you find out you were wrong. 24 of 58 picked `[1, 2, 3]`.
      **Does seeing that land the way §9 claims it does?**
- [ ] **`6 · reveal`** — check three things: your own choice carries a *count*,
      never a ✗ (AC-94); the correct option is marked (AC-40); the explanation
      spends its **middle** paragraph on `[1, 2, 3]` — what those 24 people saw
      and why it was reasonable (AC-95).
- [ ] **Step the trace yourself**, at your own speed, backwards if you want.
      That is T-15's call for B: phones step freely, nobody watches how many
      times you went back.
- [ ] **The four edge states** — `save failed`, `reconnecting`, `room full`,
      `unknown code`. Boring and load-bearing: in a room of 95 on venue wifi,
      these are the screens that actually happen.

**The question B is asking you:** is private commitment on a personal device
worth the most expensive build of the three?

---

## 2 · Direction C — Projector-first, phone as buzzer  (~8 min)

*Core assumption: fifty people reading the same nine lines is the event. The
phone is five letters, not a reading surface.*

- [ ] **`2 · question live`**, then immediately press **`6 metres back`.**
      **This is the single most important click in the whole drive-through.**
      C lives or dies on AC-78, and a mock only ever seen at full size passes a
      test the room never runs. Try `front row` and `middle` too.
- [ ] **`lights down`.** AC-80 says the brand's *no dark mode in v1* is what
      gets amended if a dim room wins. The amendment is built so you can judge
      it rather than argue about it. **Which one would you actually project?**
- [ ] **Look at the buzzer phone.** No code on it, ever — just five letters.
      Relief, or loss?
- [ ] **`4 · the split` → `5 · reveal`** — same three checks as B (count not ✗,
      answer marked, middle beat on the popular wrong answer).
- [ ] **Read the explanation as a script.** Under C the prose lives only in the
      host's hand, so speech is the *only* delivery. AC-42 stops being
      aspirational. **Read a paragraph out loud.** If it doesn't survive that,
      it fails here in a way it doesn't in B.
- [ ] Note the trace is **host-stepped** on the wall (T-15) — you set the pace
      for the room.

**The question C is asking you:** if the wall carries everything, does the phone
still earn its place as five letters?

*One thing to know while judging C:* its core assumption was arrived at
independently by `PROJECTOR_SPEC.md` three weeks before these prototypes existed
— *"the reveal+trace state is where the night lives."* That is the strongest
outside corroboration anything in the fan-out has. It is evidence, not a verdict,
and you should feel free to override it.

---

## 3 · Direction A — Hands  (~4 min)

*Core assumption: the room's voice beats its phones. No participant app exists.*

Driven entirely from the host control — one button at a time.

- [ ] **`Put it on the screen`** — the projector carries the question alone.
- [ ] **`Publish the hint to the room`.** **Stop and look at this button.** It is
      A's second strike, made visible: with no participant device, A's only
      possible hint is announced to everyone — *"we need a hint"*, out loud. A
      cannot satisfy AC-48 as T-13 rewrote it.
- [ ] **`Everyone ready — count hands`**, then **`one fewer hand`** through the
      count. Picture the newcomer: unsure, raising nothing, visibly, five times
      running. That is the first strike, and it is the exact cost §9 names.
- [ ] Read the two strikes recorded inside the take itself — they are on the
      page, not hidden in a doc.

**The question A is asking you:** the build shrinks to a fraction and the room
gets louder. Is that trade worth what it costs the person who isn't sure?

A is in the fan-out as the honest loser, on your call at T-14 — so in October it
is a **built** comparison for *do phones help or hurt*, not a hypothesis.

---

## 4 · The review surface  (~10 min, and it is timing you)

Not a direction — it doesn't fork, and it is a tool for exactly one person: you.

- [ ] **Watch the clock in the corner.** AC-21 targets **15 minutes** to review a
      batch and nobody has ever measured it. This is the first measurement.
- [ ] Work the five candidates marked **`◷ needs you`** — `q4` through `q8`.
- [ ] On each: read the explanation, then **`I have read it. It is correct.`**
      That is AC-72's blocking gate — the machine proved the *answer*, never the
      prose a host reads aloud.
- [ ] Press **`Simulate a quoted-output mismatch`** once, on any question, to see
      the gate actually refuse.
- [ ] **`Accept into the bank`** or **`Reject`** with a reason.
- [ ] **Tell me the number on the clock when you finish.** Whatever it says is
      the real AC-21, and if it's 25 minutes then AC-21 is wrong, not you.

---

## 5 · The Discord seam  (~3 min)

- [ ] Run all five sign-in cases. The one that matters is **`a server
      Administrator`** — it proves AC-65: Administrator and guild ownership
      cannot host, *by construction*, because the check is on the role **ID**.
- [ ] **`Discord is down, mid-segment`** — AC-69: a room authorized at creation
      runs to completion regardless. No degraded state to design.
- [ ] Glance at the seam diagram: what crosses the boundary, and what each side
      sees.

---

## What ends T-12

Bring back three things:

1. **The direction to converge on** — A, B, or C. One sentence on why is plenty;
   the reasons come out during iteration.
2. **Everything that annoyed you.** Wording, spacing, pacing, a control in the
   wrong place, a word that sounds wrong out loud. That list *is* the first
   iteration round, and no complaint is too small — this is the last stage where
   any of it is cheap.
3. **The three instrumented judgments**, which no agent can make for you:
   - back-row legibility at six metres (AC-78/AC-38)
   - lights up or lights down (AC-80)
   - whether split-before-answer does what §9 claims (AC-93)
   - *plus the clock number from the review surface (AC-21)*

**What does not end T-12:** picking the one you dislike least. The stage's rule
is that iteration ends when you **love** a take, not when you accept one. The
working test — *would you keep this if a better option appeared tomorrow?* If the
honest answer for all three is no, that is a real result and we fan out again.
