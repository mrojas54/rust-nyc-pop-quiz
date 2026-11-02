# PQ-66: Dedupe follow-up: the false normalized duplicates PQ-30's exact-head review left open

Follow-up to PQ-30 (PR #51, head 6c589fa). The exact-head review confirmed the pairs below. Each normalizes equal on HEAD (and on main), rustc 1.96.1 --edition 2021 --crate-type bin compiles A (exit 0) but not B (exit 1), so each silently discards a valid question (AC-15, SPEC 7.3). Items 1-3 were reproduced from the Orchestrator seat; item 4 is the reviewer's report alone, so re-probe it first.

1. (Major) A local or parameter reaches into a nested mod in its fn body, where the bare name is the library's. A: fn main() { let drop = 1; mod m { pub fn f() { drop(1); } } m::f(); let _ = drop; } vs B: drop renamed to zzq everywhere (E0425). Parameter variant: fn f(drop: i32) -> i32 { mod m { pub fn g() { drop(1); } } m::g(); drop } fn main() { f(1); } vs zzq. Fix site: scope_outside_modules (dedupe.py ~1061); only item scopes get holes for nested mod bodies today; subtract them from let, parameter and block scopes.
2. (Major) A block-level use shadows an item. A: fn drop(_: i32, _y: i32) {} fn main() { { use std::mem::drop; drop(1); } drop(1, 2); } vs B: fn zzq(_: i32, _y: i32) {} fn main() { { use std::mem::drop; zzq(1); } zzq(1, 2); } (E0061). Conservative fix: a name that sits next to a block-level use keeps its spelling.
3. (Major) _STD_MEMBERS is incomplete. A: fn count_ones(x: u32) -> u32 { x } struct S; impl S { fn count_ones(&self) -> u32 { 0 } } fn main() { let _ = S.count_ones(); count_ones(1); let _ = 3u32.count_ones(); } vs B: count_ones renamed to zzq everywhere (E0599). Documented as incomplete; the fix is a complete list or a rule that does not need one (keep a method name when a program type also declares it).
4. (Minor) The scope is anchored on the first { after the pattern, which is the initializer block. A: fn main() { if let Some(drop) = { drop(1); Some(2) } { } } vs zzq (E0425). Reachable only when the binding is unused in the then-block; the for analogue is caught only because the binding is used.

Also in scope: over-corrections (equal on main, unequal on HEAD, both compile 0/0, so a recall loss that sends the pair to an organizer): static dispatch through a program trait, trait Tr { fn go() -> i32; } struct S; impl Tr for S { fn go() -> i32 { 1 } } fn f<T: Tr>() -> i32 { T::go() } fn main() { f::<S>(); } vs go renamed to run; and a type-alias head, type A = S; A::mk() vs A::build(). Performance NIT: a 4000-line synthetic program takes 3.9 s on HEAD against 1.2 s on main (scope_outside_modules loops every item over every module); a 1000-line program 0.33 s; the largest real bank question 0.5 ms. _impl_owner/_impl_trait and _header_words/_header_keyword duplicate parsing.

Acceptance: each pair added unweakened to pipeline/tests/test_dedupe.py with a twin and a named mutation, probed with rustc (exit codes in the validation note); existing tables stay green; just test under 60 s warm; the docstring and bank/README.md lists updated to remove only what is closed. Conservative rule: where scope is ambiguous at the token level, do not rename. Standard library only. Cleared files: pipeline/src/popquiz/dedupe.py, pipeline/tests/test_dedupe.py, bank/README.md. Warning from PQ-30: this area has gone four review rounds with new Majors at the edge of the absolute criterion; the delegator should state the stopping rule in its plan (a bounded list of known shapes, not 'no false match exists'). Base: main once PR #51 merges.

# Plan (delegator, 2026-10-31)

Base `302e47b`. Rule files: `pipeline/src/popquiz/dedupe.py`, `pipeline/tests/test_dedupe.py`, `bank/README.md` only.

## The closed list (ruling 1: this is the stopping rule)

Done means these shapes and only these. "No false match exists" is not the goal.

1. A local or parameter reaches into a `mod` nested in its function body (let, parameter; the same range-drawing also covers closure parameter, `for`, `if let`, `while let`, match-arm bindings).
2. A block-level `use` shadows an item.
3. A library method name the program also declares (`count_ones`).
4. A condition's binding range anchored on its initializer block (`if let`, `while let`, and the `for` analogue).
5. Over-correction: static dispatch through a program trait (`T::go()`).
6. Over-correction: type-alias head (`type A = S; A::mk()`).

A new shape goes under `not-done` with its pair and exit codes. It is not a blocker.

## Probe table (ruling 2)

`rustc 1.96.1 (31fca3adb 2026-06-26)`, `--edition 2021 --crate-type bin`, compile only. Item 4 probed first. Script: scratchpad `probe.py`. "equal" is `normalized_tokens(A) == normalized_tokens(B)` on this base.

| Pair | A | B | equal |
|---|---|---|---|
| 4 if-let initializer block | 0 | 1 E0425 | True |
| 4 while-let initializer block | 0 | 1 E0425 | True |
| 4 for initializer block | 0 | 1 E0425 | True |
| 4 if-let `unsafe` initializer | 0 | 1 E0425 | True |
| 4 if-let `match` initializer | 0 | 1 E0425 | True |
| 1 let | 0 | 1 E0425 | True |
| 1 parameter | 0 | 1 E0425 | True |
| 1 closure parameter | 0 | 1 E0425 | True |
| 1 for binding | 0 | 1 E0425 | True |
| 1 if-let binding | 0 | 1 E0425 | True |
| 1 match-arm binding | 0 | 1 E0425 | True |
| 2 block use | 0 | 1 E0061 | True |
| 2 block use, group form | 0 | 1 E0061 | False (already kept; not reproduced, test kept since it pins behaviour) |
| 2 block glob use | 0 | 1 E0061 | True |
| 2 use in a fn body | 0 | 1 E0061 | True |
| 3 count_ones with free fn | 0 | 1 E0599 | True |
| 3 count_ones method only | 0 | 1 E0599 | True |
| 3 trait method, local receiver | 0 | 1 E0599 | True |
| Over-correction: trait static dispatch | 0 | 0 | False (want True) |
| Over-correction: type-alias head | 0 | 0 | False (want True) |
| New shape: generic into nested mod | 0 | 1 E0425 | True (outside the list: `not-done`) |

All of the ticket's pairs still reproduce. Nothing is dropped.

## The fixes (current lines on `302e47b`)

**Item 1. `_Collector.scope_outside_modules` (1079), `function` (1140), `let` (1163), `for_loop` (1199), `match_arm` (1215), `closure` (1234).**
Add `_Collector.local(names, lo, hi)`: each name reaches `[lo, hi]` minus the bodies of modules wholly inside the range. Every binding range calls it instead of `scope`. A name used inside the module then falls outside every range, so `_unscoped_names` keeps the spelling everywhere. The hole computation is shared with `scope_outside_modules`, rewritten over one sorted list of module bodies with `bisect` so a local costs only the modules inside its range (ruling 6 permits this because item 1 rewrites that loop). Top-level items still walk every top-level module: not the goal, stays listed. Generics are not changed (new shape, `not-done`).
Tests: `SCOPE_NEVER_THE_SAME` rows for let, parameter, closure, for, if-let, match arm. Twin in `SCOPED_THE_SAME`: a local and a same-spelled local inside a nested module both rename. Mutation M1: `local` delegates to plain `scope`.

**Item 2. `_declared` `use` branch (1342), `_normalize` (2001).**
A `use` whose enclosing brace is neither `None` nor a module body is block-level. Record each leaf of its tree: an identifier not followed by `::` or `as` and not preceded by `as`. For a glob, record every identifier in the enclosing block. A new `_Declared.block_use_names` joins `protected`. `use<..>` precise capture is skipped. An alias is not recorded: it is the program's own name and already scoped as an item, and renaming it consistently is what the program means.
Tests: rows for simple, group, glob, fn-body. Twin: a block-level `use` beside an unrelated program function still renames that function. Mutations: M2a no leaf recording; M2b no glob recording.

**Item 3. New `_unproven_method_calls`, `_normalize` `kept_members` (2006).**
Ruling 4's rule, read literally ("keep a method name when a program type also declares it"), keeps every method called with a dot. That breaks three existing rows: `SCOPED_THE_SAME` "a method called after a dot" (`S.get_n()`), "an item in a program trait's impl" (`S.go()`), and RICH/RICH_RENAMED (`c.bump(*s)`). Ruling 7 forbids editing them. So the plan takes ruling 4's rule with one exception, flagged as a deviation (standing clause 7): a declared member called with a dot keeps its spelling **unless the tokens tie every such call to a program type that declares it**. It needs no list. The tied receivers are exactly:

- `self`, inside an `impl` or `trait` whose owner is a sole program type or trait declaring the name;
- a bare `S` where `S` is a sole program type declaring the name;
- a bare local `c` whose every binding in the program is `let [mut] c = S;` or `let [mut] c = S { .. };` with `S` as above, and `c` is not also an item.

"Sole" means declared exactly once across all items, and not a generic. Anything else keeps the name: `3u32.x()`, `n.x()` where `n = 3u32`, `a.b.x()`, `f().x()`, `let c = S::new(); c.x()`. "Called" means `.name(` or `.name::`. A field read (`p.start`) is outside the rule (see Stays open). `_STD_MEMBERS` is unchanged.
Tests: rows for the three item 3 pairs, plus rows that attack the exception (receiver shadowed by `let c = 3u32` in an inner block; `self` in `impl Tr for u32`, both compile 0/0 but mean different programs; generic `impl<T> Tr for T`). Twins: `self.b()` inside an impl; `let c = S { .. }; c.go()`; a second method in a trait. Mutations: M3a `kept_members` loses the new set; M3b the exception always proves (receiver check returns True); M3c the exception never proves (shows the existing rows that force it).

**Item 4. `_Collector.let` (1187), `for_loop` (1211), new `_body_brace`.**
`_body_brace(s, start)`: find the first depth-0 `{` or block word (`if match loop while for unsafe async const`). If a word comes first, or the `{` stands directly at `start`, directly after a closure's `|`, or after a depth-0 `->`, the first `{` may be the initializer's, so return `None` and the binding gets no block range. Its uses in the real block are then kept (conservative rule). `if let f = |a: i32| a { f(1); }` keeps its range: the `{` follows `a`.
Tests: rows for if-let, while-let, for, `unsafe`, `match`, closure-with-block. Twin: an ordinary `if let Some(x) = y() { x }` and `for` still rename (already in tables; add one with a call initializer). Mutation M4: `_body_brace` returns the first `{`.

**Over-correction: trait static dispatch. `_Collector.generics` (1099), `_tail_reaches` (1931).**
`_generic_bounds` records, per declaration of a generic, its bound when it is exactly `T: Tr` with no `where` after the list; anything else records `None`. `_tail_reaches`, for a generic head: true only if every declaration of the spelling has a recorded bound, each a sole program trait declared as plain `trait Tr {` (no supertraits, no generics), each declaring the name, and the spelling is not also an item. Otherwise false, as today. The tables hold `T::default()`, `I::Item`, `T: Default` beside `struct T`: all stay kept.
Tests: `SCOPED_THE_SAME` row. New `SCOPE_NEVER_THE_SAME` rows: `T: Tr + Iterator` and supertrait `trait Tr: Iterator` calling a name both declare if a compiling pair exists. Mutation M5: generic head always false again (fails the twin); M5b the bound test always true (fails a guard row).

**Over-correction: type alias. `_declared` `type` branch (1331), `_tail_reaches` (1937).**
A non-associated `type A = S;` records alias `A -> S`; any other shape, or a second declaration, records `None`. `_tail_reaches`: for a head with an alias, true only if `A` is declared once and `S` is a sole program type declaring the name.
Tests: twin row; guard row where `type A = u32` beside a `S::count_ones`. Mutations: M6 alias lookup removed (fails the twin), M6b target check removed (fails the guard).

Ruling 3 applies to both over-corrections: if either reopens a listed or tabled false match, it comes out, stays conservative and is listed under `not-done`.

## Bank before and after (ruling 8)

Script `bank_timing.py`, read-only: for every pair of `load_bank` questions, fingerprint equality and bigram similarity at the threshold, plus each fingerprint. Before: 4 questions, no normalized pair, no near pair. A question pair that newly matches is a blocker, reported by mail. A fingerprint that changes is reported too: `sync_with_bank` refreshes stored entries on the next run by design, so `bank/history.json` is not edited.

## Timing (ruling 6)

Same script, on this machine, before: 1000-line 0.054 s, 4000-line 0.246 s, 4000-line worst case (2000 top-level functions beside 2000 modules) 14.724 s. The ticket's 3.9 s used a program I do not have, so these three are the comparison: after must be no more than a third slower on each.

## Stays open (docs, ruling 9)

The docstring and README open list loses items 1 to 4 and the two recall lines only as each is proven closed, and gains: a generic reaching into a nested mod (`fn f<String>() { mod m { .. String::new() .. } }`, 0/1 E0425, equal). A field read beside the library's field stays as a known class if a probe confirms it. The new recall costs are listed: method names on a receiver the tokens cannot tie, names beside a block-level `use`, names in a block with a glob `use`, bindings whose head holds a block. The docs keep "not established" and the bounded-list sentence, and do not claim dedupe is free of false matches. `_impl_owner`/`_impl_trait`, `_header_words`/`_header_keyword` are left alone. The item-by-module walk on top-level items stays listed as a speed cost.

## Deviation to flag in DONE

Ruling 4 narrowed as above so that rulings 4 and 7 can both hold. Mutation M3c shows the rows that force it.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: Sonnet subagent, no false match found against items 1, 2, 3 or the over-corrections.

1. **Major, accepted: item 4 misses labelled and macro initializers.** `if let Some(drop) = 'a: { drop(1); break 'a Some(2) } { }` and `vec!{ drop(1) }.pop()` are 0/1 E0425 and equal on base. Fix differs from the suggestion: rather than adding `:` and `!` to a denylist, `_body_brace` inverts the test. The first `{` is the body only when no block word precedes it and the token before it can end an expression: an identifier, a literal, `)`, `]`, `?`, or `self`/`true`/`false`/`await`. Anything else gives no block range, so the name is kept. Rows added for both.
2. **Minor, changed: group row.** The suggested `{drop, swap}` also passes on base, because a bare name in a group is renamed in A and kept in B. I probed it (0/1, unequal) and keep it as a pinning row. The row that exercises M2a is a nested path in a group, `use std::{mem::drop, mem::swap}` (0/1, equal on base).
3. **Minor, accepted: const named like a type.** "Sole" counts every item declaration, so a `const S` beside `struct S` is not sole. Row added (0/1 E0599).
4. **Minor, no change: narrowing of ruling 4.** Reviewer confirms the three rows justify it.
5. **Minor, no change: the loop rewrite.** The hole computation is shared with item 1, so ruling 6's condition holds. Timing is reported on all three programs.

From the row probes: the field-read probe does not reproduce (unequal on base), so it is not an open shape and is removed from "Stays open". The ambiguity guards for over-correction 5 are A 1 E0034 / B 0. The generic-impl receiver row is dropped: it cannot produce a false match, because the owner must already be a sole program type. "Sole" therefore does not need a generic exclusion.

## Reset 2026-10-31 by agent:delegator-pq66

## Reset 2026-10-31 by agent:delegator-pq66

## Reset 2026-10-31 by agent:delegator-pq66
