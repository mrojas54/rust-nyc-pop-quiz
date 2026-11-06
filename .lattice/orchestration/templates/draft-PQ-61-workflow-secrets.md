# Workflow credential policy (PQ-61)

`tests/workflow_secrets.rs` runs inside `just test`. It parses each `.yml`
and `.yaml` file directly under `.github/workflows` with `serde_yaml_ng`, a
test-only dependency. File reads and YAML parsing must succeed.

The policy has two parts:

- `deployed-burst.yml` must match `deployed-burst.pinned.yml` byte for byte.
  Every step can affect later steps that hold credentials. The pin makes a
  change to scripts, actions, environment entries, triggers, or job setup
  require an explicit review. Parsed checks also require manual dispatch,
  the protected environment on the burst job, and secret expressions under
  that job's step environment mappings.
- Other workflows cannot bind the `deployed-burst` environment, choose an
  environment through an expression, or forward secrets to reusable
  workflows. Their only approved secret expression is the existing Fly
  token value at its specific step environment path in `deploy.yml`.

YAML decoding happens before policy checks. Quotes, escapes, flow mappings,
folded scalars, and aliases therefore cannot hide a value or move a read
away from its real location. YAML comments disappear; comments inside a
script remain part of the script and are checked. Duplicate keys, multiple
YAML documents, custom tags, merge keys, and non-string mapping keys are
rejected rather than interpreted ambiguously.

The secret-expression check is deliberately conservative: an expression
bearing scalar containing `secrets` (case insensitive), or an `if` value
containing it, must match a complete approved value at an approved path.
It does not evaluate GitHub expressions. A literal mention inside such an
expression may need policy review even if it would not read a secret.

## Changing a workflow

Ordinary YAML formatting in other workflows needs no exception. New secret
reads, computed environment names, and changes to an approved read's path
need an explicit policy change and regression coverage.

For a burst workflow edit, review every affected step against SPEC section
8.4 before updating the pinned fixture. Check earlier steps, shell setup,
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

This is a repository regression guard. The pin does not prove arbitrary
shell code safe, and changing both the workflow and its fixture still needs
human review. Runtime behavior of built binaries, action internals, cache
contents, and GitHub-side environment configuration are outside this test.
Required reviewers, branch restrictions, and setting/deleting the two
credentials remain operator responsibilities described in SPEC section 8.4.
