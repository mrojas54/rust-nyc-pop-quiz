scanned 362 text files; 11 files hit; 63 lines

## BUILDPLAN.md (1)
  56: Message Batches API** — the pipeline is offline, so batch's 50% discount is

## PRD.md (8)
  346: participant-observable state. The version-one weights are 79.99% deterministic
  347: output, 0.01% compiler error (`0.0001` probability), and 20% undefined
  351: 26.663% prior, **does not compile** has a 0.01% prior, and **exhibits
  352: undefined behavior** has a 20% prior. Secure shuffling still makes each final
  353: screen position 20% likely to contain the correct choice.
  773: between 0.007% and 0.013%, undefined-behavior targets remain within 0.25
  774: percentage points of 20%, and deterministic-output targets make up the
  974: - The 0.01% compiler-error target is an intentional, learnable answer prior and

## docs/audit/2026-10-guardrail-audit.md (1)
  483: `web/home/`. M11a (*About 40% of the answers are does not compile…* in `README.md`)

## mvp/2026-08-12/pop-quiz-2026-08-12-q3.html (1)
  140: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat

## mvp/2026-08-12/pop-quiz-2026-08-12.html (8)
  163: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat
  200: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat
  220: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat
  244: <div class="explain">Rust&#x27;s / and % truncate toward zero, so -7 / 2 is -3, and the remainder takes the sign of the dividend: -1. div_euclid and rem_euclid 
  245: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat
  277: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat
  313: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat
  333: <details class="receipt"><summary>Verification receipt</summary><ul><li>rustc 1.96.1 (31fca3adb 2026-06-26) · edition 2021 · compiled cleanly</li><li>5 of 5 nat

## pipeline/tests/test_audit.py (12)
  474: "About 20% of answers are *does not compile*.",
  475: "Roughly one in five questions does not compile.",
  476: "3/10 of the answers are a panic.",
  477: "Half the answers are UB, so guess it.",
  478: "Only 2 of 8 printed output worth reading.",
  479: "The answer is undefined behaviour 40 percent of the time.",
  487: text = "Last season:\n20% of the time\nthe answer did not compile.\n"
  494: "20% of the room\n\n\nsaw the output",  # the kind word is three lines away
  496: "20% of pubs close early.",  # "ub" inside a word is not UB
  508: exempt = "with perfect memory of every previous answer is still guessing 1 in 5."
  588: cycle, and a fifth of the answers are *does not compile*, spread across its
  791: exactly the questions 'pick does_not_compile' finds: two of four is a WARN in

## sequence/research/00-synthesis.md (2)
  42: | The published **0.01% compiler-error prior** | "Never pick *does not compile*" — a **33% relative EV lift over random guessing, with zero code reading** | One
  75: | UB-path budget compounding: ~187s = 62% of the deadline on first attempt; ~48.8% of 3-question rooms contain at least one UB slot, with one attempt of slack |

## sequence/research/01-problem-and-people.md (4)
  80: Assumptions section states outright: *"The 0.01% compiler-error target is an
  83: distribution is a fixed, versioned product constant (79.99% deterministic
  84: output / 0.01% compiler error / 20% undefined behavior), any attendee who
  324: contradiction (0.01% compiler-error prior weakening the anti-gaming

## sequence/research/03-economics.md (1)
  68: | A7 | Semantic-judge trigger rate | 15% of candidates | Only candidates with cosine similarity ≥ 0.82 against the top-20 historical bucket go to the semantic j

## sequence/research/04-generation-verification.md (21)
  29: Since 20% of slots target UB independently, **48.8% of default
  89: paths exercised. ([What's "new" in Miri — ralfj.de, 2025-12-22](https://www.ralfj.de/blog/2025/12/22/miri.html))
  150: | Compiler error (0.01% of slots) | queue+model+sandbox+static+compile-fail+callback (no Miri) | ~67s | 22% |
  151: | Deterministic output (79.99%) | + one Miri config (Stacked Borrows) + 3 native runs | ~133–140s | ~46% |
  152: | Undefined behavior (20%) | + **two** Miri configs (Stacked Borrows and Tree Borrows) | ~187s | **62%** |
  161: slot samples its target independently at 20% UB, **P(a 3-slot room has ≥1
  162: UB-targeted slot) = 1 − 0.8³ ≈ 48.8%** — call it half of all default rooms
  261: **Is a 20% UB target sane?** Pedagogically, yes in principle — recognizing
  389: The PRD's own closing assumption (PRD.md:974-975) concedes: *"The 0.01%
  395: *category* priors are: deterministic output ≈79.99% (split ~26.663% each
  396: across three output-shaped choices), undefined behavior 20%, compiler error
  397: 0.01% (PRD.md:346-353). Final on-screen *position* is uniformly shuffled, so
  406: 0.01%) and pick uniformly among the three output-shaped choices:
  408: > EV(exclude-compiler-error, guess output) = 0.7999 / 3 ≈ **26.66%**
  409: > vs. EV(uniform-random-among-5) = **20%**
  418: absent, exclude both "does not compile" (0.01%) and "exhibits undefined
  547: - **Compiler-error path in practice.** At a true 0.01% per-slot rate, the
  578: - [What's "new" in Miri (and also, there's a Miri paper!) — ralfj.de, 2025-12-22](https://www.ralfj.de/blog/2025/12/22/miri.html) — maintainer's own framing of 
  581: - [Tree Borrows PR #3766 discussion / DeepWiki summary](https://github.com/rust-lang/miri/pull/3766) — Stacked vs. Tree Borrows permissiveness (54% fewer reject
  589: - [Enhancing Student Learning with LLM-Generated Retrieval Practice Questions, arXiv:2507.05629](https://arxiv.org/abs/2507.05629) — LLM-MCQ quality variance, n
  590: - [Unlocking a New Rust Programming Experience: Fast and Slow Thinking with LLMs to Conquer Undefined Behaviors, arXiv:2503.02335](https://arxiv.org/abs/2503.02

## sequence/run-state.md (4)
  160: | 04 | Generation & verification | Feasible but unproven. UB path ≈187s = 62% of the 5-min deadline; ~48.8% of rooms hit it. **`unsafe` in source is a near-perf
  182: | Live vs. batch generation | **Resolved — batch.** Client call, T-4. | Freshness is a supply problem, not a runtime problem. Decoupling dissolves 4 of 5 top ri
  347: | **Fix** | Slot is drawn **uniformly from the date and nothing else**. `slot_for_day()` is pure and takes no history; `mvp/answer-history.json` demoted from in
  1862: **D-25 judge round 2: 8/10, PASS, no blocking finding.** Applied: the Miri line