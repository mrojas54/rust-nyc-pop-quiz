scratch: /var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq22-validate-8ft9gdx0
cwd=/var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq22-validate-8ft9gdx0 args=['--bank', 'bank'] exit=0
history: 0 programs recorded; this run would make it 4 (+4: 4 to backfill from the bank, 0 to admit) — dry run, nothing written
cwd=/var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq22-validate-8ft9gdx0 args=['--bank', 'bank', 'v-exact.json', 'v-renamed.json', 'v-near.json', 'v-clear.json'] exit=1
v-exact: rejected — exact duplicate of q3: the source is byte-identical
v-renamed: rejected — normalized duplicate of q3: the same tokens once declared names are renamed and comments, layout and formatting commas are ignored
v-near: no exact or normalized duplicate found; near-duplicate of q3 at similarity 0.94 (threshold 0.60, uncalibrated) — added to the review queue, marked near-duplicate of q3
v-clear: no exact or normalized duplicate found; closest is q8 at similarity 0.32 (threshold 0.60, uncalibrated) — added to the review queue
history: 0 → 6 programs recorded (+6: 4 backfilled from the bank, 2 admitted)
v-near review: {'near_duplicate_of': 'q3'}
cwd=/var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq22-validate-8ft9gdx0 args=['--bank', 'bank', 'v-clear.json'] exit=2
dedupe: v-clear: the bank already holds this id. Ids are never reused, so a candidate needs one of its own.
cwd=/ args=['--dry-run'] exit=0
history: 0 programs recorded; this run would make it 4 (+4: 4 to backfill from the bank, 0 to admit) — dry run, nothing written
Repository bank unchanged: all file hashes match.

Compiler probes:
rustc 1.96.1 (31fca3adb 2026-06-26)
a block-local declaration does not bind a call outside its scope rustc exits= [0, 1] dedupe= near_duplicate 
a binding does not shadow a name in its own initializer rustc exits= [0, 1] dedupe= cleared 
derived Debug observes type and field names rustc exits= [0, 0] dedupe= near_duplicate distinct observed output hashes
doc comments cannot document function parameters rustc exits= [0, 1] dedupe= near_duplicate 
the binary entry point is not an arbitrary function rustc exits= [0, 1] dedupe= cleared 
an empty call cannot contain a comma rustc exits= [0, 1] dedupe= near_duplicate 
an empty array cannot contain a comma rustc exits= [0, 1] dedupe= near_duplicate 
an empty struct cannot contain a comma rustc exits= [0, 1] dedupe= near_duplicate 
an empty enum cannot contain a comma rustc exits= [0, 1] dedupe= near_duplicate 
a repeated trailing comma is not layout rustc exits= [0, 1] dedupe= near_duplicate 
compiler evidence: /var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq22-compiler-8jxj3kxw

Remaining blocker regression:
============================= test session starts ==============================
platform darwin -- Python 3.12.13, pytest-9.1.1, pluggy-1.6.0
rootdir: /tmp
collected 1 item

../../../../../tmp/test_pq22_remaining_scope.py F                        [100%]

=================================== FAILURES ===================================
_______ test_a_function_parameter_does_not_bind_a_separate_function_call _______

    def test_a_function_parameter_does_not_bind_a_separate_function_call():
        a = 'fn f(drop: i32) {} fn main() { drop(1); }'
        b = 'fn f(nope: i32) {} fn main() { nope(1); }'
>       assert check('new', b, record(History(), 'old', a)).kind != 'normalized_duplicate'
E       AssertionError: assert 'normalized_duplicate' != 'normalized_duplicate'
E        +  where 'normalized_duplicate' = Verdict(candidate_id='new', kind='normalized_duplicate', threshold=0.6, duplicate_of='old', closest=None, similarity=None).kind
E        +    where Verdict(candidate_id='new', kind='normalized_duplicate', threshold=0.6, duplicate_of='old', closest=None, similarity=None) = check('new', 'fn f(nope: i32) {} fn main() { nope(1); }', History(exact_hashes={'old': '3a69ea28760bb79a577968458dd41868951ae159600fe7e4d1c574bd7c27b798'}, ast_fingerprints={'o...', '( 1', ') ;', ') {', '1 )', ': i32', '; }', 'fn $', 'fn main', 'i32 )', 'main (', '{ $', '{ }', '} fn']}, version=1))
E        +      where History(exact_hashes={'old': '3a69ea28760bb79a577968458dd41868951ae159600fe7e4d1c574bd7c27b798'}, ast_fingerprints={'o...', '( 1', ') ;', ') {', '1 )', ': i32', '; }', 'fn $', 'fn main', 'i32 )', 'main (', '{ $', '{ }', '} fn']}, version=1) = record(History(exact_hashes={}, ast_fingerprints={}, token_bigrams={}, version=1), 'old', 'fn f(drop: i32) {} fn main() { drop(1); }')
E        +        where History(exact_hashes={}, ast_fingerprints={}, token_bigrams={}, version=1) = History()

/tmp/test_pq22_remaining_scope.py:8: AssertionError
=========================== short test summary info ============================
FAILED ../../../../../tmp/test_pq22_remaining_scope.py::test_a_function_parameter_does_not_bind_a_separate_function_call
============================== 1 failed in 0.05s ===============================
