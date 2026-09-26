Round 3 validation, HEAD 20f5cc9 (checks at 27b93fd, re-run gate at 20f5cc9).

just test: green. 407 pipeline tests (2.18 s), web 103 pass 0 fail, room ok; 2.76 s wall warm. mypy --strict --follow-imports=silent dedupe.py clean. Ruff: dedupe.py and test_dedupe.py clean; test_migration.py has one pre-existing import-order finding (present at HEAD 285b22d, out of scope, untouched).

Scratch-bank CLI exercise (scratch copy of bank/, python -m popquiz.dedupe):
--dry-run exit=0: history: 0 programs recorded; this run would make it 4 (+4: 4 to backfill from the bank, 0 to admit) — dry run, nothing written
batch exit=1:
 v-exact: rejected — exact duplicate of q3: the source is byte-identical
 v-renamed: rejected — normalized duplicate of q3 (the same tokens once declared names are renamed …)
 v-near: no exact or normalized duplicate found; near-duplicate of q3 at similarity 0.94 (threshold 0.60, uncalibrated) — added to the review queue, marked near-duplicate of q3
 v-clear: no exact or normalized duplicate found; closest is q8 at similarity 0.23 — added to the review queue
 history: 0 → 6 programs recorded (+6: 4 backfilled from the bank, 2 admitted); v-near review: {'near_duplicate_of': 'q3'}
round-2 blocker pair end to end:
 v-scope-a (fn f(drop: i32) {} … drop(1)) exit=0: no exact or normalized duplicate found; closest q4 0.15 — review queue; history 6 → 7
 v-scope-b (nope variant) exit=0: no exact or normalized duplicate found; near-duplicate of v-scope-a at 0.64 — review queue, marked near-duplicate of v-scope-a; history 7 → 8
resubmitting v-clear exit=2: the bank already holds this id.
Repository bank unchanged: all file hashes match.

Compiler probes — rustc 1.96.1 (31fca3adb 2026-06-26), --edition 2021, compile/no-compile only (nothing run, no output asserted). Exit codes A / B, B's first error, dedupe verdict of B against A:
 function parameter vs separate call 0/1 E0425 near_duplicate
 closure parameter vs call outside 0/1 E0425 near_duplicate
 closure as argument ends with call 0/1 E0425 near_duplicate
 match arm binding past its arm 0/1 E0425 cleared
 comma-less arm ends at next arm 0/1 E0425 near_duplicate
 if-let past its block 0/1 E0425 near_duplicate
 if-let vs statements after 0/1 E0425 near_duplicate
 while-let past its loop 0/1 E0425 near_duplicate
 for pattern past its loop 0/1 E0425 cleared
 parameter pattern past its function 0/1 E0425 near_duplicate
 generic parameter past its item 0/1 E0433 near_duplicate
 method is not a free function 0/1 E0425 near_duplicate
 block-local fn 0/1 E0425 near_duplicate
 fn in a module 0/1 E0425 near_duplicate
 field is not a free function 0/1 E0425 near_duplicate
 macro is not a function 0/1 E0423 near_duplicate
 library-field shorthand 0/1 E0560 near_duplicate
 library field beside param 0/1 E0560 near_duplicate
 library-field struct pattern 0/1 E0026 near_duplicate
All 19: A compiles, B does not, never a normalized duplicate. Round-2's ten controls remain green in the suite.