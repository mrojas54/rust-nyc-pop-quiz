# PQ-49 checkpoint
Plan posted (lattice plan PQ-49). Baseline: just bank-audit exit 0, passed, on 622e0fd. just setup done.
Next: status in_progress, rebase origin/main, add the step to .github/workflows/ci.yml after `just test`, plant in $TMPDIR copy, just test.
Files: .github/workflows/ci.yml only. Leave .claude/* uncommitted.

After implement: commit a682f54 (signed); just test green, cold 248s. Plants run in scratchpad copies: real exit 0, answer-last exit 0 (WARN), four-options exit 1, warn-too-long exit 0.
Next: Codex review of origin/main...HEAD, validate, push, PR, status review, DONE.

PR #117 open at b0264ea84dca4b051208bcc859031f086b584617, CI green, ticket in review. DONE posted. Waiting on Michelle to merge and to decide repo visibility. Nothing further to do here.
Ruling 2026-11-02: Michelle keeps the repo public. Flag lowered. PR #117 still waits on her merge.
