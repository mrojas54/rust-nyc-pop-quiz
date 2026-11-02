# PQ-66 checkpoint: PR open, waiting

Latest state, overrides everything below: PR #113 https://github.com/mrojas54/rust-nyc-pop-quiz/pull/113, HEAD 14fa1ff pushed (docs-only, by Michelle's ruling relayed by the Orchestrator 2026-11-02T03:27Z; doc gap closed), base origin/main 2fe8d72. CI just test and just test-full SUCCESS. PR MERGED, not by this panel. DOCS result and a correction sent by comment and mail. No timers armed. Nothing further from this panel.

---
Earlier: fix the third-pass pair, then a fourth Codex pass

CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-66-dedupe-followup. Branch ai-c11-cc/pq-66-dedupe-followup, HEAD c7cac93afa29e1676ad15a07c4f36bb94078e483, base origin/main 302e47b (at last fetch), NOT pushed. Lattice in_progress. Flag lowered. Plan: /Users/michellerojas/rust-nyc-pop-quiz/.lattice/plans/task_01M4D08QSJ40GTES3B2V4X9QAR.md.

## Rulings (Michelle, 2026-10-31)
- B at about 21:44Z: keep both over-corrections, add module-scope checks, one further Codex pass above the cap. Done; that pass is spent.
- Fourth pass at 2026-10-31T21:54Z: fix the third-pass pair and run one fourth Codex pass. Yes to a handoff and fresh session.

## Next, in order
Steps 1-4 DONE: fix committed as 9db9878 (let_types keeps each initializer's position; local_types requires sole_here at each `let`; a bare receiver still checks at the call). New rows: initializer outside block, bare receiver outside block, twin. Mutations: 31 of 31 killed (M7f added; M7d survived until the bare-receiver row, then killed). pytest test_dedupe.py rc 0.
5. Fourth Codex pass on 9db9878: prompt at /private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz-worktrees-pq-66-dedupe-followup/7407572d-21bc-4d87-b284-e0ddc4186dc8/scratchpad/pq66-codex-prompt4.txt, output $TMPDIR/pq66-codex-review-4.txt. No further pass without Michelle; a new Major stops and goes to her. 6. Validate. 7. PR arc.

## Ruling after pass 4 (Michelle, 2026-10-31, reply "4")
Option 4: when normalized tokens are equal, compare the verifier's record (compile result, error code, exit code; not stdout). Same: normalized_duplicate. Different: near_duplicate to the organizer. No record: today's behaviour. u32 pair is its first test; add a mutation. Keep branch's fixes; name-rule cleanup is a later ticket. Identity false match caught by the check is a NOTE, not a blocker. One final Codex pass, on this check only. New organizer-visible wording halts.

Done: commit e54d782 (signed, not pushed). test_dedupe.py 241 passed. Mutations 42 of 42 killed (M8a-k added). Fifth and final Codex pass on e54d782 only: prompt pq66-codex-prompt5.txt in session scratchpad 7407572d-21bc-4d87-b284-e0ddc4186dc8, output /var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq66-codex-review-5.txt. Then Validate, then PR arc. A new Major goes to Michelle.

Pass 5 on e54d782: PASS, no defects; no VERDICT first line, its words are "PASS for commit e54d7822..., no actionable defects found". Output pq66-codex-review-5.txt.
Rebased on origin/main 2fe8d72 (#112): HEAD 6f225254a7fd83237c6e47bc19aa6fb7262933d5, all 11 signed; dedupe.py, test_dedupe.py, bank/README.md byte-equal to e54d782. Not pushed.
At 6f22525: just test rc 0, 27.2 s (pipeline 1104 passed, 1 skipped); test_dedupe.py 241 passed; rows.py --final 42 rows, mismatches 0; bank 4 questions, normalized [] near [] before and after; timing 4000-line 0.246→0.185 s, worst case 14.7→3.5 s. Mutations 42/42 ran at e54d782. just test-full running: $TMPDIR/pq66-testfull.log. Then attach Validation, push, PR, wait for CI, review status, DONE.

## Superseded: pass 4 on 9db9878 found a new Major (2026-10-31)
HEAD 9db9878e898f9a7453e61780af3901553aab4b0d, not pushed. Nothing uncommitted under pipeline/ or bank/. No timers armed. Output: /var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq66-codex-review-4.txt. Codex gave a P1 and no VERDICT line; treat it as FAIL. Its finding on pass 3's pair: fixed, holds.
Shape 3: `_impl_owner` cuts `std::primitive::u32` to `u32`, so `self` in that impl ties to the program's `struct u32`.
A: `struct u32; impl u32 { fn count_ones(&self) -> u8 { 0 } } trait Tr { fn go(&self); } impl Tr for std::primitive::u32 { fn go(&self) { let _ = self.count_ones(); } } fn main() { Tr::go(&3u32); }`
B: the same with count_ones renamed to zzq. Reproduced by me: rustc 1.96.1 0 / 1 E0599; normalized equal on HEAD and on origin/main, so not a regression.
Question to Michelle: (A, recommended) fix at the tie check, row, twin, mutation, no fifth pass, PR states the fix after the last review was not reviewed by Codex; (B) no fix, list under not-done, go to Validate; (C) fix and a fifth pass. Resume: apply her answer, then Validate, then PR arc.

## The pair fixed in 9db9878
Pass 3 on c7cac93: FAIL, one Major in shape 3. `_call_is_tied` checks the type's scope at the call, not at the `let` that supplied it.
A: `fn main() { let mut c = None; { struct None { a: u8 } impl None { fn is_some_and(&self, _: fn(u8) -> bool) -> bool { true } } c = Some(1u8); let _ = c.is_some_and(|_| true); } }`
B: the same with is_some_and renamed to zzq. rustc 1.96.1: 0 / 1 E0599. Normalized equal on base and on HEAD, so it is on main already, not a regression from this branch.
Ruled: fix and a fourth pass.

## Passes
1 on 633eab8 FAIL (2 Majors, fixed a96c55a). 2 on a96c55a FAIL (1 Major, fixed c7cac93 with _in_scope). 3 on c7cac93 FAIL (above). Output: /var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq66-codex-review-3.txt (and -2, and pq66-codex-review.txt).

## Evidence at c7cac93
221 dedupe tests pass per Codex; pytest test_dedupe.py rc 0 on my run. 30 of 30 named mutations killed. All probed pairs apart, twins equal.
A stray commit failure and a review launched on the wrong HEAD were stopped before any verdict; that run is not counted.

## Not run on a final HEAD
`just test` (last at a96c55a: rc 0, 60 s, at the budget; re-time), direct test_dedupe.py count, rows.py --final, bank before/after, timing, just test-full.

## Resume
Apply the ruling, then Validate, then section 3's PR arc in .claude/boot-prompt.md. Commit from the worktree root: the shell may sit in pipeline/. Push and PR prompt the client. Do not merge. The PR and DONE state the final Codex verdict and the sha it reviewed.

## For DONE
Deviations: ruling 4 narrowed; item 1 and item 4 extended; item 4 test inverted; a program with a glob `use`, or a binding or `use` sharing the type's spelling, or a type out of scope, ties nothing. Third and fourth passes above the cap, by Michelle's rulings.
not-done: generic into nested mod (0/1 E0425). Left alone per ruling 6: _impl_owner/_impl_trait, _header_words/_header_keyword, the item-by-module walk. Not reproduced: bare name in a use group; field read beside a library field; type head outside its module (unequal on base and HEAD).
Scripts: scratchpads of sessions 0ebdf1fb (sib.py, mutate.py, rows3.py, p3.py) and fb9a3019 (rows.py, bank_timing.py, compare.py, probe.py, dedupe_base.py) under /private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz-worktrees-pq-66-dedupe-followup/.
