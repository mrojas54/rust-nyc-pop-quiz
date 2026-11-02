Validation for PQ-66 at HEAD 6f22525 (base origin/main 2fe8d72). rustc 1.96.1 for probes.

RAN at 6f22525
- just test: rc 0, 27.2 s warm (pipeline 1104 passed, 1 skipped).
- pipeline/tests/test_dedupe.py run directly from pipeline/: 241 passed.
- Probe table, rows.py --final, re-run this session at 6f22525: 43 rows, mismatches 0. Every listed pair is apart, every twin equal, compile only.
- Bank, read-only: 4 questions; normalized [] and near [] before and after.
- Timing: 4000-line program 0.246 s to 0.185 s; worst case 14.7 s to 3.5 s.

RAN at e54d782 (not repeated at 6f22525)
- Named mutations: 42 of 42 killed, 0 survived (log /tmp/claude-501/pq66-mutate5.log).
- dedupe.py, test_dedupe.py and bank/README.md are byte-equal between e54d782 and 6f22525: git diff --quiet rc 0, re-confirmed this session. The rebase changed none of them.

CODE REVIEW
Pass 5, the final Codex pass, on e54d782: PASS, no actionable defects. Pass 5 stands for 6f22525 by the byte-equality above.

just test-full: NOT RE-RUN, by Michelle's ruling
The one local run failed at sandbox-build because Docker was not running: "ERROR: Cannot connect to the Docker daemon ... Is the docker daemon running?" then "error: recipe `sandbox-build` failed with exit code 1". Docker was not started. PR CI's just test-full job covers it.
Ran and passed before that failure, all inside the `test` dependency: test-room (cargo fmt --check, cargo test), test-pipeline (1104 passed, 1 skipped), test-web (303 passed), test-organizer (cargo test, clippy, fmt, build, terminal_repeat).
Did not run locally: sandbox-build, test-sandbox (AC-12 containment), test-verify-full, test-generate-full, test-transport-full, canary --full-only, harness-full, and test-full's lifecycle test_full_ tests.
Log: /var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq66-testfull.log

OPEN PAIR: generic into nested mod, probed this session at 6f22525
A: fn f<String>() { mod m { pub fn g() -> String { String::new() } } m::g(); } fn main() { f::<u8>(); }
B: the same with String renamed to Zzq.
- rustc: A exits 0; B exits 1 with E0425 and E0433. The earlier note listed E0425 alone; the full probe shows both codes.
- Normalized tokens: still equal. No name rule closes this pair.
- Real dedupe run with records taken from those rustc results (A ran, B does not compile): B is near_duplicate of A, so it reaches the organizer and is not discarded. The new check separates it.
- Same run with no records on either: B is normalized_duplicate of A, so B is still discarded silently.
The pair therefore stays listed as open. It is separated only when both programs carry a verifier record of a shape the check can read. This run was a scratch test over a private copy of the bank, using test records built from rustc's results rather than a run of the sandbox verifier.