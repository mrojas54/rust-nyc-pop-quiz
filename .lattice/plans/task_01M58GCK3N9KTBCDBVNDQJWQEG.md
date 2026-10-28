# PQ-79: The club migration's backups land in bank/backups/, which git does not ignore

Deferred from PQ-76 (PR #95, merged at fe092af; the delegator's deviation 3). `just migrate-club --write` (pipeline/src/popquiz/schedule.py, migrate-club; default backup_dir is <bank>/backups/club-rename-<UTC time>/, around line 700) copies each record it changes into bank/backups/ before replacing it. .gitignore on main has no rule for bank/backups/, so a real migration leaves an untracked folder that `git add bank` would sweep into a commit; docs/RUNBOOK.md 'Releasing the club rename' step 3 only says to leave it out of the commit. Decide: add bank/backups/ to .gitignore, or move the default backup_dir outside the repo (and update the RUNBOOK and the --backup-dir help to match). Re-find line numbers first. Acceptance: after a --write run that changes a record, git status shows no bank/backups/ entry (or the default lands outside the repo); the migrate-club tests stay green under just test; no change to the bank's records, the alias tables, or any visible copy.

## Plan

Client ruling 2026-10-28: ignore it in git. Fast-track, built by the orchestrator seat.

1. RED: in `pipeline/tests/test_schedule.py`, a test that runs `migrate-club --write` with the default backup dir, maps the backup it made onto the repository's `bank/`, and asserts `git check-ignore` matches it.
2. GREEN: `/bank/backups/` in `.gitignore`.
3. DOCS: `docs/RUNBOOK.md` *Releasing the club rename* step 3: the backup folder is ignored by git, so `git add bank` leaves it out.
4. `just test`, PR, exact-head CI; merge on the client's word.
