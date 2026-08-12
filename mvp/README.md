# MVP — the hand-run quiz

The smallest thing that solves the observed problem: **one fresh question nobody
in the room has seen, displayed as legible code.** No server, no accounts, no
room codes, no capacity risk. One HTML file you open on the projector.

This is deliberately not the product. It exists to put fresh verified questions
in front of real attendees now, and to generate the user evidence the design
stages need — *does the room actually argue about the answer?*

## The shape: one question, 3–5 minutes, at the very end

Not a quiz round. A closing bit.

The segment runs **after the last talk**, takes **one question**, and is over in
three minutes — five if the argument catches. That is the whole format.

Three reasons it is this and not a longer round:

1. **The argument does not have to be cut off.** Mid-meetup, you must stop the
   disagreement to start the next talk — which kills the one thing the segment
   exists to produce. At the end, it walks to the bar with them and keeps going.
2. **One question a night is eight months of supply from a single batch.** The
   generation pipeline stops being urgent, which is `PHILOSOPHY.md` §8 paying
   out for the second time.
3. **Nobody leaves during it.** A three-minute closer at the door is a thing
   people stay for. A ten-minute round competes with the trip home.

The cost is that it broke answer-position fairness — a deck of one cannot
balance anything — and the obvious repair broke it worse. Both are fixed; see
*Anti-gaming* below, and read it before touching the slot logic.

## Two builds

| File | For | Contains |
|---|---|---|
| `pop-quiz-<day>-<qid>.html` | **The host.** Your laptop only. | Everything — answer, explanation, hint, receipt |
| `pop-quiz-<day>-<qid>-participant.html` | **Attendees.** Safe to hand out. | The question and the five options. Nothing else. |

The `-<qid>` suffix names the question the deck is loaded with, so a single-question
build never overwrites a batch build you were reviewing from.

The participant build is answer-free **by construction**, not by hiding: no
`data-correct`, no explanation, no hint, no receipt, and the reveal code is
excised from the JavaScript rather than merely disabled. The build asserts this
and fails if any of it leaks. The correct answer *text* is of course present —
it is necessarily one of the five visible options (AC-62). What is absent is any
way to tell which one.

Regenerate both:

```
python3 tools/build_deck.py 2026-08-12 host        --only q3
python3 tools/build_deck.py 2026-08-12 participant --only q3
```

Drop `--only` to build the whole verified batch into one deck. That form is for
reviewing questions on your own machine, not for running the segment.

## No projector? Three ways to run it

Code cannot be read aloud — people have to see it. That leaves:

**1. Their own phones.** Put the participant build somewhere they can open, and
drop the link in the meetup Discord. Everyone reads the code on their own
screen; you read the answer and explanation aloud off your laptop. This is the
best fallback and it is also the most faithful to where the product is going.
It needs the file reachable at a URL — a gist, a static host, anything.

**2. Paper.** Open the participant build and print it. There is a print
stylesheet: controls disappear, the question prints one to a page, and syntax
highlighting becomes weight rather than colour so it survives a mono printer.
At one question a night this is a **single sheet** — photocopy it that morning
and the segment has no technology in it at all. This is now the most robust
fallback, not the last resort.

**3. Any screen at all.** A wall-mounted TV, a large monitor turned toward the
room, or a laptop on a table for a small group. The deck is just a web page;
`◐` gives you a dim-room mode.

Whichever you pick, **the host build stays on your machine.** It has the answers
in it.

## Running it at the meetup

Open `2026-08-12/pop-quiz-2026-08-12-q3.html`, fullscreen it, and have it on
screen **before** you announce it. The clock starts when people can see code.

| | | |
|---|---|---|
| **0:00** | 0:30 | Code up, **say nothing**. Let them read. The silence is the format — if you talk over it, half the room never forms an opinion and the vote is worthless. |
| **0:30** | 1:00 | *"Hands up for A."* Straight through A to E. No discussion yet. Count out loud, badly. It does not matter. |
| **1:30** | 0:30 | *"Someone who said E — why?"* One person, twenty seconds. Then someone who said B. **This is the segment.** Everything else is setup for it. |
| **2:00** | 0:30 | **Reveal** (`Space`). Read the explanation aloud verbatim — it is written to be read aloud, not to be paraphrased. |
| **2:30** | 0:30 | Receipt only if you want it (`◐` for a dim room). Then: *"That's it — go argue about it downstairs."* |

Three minutes. Let it run to five only if the 1:30 row is going well; that is
the only row worth spending extra time on.

**If the room is silent at 1:30**, hit **Hint** (`H`) and re-take the vote. A
silent room is a question that was too hard or a screen nobody could read —
write down which, because that is exactly the evidence the prototype stage needs.

Everything works offline. Venue wifi is assumed hostile.

### Do not

- **Do not run two questions because the first went well.** The second one is
  always worse — the room has spent its appetite, and you have burned a question
  that would have carried a whole other meetup.
- **Do not announce the segment at the start of the night.** Announced, it
  becomes a thing people prepare for. Unannounced, it stays a pop quiz.
- **Do not read the answer off your laptop before the reveal.** You are playing
  too (`PHILOSOPHY.md` §7), and a Rust-literate host who has read the source
  already knows — just do not say it.

## What "verified" means here, exactly

Every question in the deck was, before anyone saw it:

- compiled with **rustc 1.96.1, edition 2021**
- run **5 times**, with all 5 runs producing **byte-identical** output
- run under **Miri with strict provenance**, reporting no undefined behaviour —
  and Miri's output matched the native output byte-for-byte
- the one non-compiling question was confirmed rejected by the compiler, with
  its error code recorded (**E0502**)

**No answer in this deck was written by hand.** `verify.py` records whatever the
program actually printed and `build_deck.py` reads the answer from that record.
A human wrote the programs, the distractors and the explanations; the machine
decided every correct answer.

**What it does not mean.** A clean Miri run proves there was no undefined
behaviour *on the paths actually executed*. It is strong evidence, not a proof
of total correctness. This set deliberately contains **no** question whose answer
is "exhibits undefined behavior" — that category needs a much stronger claim than
this MVP makes, and it is also the category that leaks (see the Phase 2
synthesis: `unsafe` in the source is a near-perfect tell for a UB answer).

## Anti-gaming, such as it is

The whole point of this project is that the meta must not become learnable.
Two things are enforced by the build, not by good intentions:

- **Correct-answer position is drawn uniformly from the date, and from nothing
  else.** Not from the ledger, not from what came up last month. An attendee
  with perfect memory of every previous answer is still guessing 1 in 5.
- **"does not compile" appears on every question**, so its presence tells you
  nothing.

There is no published distribution of answer categories, and there should never
be one.

### The letters will look lopsided. Leave them alone.

Over a year you will see `B` twice in a row, and some letter will go missing for
six months. **That is what random looks like, and it is not exploitable** —
under a uniform draw, past frequency tells you exactly nothing about the next
one. The urge to even it out is the trap, and this project already fell in it.

The first repair for the one-question format did exactly that: take the
least-used letter next, never repeat last month's. It reads like good practice.
It hands the room a free answer every fifth meetup:

| meetup in cycle | letters you can eliminate | your odds |
|---|---|---|
| 1 | 0 of 5 | 1 in 5 |
| 2 | 1 of 5 | 1 in 4 |
| 3 | 2 of 5 | 1 in 3 |
| 4 | 3 of 5 | 1 in 2 |
| **5** | **4 of 5** | **certain** |

Measured over 20 meetups that scheme gave a **45.7%** hit rate against a 20%
baseline — worse than the always-`A` bug it was written to fix. Any rule that
reads history constrains the next answer, and a constraint is information.

So the build audits the **generator**, not the sequence: 20,000 synthetic draws
tested for uniformity in *both* tails — too skewed means the generator broke,
**too even means someone reintroduced balancing** — and the audit is structurally
unable to change what tonight's answer is. An audit that can move tonight's
output is a rule, and rules leak. `answer-history.json` is a record now; nothing
reads it back.

## Regenerating

```
python3 tools/verify.py                            # authors + verifies, writes verified.json
python3 tools/build_deck.py 2026-08-12 host --only q3
```

`verify.py` holds the question programs. `<day>/content.json` holds distractors
and explanations and never names a correct answer. Miri is a separate pass —
see the repo's run-state for how it was run.

The build audits the generator on every run and prints the result, so there is
nothing separate to remember. To look at the series so far:

```
python3 - <<'EOF'
import json, collections
h = json.load(open('answer-history.json'))["meetups"]
seq = ["ABCDE"[m["slot"]] for m in h]
print(" ".join(seq), "|", dict(sorted(collections.Counter(seq).items())))
print("questions used:", ", ".join(f"{m['question']}@{m['day']}" for m in h))
EOF
```

This is for looking — mainly to check you are not about to re-run a question.
**Do not act on the letter counts.** If they look lopsided, that is the system
working; see *Anti-gaming*.

## Known rough edges

- Question programs are hand-written, not LLM-generated. The generation half of
  the pipeline is October's work; this proves the *verification* and *delivery*
  halves.
- Voting is by show of hands. That is a feature at this size — it is louder and
  more argumentative than phones — but it collects no data, so "how the room
  voted" is whatever you remember. **Write the split down after each meetup**;
  at one question a night it is four numbers, and it is the only record the
  segment produces.
- The 8 verified questions are now **8 meetups of supply**, not one night's
  deck. Used so far: q3 (Aug 12). With Aug 20 as the second run and September
  off for RustConf, that batch carries the segment into spring — which is why
  the generation pipeline is no longer on anyone's critical path.
- Nothing stops you rebuilding with a question you already ran. The ledger
  records which question each night used, but it does not refuse a repeat —
  check it before you pick.
