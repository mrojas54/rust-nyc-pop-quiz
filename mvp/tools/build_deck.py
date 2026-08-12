#!/usr/bin/env python3
"""Build the projector deck from verified.json + content.json.

Correct answers come ONLY from verified.json (machine-established). content.json
holds authored distractors and explanations and never names the answer.

Output is a single self-contained HTML file: no network, no fonts to fetch, no
server. Venue wifi is assumed hostile.
"""

import hashlib
import html
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent

# usage: build_deck.py [day] [host|participant] [--only <question-id>]
#   --only builds the single-question deck the live segment actually runs. The
#   multi-question form is kept for reviewing a whole batch on your own machine.
ARGV = sys.argv[1:]
ONLY = None
if "--only" in ARGV:
    i = ARGV.index("--only")
    assert i + 1 < len(ARGV), "--only needs a question id, e.g. --only q4"
    ONLY = ARGV[i + 1]
    del ARGV[i:i + 2]

DAY = ARGV[0] if len(ARGV) > 0 else "2026-08-12"
MODE = ARGV[1] if len(ARGV) > 1 else "host"
assert MODE in ("host", "participant"), "mode must be host or participant"
PARTICIPANT = MODE == "participant"
DDIR = HERE.parent / DAY

KEYWORDS = {
    "as","break","const","continue","crate","dyn","else","enum","extern","false","fn",
    "for","if","impl","in","let","loop","match","mod","move","mut","pub","ref","return",
    "self","Self","static","struct","super","trait","true","type","unsafe","use","where","while",
}
TYPES = {"i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize",
         "f32","f64","bool","char","str","String","Vec","Option","Some","None","Ok","Err","Box"}

TOKEN = re.compile(
    r'(?P<comment>//[^\n]*)'
    r'|(?P<string>"(?:[^"\\]|\\.)*")'
    r'|(?P<macro>\b[a-zA-Z_][a-zA-Z0-9_]*!)'
    r'|(?P<word>\b[A-Za-z_][A-Za-z0-9_]*\b)'
    r'|(?P<num>\b\d[\d_]*\b)'
)


def highlight(src: str) -> str:
    out, last = [], 0
    for m in TOKEN.finditer(src):
        out.append(html.escape(src[last:m.start()]))
        kind = m.lastgroup
        text = html.escape(m.group())
        if kind == "word":
            if m.group() in KEYWORDS:
                cls = "kw"
            elif m.group() in TYPES:
                cls = "ty"
            else:
                cls = None
            out.append(f'<span class="{cls}">{text}</span>' if cls else text)
        else:
            out.append(f'<span class="{kind}">{text}</span>')
        last = m.end()
    out.append(html.escape(src[last:]))
    return "".join(out)


def _rng(*parts):
    """Deterministic, well-mixed stream. Derived from a real hash, not from the
    question id's arithmetic -- ids like q1..q8 are far too close together to
    seed an LCG with, which produced a visible B,D,B,D pattern on first build."""
    seed = int(hashlib.blake2b("|".join(map(str, parts)).encode(), digest_size=8).hexdigest(), 16)
    def nxt(bound):
        nonlocal seed
        seed = (seed * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        return (seed >> 33) % bound
    return nxt


def shuffled(items: list, *seed_parts) -> list:
    out = list(items)
    nxt = _rng(*seed_parts)
    for i in range(len(out) - 1, 0, -1):
        j = nxt(i + 1)
        out[i], out[j] = out[j], out[i]
    return out


def answer_slots(n_questions: int, n_options: int, salt: str) -> list:
    """Where the correct answer sits, per question, within a multi-question deck.

    Balanced by construction -- every option letter is used as close to equally
    often as the counts allow -- then shuffled so the sequence carries no
    pattern. A quiz whose answer positions are guessable is the same failure as
    a quiz whose questions are memorised.

    This only balances anything when a deck holds several questions. For the
    one-question-a-night format the balancing unit is the meetup series, not the
    deck -- see slot_for_meetup()."""
    assert n_questions > 1, "single-question decks must use slot_for_meetup()"
    base = [i % n_options for i in range(n_questions)]
    return shuffled(base, "slots", salt, n_questions, n_options)


# ---- answer position across meetups -----------------------------------------
#
# One question a night breaks within-deck balancing completely: with n=1 there is
# exactly one arrangement, so answer_slots(1, 5, day) returned slot 0 for every
# date -- the answer would have been A at every meetup, forever.
#
# The obvious repair -- balance across the series instead, least-used slot next,
# never repeat last month -- is WORSE, and it is worth writing down why, because
# it is the repair anyone would reach for second.
#
# Balance and unpredictability are in direct conflict. Every rule that reads
# history constrains the next answer, and a constrained answer is information an
# attendee can have for free. Enforcing "counts stay within one of each other"
# means that after four meetups of a cycle the fifth is FULLY DETERMINED: an
# attendee who remembers four letters knows the fifth with certainty. Measured
# over 20 meetups, that scheme handed out a guaranteed answer every fifth night
# and a 45.7% average hit rate against a 20% baseline. It was strictly worse
# than the always-A bug it replaced, and it looked responsible.
#
# So: the slot is drawn UNIFORMLY and depends on nothing but the night itself.
# The attendee's best guess is 1/5 and stays 1/5 no matter how perfectly they
# remember every previous answer. That is the floor -- they have to guess
# something -- and no memoryful scheme can reach it.
#
# The price is that the letters will NOT look evenly spread. B will come up
# twice in a row; some letter will be missing for half a year. That is what
# random looks like, and it is not exploitable: under a uniform draw, past
# frequency carries exactly zero information about the next one. Do not "fix"
# it. Making the sequence look tidier is the bug.
#
# The ledger below is therefore a RECORD, not an input. Nothing reads it back
# to decide anything.

HISTORY_PATH = HERE.parent / "answer-history.json"
HISTORY_NOTE = (
    "A record of which option slot held the correct answer each meetup, and "
    "which question was used. Written by build_deck.py. Deliberately NOT an "
    "input to slot selection -- slots are drawn uniformly from the date alone, "
    "because any rule that reads this file back would let an attendee who "
    "remembers past answers narrow the next one. Uneven counts and repeated "
    "letters are expected and must not be 'corrected'."
)


def load_history() -> dict:
    if HISTORY_PATH.exists():
        return json.loads(HISTORY_PATH.read_text())
    return {"_note": HISTORY_NOTE, "meetups": []}


def slot_for_day(day: str, n_options: int) -> int:
    """Where this meetup's correct answer sits. Uniform over n_options.

    PURE, and it must stay pure. It takes the day and nothing else -- no
    ledger, no counts, no previous slot. That signature is the security
    property: a function that cannot see history cannot leak it. If you are
    about to add a parameter here, read the block comment above first.

    Keyed on the day rather than the question so that swapping tonight's
    question does not move the answer; the slot belongs to the night."""
    return _rng("meetup-slot", day)(n_options)


# chi-square critical values, df = 4 (5 options). Two-tailed on purpose: the
# upper tail catches a skewed generator, the LOWER tail catches someone
# reintroducing balancing -- output that is too evenly spread is not evidence
# of fairness, it is evidence of a constraint, which is the exploitable case.
CHI2_DF4_UPPER = 18.467   # p = 0.001
CHI2_DF4_LOWER = 0.297    # p = 0.999


def audit_generator(n_options: int, sample: int = 20000) -> str:
    """Fails the build if the slot generator is biased.

    Tested against a large synthetic run of dates, never against the real
    ledger. That is deliberate: it gives the test enough power to actually
    catch a broken generator, while constraining nothing about what tonight's
    answer position turns out to be. An audit that can change tonight's output
    is not an audit, it is a rule -- and rules leak."""
    days = [f"{2026 + i // 366:04d}-{i % 12 + 1:02d}-{i % 28 + 1:02d}-{i}"
            for i in range(sample)]
    counts = [0] * n_options
    for d in days:
        counts[slot_for_day(d, n_options)] += 1

    expected = sample / n_options
    chi2 = sum((c - expected) ** 2 / expected for c in counts)
    assert chi2 < CHI2_DF4_UPPER, (
        f"slot generator is skewed (chi2={chi2:.1f} over {sample} draws): "
        + ", ".join(f"{'ABCDE'[i]}={c}" for i, c in enumerate(counts))
    )
    assert chi2 > CHI2_DF4_LOWER, (
        f"slot generator is TOO EVEN (chi2={chi2:.2f} over {sample} draws) -- "
        "something is balancing the output. Balanced answer positions are "
        "predictable answer positions; see the comment above slot_for_day()."
    )

    # A memoryless draw must produce repeats. If it never does, something is
    # suppressing them, which is exactly the "never last month's letter" leak.
    seq = [slot_for_day(d, n_options) for d in days[:2000]]
    repeats = sum(1 for a, b in zip(seq, seq[1:]) if a == b)
    assert repeats > 0, (
        "slot generator never repeats a position on consecutive draws -- "
        "an attendee could rule out last meetup's letter for free"
    )
    return f"generator uniform (chi2={chi2:.1f}, df=4) over {sample} draws"


def slot_for_meetup(day: str, qid: str, n_options: int) -> int:
    """slot_for_day(), plus a line in the ledger. The ledger is write-only."""
    slot = slot_for_day(day, n_options)

    history = load_history()
    meetups = history["meetups"]
    entry = next((m for m in meetups if m["day"] == day), None)
    if entry is None:
        meetups.append({"day": day, "question": qid, "slot": slot})
        meetups.sort(key=lambda m: m["day"])
    else:
        entry["question"], entry["slot"] = qid, slot
    history["_note"] = HISTORY_NOTE
    HISTORY_PATH.write_text(json.dumps(history, indent=2) + "\n")
    return slot


def main() -> int:
    verified = json.loads((DDIR / "verified.json").read_text())
    content = json.loads((DDIR / "content.json").read_text())

    live = [q for q in verified["questions"] if q.get("verified")]
    for q in verified["questions"]:
        if not q.get("verified"):
            print(f"skipping unverified {q['id']}")

    if ONLY:
        live = [q for q in live if q["id"] == ONLY]
        assert live, f"{ONLY} is not a verified question in {DAY}/verified.json"
        print(audit_generator(5))
        slots = [slot_for_meetup(DAY, ONLY, 5)]

        seq = [m for m in load_history()["meetups"]]
        used = [m["question"] for m in seq if m["day"] != DAY]
        if ONLY in used:
            print(f"  !! {ONLY} was already run on "
                  f"{', '.join(m['day'] for m in seq if m['question'] == ONLY and m['day'] != DAY)}")
        print(f"  {DAY}: {ONLY}, answer at {'ABCDE'[slots[0]]}")
        print(f"  series so far: {' '.join('ABCDE'[m['slot']] for m in seq)}"
              f"   ({len(seq)} meetups -- uneven is correct, do not 'fix' it)")
    else:
        slots = answer_slots(len(live), 5, DAY)

    cards = []
    for n, (q, slot) in enumerate(zip(live, slots), start=1):
        c = content[q["id"]]
        answer = q["answer"].rstrip("\n") if q["compiled"] else "does not compile"
        distractors = list(c["distractors"])
        # every question carries "does not compile" so its presence is never a tell
        if answer != "does not compile" and "does not compile" not in distractors:
            distractors.append("does not compile")
        distractors = shuffled(distractors, "distractors", DAY, q["id"])[:4]

        options = list(distractors)
        options.insert(slot, answer)
        correct = slot
        assert len(options) == 5, f"{q['id']}: {len(options)} options, expected 5"
        assert options[correct] == answer, f"{q['id']}: correct index does not point at the answer"
        assert len(set(options)) == 5, f"{q['id']}: duplicate options"

        receipt = [f"{verified['rustc']} · edition {verified['edition']} · compiled cleanly"]
        if q["compiled"]:
            receipt.append(f"{q['runs']} of {q['runs']} native runs produced byte-identical output")
            if q.get("miri") == "clean":
                receipt.append("Miri (strict provenance): no undefined behaviour on the executed path"
                               + (" · output matched native byte-for-byte" if q.get("miri_output_matches") else ""))
        else:
            receipt.append(f"rejected by the compiler: {', '.join(q.get('error_codes', []))}")

        cards.append({
            "n": n, "id": q["id"], "topic": q["topic"], "difficulty": q["difficulty"],
            "source": q["source"].rstrip("\n"), "hint": q["hint"],
            "choices": options, "correct": correct,
            "explanation": c["explanation"], "receipt": receipt,
        })

    body = []
    for c in cards:
        letters = "ABCDE"
        # "Question 1 of 1" is noise on the single-question deck.
        qno = "Pop quiz" if len(cards) == 1 else f"Question {c['n']} of {len(cards)}"
        choices = "".join(
            f'<button class="choice" data-i="{i}" aria-label="Option {letters[i]}">'
            f'<span class="letter">{letters[i]}</span><pre>{html.escape(t)}</pre>'
            f'<span class="mark" aria-hidden="true"></span></button>'
            for i, t in enumerate(c["choices"])
        )
        receipt = "".join(f"<li>{html.escape(r)}</li>" for r in c["receipt"])
        if PARTICIPANT:
            # Nothing that reveals the answer may exist in this payload -- not the
            # index, not the explanation, not the receipt, not the hint. The option
            # TEXT stays public: the correct answer is necessarily one of the five
            # visible options (AC-62). This is AC-60 enforced at build time.
            body.append(f'''<section class="card" id="{c['id']}" hidden>
  <header class="meta">
    <span class="qno">{qno}</span>
    <span class="badges"><span class="badge">{html.escape(c['topic'])}</span>
    <span class="badge dim">Difficulty {c['difficulty']}</span></span>
  </header>
  <p class="prompt">What does this program print?</p>
  <pre class="code"><code>{highlight(c['source'])}</code></pre>
  <div class="choices">{choices}</div>
</section>''')
            continue
        body.append(f'''<section class="card" id="{c['id']}" data-correct="{c['correct']}" hidden>
  <header class="meta">
    <span class="qno">{qno}</span>
    <span class="badges"><span class="badge">{html.escape(c['topic'])}</span>
    <span class="badge dim">Difficulty {c['difficulty']}</span></span>
  </header>
  <p class="prompt">What does this program print?</p>
  <pre class="code"><code>{highlight(c['source'])}</code></pre>
  <div class="choices">{choices}</div>
  <div class="hint" hidden><strong>Hint</strong> {html.escape(c['hint'])}</div>
  <div class="reveal" hidden>
    <p class="answer-label">Answer <span class="letter-out"></span></p>
    <div class="explain">{html.escape(c['explanation'])}</div>
    <details class="receipt"><summary>Verification receipt</summary><ul>{receipt}</ul></details>
  </div>
</section>''')

    doc = TEMPLATE.replace("__CARDS__", "\n".join(body)) \
                  .replace("__COUNT__", str(len(cards))) \
                  .replace("__DAY__", DAY) \
                  .replace("__MODE__", MODE) \
                  .replace("__RUSTC__", html.escape(verified["rustc"]))

    if PARTICIPANT:
        # Excise host-only JS rather than merely guarding it. Nothing that could
        # reveal an answer should exist in a payload handed to attendees.
        doc = re.sub(r"/\*HOSTJS\*/.*?/\*ENDHOSTJS\*/", "", doc, flags=re.S)
        for leak in ("data-correct", "letter-out", "byte-identical", "Verification receipt"):
            assert leak not in doc.split("</style>")[-1], f"participant build leaks {leak!r}"

    # The single-question deck gets its own name so it never overwrites a batch
    # build, and so the filename says which question is loaded.
    stem = f"pop-quiz-{DAY}-{ONLY}" if ONLY else f"pop-quiz-{DAY}"
    suffix = "-participant" if PARTICIPANT else ""
    out = DDIR / f"{stem}{suffix}.html"
    out.write_text(doc)
    print(f"wrote {out}  ({len(cards)} questions, {out.stat().st_size // 1024} KB)")
    return 0


TEMPLATE = r"""<!DOCTYPE html>
<html lang="en" data-mode="__MODE__"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Rust NYC Pop Quiz — __DAY__</title>
<style>
:root{
--gray-50:#f8f9fa;--gray-100:#e9ecef;--gray-200:#dee2e6;--gray-300:#cbd5e0;
--gray-500:#718096;--gray-600:#4a5568;--gray-800:#2d3748;--white:#fff;
--amber-500:#d69e2e;--amber-700:#945c0a;--green-500:#38a169;--red-500:#e53e3e;
--bg:var(--gray-50);--panel:var(--white);--ink:var(--gray-800);--ink-2:var(--gray-600);
--muted:var(--gray-500);--line:var(--gray-300);--accent:var(--amber-500);--accent-ink:var(--amber-700);
--code-bg:var(--gray-100);
--mono:'Cascadia Mono','SF Mono',SFMono-Regular,Menlo,Monaco,Consolas,'Liberation Mono',monospace;
--serif:'Instrument Serif',Georgia,'Times New Roman',serif;
--radius:4px;
}
html[data-theme="dark"]{
--bg:#1a202c;--panel:#22293a;--ink:#edf2f7;--ink-2:#cbd5e0;--muted:#a0aec0;
--line:#3d4759;--code-bg:#161b26;--accent:#f6c454;--accent-ink:#f6c454;
}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);font-family:var(--mono);
font-size:17px;line-height:1.6;-webkit-font-smoothing:antialiased}
h1{font-family:var(--serif);font-weight:400;font-size:30px;margin:0;line-height:1.2}
pre{margin:0;font-family:var(--mono)}
button{font-family:var(--mono);font-size:inherit;color:inherit}
:focus-visible{outline:2px solid var(--accent);outline-offset:2px}

.bar{position:sticky;top:0;z-index:10;display:flex;gap:10px;align-items:center;flex-wrap:wrap;
padding:12px 20px;background:var(--panel);border-bottom:1px solid var(--line)}
/* flex-basis, not min-width:0 -- shrinking below content width made the
   subtitle wrap to one word per line in a narrow pane. Let the BAR wrap. */
.bar .grow{flex:1 1 260px;min-width:0}
.bar .sub{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.proto{display:inline-block;padding:3px 9px;border-radius:var(--radius);
background:var(--accent);color:#231a05;font-size:12px;letter-spacing:.08em;text-transform:uppercase;font-weight:700}
.sub{font-size:12px;color:var(--muted);letter-spacing:.04em}
.btn{border:1px solid var(--line);background:var(--bg);border-radius:var(--radius);
padding:8px 14px;min-height:40px;cursor:pointer}
.btn:hover{border-color:var(--muted)}
.btn.primary{background:var(--accent);border-color:var(--accent);color:#231a05;font-weight:700}
.timer{font-variant-numeric:tabular-nums;font-size:22px;min-width:64px;text-align:center}
.timer.low{color:var(--red-500);font-weight:700}

main{max-width:1000px;margin:0 auto;padding:28px 20px 96px}
.card{background:var(--panel);border:1px solid var(--line);border-radius:var(--radius);padding:28px}
.meta{display:flex;justify-content:space-between;align-items:baseline;gap:12px;flex-wrap:wrap;margin-bottom:6px}
.qno{font-size:13px;color:var(--muted);letter-spacing:.06em;text-transform:uppercase}
.badge{display:inline-block;border:1px solid var(--line);border-radius:var(--radius);
padding:2px 8px;font-size:12px;color:var(--ink-2)}
.badge.dim{color:var(--muted)}
.prompt{font-family:var(--serif);font-size:26px;margin:10px 0 16px}
.code{background:var(--code-bg);border:1px solid var(--line);border-radius:var(--radius);
padding:20px;overflow-x:auto;font-size:19px;line-height:1.55;tab-size:4}
.kw{color:#b7791f;font-weight:600}html[data-theme="dark"] .kw{color:#f6c454}
.ty{color:#2b6cb0}html[data-theme="dark"] .ty{color:#7fb3e8}
.string{color:#2f855a}html[data-theme="dark"] .string{color:#68d391}
.comment{color:var(--muted);font-style:italic}
.macro{color:#805ad5}html[data-theme="dark"] .macro{color:#b794f4}
.num{color:#c05621}html[data-theme="dark"] .num{color:#f6ad55}

.choices{display:flex;flex-direction:column;gap:9px;margin-top:18px}
.choice{display:flex;align-items:flex-start;gap:12px;width:100%;text-align:left;
border:1px solid var(--line);background:var(--bg);border-radius:var(--radius);
padding:12px 14px;min-height:48px;cursor:pointer}
.choice:hover{border-color:var(--muted)}
.choice .letter{flex:0 0 auto;width:26px;height:26px;border-radius:var(--radius);
background:var(--code-bg);border:1px solid var(--line);display:grid;place-items:center;
font-size:14px;font-weight:700;color:var(--ink-2)}
.choice pre{flex:1;min-width:0;white-space:pre-wrap;word-break:break-word;font-size:17px}
.choice .mark{flex:0 0 auto;font-weight:700;font-size:19px}
.choice.correct{border-color:var(--green-500);border-width:2px;background:color-mix(in srgb,var(--green-500) 8%,var(--bg))}
.choice.correct .mark::after{content:"\2713";color:var(--green-500)}
.choice.wrong{opacity:.62}
.choice.picked:not(.correct){border-color:var(--red-500);opacity:1}
.choice.picked:not(.correct) .mark::after{content:"\2717";color:var(--red-500)}

.hint,.reveal{margin-top:18px;border-top:1px solid var(--line);padding-top:16px}
.hint{color:var(--ink-2)}
.answer-label{font-family:var(--serif);font-size:22px;margin:0 0 10px}
.explain{color:var(--ink-2);text-wrap:pretty}
.receipt{margin-top:14px;font-size:13px;color:var(--muted)}
.receipt summary{cursor:pointer}
.receipt ul{margin:8px 0 0;padding-left:18px}
.receipt li{margin:3px 0}
.end{text-align:center;padding:48px 20px}
.end h1{margin-bottom:10px}
.foot{max-width:1000px;margin:0 auto;padding:0 20px 40px;font-size:12px;color:var(--muted);text-wrap:pretty}
html[data-mode="participant"] .host-only{display:none}
@media (prefers-reduced-motion:reduce){*{transition:none!important;animation:none!important}}
/* Print = paper handouts: every question, no controls, one per page. */
@media print{
 html,body{background:#fff;color:#000;font-size:11pt}
 .bar,.foot,#end,.host-only{display:none!important}
 main{max-width:none;padding:0}
 .card{display:block!important;page-break-after:always;break-after:page;
  border:none;padding:0 0 12pt;box-shadow:none;background:#fff}
 .card[hidden]{display:block!important}
 .code{background:#f4f4f4;border:1px solid #bbb;font-size:10.5pt;overflow:visible;white-space:pre-wrap}
 .choice{border:1px solid #999;background:#fff;page-break-inside:avoid;break-inside:avoid}
 .choice .mark{display:none}
 .prompt{font-size:14pt}
 .kw,.ty,.string,.macro,.num{color:#000!important;font-weight:600}
 .comment{color:#555!important}
}
@media (max-width:760px){.bar{padding:10px 14px;gap:8px}.btn{padding:7px 11px}.timer{font-size:19px;min-width:54px}}
@media (max-width:640px){.code{font-size:14px;padding:14px}.prompt{font-size:20px}body{font-size:15px}main{padding:18px 12px 80px}.card{padding:18px 14px}.choice pre{font-size:15px}}
</style></head><body>

<div class="bar">
  <span class="proto">MVP</span>
  <div class="grow"><span class="sub">Rust NYC Pop Quiz · __DAY__ · questions generated fresh and machine-verified</span></div>
  <span class="timer" id="timer" role="timer" aria-live="off">1:30</span>
  <button class="btn" id="tbtn">Start</button>
  <button class="btn host-only" id="hbtn" title="H">Hint</button>
  <button class="btn primary host-only" id="rbtn" title="Space">Reveal</button>
  <button class="btn" id="prev" aria-label="Previous">&larr;</button>
  <button class="btn" id="next" aria-label="Next">&rarr;</button>
  <button class="btn" id="theme" aria-label="Toggle dark mode">&#9681;</button>
</div>

<main id="main">
__CARDS__
<section class="end card" id="end" hidden>
  <h1>That's the quiz</h1>
  <p class="sub" style="font-size:15px">__COUNT__ questions, all generated for tonight and verified before anyone saw them.</p>
  <p style="margin-top:16px;color:var(--ink-2)">Nothing was recorded. No accounts, no scores, no leaderboard — the argument was the point.</p>
</section>
</main>

<p class="foot" role="status" aria-live="polite" id="status"></p>
<p class="foot">Verified with __RUSTC__ and Miri. A clean Miri run proves no undefined behaviour <em>on the paths actually executed</em> — it is evidence, not a proof of total correctness. Deliberately excluded from this set: questions whose answer is “exhibits undefined behavior”.</p>

<script>
(function(){
  var cards=[].slice.call(document.querySelectorAll('.card:not(.end)'));
  var end=document.getElementById('end'), i=0, revealed=[], picked=[];
  var timer=document.getElementById('timer'), status=document.getElementById('status');
  var iv=null, left=90;

  function fmt(s){return Math.floor(s/60)+':'+String(s%60).padStart(2,'0')}
  function stop(){if(iv){clearInterval(iv);iv=null}document.getElementById('tbtn').textContent='Start'}
  function resetTimer(){stop();left=90;timer.textContent=fmt(left);timer.classList.remove('low')}
  document.getElementById('tbtn').onclick=function(){
    if(iv){stop();return}
    this.textContent='Pause';
    iv=setInterval(function(){
      left--;timer.textContent=fmt(Math.max(0,left));
      timer.classList.toggle('low',left<=10);
      if(left<=0){stop();say("Time's up.")}
    },1000);
  };

  function say(t){status.textContent=t}
  function show(n){
    cards.forEach(function(c){c.hidden=true});end.hidden=true;
    if(n>=cards.length){end.hidden=false;say('End of quiz.');window.scrollTo(0,0);return}
    i=n;cards[i].hidden=false;resetTimer();
    say('Question '+(i+1)+' of '+cards.length+'. '+(revealed[i]?'Answer revealed.':'Open.'));
    window.scrollTo(0,0);
  }
  /*HOSTJS*/
  function reveal(){
    var c=cards[i];if(!c||revealed[i])return;revealed[i]=true;stop();
    var correct=+c.dataset.correct, btns=c.querySelectorAll('.choice');
    btns.forEach(function(b,k){
      b.classList.add(k===correct?'correct':'wrong');
      if(picked[i]===k)b.classList.add('picked');
    });
    c.querySelector('.letter-out').textContent='ABCDE'[correct];
    c.querySelector('.reveal').hidden=false;
    c.querySelector('.hint').hidden=false;
    say('Answer revealed: option '+'ABCDE'[correct]+'.');
  }
  /*ENDHOSTJS*/
  cards.forEach(function(c,n){
    c.querySelectorAll('.choice').forEach(function(b,k){
      b.onclick=function(){
        if(revealed[n])return;
        picked[n]=k;
        c.querySelectorAll('.choice').forEach(function(x){x.style.borderColor=''});
        b.style.borderColor='var(--accent)';
        say('Marked option '+'ABCDE'[k]+' for the room.');
      };
    });
  });
  /*HOSTJS*/document.getElementById('rbtn').onclick=reveal;/*ENDHOSTJS*/
  /*HOSTJS*/document.getElementById('hbtn').onclick=function(){
    var c=cards[i];if(!c)return;var h=c.querySelector('.hint');
    if(!h)return;
    h.hidden=!h.hidden;say(h.hidden?'Hint hidden.':'Hint shown to the room.');
  };/*ENDHOSTJS*/
  document.getElementById('next').onclick=function(){show(Math.min(i+1,cards.length))};
  document.getElementById('prev').onclick=function(){show(Math.max(i-1,0))};
  document.getElementById('theme').onclick=function(){
    var d=document.documentElement.getAttribute('data-theme')==='dark';
    document.documentElement.setAttribute('data-theme',d?'light':'dark');
  };
  document.addEventListener('keydown',function(e){
    if(e.target.tagName==='INPUT')return;
    if(e.key==='ArrowRight'){show(Math.min(i+1,cards.length))}
    else if(e.key==='ArrowLeft'){show(Math.max(i-1,0))}
    /*HOSTJS*/
    else if(e.key===' '){e.preventDefault();reveal()}
    else if(e.key==='h'||e.key==='H'){document.getElementById('hbtn').click()}
    /*ENDHOSTJS*/
    else if(e.key==='t'||e.key==='T'){document.getElementById('tbtn').click()}
  });
  show(0);
})();
</script>
</body></html>
"""

if __name__ == "__main__":
    sys.exit(main())
