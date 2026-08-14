/* ===========================================================================
   Realistic mock data.

   Sources, answers, hints, topics, difficulties and receipt facts are copied
   VERBATIM from mvp/2026-08-12/verified.json — a real, machine-verified batch
   (pinned rustc 1.96.1, 5 byte-identical native runs each, Miri clean under
   strict provenance, one confirmed E0502 rejection). Distractors and
   explanations are copied verbatim from mvp/2026-08-12/content.json.

   Nothing here is prettied up. The explanations are the real long ones, the
   sources are the real multi-line ones, and the room numbers below are drawn
   from sequence/research/06-attendees.md rather than invented:

     - the venue caps at 110 RSVPs, median show rate 88%  ->  95 in the room
     - the participation rate has NEVER been measured (run-state, T-7), so the
       58/95 below is a deliberate placeholder the prototypes display rather
       than hide. It is the number October is supposed to tell us.
   =========================================================================== */

const RUSTC   = "rustc 1.96.1 (31fca3adb 2026-06-26)";
const EDITION = "2021";
const MIRI    = "miri under -Zmiri-strict-provenance, nightly";

/* The question the Aug 12 deck actually drew, with its real answer position.
   slot_for_day() put the correct answer at E; the option order below preserves
   that. Note that "does not compile" sits at C and is not the answer — it is
   present on every question precisely so its presence signals nothing (AC-24). */
const TONIGHT = {
  id: "q3",
  topic: "Collections",
  difficulty: 2,
  expect: "compiles",
  hint: "Read dedup's documentation in your head, then read it again. The key word is one you might skip over.",
  source: 'fn main() {\n    let mut v = vec![1, 2, 2, 3, 2, 1, 1];\n    v.dedup();\n    println!("{:?}", v);\n}\n',
  options: [
    { letter: "A", text: "[1, 2, 3]" },
    { letter: "B", text: "[1, 2, 2, 3, 2, 1, 1]" },
    { letter: "C", text: "does not compile" },
    { letter: "D", text: "[1, 2, 3, 2, 1, 1]" },
    { letter: "E", text: "[1, 2, 3, 2, 1]", correct: true }
  ],
  explanation: "dedup removes *consecutive* duplicates only — the word is easy to skip in the docs. The adjacent 2,2 and the trailing 1,1 collapse; the later 2 and 1 are not adjacent to their earlier occurrences, so they survive. If you want genuinely unique elements you have to sort first (making duplicates adjacent) or reach for a HashSet.",

  /* --------------------------------------------------------------------
     The same explanation, restructured into three beats.

     The one-paragraph form above is what the batch actually holds, and it
     is kept verbatim — AC-73's quoted-output check runs against it and the
     receipt's "this does not cover the prose" claim is about it.

     But AC-44 asks that someone who knows only beginner Rust can explain
     the solution TO SOMEONE ELSE, and one intermediate-pitched paragraph
     does not hand anyone that. The real explanations in the bank already
     have this shape latent in them — every one of them ends on a "worth
     arguing about" clause. This makes it explicit, and puts the popular
     WRONG answer in the middle beat rather than leaving it unmentioned.

     Design consequence for tone-architect: `explanation` stops being one
     string and becomes three fields. That is a content-model change, not a
     copy change, and it needs a criterion of its own.
     -------------------------------------------------------------------- */
  explains: {
    what: "dedup only removes duplicates that are sitting next to each other. The 2,2 in the middle and the 1,1 at the end were adjacent, so they collapsed into one. The other 2 and the other 1 weren't next to their twins, so they stayed exactly where they were.",
    whyWrong: {
      option: "A",
      text: "In almost every other language, a method called dedup means unique — give me each value once. Rust's doesn't. The word doing all the work is consecutive, it appears once in the docs, and it is extremely easy to read straight past."
    },
    argue: "If you want genuinely unique elements you have to sort first — which makes the duplicates adjacent — or reach for a HashSet. Which of those you pick is a real decision, and it is mostly about whether you care about order."
  },
  receipt: {
    rustc: RUSTC,
    edition: EDITION,
    runs: 5,
    byteIdentical: true,
    miri: "clean",
    miriMatches: true,
    target: "aarch64-apple-darwin"
  },

  /* --------------------------------------------------------------------
     The trace — stepping the program, one comparison at a time.

     This is the beginner lane made concrete. AC-96 asks that the first
     beat of an explanation land for someone who knows only beginner Rust,
     and prose can only assert that `dedup` compares neighbours. A trace
     SHOWS it, and the step where it shows it is step 4: the second 2
     survives because it is sitting next to a 3. That single frame is the
     entire reason 24 people answered [1, 2, 3].

     PROVENANCE — open question, carried to tone-architect. This trace is
     AUTHORED, so under AC-74 it is human prose and belongs on the human
     side of the line with the explanation. It does not have to stay that
     way: a trace of this shape is mechanically derivable by instrumenting
     the program under the pinned toolchain and recording state per step,
     which would make it machine-established and put it on the SAME side of
     the line as the answer. That is a real pipeline capability, not a
     formatting choice, and it decides which side of AC-74 the trace lives
     on. See run-state.
     -------------------------------------------------------------------- */
  trace: {
    call: "v.dedup()",
    subtitle: "one pass, comparing each element with the one before it",
    cells: [1, 2, 2, 3, 2, 1, 1],
    steps: [
      { cursor: null, against: null, kept: [0],
        say: "We start with all seven. The first element is always kept — there is nothing in front of it for it to be a duplicate of." },
      { cursor: 1, against: 0, verdict: "keep", kept: [0, 1],
        say: "2 against 1. Different, so the 2 stays." },
      { cursor: 2, against: 1, verdict: "drop", kept: [0, 1],
        say: "2 against 2. The same — and sitting right next to each other. This one goes." },
      { cursor: 3, against: 1, verdict: "keep", kept: [0, 1, 3],
        say: "3 against 2. Different, so the 3 stays." },
      { cursor: 4, against: 3, verdict: "keep", kept: [0, 1, 3, 4], pivot: true,
        say: "2 against 3. Different — so this 2 stays. There is already a 2 in the list, back at the start, and dedup does not look. It only ever compares neighbours. This is the step the whole question turns on." },
      { cursor: 5, against: 4, verdict: "keep", kept: [0, 1, 3, 4, 5],
        say: "1 against 2. Different, so the 1 stays — same story. There is a 1 at the very front, and it is nowhere near this one." },
      { cursor: 6, against: 5, verdict: "drop", kept: [0, 1, 3, 4, 5],
        say: "1 against 1. The same, and adjacent. This one goes too." },
      { cursor: null, against: null, done: true, kept: [0, 1, 3, 4, 5],
        say: "Five left. Two were dropped, and both of them only because of who they happened to be sitting next to." }
    ]
  }
};

/* How the room voted. The [1, 2, 3] distractor catches everyone who reads
   dedup as "unique", so it beats the correct answer — which is the point of
   the segment, not a flaw in the mock. */
const ROOM = {
  present: 95,
  answered: 58,
  votes: { A: 24, B: 5, C: 3, D: 9, E: 17 },
  code: "RUST-4417"
};

/* The rest of the verified bank, for the organizer review surface.
   Correct answers live here because the reviewer is explicitly someone who has
   seen them (AC-22); no participant surface ever reads this array. */
const BANK = [
  {
    id: "q1", topic: "Ownership and Drop", difficulty: 2, expect: "compiles",
    hint: "Two things decide the order: when each value goes out of scope, and what order values in the SAME scope are dropped in.",
    source: 'struct Noisy(&\'static str);\n\nimpl Drop for Noisy {\n    fn drop(&mut self) {\n        println!("drop {}", self.0);\n    }\n}\n\nfn main() {\n    let _a = Noisy("a");\n    let _b = Noisy("b");\n    {\n        let _c = Noisy("c");\n    }\n    println!("end of main");\n}\n',
    answer: "drop c\nend of main\ndrop b\ndrop a\n",
    distractors: ["drop c\nend of main\ndrop a\ndrop b", "end of main\ndrop a\ndrop b\ndrop c", "drop a\ndrop b\ndrop c\nend of main"],
    explanation: "Two rules combine. Values in an inner scope drop when that scope ends, so _c drops before \"end of main\" ever prints. And within a single scope, values drop in reverse order of declaration -- so at the end of main, _b goes before _a. Worth arguing about afterward: the underscore prefix only silences the unused-variable warning, it does not change drop timing. Writing `let _ = Noisy(\"a\");` instead would drop it immediately, on that line.",
    runs: 5, byteIdentical: true, miri: "clean", miriMatches: true,
    state: "affirmed", affirmedBy: "michelle", affirmedAt: "2026-08-11 22:40"
  },
  {
    id: "q2", topic: "Iterators and closures", difficulty: 2, expect: "compiles",
    hint: "Adaptors don't run on their own. Something has to pull values through -- and watch how many get pulled.",
    source: 'fn main() {\n    let v = vec![1, 2, 3];\n    let it = v.iter().map(|x| {\n        println!("mapping {}", x);\n        x * 2\n    });\n    println!("created");\n    let total: i32 = it.take(2).sum();\n    println!("total {}", total);\n}\n',
    answer: "created\nmapping 1\nmapping 2\ntotal 6\n",
    distractors: ["mapping 1\nmapping 2\nmapping 3\ncreated\ntotal 6", "created\nmapping 1\nmapping 2\nmapping 3\ntotal 6", "created\ntotal 6"],
    explanation: "map is lazy: it builds a pipeline and runs nothing, so \"created\" prints first. Then take(2).sum() pulls exactly two values through the chain -- so only 1 and 2 are ever mapped, and 3 is never touched at all. The total is 2 + 4 = 6. This laziness is why a long adaptor chain compiles down to a single pass with no intermediate allocation.",
    runs: 5, byteIdentical: true, miri: "clean", miriMatches: true,
    state: "affirmed", affirmedBy: "michelle", affirmedAt: "2026-08-11 22:44"
  },
  {
    id: "q3", topic: "Collections", difficulty: 2, expect: "compiles",
    hint: TONIGHT.hint, source: TONIGHT.source,
    answer: "[1, 2, 3, 2, 1]\n",
    distractors: ["[1, 2, 3]", "[1, 2, 2, 3, 2, 1, 1]", "[1, 2, 3, 2, 1, 1]"],
    explanation: TONIGHT.explanation,
    runs: 5, byteIdentical: true, miri: "clean", miriMatches: true,
    state: "affirmed", affirmedBy: "michelle", affirmedAt: "2026-08-11 22:51"
  },
  {
    id: "q4", topic: "Integer arithmetic", difficulty: 3, expect: "compiles",
    hint: "Rust has two different notions of division for signed integers, and they disagree here. Which one do / and % use?",
    source: 'fn main() {\n    let a: i32 = -7;\n    let b: i32 = 2;\n    println!("{} {}", a / b, a % b);\n    println!("{} {}", a.div_euclid(b), a.rem_euclid(b));\n}\n',
    answer: "-3 -1\n-4 1\n",
    distractors: ["-4 1\n-4 1", "-3 1\n-4 1", "-4 -1\n-3 -1"],
    explanation: "Rust's / and % truncate toward zero, so -7 / 2 is -3, and the remainder takes the sign of the dividend: -1. div_euclid and rem_euclid floor instead, giving -4 with a remainder that is always non-negative: 1. Both are correct division -- they satisfy the same identity a == b * q + r -- they just make different choices about which way to round. If you have ever indexed into a ring buffer with a negative offset, you have met this bug.",
    runs: 5, byteIdentical: true, miri: "clean", miriMatches: true,
    state: "unaffirmed"
  },
  {
    id: "q5", topic: "Strings and UTF-8", difficulty: 3, expect: "compiles",
    hint: "len() is not a character count. And the second string may not contain the character you think it does.",
    source: 'fn main() {\n    let s = "caf\\u{e9}";\n    println!("{} {}", s.len(), s.chars().count());\n\n    let t = "cafe\\u{301}";\n    println!("{} {}", t.len(), t.chars().count());\n\n    println!("{}", s == t);\n}\n',
    answer: "5 4\n6 5\nfalse\n",
    distractors: ["4 4\n5 5\nfalse", "5 4\n6 5\ntrue", "5 5\n6 6\nfalse"],
    explanation: "len() returns BYTES, not characters. Written with the precomposed e-acute (U+00E9) the string is 5 bytes and 4 chars. Written as a plain 'e' followed by a combining acute accent (U+0301) it is 6 bytes and 5 chars. The two render identically on screen and are not equal, because == on str compares bytes -- Rust does no Unicode normalization for you. Byte length, char count, and what a human calls a character are three genuinely different questions, and the third one needs a grapheme-cluster crate.",
    runs: 5, byteIdentical: true, miri: "clean", miriMatches: true,
    state: "unaffirmed"
  },
  {
    id: "q6", topic: "Bindings and shadowing", difficulty: 2, expect: "compiles",
    hint: "There are two different variables called count here. Only one of them survives the loop.",
    source: 'fn main() {\n    let mut count = 0;\n    for i in 0..3 {\n        let count = i * 10;\n        println!("{}", count);\n    }\n    count += 1;\n    println!("final {}", count);\n}\n',
    answer: "0\n10\n20\nfinal 1\n",
    distractors: ["0\n10\n20\nfinal 21", "0\n10\n20\nfinal 31", "0\n0\n0\nfinal 1"],
    explanation: "The `let count` inside the loop body *shadows* the outer count -- it is a brand-new binding that lives only for that iteration, and it is what the println sees. The outer `mut count` is never written to by the loop at all, so when the loop ends it is still 0, and count += 1 makes it 1. Shadowing looks like assignment and is not; this is one of the few places Rust will let a name quietly mean two things in the same function.",
    runs: 5, byteIdentical: true, miri: "clean", miriMatches: true,
    state: "unaffirmed"
  },
  {
    id: "q7", topic: "Sorting", difficulty: 2, expect: "compiles",
    hint: "Two entries share a key. What does Rust's sort promise about their relative order?",
    source: 'fn main() {\n    let mut v = vec![("bob", 2), ("amy", 1), ("cal", 2), ("dan", 1)];\n    v.sort_by_key(|&(_, k)| k);\n    println!("{:?}", v);\n}\n',
    answer: '[("amy", 1), ("dan", 1), ("bob", 2), ("cal", 2)]\n',
    distractors: ['[("dan", 1), ("amy", 1), ("bob", 2), ("cal", 2)]', '[("amy", 1), ("dan", 1), ("cal", 2), ("bob", 2)]', '[("amy", 1), ("bob", 2), ("cal", 2), ("dan", 1)]'],
    explanation: "sort_by_key is a STABLE sort: elements whose keys compare equal keep their original relative order. bob came before amy in the input, but they have different keys so that does not matter; what matters is that amy preceded dan (both key 1) and bob preceded cal (both key 2), and both of those orders survive. sort_unstable_by_key makes no such promise and is usually a little faster -- the standard library gives you the choice deliberately.",
    runs: 5, byteIdentical: true, miri: "clean", miriMatches: true,
    state: "unaffirmed"
  },
  {
    id: "q8", topic: "Borrow checking", difficulty: 3, expect: "fails",
    hint: "Ask what push might need to do to the vector's storage, and what that would mean for anything already pointing into it.",
    source: 'fn main() {\n    let mut v = vec![1, 2, 3];\n    let first = &v[0];\n    v.push(4);\n    println!("{}", first);\n}\n',
    answer: "does not compile",
    errorCodes: ["E0502"],
    stderrHead: "error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable\n --> q8.rs:4:5\n  |\n3 |     let first = &v[0];\n  |                  - immutable borrow occurs here\n4 |     v.push(4);",
    distractors: ["1", "4", "[1, 2, 3, 4]", "1\n4"],
    explanation: "&v[0] takes a shared borrow pointing into the vector's heap buffer. push needs &mut v and may reallocate that buffer to grow it -- which would leave `first` dangling at a freed address. The borrow checker rejects this at compile time with E0502: cannot borrow `v` as mutable because it is also borrowed as immutable. The detail worth arguing about: the borrow is only still alive because `first` is USED after the push. Delete the final println and the same program compiles -- that is non-lexical lifetimes, and it is why this error moves around when you edit seemingly unrelated lines.",
    runs: 0, byteIdentical: null, miri: "n/a (does not compile)", miriMatches: null,
    state: "unaffirmed"
  }
];
