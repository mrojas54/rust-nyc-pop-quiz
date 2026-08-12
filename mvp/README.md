# MVP — the hand-run quiz

The smallest thing that solves the observed problem: **fresh questions nobody in
the room has seen, displayed as legible code.** No server, no accounts, no room
codes, no capacity risk. One HTML file you open on the projector.

This is deliberately not the product. It exists to put fresh verified questions
in front of real attendees now, and to generate the user evidence the design
stages need — *does the room actually argue about the answer?*

## Running it at the meetup

1. Open `2026-08-12/pop-quiz-2026-08-12.html` in any browser. Fullscreen it.
2. Read the question aloud. Hit **Start** for a 90-second timer (or don't — the
   timer is optional and the room can just talk).
3. Take answers by show of hands, A through E.
4. **Hint** (`H`) publishes the hint if the room is stuck.
5. **Reveal** (`Space`) shows the answer, the explanation, and the verification
   receipt. Read the explanation aloud — it is written to be read aloud.
6. `←` / `→` to move between questions. `◐` toggles dark mode for a dim room.

Everything works offline. Venue wifi is assumed hostile.

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

- **Correct-answer positions are balanced and shuffled.** In this deck:
  E, C, A, B, D, C, A, B. The first build produced B, D, B, D, B, D, B — a
  visible alternating pattern from a badly seeded shuffle. It was caught by an
  audit and fixed; the audit is worth re-running every time.
- **"does not compile" appears on every question**, so its presence tells you
  nothing. It is the answer exactly once, and it is not announced.

There is no published distribution of answer categories, and there should never
be one.

## Regenerating

```
python3 tools/verify.py                 # authors + verifies, writes verified.json
python3 tools/build_deck.py 2026-08-12  # builds the self-contained HTML
```

`verify.py` holds the question programs. `<day>/content.json` holds distractors
and explanations and never names a correct answer. Miri is a separate pass —
see the repo's run-state for how it was run.

To audit a built deck:

```
python3 - <<'EOF'
import re,pathlib,collections
h=pathlib.Path('2026-08-12/pop-quiz-2026-08-12.html').read_text()
pos=[int(m) for m in re.findall(r'data-correct="(\d+)"',h)]
print([ "ABCDE"[p] for p in pos ], dict(sorted(collections.Counter(pos).items())))
EOF
```

## Known rough edges

- Question programs are hand-written, not LLM-generated. The generation half of
  the pipeline is October's work; this proves the *verification* and *delivery*
  halves.
- Voting is by show of hands. That is a feature at this size — it is louder and
  more argumentative than phones — but it collects no data, so "how the room
  voted" is whatever you remember.
- 8 questions is more than one meetup needs. Pick 3–5; keep the rest.
