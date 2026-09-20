VALIDATION EVIDENCE. Filed under role 'review' because this install accepts no other note role -- 'validation' is rejected with 'Valid roles: review'. The boot prompt asked for --role validation; flagging rather than inventing one.

Measured on this machine (Apple Silicon, cargo 1.96.1, just 1.57.0, uv 0.12.1, node 22.22.3).

TIMINGS against EVALUATION.md's 60 s budget for test:
  just test  COLD  12.21 s   (after cargo clean and rm -rf pipeline/.venv)
  just test  WARM   0.28 s   (median of 0.77 / 0.16 / 0.28)
  just setup        0.58 s   (network; package caches already warm)
  per suite, warm: room 0.25 s, pipeline 0.16 s, web 0.12 s
Both are far inside 60 s, so there is no contract defect to escalate. A truly cold machine would pay more in setup -- an empty cargo registry and uv cache -- but setup is not the budgeted hook, and CI caches both.

EXIT CODES, every reserved recipe, run directly:
  test 0, test-full 0, canary 0
  verify 1, bank-audit 1, burst 1, a11y 1, smoke 1
Each of the five prints 'just <name>: not built yet -- BUILDPLAN <ticket> delivers it.' to stderr. test and test-full never invoke them, which is how both stay green while the stubs stay red.

HERMETICITY, evidenced rather than asserted: just test passes with exit 0 run fully inside the network-filtered sandbox with an empty uv cache directory. The --offline flags on cargo and uv are the enforcement; that run is the proof. The web suite has nothing to fetch -- no package.json, no node_modules.

THE SHELL-OUT GUARD, proven by negative control: planting pipeline/tests/test_negative_control_tmp.py containing 'import subprocess' made the suite fail with 'something just test imports can start a process ... tests/test_negative_control_tmp.py:1 imports subprocess'; removing it returned the suite to green. The file was temporary and is in no commit.

CANARY SEAM: room/tests/canary.rs drives router() through tower::ServiceExt::oneshot and asserts 404 on an unrouted path. No listener, no port, no route pre-decided for T-04a or T-09.

NOT RUN LOCALLY, and saying so plainly: .github/workflows/ci.yml. A workflow cannot be executed from this worktree, so both jobs are unproven until the PR runs them. Reviewed by reading. Two risks worth naming:
 1. the pinned just checksum -- verified against the release's published SHA256SUMS and an independent local download, both 45b548094283cb9739af8f13273b8cddeee869f5b4ef2bb631b1f311cb566155. If it were wrong, both jobs fail at the install step, loudly and immediately.
 2. astral-sh/setup-uv@v5, and the ubuntu runner shipping Docker. The Docker step asserts rather than assumes, so a runner image that drops it fails visibly instead of silently skipping what T-15a and T-21 will need.