# PQ-30: Dedupe: close the remaining false normalized duplicates

Follow-up to PQ-22 (BUILDPLAN T-17). PQ-22 round 3 fixed round 2's Critical (declared names are now renamed only inside their scope) and was pushed as ai-c11-cc/dedupe @ 20f5cc9. The final review (PQ-22 artifact "Code review round 3 (FAIL)") found the false normalized duplicates below. Each pair normalizes equal, and rustc 1.96.1 compiles A (exit 0) but not B (exit 1). A normalized duplicate is rejected with no person in the loop, so each one silently discards a valid question (AC-15, SPEC 7.3: no normalization may make two different programs equal).

Scope ranges drawn too wide (same class as round 2):
- C1: a closure's range runs past its body into an if-let / while-let block. `fn main() { if let f = |drop: i32| drop { drop(f); } }` vs nope. Fix site: `_Collector.closure` (the body end is found by the next `,`/`;`).
- C2: a top-level item reaches into nested `mod` bodies. `fn drop(_: i32) {} mod m { pub fn g() { drop(1); } } fn main() { m::g(); }` vs nope. Fix site: `_Collector.item`.
- C3: `macro_rules!` names are renamed at every call regardless of scope. `fn main() { vec![1]; macro_rules! vec { ($($t:tt)*) => { () } } }` vs zz. Fix site: `_normalize` (`rename = name in declared.macros`) and `_unscoped_names` (skips macro calls).
- C4: `_arm_start` walks back into a comma-less previous arm body (`while a == b {..}`, `match s.a {..}`, `for .. in xs {..}`) and binds its names. `fn main(){ let (a,b)=(1,2); let v:Option<i32>=None; match v { None => while a == b { let _f: fn(i32) = drop; break; } Some(_z) => {} } }` vs nope.

New classes:
- C5: a path's tail is renamed when its head is, without resolving the tail. Examples: `I::Item` with a generic `I: Iterator` beside a user `type Item`; `T::default()` beside a user `fn default`; `m::drop` re-exported from std beside a parameter named `drop`. Fix site: the path branch of `_normalize`.
- C6: a lifetime `'a` and an identifier `a` share placeholder text. `fn f<'a>(x: &'a i32) {}` vs `fn f<a>(x: &a i32) {}`. Likely fix: give lifetime placeholders a distinct form (e.g. `'$n`).
- C7: every free `fn` counts as a member, so a `.name()` call to a library method is renamed beside a free `fn name`. `fn count_ones(x: u32) -> u32 { x } fn main(){ let _ = 5u32.count_ones(); count_ones(1); }` vs zzz. Fix site: `_Collector.function` (`members.add` for every fn, not only impl/trait methods).

Acceptance: each pair above is added to `pipeline/tests/test_dedupe.py` unweakened, with a twin where useful, and each pair is probed with rustc (record the exit codes in the validation note). All existing NEVER_THE_SAME, SCOPE_NEVER_THE_SAME and SCOPED_THE_SAME cases and the AC-15 fixtures stay green. `just test` stays green and under 60 s warm. Remove each fixed item from the "not met yet" lists in the `dedupe.py` module docstring and in `bank/README.md`. Conservative rule: where scope is ambiguous at the token level, do not rename. Standard library only. Cleared files are the same as PQ-22's.

Base: branch from ai-c11-cc/dedupe @ 20f5cc9, or from main once PQ-22 merges.

# Plan (delegator-pq30, 2026-10-07)

Base `origin/main` @ 2877149. Worktree `~/rust-nyc-pop-quiz-worktrees/dedupe-false-duplicates`.
Files: `pipeline/src/popquiz/dedupe.py`, `pipeline/tests/test_dedupe.py`, `bank/README.md` (not-met-yet passage only).

## Probes already run (rustc 1.96.1, `--edition 2021 --crate-type bin --crate-name probe`)

| Pair | A exit | B exit | normalized-equal on origin/main |
|---|---|---|---|
| C1 | 0 | 1 (E0425 `nope`) | True |
| C2 | 0 | 1 (E0425) | True |
| C3 | 0 | 1 (cannot find macro `zz`) | True |
| C4 (`while a == b`) | 0 | 1 (E0425) | True |
| C4 (`match s.a`) | 0 | 1 (E0425) | True |
| C4 (`for x in xs`) | 0 | 1 (E0425) | True |
| C5 `I::Item` | 0 | 1 (E0220) | True |
| C5 `T::default()` | 0 | 1 (E0599) | True |
| C5 `m::drop` (`pub use std::mem::drop`) | 0 | 1 (E0425 in module `m`) | True |
| C6 verbatim from ticket | **1 (E0601 no `main`)** | 1 (syntax) | True |
| C6 + `fn main() {}` | 0 | 1 (syntax) | True |
| C7 | 0 | 1 (E0599) | True |

Deviation-with-flag (ruling 5): the ticket's C6 A has no `main`, so it does not compile as a bin.
The test uses the pair with `fn main() {}` appended (A 0 / B 1). Both forms are recorded.

Candidates probed and dismissed: a `let`/parameter named `drop` reaching a nested `fn g() { drop(2) }`
fails on both sides (E0434, A and B), so it is not a false duplicate. A glob re-export
(`fn drop..; mod m { pub use std::mem::*; } m::drop(1)` vs `zz`): A 0 / B 1, normalized equal —
the C5 class; the C5 fix below closes it too.

## Fixes

**C1 — closure in a condition head.** `_Collector.closure`: new `_in_condition_head(s, k)` walks
back from the `|` over whole groups to `;`/`{`/`}`/`,`/opener/`=>`; true on `if`, `while`,
`match` or `in`. There, a closure without `-> T` ends at its body block if the body starts with
`{`, else at the first depth-0 `{` (minus one), and never later than the old `,`/`;` end.
Conservative default: `|x| if x {..}` inside a condition head is cut short → `x` unreached →
kept verbatim (a recall gap, not a merge). Pair: ticket C1 → `SCOPE_NEVER_THE_SAME`.
Twin: `fn main() { if let f = |a: i32| a { f(1); } }` vs `b` → `SCOPED_THE_SAME`.
Mutation: drop the condition-head branch → C1 pair fails.

**C2 — items do not reach into nested `mod` bodies.** `_Collector.__init__` collects every
`mod NAME {` body; `item()` scopes the name over its module/block range minus every nested
`mod` body that does not contain the declaration (several intervals). Pair: ticket C2.
Twin: `fn helper(_: i32) {} mod m { pub fn g() { super::helper(1); } } fn main() { m::g(); helper(2); }`
vs `aid` → same. Mutation: skip the subtraction → C2 pair fails.

**C3 — `macro_rules!` textual scope.** Collector records `macro_scopes[name]` = from the
name token to the end of the enclosing brace group (end of file at top level); the bare-name
scope stays `[k+2, k+2]` so "a macro is not a function" is unchanged. `_unscoped_names`: a
macro call of a declared macro outside every macro scope protects the spelling (was:
skipped). `_normalize`: macro call renamed iff `name in declared.macros and name in renamable`.
Conservative default: a `#[macro_use]` module's macro used after the module is kept (narrower
than Rust). Pair: ticket C3. Twin: `fn main() { macro_rules! m { () => { 1 } } let _ = m!(); }`
vs `n` → same. Mutation: macro-call rename back to `name in declared.macros` → C3 fails.

**C4 — `_arm_start` and comma-less previous arms.** A `{...}` group before the arrow is part of
the pattern only when a path names it *and* the token before that path's head is an arm
boundary: start of stream, `,`, `{`, `}`, `@`, a non-`||` `|`, or a `=>`. Otherwise (`==`,
`.`, `in`, `while`, an ident...) the group ends a previous arm. Conservative default: a
pattern like `&P { x }` stops early → its names unbound → kept. Pairs: C4 `while`, plus
`match s.a` and `for x in xs` variants. Twin: `struct P { x: i32 } fn main() { match (P { x: 1 }) { P { x: 0 } => while false {} P { x } => { let _ = x; } } }`
vs `Q`/`y` → same. Mutation: restore the old `named_by_path` test → C4 pairs fail.

**C5 — path tails resolved only where tokens can tell.** One helper decides whether a tail
`H::name` reaches the program's declaration of `name` (assuming `H` is renamed):
`Self`/`self`/`crate`/`super` head → yes (unchanged); `H` a generic parameter → no;
`H` a declared `mod` → only if `name` is an item declared directly in a body of a module
spelt `H`; any other head → only if `name` is an associated item (impl/trait item or enum
variant). Tail renamed iff head renamed, helper yes, and `name` renamable. Every tail kept
verbatim whose spelling is declared somewhere protects that spelling everywhere (in
`_unscoped_names`, iterated to a fixpoint because a protected head un-renames its tails) —
otherwise the declaration loses its link to the verbatim use and two programs merge.
Pairs: `I::Item`, `T::default()`, `m::drop` (+ glob variant). Twins: `fn f<T: Default>() -> T { T::default() }`
vs `U`; `mod m { pub fn g() {} } fn main() { m::g(); }` vs `n`/`h`; `enum E { A } ... E::A` vs `F`/`B`.
Mutations: (a) generic head → yes: `I::Item`/`T::default` fail; (b) module check → "any item":
glob variant fails; (c) drop the tail protection: `m::drop` fails.

**C6 — lifetime placeholders.** `_normalize` emits `'` + placeholder for a lifetime (`'$1`,
unnumbered `'$`). Pair: C6 + main. Twin: `'a` vs `'b` → same. Mutation: plain placeholder → C6 fails.

**C7 — only impl/trait fns are members.** `_Collector.function` adds to `members` only when
`item()` found it inside an `impl`/`trait` (the same test that gives it the `[d, d]` scope).
Safe because a `.name` use can only reach fields and associated fns. Pair: ticket C7.
Twins: `struct S; impl S { fn get_n(&self) -> u8 { 1 } } fn main() { S.get_n(); }` vs `fetch_n`;
free `fn helper` vs `fn aid` both still renamed. Mutation: always add → C7 fails.

**Docs.** Docstring and README: remove the closed items; say what is now conservative
(recall gaps: conditions' closures, `#[macro_use]`, `&P{..}` arm patterns, verbatim path tails).
Do not claim the aim is met — the `_STD_MEMBERS` gap (e.g. `type Item` inside `impl Iterator`)
remains and is already named.

## Size estimate
dedupe.py ~+170/−40; test_dedupe.py ~+120; README ~±15. Total ~350 lines. Escalation line (3×): ~1000.

## Contract tension
None beyond the C6 probe deviation. SPEC 7.3's absolute rule is held as a requirement; the
docstring keeps saying the approximation has not established it.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: general-purpose / sonnet, 2026-10-07. Eight findings.

1. **Major — `crate`/`self`/`super` heads unchecked (accepted).** Those heads now count as
   reaching `name` only when `name` is an item declared at top level or directly in a `mod`
   body. Pair added: `use std::mem::drop; fn f(drop: i32) -> i32 { drop } fn main() { f(1); crate::drop(1); }`
   vs `nope` (to be probed). Twin: `fn helper() {} mod m { pub fn g() { super::helper(); } }` stays the same.
2. **Major — `Self::name` (accepted).** A `Self` head reaches `name` only when `name` is an
   associated item (impl/trait item or variant). Pair added: the reviewer's `Self::default()` pair
   (to be probed). Twin: `impl S { fn make() -> S { S } fn g() -> S { Self::make() } }` stays the same.
3. **Major — other heads accept any associated name (partly accepted).** Not tying a tail to its
   head's own impl is name resolution (ruling 3). The common names are already on `_STD_MEMBERS`
   (`max`, `default` is not — but an associated `fn default` beside a library `default` on the
   same head is the existing "declared member named like the library's" gap). Recorded as a
   remaining gap in the docstring and README; no new mechanism.
4. **Minor — C4 `|` (accepted).** `|` is no longer an arm boundary for the struct-pattern test;
   `A | P { x } =>` stops early (names unbound → kept): a recall gap, conservative.
5. **Minor — C2 recall (accepted).** `mod tests { use super::*; helper(1) }` now protects `helper`;
   named among the recall gaps in docstring and README.
6. **Minor — generic heads (accepted).** A head whose spelling is declared as a generic parameter
   *anywhere* in the program is treated as a generic: tail not renamed, spelling protected.
7. **Minor — C6 labels share the `'a` key.** No action: errs toward not merging.
8. **Minor — C1 `in`.** No action; kept for `for` heads, harmless.

Existing tables and history fixtures: reviewer found no breakage; to be confirmed by the suite.

## Reset 2026-10-07 by agent:delegator-pq30

## Reset 2026-10-08 by agent:delegator-pq30
