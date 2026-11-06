# Workflow credential policy (PQ-61)

`tests/workflow_secrets.rs` runs inside `just test`. It parses each `.yml`
and `.yaml` file directly under `.github/workflows` with `serde_yaml_ng`, a
test-only dependency. File reads and YAML parsing must succeed.

The policy has two parts, both applied to the decoded YAML tree:

- `deployed-burst.yml` must match the reviewed execution configuration in
  `deployed-burst.pinned.yml`. The comparison ignores YAML comments, mapping
  order, and scalar presentation. Decoded script contents, step order, and
  every configuration value must still match.
  Every step can affect later steps that hold credentials. The baseline makes a
  change to scripts, actions, environment entries, triggers, or job setup
  require an explicit review. Parsed checks also require manual dispatch,
  the protected environment on the burst job, and secret expressions under
  that job's step environment mappings.
- Other workflows cannot bind the `deployed-burst` environment, choose an
  environment through an expression, or forward secrets to reusable
  workflows. Their only approved secret expression is the existing Fly
  token value in a step's `env.FLY_API_TOKEN` under `jobs.deploy` in
  `deploy.yml`. Inserting another step does not invalidate that permission.

YAML decoding happens before policy checks. Quotes, escapes, flow mappings,
folded scalars, and aliases therefore cannot hide a value or move a read
away from its real location. YAML comments disappear; comments inside a
script remain part of the script and are checked. Duplicate keys, multiple
YAML documents, custom tags, merge keys, and non-string mapping keys are
rejected rather than interpreted ambiguously.

The secret-expression check is deliberately conservative: an expression
bearing scalar containing `secrets` (case insensitive), or an `if` value
containing it, must match a complete approved value at an approved path.
It does not evaluate GitHub expressions. A literal mention anywhere in the
same scalar requires review even when it is outside the expression. For
example, a script containing both `echo 'No secrets are printed'` and
`${{ github.sha }}` is rejected. The diagnostic describes this conservative
rule; it does not claim that every rejected value reads a credential.

## Acceptance examples

| Change | Result |
|---|---|
| YAML comment ending in `: |` before an escaped secret read | Rejected after decoding the read |
| Flow-style job binding `deployed-burst` in another workflow | Rejected at that job's environment |
| Approved Fly-token value aliased into `run` | Rejected at the alias destination |
| Secret interpolation in a script or `with` mapping | Rejected outside the approved step `env` |
| Earlier burst step changes curl defaults, or a credential step gains tracing | Rejected by the execution baseline |
| Burst YAML reserialized with different quotes and no YAML comments | Accepted when decoded values match |
| Harmless step inserted before Fly deployment | Accepted |
| A normal `run: | # comment` or `restore-keys: |` scalar | Accepted |

## Changing a workflow

Ordinary YAML formatting in other workflows needs no exception. New secret
reads, computed environment names, and changes to an approved read's path
need an explicit policy change and regression coverage.

For a burst execution edit, review every affected step against SPEC section
8.4 before updating the baseline fixture. Check earlier steps, shell setup,
actions, and environment entries as well as the scripts that consume the
credentials. Copying the workflow into the fixture is not that review.

Run from `popquiz/`:

```sh
cargo test --offline --locked --test workflow_secrets --test harness_config
```

Then run `just test` from the worktree root. Negative semantic fixtures are
parsed first so invalid YAML cannot masquerade as successful policy
rejection. Separate cases exercise malformed or deliberately unsupported
YAML. No test dispatches a workflow or reads real credentials.

## Evidence boundary

This is a repository regression guard. The baseline does not prove arbitrary
shell code safe, and changing both the workflow and its fixture still needs
human review. Runtime behavior of built binaries, action internals, cache
contents, and GitHub-side environment configuration are outside this test.
Required reviewers, branch restrictions, and setting/deleting the two
credentials remain operator responsibilities described in SPEC section 8.4.
