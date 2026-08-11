# Dimension 4 — Feasibility of the core technical claim

**Scope:** does "LLM-generated Rust output question, gated by pinned rustc +
Miri, ready in five minutes, resistant to gaming" actually hold together as
engineering? This reads the PRD's own requirements
(`/Users/michellerojas/rust-nyc-pop-quiz/PRD.md`) against Miri's documented
capabilities, published performance data, and the literature on LLM question
generation and code near-duplicate detection.

---

## Summary verdict

**The core claim is plausible but unproven, and the PRD already knows it —
its own status is "Draft, blocked on feasibility evidence" pending a
100-room Generation Latency Spike that has not run.** Three independent
findings from this review:

1. **Miri's raw execution cost is not the bottleneck.** For 35-line,
   std-only, no-I/O programs, Miri's interpreter overhead is real but small
   in absolute terms; the PRD's own stage budgets (60s per Miri model
   configuration) look generous relative to published slowdown data. The
   bottleneck is **budget math**, not Miri speed: an undefined-behavior
   (UB) targeted slot pays for two full Miri configurations
   (Stacked Borrows + Tree Borrows) plus model-generation latency, consuming
   an estimated ~187 seconds (62% of the five-minute deadline) on its
   **first** attempt at stated p95 costs — leaving no room for a second
   attempt under the PRD's own "don't start an attempt that can't fit" rule.
   Since 20% of slots target UB independently, **48.8% of default
   three-question rooms will contain at least one UB-targeted slot**
   (1 − 0.8³), and that is precisely the path with the least attempt slack
   and the least published evidence on LLM first-attempt yield.
2. **A close read of the PRD finds a self-contradiction that is a bigger
   gaming vector than the one the PRD already flags.** The PRD requires
   `unsafe` to appear **only** in UB-targeted questions (PRD.md:341-342),
   while separately claiming "no ... token ... may reveal additional
   information about the correct option or target category" (PRD.md:372-375).
   Because safe Rust cannot cause undefined behavior, this rule is a
   near-deterministic tell: **any question containing `unsafe` is UB with
   near-certainty, and any question without it is provably not UB.** A
   repeat attendee needs zero Rust reasoning to exploit this — just scan for
   the keyword. This is worse than the compiler-error prior the PRD's own
   closing assumption calls out.
3. **The design is appropriately humble about semantic-uniqueness matching**
   ("a measured risk reduction, not proof of program inequivalence") **but
   is not equally humble about Miri.** Miri's own README states plainly that
   a clean run only proves the absence of UB *on the executions actually
   sampled* — and the PRD samples only 4 seeds (`-Zmiri-many-seeds=0..4`),
   narrower than Miri's own shipped default range of 0..64 for that exact
   flag, on a topic list that explicitly includes concurrency.

Nothing found here says "this cannot work." Everything found here says "this
needs the two pending spikes plus the fixes below before the five-minute
target and the anti-gaming claim can be trusted."

---

## Miri capabilities and limits

**What Miri detects** (per the Miri README): out-of-bounds accesses and
use-after-free; use of uninitialized data; violated intrinsic preconditions
(e.g., reaching `unreachable_unchecked`, misusing `copy_nonoverlapping`);
insufficiently aligned accesses; violated basic type invariants (an invalid
`bool` or enum discriminant); data races and *some* weak-memory effects; and,
as **experimental** checks, Stacked Borrows / Tree Borrows aliasing
violations. It also flags unreachable allocations at program end (memory
leaks). ([rust-lang/miri README](https://github.com/rust-lang/miri/blob/master/README.md))

**What it explicitly does not or cannot do:**

- "Miri does not catch every violation of the Rust specification" because
  **no formal specification of Rust's semantics exists** — the Reference's
  own UB list is explicitly non-exhaustive: "There is no formal model of
  Rust's semantics for what is and is not allowed in unsafe code... We also
  reserve the right to make some of the behavior in that list defined in the
  future." ([Rust Reference — Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html))
- Miri is a platform-independent interpreter with **no access to most
  platform-specific APIs or FFI**, most syscalls, and networking. It cannot
  run most `std` internals that shell out to the OS. This is irrelevant to
  the PRD's std-only, no-I/O programs, but it means Miri coverage does not
  generalize past that constraint.
- **Miri only checks the execution(s) it actually runs.** Direct quote from
  the README: *"Miri can just tell you if a particular way of interacting
  with your code causes any undefined behavior in a particular execution...
  when Miri does not find UB, then you may just have to test more inputs or
  more possible non-deterministic choices."* The maintainer is blunter in a
  2025 retrospective: Miri *"fundamentally cannot ensure that your code is
  sound"* the way a static/formal proof would; it can only rule out UB along
  paths exercised. ([What's "new" in Miri — ralfj.de, 2025-12-22](https://www.ralfj.de/blog/2025/12/22/miri.html))
- **Non-determinism is sampled, not enumerated.** `-Zmiri-seed=<num>` fixes
  one RNG seed controlling allocation base addresses, scheduling/preemption
  choices, `compare_exchange_weak` failure, and weak-memory store-buffer
  behavior. `-Zmiri-many-seeds=[from]..to` reruns the program across a seed
  range to sample different resolutions of that non-determinism; **Miri's
  own default range for that flag is `0..64`**, and the docs still caveat
  that "this will still by far not explore all possible executions" and that
  going further requires model checking or deductive verification — testing
  in general cannot enumerate all schedules. ([rust-lang/miri README](https://github.com/rust-lang/miri/blob/master/README.md))
- **Neither aliasing model is finalized policy.** Stacked Borrows is Miri's
  default; Tree Borrows is opt-in via `-Zmiri-tree-borrows`. An evaluation
  across the 30,000 most-downloaded crates found **Tree Borrows rejects 54%
  fewer test cases than Stacked Borrows** — i.e., materially more permissive
  — and Rust has not committed to either as the final language model. Both
  are explicitly labeled experimental / works-in-progress; Tree Borrows has
  a known gap where **combining it with int-to-pointer casts "will miss a
  lot of undefined behavior."** ([Tree Borrows PR discussion; DeepWiki summary](https://github.com/rust-lang/miri/pull/3766))

**Does the PRD over-claim here?** Partially. To its credit, the PRD does
*not* claim Miri proves soundness — the UB section correctly treats Miri's
verdict as authoritative only for the sampled runs, and it smartly requires
**both** Stacked Borrows and Tree Borrows to agree, rejecting model-dependent
examples (PRD.md:404-407) — a stronger check than either model alone, and a
direct, sound mitigation for the "which model is right" uncertainty above.
Where it does not carry the same rigor: the seed range is `0..4`, four times
narrower than Miri's own out-of-the-box `many-seeds` default of 64, on a
topic list ("Async and concurrency," PRD.md:219) that explicitly invites the
exact class of non-determinism seeds exist to sample. And nowhere does the
PRD attach the "not proof of no UB" caveat to its verification claims the
way it explicitly attaches "measured risk reduction, not proof" to semantic
uniqueness (PRD.md:429-431). The asymmetry is the tell: the design authors
clearly understand the concept (they applied it to embeddings) and did not
apply it to Miri.

---

## Performance budget analysis vs the five-minute target

**Published slowdown data.** Miri is commonly cited at **10–100× slower
than native execution**, with reported outliers far higher for
allocation-heavy or intrinsic-heavy code (one blogged example: ~7,000× on a
1.6ms native workload). ([Microsoft RustTraining — Miri, Valgrind, and Sanitizers](https://microsoft.github.io/RustTraining/engineering-book/ch05-miri-valgrind-and-sanitizers-verifying-u.html); [Data-driven performance optimization with Rust and Miri — Medium](https://medium.com/source-and-buggy/data-driven-performance-optimization-with-rust-and-miri-70cb6dde0d35))
Separately, Miri pays a **per-invocation startup/recompilation cost**
independent of program runtime — the `cargo-nextest` Miri integration notes
Miri recompiles the crate under test on every invocation, and one cited
optimization cut a `regex`-crate test-listing invocation from 87s to 17s.
([cargo-nextest — Miri interpreter](https://nexte.st/docs/integrations/miri/); [Miri is slow on what should be a straightforward program — miri#4616](https://github.com/rust-lang/miri/issues/4616))
That figure is for a real crate with a nontrivial dependency graph, not a
35-line PRD-sized program, but it establishes that Miri's fixed per-process
overhead is measured in seconds, not milliseconds, even before counting the
interpreted execution itself.

**Applying this to the PRD's own numbers.** The PRD's Generation Latency
Spike section (PRD.md:815-832) sets p95 stage budgets: 5s queue/dispatch,
30s per model request, 5s Sandbox start, 2s static checks, 10s compilation,
**60s per Miri configuration**, 15s for uniqueness/quality/callback. Summing
by target category, using one generation+verification attempt:

| Target category | Stages paid | Estimated attempt cost (p95 budgets) | % of 5-min deadline |
|---|---|---|---|
| Compiler error (0.01% of slots) | queue+model+sandbox+static+compile-fail+callback (no Miri) | ~67s | 22% |
| Deterministic output (79.99%) | + one Miri config (Stacked Borrows) + 3 native runs | ~133–140s | ~46% |
| Undefined behavior (20%) | + **two** Miri configs (Stacked Borrows and Tree Borrows) | ~187s | **62%** |

Given the PRD's own rule — *"Do not begin another candidate attempt when its
remaining stage budget cannot fit before the deadline"* (PRD.md:327-329) —
a UB-targeted slot whose first attempt runs anywhere near its stated p95
cost has **roughly 113 seconds left, not enough for a second full attempt at
p95 cost (~187s)**. In effect, at the stated budgets, UB-targeted slots get
one realistic try; everything downstream (yield, retries, judge rejection)
has to work on the first pass or the room fails the deadline. Since each
slot samples its target independently at 20% UB, **P(a 3-slot room has ≥1
UB-targeted slot) = 1 − 0.8³ ≈ 48.8%** — call it half of all default rooms
riding on the tightest, least-tested budget line.

The compiler-error path is fast (no Miri needed at all, since a failed
compile already answers the question) but so rare — expected ≈1-in-3,333
slots, ≈1-in-1,111 three-slot rooms — that the 100-room Generation Latency
Spike is very unlikely to exercise it even once. Its real-world timing and
LLM yield will effectively be discovered in production, not validated
pre-launch.

**Bottom line on performance:** Miri's raw interpreted-execution cost for a
35-line, std-only program should comfortably fit the 60-120s Miri budget —
this is not where I'd expect the five-minute target to fail. The genuine
risk is compounding: model-call latency (30s budget, paid every attempt) ×
retries needed to hit a verifier-confirmed, judge-approved, correctly-typed
candidate, especially on the UB path where a wrong-type candidate (Miri says
"no UB" when UB was the target, or vice versa) burns a full ~187s attempt
and likely the room's remaining budget. **Unknown — measure by: running the
still-pending Generation Latency Spike and recording, per target category,
the attempt count needed to reach acceptance, not just wall-clock time.**
The PRD's spike design (PRD.md:815-832) already asks for this; it just
hasn't run yet.

---

## Determinism hazards

The PRD requires three fresh native runs to produce byte-identical stdout
(PRD.md:382-388) and explicitly bans several nondeterminism sources
(PRD.md:389-393, PRD.md:894-895). Assessed hazard-by-hazard:

| Hazard | Root cause | Does the PRD catch it? | Residual risk |
|---|---|---|---|
| `HashMap`/`HashSet` iteration order | `RandomState` reseeds SipHash-1-3 per process from OS entropy; iteration order is intentionally unstable across runs ([internals.rust-lang.org — RandomState](https://internals.rust-lang.org/t/randomstate-new-non-random/14037)) | Explicitly banned by rule (PRD.md:389-390) *and* would likely diverge across 3 native runs anyway for maps with ≥3-4 entries | Low — belt-and-suspenders; residual risk is a small map that happens to produce the same order 3 times by chance and evades the rule-based reject too |
| Pointer/address formatting (`{:p}`) | Addresses are allocator- and ASLR-dependent; the stdlib documents printed addresses as unstable, non-unique identifiers | Explicitly banned (PRD.md:390) | Low, if the ban is enforced lexically/statically as well as behaviorally |
| Float `Debug`/`Display` formatting | Deterministic for a fixed value and fixed Rust version, but the format itself changed in 1.58 (scientific notation above/below a threshold); **NaN bit patterns and sign are not guaranteed stable across arithmetic** ([f32 docs](https://doc.rust-lang.org/std/primitive.f32.html)) | Not explicitly named in the ban list | Medium for NaN-producing programs specifically — `is_sign_positive`/`is_sign_negative` on a NaN "can produce surprising results," and this is invisible to a same-machine, same-build 3-run check since it may reproduce consistently locally while remaining formally unspecified |
| Thread interleaving / scheduling | Real OS-level scheduling nondeterminism; the topic list explicitly includes "Async and concurrency" (PRD.md:219) | Banned by rule (PRD.md:391-392: "thread scheduling... unless the program canonicalizes"); Miri's `-Zmiri-many-seeds` samples schedules, but only 4 seeds | **High** — rare-window races can easily reproduce identically across both 3 native runs and 4 Miri seeds and still be latently nondeterministic; this is exactly the class Miri's authors say needs many more seeds or a model checker |
| Time / system entropy | Wall clock, `SystemTime`, OS RNG | Prohibited by policy — "Prohibit... time, randomness" (PRD.md:339-340) | Low — removed by category ban rather than detection, which is the right call |
| `size_of` across targets | `usize`/pointer width is fixed per target triple ([Rust Reference — Type Layout](https://doc.rust-lang.org/reference/type-layout.html)) | Verifier manifest pins one target triple (PRD.md:333-336) | Low — a single pinned target makes this fully deterministic by construction |
| Integer overflow behavior (panic vs. wrap) | **Not** the same across compilation profiles: plain `rustc` defaults overflow-checks *off* (silent two's-complement wrap); `cargo`'s dev profile turns them *on* (`-C debug-assertions=on -C overflow-checks=on`) by default, causing a panic instead | **Gap.** The pinned "complete verifier manifest" (PRD.md:333-336) lists `rustc -Vv`, target triple, `rustfmt` version, Miri build, Image digest, seed range, and `MIRIFLAGS` — it does **not** list `-C overflow-checks` / `-C debug-assertions` / optimization level for the compile-and-native-run step | **Medium** — whichever invocation path the verifier harness actually uses (bare `rustc` vs. a generated `Cargo.toml` with a dev profile) silently decides whether overflowing arithmetic panics (rejected via the nonempty-stderr rule) or wraps and prints a value (accepted as "the" answer). Undocumented and unpinned, this is a real reproducibility gap even though each individual choice is internally deterministic |

**Is 3 native runs enough?** It is a reasonable low-cost tripwire for the
common, high-frequency hazards (hash-order, obvious races) but it is
explicitly a *sampling* check, not a proof, and the PRD says so itself:
"This is a bounded nondeterminism detector, not a proof" (PRD.md:384-385).
That's the right framing. The hazard it is weakest against is exactly the
one the topic list invites: low-probability thread-interleaving bugs, where
3-of-3 and 4-of-4-seeds agreement is unfortunately easy to hit by chance.

---

## The UB-as-a-choice problem

Two distinct issues, one mechanical, one epistemic.

**1. Mechanical: the design's own `unsafe`-gating rule creates a
category-revealing token.** PRD.md:341-342: *"Permit `unsafe` only for
questions whose intended answer is undefined behavior."* Safe Rust, by the
language's foundational guarantee, cannot cause undefined behavior — *"if
all you do is write Safe Rust, you will never have to worry about
type-safety or memory-safety"* ([The Rustonomicon — Meet Safe and Unsafe](https://doc.rust-lang.org/nightly/nomicon/meet-safe-and-unsafe.html)).
Combine the two: **every accepted UB-targeted question will contain an
`unsafe` block (there is essentially no other way to reach UB in a 35-line
std-only program), and no accepted non-UB question ever will.** The
displayed source is shown to participants before they answer (Core
Experience step 8, PRD.md:88; PRD.md:121). This makes `unsafe`'s presence a
**perfect, zero-Rust-knowledge tell**: scan for the keyword, answer "exhibits
undefined behavior" if present, exclude it if absent. That directly
contradicts the PRD's own claim two pages later that "no position, markup,
wording variant, distractor style, receipt field, or timing behavior may
reveal additional information about the correct option or target category"
(PRD.md:372-375) — the source code itself, which participants are required
to read, is exactly such a leak. This is a genuine design defect, not a
theoretical one, and it is more exploitable than the compiler-error prior
the PRD flags, because it requires no memorized statistic — just visual
pattern-matching on one keyword.

**2. Epistemic: UB has no defined output, but the quiz still needs
plausible-looking distractors for it.** The Reference and Nomicon are
explicit that once UB occurs, the compiler is free to assume it never
happens and may miscompile around that assumption — there is no "real"
answer to appeal to. The PRD handles this correctly in one respect: it bars
inferring UB from what a native run happens to print ("Normal program
execution must not be used to infer undefined behavior," PRD.md:408), so
the pipeline never presents "what it printed on our machine" as a candidate
choice. But the *distractors themselves* are still required to be "plausible
output a reader who misses the trap could accept" (PRD.md:370-371) — which
means the generation pipeline must synthesize numbers that look like they
could be the answer to a program that, by construction, has none. This
creates a live adjudication risk at an in-person meetup: a participant with
a laptop can compile and run the exact pinned toolchain themselves, get a
concrete number on that specific rustc build, and reasonably object that the
"official" answer (UB) contradicts an empirically reproducible result they
just watched happen. The quiz's ground truth is a *meta-fact about the
language specification*, not an *empirical fact about program behavior* —
that gap is pedagogically defensible (it's the whole point of a UB
question) but socially friction-prone in a room of Rust experts, and the PRD
doesn't address how a host should adjudicate that objection live.

**Is a 20% UB target sane?** Pedagogically, yes in principle — recognizing
"this looks fine but isn't" is a valuable, distinctly Rust-flavored skill for
this audience. Mechanically, given finding 1 above, **20% of every quiz is
currently a near-freebie for anyone who has attended once and learned to
scan for `unsafe`** — which inverts the intended difficulty and directly
undermines the stated Problem statement (repeat attendees "recognize
questions and answers," PRD.md:23-24). Fixing the mechanical leak (e.g.,
allowing `unsafe` to also appear, unused for UB, in some non-UB questions as
camouflage) would need to be reconciled with the "verified answer type must
match assignment" rule and re-evaluated for its own false-signal risk before
20% is defensible as-is.

---

## LLM yield expectations

No published source measures the specific task this PRD needs — an LLM
*deliberately constructing* a short, correct, single-defined-output Rust
program, or a short program with intentional, confirmable UB, against a
rustc+Miri gate. The closest available evidence points the same direction on
two fronts:

- **General LLM-generated MCQ quality needs heavy filtering.** Across
  education-research literature, common defects in LLM-authored
  multiple-choice questions include hallucinated content, answer hints
  leaking into the stem, duplicated or ambiguous options, easy/trivial
  distractors, and inconsistent formatting — "human verification and
  validation is indispensable." One study on LLM-generated retrieval-practice
  questions reports strong downstream learning benefit (89% vs. 73% average
  quiz accuracy in practiced vs. unpracticed weeks) but explicitly cautions
  that "the quality of LLM-generated questions can vary" and that
  instructors "must still manually verify and revise" output before release.
  ([Enhancing Student Learning with LLM-Generated Retrieval Practice Questions, arXiv:2507.05629](https://arxiv.org/abs/2507.05629))
  This PRD substitutes a compiler/Miri/judge pipeline for human review, which
  is a stronger gate than most education literature assumes — but it also
  means rejected candidates burn wall-clock budget rather than a human's
  editing time, which is exactly the scarce resource under a five-minute
  deadline.
- **LLMs are measurably unreliable specifically on unsafe-Rust/UB
  reasoning.** Reported patterns: unsafe blocks in LLM-generated Rust are
  "3–5× more frequent than in human-authored Rust," and "most are
  unnecessary and introduce undefined behavior paths" — i.e., LLMs tend to
  reach for `unsafe` accidentally rather than deliberately and correctly.
  ([Rust LLM Generated Code: Security Risk Guide — Medium](https://medium.com/@krun_dev/rust-llm-generated-code-security-risk-guide-0487c1120591))
  A research framework purpose-built to make LLM-touched unsafe Rust
  Miri-clean — RustBrain, using a "fast thinking / slow thinking" generate-
  then-verify-then-refine loop — reports **94.3% pass rate and 80.4%
  execution rate on a Miri-based benchmark**, but that is for *repairing*
  existing unsafe code to eliminate UB, the opposite task from this PRD's
  need to *construct* a minimal program with confirmed, judge-approved UB in
  one or a few generation attempts. ([Unlocking a New Rust Programming Experience: Fast and Slow Thinking with LLMs to Conquer Undefined Behaviors, arXiv:2503.02335](https://arxiv.org/abs/2503.02335))
  Constructing correct-and-pedagogically-clean UB on the first attempt is
  plausibly *harder* than repairing UB out of code, since it requires
  reasoning forward about a subtle violation rather than reacting to a
  diagnostic.

**Net read:** expect first-attempt acceptance rates for deterministic-output
candidates to be moderate-to-good (compilation and a Miri clean run under
Stacked Borrows are comparatively easy bars if the LLM avoids the banned
nondeterminism sources — which is itself an LLM discipline problem, since
`HashMap` iteration is an extremely common "looks fine" pattern for an LLM
to reach for). Expect first-attempt acceptance for UB-targeted and
compiler-error-targeted candidates to be materially lower, because they
require the LLM to hit a narrow target deliberately rather than merely avoid
banned patterns. **Unknown — measure by:** running the Generation Latency
Spike's 100 rooms and reporting acceptance rate *per target category*, not
only overall room-readiness rate; the PRD's spike plan doesn't currently ask
for that breakdown explicitly (PRD.md:823-824 lists stages measured but not
per-category yield).

---

## Semantic uniqueness assessment

The PRD's pipeline (PRD.md:410-431) is a hybrid: exact hash + versioned AST
fingerprint (alpha-renamed locals, comments stripped, but literals/operators/
types/control-flow preserved) for exact-and-structural duplicates, plus
embedding cosine similarity over a semantic descriptor (concept, trap
mechanism, outcome category, diagnostic family, control/data-flow features)
with top-K=20 retrieval at a 0.82 threshold feeding a separate semantic
judge for near-duplicates.

**Does cosine similarity on code embeddings actually separate "renamed
variables" from "genuinely different"?** Partially, and with known failure
modes in both directions. Embedding-based clone detectors are reported to
work well as a *retrieval* mechanism (nearest-neighbor search scales far
better than pairwise comparison across a growing corpus — directly relevant
here since the history only grows) but suffer from **opacity**: "embeddings
are opaque vectors with no intuitive meaning, making it difficult... to
understand why two fragments match." Automated approaches, more broadly,
find **syntactically-divergent-but-semantically-identical clones "very
challenging"** to detect reliably. ([A parallel deep learning-based code clone detection model — ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S074373152300117X); [Nearest-neighbor, BERT-based, scalable clone detection — Ahmed et al. 2024](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3355))
Traditional token/AST fingerprinting tools (MOSS's Winnowing k-gram hashing,
JPlag's Greedy String Tiling) are more interpretable and, in at least one
comparison, an AST-vectorized (DECKARD-style) approach modestly outperformed
JPlag and substantially outperformed MOSS on plagiarism recall — suggesting
**AST-normalized fingerprinting is not a strictly inferior alternative to
embeddings**; it is complementary, catching structural equivalence that
embeddings can miss, while embeddings catch surface-different-but-
conceptually-same cases that pure fingerprinting misses. The PRD already
runs both (AST fingerprint for exact/structural, embedding for semantic) —
that combination is the right shape given the literature, not an arbitrary
choice.

**Is top-K=20 / threshold=0.82 grounded or arbitrary?** Arbitrary as stated,
but not unreasonably so, and the PRD is honest about it. General embedding
near-duplicate literature commonly cites **~0.85 as a "high-precision"
near-duplicate cutoff**, with domain-specific calibration studies finding
optimal values anywhere from **0.75 to 0.92** depending on the embedding
model and what counts as a "duplicate" for the task. ([Zilliz — How do I use embeddings for duplicate detection?](https://zilliz.com/ai-faq/how-do-i-use-embeddings-for-duplicate-detection)) 0.82
sits squarely inside that observed range, so it's a defensible starting
guess, not a random number — but it has **no Rust-code-specific or
task-specific calibration behind it** in the PRD as written. Credit where
due: the PRD does not claim otherwise. It explicitly requires maintaining
"curated equivalent and distinct fixture pairs" and publishing false-accept/
false-reject rates *with every algorithm version* (PRD.md:429-431), and the
Success Metrics section sets a concrete bar — reject ≥95% of known
superficial variants while rejecting ≤10% of intentionally distinct
questions (PRD.md:785-787). That is the correct discipline for an admittedly
arbitrary starting threshold. **Unknown — measure by:** running that fixture
suite before launch, not deferring it to "measured with every algorithm
version" after the fact; a first threshold with zero fixture-suite results
behind it is still just a guess in the meantime.

---

## The answer-prior gaming analysis

The PRD's own closing assumption (PRD.md:974-975) concedes: *"The 0.01%
compiler-error target is an intentional, learnable answer prior and weakens
the original repeat-attendee anti-gaming objective."* Here is the quantified
version of exactly how much it weakens it, plus the bigger leak found above.

**Setup.** Five choices per question. Before reading any source code, the
*category* priors are: deterministic output ≈79.99% (split ~26.663% each
across three output-shaped choices), undefined behavior 20%, compiler error
0.01% (PRD.md:346-353). Final on-screen *position* is uniformly shuffled, so
position itself leaks nothing (PRD.md:353) — the exploitable signal is
purely the semantic *label*, which is always legible: "does not compile" and
"exhibits undefined behavior" are always displayed as literal, unambiguous
text, distinct from the three output-shaped choices.

**Strategy 1 — the prior the PRD already flags.** A player who has simply
learned the published weight vector, without reading a single line of Rust,
can always identify and exclude the "does not compile" choice (true prior
0.01%) and pick uniformly among the three output-shaped choices:

> EV(exclude-compiler-error, guess output) = 0.7999 / 3 ≈ **26.66%**
> vs. EV(uniform-random-among-5) = **20%**
> → a **33% relative improvement in expected value with zero Rust
> knowledge**, purely from memorizing one published number.

**Strategy 2 — the leak this review found, which compounds it.** Add the
`unsafe`-keyword tell from the UB section above. A player's decision
procedure becomes: *scan the source for `unsafe`; if present, answer
"exhibits undefined behavior" (correct with near-certainty, since the
permission rule makes `unsafe` near-exclusive to accepted UB candidates); if
absent, exclude both "does not compile" (0.01%) and "exhibits undefined
behavior" (now excluded with high confidence, not just low prior) and reason
only among the three output choices using ordinary Rust code-reading
skill.* This doesn't just shave off a rare distractor — it **converts a
fifth of the quiz into a near-freebie** and turns the remaining 80% into a
cleaner three-way reasoning problem than five-way, materially inflating the
score of anyone who has ever noticed the pattern. This is strictly stronger
than Strategy 1 and costs the player nothing but reading comprehension
they'd have anyway.

**Verdict:** yes, a fixed, self-consistent answer-distribution prior is
learnable and gameable in expectation, exactly as the PRD concedes — but the
PRD's own text only quantifies (implicitly) the compiler-error piece. The
`unsafe`-gating rule is a second, larger, structurally-guaranteed leak that
the "no token may reveal category" claim (PRD.md:372-375) directly
contradicts, and it is not acknowledged anywhere in the PRD's assumptions
section the way the compiler-error prior is.

---

## Sandbox requirements

This is the most solidly covered dimension of the whole design, with two
verifiable gaps.

**What must be blocked, and does the PRD block it:**

- **Network** — blocked (`block_network=True`, PRD.md:643-646). Confirmed
  achievable: Modal Sandboxes are explicitly marketed for "executing
  untrusted user or agent code," including "code generated by a language
  model," with network egress as a documented control point. ([Modal — Sandbox guide](https://modal.com/docs/guide/sandbox))
- **Filesystem / secrets / persistent volumes** — "no secrets, no mounted
  Volumes" (PRD.md:643-644); ephemeral 2 GiB writable disk only.
- **Build-time arbitrary code execution (build.rs, proc macros)** — this is
  the classic Rust supply-chain attack surface: build scripts and procedural
  macros run with the full permissions of the compiling process and can
  exfiltrate secrets, hit the network, or write files, "by design" in
  Cargo's model. ([Rust and Cargo Supply Chain Security — systemshardening.com](https://www.systemshardening.com/articles/cicd/rust-cargo-supply-chain-security/); [Procedural Macros — The Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html))
  The PRD **structurally avoids this whole class**, not just mitigates it:
  "Use the Rust standard library only... Prohibit... external dependencies"
  (PRD.md:337-340). With no `Cargo.toml` dependency graph, there is no
  build.rs and no third-party proc macro to ever invoke. This is a stronger
  guarantee than sandboxing alone would give.
- **Compile-time resource exhaustion** (macro recursion, const-eval loops,
  monomorphization blowup) — bounded by the 35-line/4KiB source cap plus a
  10s compilation timeout inside a CPU/memory/disk-limited Sandbox
  (PRD.md:338, 647-650).
- **Runtime resource exhaustion** (infinite loops, huge output, fork bombs)
  — bounded by the 2-second/4KiB-stdout native-run cap (PRD.md:388) and the
  overall 180-second Sandbox lifetime (PRD.md:648), inside a Sandbox with no
  subprocess/network capability to fork-bomb outward even if it tried.
- **Running the resulting binary** — same Sandbox as compilation, always
  terminated after verification (PRD.md:645-646); worst case of a hostile
  `unsafe` block is a crashed sandboxed process, not host compromise,
  *conditional on* Modal's isolation actually holding.

**Is `rustc` alone safe on untrusted input?** No — and the PRD does not rely
on it alone. `rustc`'s own frontend assumes a benign author writing a real
program, not an adversary; the actual safety property here comes entirely
from Sandbox-level OS/process isolation plus the no-external-dependencies
rule, not from any property of `rustc` itself. That's the correct
architecture.

**Gap found:** Modal's public Sandbox documentation states resource-limit
and network-blocking *capabilities* exist and are designed for untrusted
code, but **does not disclose the underlying isolation mechanism** (VM vs.
gVisor-style userspace kernel vs. container) in the pages fetched for this
review. The PRD's safety argument is only as strong as that undisclosed
mechanism. **Unknown — measure/verify by:** pulling Modal's specific
isolation-technology documentation or asking Modal directly, and treating
"Sandbox isolation is sufficient for untrusted LLM-authored `unsafe` Rust"
as an explicit assumption to validate, not a given.

---

## Where the design over-claims

1. **"No token... may reveal additional information about the correct
   option or target category" (PRD.md:372-375) is contradicted by the
   design's own `unsafe`-only-for-UB rule (PRD.md:341-342).** This is the
   single most concrete, fixable over-claim found in this review — see "The
   UB-as-a-choice problem" and "The answer-prior gaming analysis" above.
2. **The "complete verifier manifest" (PRD.md:333-336) is not actually
   complete.** It pins `rustc -Vv`, target triple, `rustfmt` version, Miri
   build, Image digest, Miri seed range, and `MIRIFLAGS`, but omits
   compilation flags that change program *behavior*, not just verifier
   identity — specifically `-C overflow-checks` / `-C debug-assertions` /
   optimization level, which decide whether integer overflow panics or
   silently wraps. See the Determinism table above.
3. **Miri's verdict is treated with more implied certainty than the PRD
   applies to its own semantic-uniqueness system**, despite both being
   fundamentally sampling-based, non-exhaustive checks. The PRD explicitly
   calls semantic uniqueness "a measured risk reduction, not proof" but
   attaches no equivalent caveat to the Miri-based deterministic-output or
   undefined-behavior verdicts, even though Miri's own documentation
   supplies almost the identical sentence for free ("Miri can just tell you
   if a particular way of interacting with your code causes any undefined
   behavior in a particular execution").
4. **`-Zmiri-many-seeds=0..4` reads as more rigorous than it is**, given
   that (a) Miri's own shipped default for that exact flag is `0..64`, and
   (b) the quiz's topic list explicitly includes concurrency, the class of
   nondeterminism seed-sampling exists to catch. The PRD does not explain
   why 4 was chosen over Miri's own suggested default, and nothing in the
   PRD flags this as a known narrowing (contrast with the explicit
   "three [native runs]... is the initial latency/coverage tradeoff and may
   only change through a versioned verifier decision" framing given to the
   native-run count, PRD.md:384-385 — the same honesty is not extended to
   the seed count).
5. **The five-minute target and its component stage budgets are asserted in
   the PRD body as normative requirements** (PRD.md:780, 825-828) **before
   the Generation Latency Spike that is supposed to validate them has run.**
   To the PRD's credit this is not hidden — the document's own Status line
   calls it "Draft - blocked on feasibility evidence," and the spike section
   is written as a gate, not a formality. This is flagged here not as a
   hidden over-claim but as the reason the summary verdict above is
   "plausible, not proven."

---

## Open questions

- **Per-category acceptance yield.** What fraction of first-attempt
  candidates pass verification, split by target category (deterministic /
  UB / compiler-error)? Unknown — measure by instrumenting the pending
  Generation Latency Spike to report this breakdown, not just aggregate
  room-readiness.
- **Real UB-path timing under load.** The 187-second UB-path estimate above
  uses stated p95 stage budgets, not measured data. Unknown — measure by
  the same spike, specifically isolating UB-targeted slots.
- **Compiler-error path in practice.** At a true 0.01% per-slot rate, the
  100-room spike will almost certainly produce zero real compiler-error
  attempts. Its actual latency and yield remain unvalidated pre-launch.
  Unknown — measure by a targeted, artificially-boosted-probability test
  batch specifically for this category, separate from the production
  distribution.
- **Overflow-check / debug-assertion pinning.** Does the actual verifier
  harness invoke bare `rustc` or a generated `Cargo.toml` with cargo's dev
  profile? This changes whether overflow panics or wraps. Unknown from the
  PRD text — needs to be pinned explicitly and added to the verifier
  manifest.
- **Modal Sandbox isolation mechanism.** VM-based or container/gVisor-based?
  Not disclosed in the docs pages reviewed. Unknown — verify directly against
  Modal's architecture documentation or support channel before treating
  Sandbox isolation as a hard boundary for hostile `unsafe` code.
- **Fixture-suite results for the 0.82 / top-K=20 semantic-uniqueness
  configuration.** The PRD requires these to be published "with every
  algorithm version" but does not show them for version one. Unknown until
  that suite is actually run.
- **The `unsafe`-tell fix.** Does the PRD intend to address the finding in
  this review (allow `unsafe` as camouflage in some non-UB questions, or
  otherwise break the near-1:1 correlation), and if so, how does that
  interact with the "verified type must match assignment" rule? Not
  addressed in the current text — flagged here as a design decision still
  needed, not merely a bug to patch.

---

## Sources

- [rust-lang/miri README](https://github.com/rust-lang/miri/blob/master/README.md) — UB categories detected, isolation flags, `-Zmiri-seed`/`-Zmiri-many-seeds` semantics and default range, explicit "not proof of soundness" language.
- [What's "new" in Miri (and also, there's a Miri paper!) — ralfj.de, 2025-12-22](https://www.ralfj.de/blog/2025/12/22/miri.html) — maintainer's own framing of Miri's soundness limits, POPL 2026 paper, recent capability growth.
- [Rust Reference — Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html) — canonical (non-exhaustive) UB list, explicit "no formal model" disclaimer.
- [The Rustonomicon — Meet Safe and Unsafe](https://doc.rust-lang.org/nightly/nomicon/meet-safe-and-unsafe.html) — the safe-Rust-cannot-cause-UB guarantee underlying the `unsafe`-tell finding.
- [Tree Borrows PR #3766 discussion / DeepWiki summary](https://github.com/rust-lang/miri/pull/3766) — Stacked vs. Tree Borrows permissiveness (54% fewer rejections), int2ptr gap, experimental status.
- [Microsoft RustTraining — Miri, Valgrind, and Sanitizers](https://microsoft.github.io/RustTraining/engineering-book/ch05-miri-valgrind-and-sanitizers-verifying-u.html) and [Data-driven performance optimization with Rust and Miri — Medium](https://medium.com/source-and-buggy/data-driven-performance-optimization-with-rust-and-miri-70cb6dde0d35) — 10–100× (up to ~7,000× outlier) slowdown figures.
- [cargo-nextest — Miri interpreter integration](https://nexte.st/docs/integrations/miri/) and [miri#4616](https://github.com/rust-lang/miri/issues/4616) — per-invocation recompilation/startup overhead.
- [internals.rust-lang.org — RandomState not random](https://internals.rust-lang.org/t/randomstate-new-non-random/14037) — `HashMap`/`HashSet` iteration-order nondeterminism mechanism.
- [f32 primitive docs](https://doc.rust-lang.org/std/primitive.f32.html) and [Pointer fmt trait docs](https://doc.rust-lang.org/std/fmt/trait.Pointer.html) — float/NaN and address-printing nondeterminism.
- [Rust Reference — Type Layout](https://doc.rust-lang.org/reference/type-layout.html) — `usize`/`size_of` target-triple dependence.
- [Rust and Cargo Supply Chain Security — systemshardening.com](https://www.systemshardening.com/articles/cicd/rust-cargo-supply-chain-security/) and [Procedural Macros — The Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html) — build.rs/proc-macro arbitrary-code-execution risk.
- [Modal — Sandbox guide](https://modal.com/docs/guide/sandbox) — untrusted-code positioning, timeout/resource configurability, isolation mechanism not disclosed.
- [Enhancing Student Learning with LLM-Generated Retrieval Practice Questions, arXiv:2507.05629](https://arxiv.org/abs/2507.05629) — LLM-MCQ quality variance, need for human/automated verification, 89% vs. 73% learning-outcome data.
- [Unlocking a New Rust Programming Experience: Fast and Slow Thinking with LLMs to Conquer Undefined Behaviors, arXiv:2503.02335](https://arxiv.org/abs/2503.02335) — RustBrain, 94.3% pass rate / 80.4% execution rate on a Miri benchmark for UB *repair* (not construction).
- [Rust LLM Generated Code: Security Risk Guide — Medium](https://medium.com/@krun_dev/rust-llm-generated-code-security-risk-guide-0487c1120591) — LLM unsafe-block over-use and accidental-UB pattern.
- [A parallel deep learning-based code clone detection model — ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S074373152300117X) and [Nearest-neighbor, BERT-based, scalable clone detection — Ahmed et al. 2024](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3355) — embedding-based clone detection strengths/opacity weaknesses.
- [Zilliz — How do I use embeddings for duplicate detection?](https://zilliz.com/ai-faq/how-do-i-use-embeddings-for-duplicate-detection) — cosine-similarity threshold calibration ranges (0.75–0.92 observed, ~0.85 common high-precision cutoff).
- `/Users/michellerojas/rust-nyc-pop-quiz/PRD.md` — the design under review (all `PRD.md:<line>` citations above refer to this file as read on 2026-08-11).
