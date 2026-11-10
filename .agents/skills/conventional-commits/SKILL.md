---
name: conventional-commits
description: Write Conventional Commit messages for this repository, including its RED, GREEN, VERIFY and DOCS phase markers and PQ ticket ids, and satisfy the opt-in commit-msg hook. Use when writing, rewording or reviewing a commit message, squash title or PR title.
---

# Conventional commits

Write the subject as `type(scope): description`. This skill covers the message only. It does not authorize committing: commit when the user asks, or when the run's required phase commits call for it (see the run guardrails in `AGENTS.md`).

## Availability

Stored in `.agents/skills/` for project-local discovery. Claude Code uses `.claude/skills/` and does not load this copy automatically; read this file explicitly there.

## Format

```
type(scope)!: description

Body: why, not what. Wrap near 72 columns.

BREAKING CHANGE: what breaks and what to do instead.
Refs: PQ-52
```

- **type**: one of `feat` `fix` `docs` `test` `chore` `refactor` `ci` `build` `perf` `style` `revert`.
- **scope**: optional, no spaces, the area touched. History uses `organizer-tui`, `spec`, `a11y`, `fallback`, `review`, `runbook`, `migrate-club`, `q8`. Reuse one before inventing another.
- **`!`** before the colon, and a `BREAKING CHANGE:` footer, mark a breaking change.
- **description**: imperative, lowercase first word unless it is an identifier, no trailing period. Aim for 72 characters on the whole subject; the hook does not enforce length.
- A blank line separates the subject from the body.

## Choosing the type

| Change                                                            | Type       |
| ----------------------------------------------------------------- | ---------- |
| User-visible capability: a new screen, command or refusal         | `feat`     |
| Behaviour that was wrong and is now right                         | `fix`      |
| Only documentation, specs, ADRs, runbook, lessons                 | `docs`     |
| Only tests or fixtures, including a RED commit                    | `test`     |
| Restructure with no behaviour change                              | `refactor` |
| `.github/workflows/`, CI config                                   | `ci`       |
| Build, dependencies, the `justfile`, container image              | `build`    |
| Ignore files, config, housekeeping, key or secret-reference fixes | `chore`    |
| Faster, same result                                               | `perf`     |
| Formatting only                                                   | `style`    |
| Undoing an earlier commit                                         | `revert`   |

When a commit does two things, split it. A commit that needs `fix` and `docs` is two commits.

## This repository's markers

The phase and ticket markers go at the **end of the description**, after the type, so the hook sees a normal subject:

```
test(fallback): the retired command fails loudly (PQ-52 RED)
fix(organizer-tui): a refused decision no longer reloads the preview (GREEN)
test(review): edits through the re-verifier keep a question's standing (VERIFY)
docs(review): say how long a checked quote lasts (DOCS)
```

| Phase  | Usual type                                           |
| ------ | ---------------------------------------------------- |
| RED    | `test`                                               |
| GREEN  | `fix`, `feat` or `refactor`, whichever the change is |
| VERIFY | `test`, or `chore` when it records a run             |
| DOCS   | `docs`                                               |

Put a ticket id as `(PQ-52)`, or inside the phase marker as `(PQ-52 RED)`. Skipping a phase commit is not a style choice; the run guardrails require them.

## Truthfulness

A subject states what the commit does and what was verified, no more. Do not write "fix" for a change you did not run, and do not describe a program's output from memory. Miri proves absence of UB on executed paths only, so say "verified" only where a recorded run backs it.

## The hook

`.githooks/commit-msg` is opt-in and local. Turn it on in a clone with `just hooks`. It rejects a subject that is not `type(scope): description` and a body that does not follow a blank line.

`core.hooksPath` replaces Git's hook directory instead of adding to it, so `just hooks` refuses, changes nothing and prints how to chain the check when it would switch off hooks already running: a `core.hooksPath` you set, or an executable hook in `.git/hooks` (for a linked worktree, the shared one). Call `.githooks/commit-msg "$1"` from your own `commit-msg` hook and keep its exit status.

A line starting with `#` is a comment only where Git strips it. Git strips comment lines when an editor opens, and keeps them for `git commit -m` and `-F`, so there `#123 fix key reference` is the subject and is refused. `commit.cleanup` and `core.commentChar` are honored. `git commit --cleanup=<mode>` is not visible to a hook and is read as if it had not been given.

Exempt: `Merge ...`, `Revert "..."`, and `fixup!`, `squash!`, `amend!` subjects. Skip once with `git commit --no-verify`; turn it off with `git config --unset core.hooksPath`.

The hook runs only on a machine that turned it on. Commits GitHub makes (the merge button, edits in the web UI) never pass through it, so write a PR title and any squash title as a conventional subject yourself.

Of 728 non-merge commits before this hook, 90 follow the convention. Do not rewrite history to match.

## Check

```
bash .githooks/commit-msg.test.sh
bash .githooks/install.test.sh
```

The fixtures run the hook on message files and through real `git commit` (with `-m` and with an editor) in a throwaway repository, and run the installer against repositories that already have hooks, including a linked worktree.
