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
