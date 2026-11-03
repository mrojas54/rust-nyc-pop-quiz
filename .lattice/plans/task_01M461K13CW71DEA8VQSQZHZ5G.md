# PQ-55: A RevealWitness can be forged without boundary.rs noticing

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-9** (G-3, Major, new): A RevealWitness can be forged without boundary.rs noticing
- Evidence: phase.rs:336; boundary.rs:98-113,148-156; M3e, M3e2 survived
- What would close it: Put RevealWitness in the no-derive, no-impl scan and count every constructor form in phase.rs

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.


# Plan (delegator, 2026-11-02)

Tests only: `popquiz/tests/boundary.rs`. `phase.rs` and the vault do not change.

What the scan does today, read at 2e8f6ef:
- `sealed_types` finds a declaration with `text.find(decl)`, and panics if it is absent, so an entry is not vacuous. The type name is the declaration's last word, so the entry must be `pub struct RevealWitness`, not `...<'m>`: with the lifetime in the name, `Clone for RevealWitness<'_>` would slip past.
- The impl half only looks for `Debug|Clone|Copy|Serialize for <ty>`. It misses `impl Default` and every inherent impl, whatever the lifetime spelling.

Changes:
1. Add `RevealWitness` to `sealed_types` (no derive; those four traits).
2. New `witness_impls` helper and test: no impl header anywhere in the room's source names `RevealWitness`, in any lifetime spelling, inherent or trait, including a header wrapped over several lines. The helper is checked against written-out forms, so it cannot go blind unnoticed, and the test asserts it walked `phase.rs`.
3. `the_witness_is_built_in_one_place` also counts `RevealWitness::<` as a literal. `Self { .. }` needs an impl, which rule 2 forbids. The mint stays at `Machine::revealed`.

Out of scope by ruling (goes in not-done): aliases, macros, `unsafe`, and similar.

Proof: plant M3e, M3e2, `impl Default`, hand `impl Clone`, a second literal, and a wrong declaration; each must fail a named test, then be reverted. Note the audit's `impl RevealWitness {` does not compile without a lifetime, so M3e is planted as `impl<'m> RevealWitness<'m>` and `impl RevealWitness<'_>`.

## Reset 2026-11-03 by agent:delegator-pq55
