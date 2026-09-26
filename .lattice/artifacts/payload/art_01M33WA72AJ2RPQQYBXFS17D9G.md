FAIL — review round 2 of 2, capped. Reviewed starting HEAD ad22b701c36215ac546cdb1b991b04d7af60f63b; fixes and reproduction verified at final HEAD 285b22dec060df6336aa284daf2e036024e14a08.

Independent reviewer returned FAIL with five Major findings. Delegator treats the remaining compile/noncompile false rejection as Critical, consistent with round one's severity and the user's stop rule.

Resolved findings:
- Entry-point identity (starting dedupe.py:694): main is preserved.
- Invalid empty lists (starting dedupe.py:955-966): commas without a preceding entry are preserved. Calls, arrays, structs, enums and repeated separators have controls.
- Derived output exposes type/field names (starting dedupe.py:1009-1011): derives conservatively preserve declared names.
- Documentation on function parameters (starting dedupe.py:998-1000): comments outside recognized item positions are retained.
- Let-scope and initializer examples of global spelling collisions are guarded, but the scope finding is NOT fully resolved.

OPEN CRITICAL — pipeline/src/popquiz/dedupe.py:750, :1294, :1371, :1431. Function parameters are still collected into a global spelling map. The let guard does not resolve parameter scopes. These programs normalize equal:
A: fn f(drop: i32) {} fn main() { drop(1); }
B: fn f(nope: i32) {} fn main() { nope(1); }
Local rustc 1.96.1, edition 2021: A exit 0; B exit 1, E0425. check() returns normalized_duplicate for B against A. The attached standalone never-equal regression fails (1 failed); it is not skipped or weakened. Closure/pattern scopes also remain unresolved. No claim that all false matches are eliminated.

Checks: repository just test passes, 380 pipeline tests, 3.44 s warm; 94 dedupe tests after documentation edits pass in 0.28 s. Ruff checks dedupe.py and test_dedupe.py clean; mypy --strict --follow-imports=silent checks dedupe.py clean (scope: this module, not imported bank.py). Ten added regression controls; nine observed RED before fixes, repeated-comma control already passed. Ten compiler probes confirm compile/noncompile or actually observed distinct derived-output hashes. Scratch-bank CLI exercise passes, repository bank file hashes unchanged.

AC-14/16/17/18 have direct tests and scratch-bank evidence. AC-15 has passing positive fixtures but remains blocked on this false rejection. AC-17 review-surface rendering is T-18's responsibility, not proven here. test_migration.py:90 remains untouched per the latest explicit user scope, despite older ticket clearance; narrow its empty-history assertion before the first real dedupe run.

Disposition: needs_human under the user's two-round cap. No PR, no merge, no force-push. Six signed local commits preserve RED, GREEN, VERIFY and DOCS. Automatic approval review rejected publication; push awaits direct user confirmation. Original remote branch remains ad22b70 until confirmed otherwise. Existing deviations remain, including token-level rather than AST normalization, incomplete standard-library member keep-list, uncalibrated threshold, bank-backed review queue, in-process AC-17 tests, and run-report-only growth reference. New conservative misses include ambiguous let names, derive-bearing source names, and docs before unrecognized item forms.
