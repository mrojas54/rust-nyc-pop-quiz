# Generator fixtures

Each file is one Message Batches result set, replayed by
`popquiz.generate.ReplayBatchClient` so `just test` can drive the generator with no
key and no network.

**None of these is a recording.** They are hand-built in the SDK's documented batch
result shape (`{custom_id, result: {type: succeeded | errored | canceled | expired, …}}`,
a succeeded result carrying a `message` with `content`, `stop_reason` and `usage`),
standing in for what a model would return. No real batch produced them; the token
counts are invented, which is why a fixture run's report says *FIXTURE NUMBERS*.

**Every candidate carries a drafted trace with no `stdout` row.** The steps are
hand-written stand-ins for what a model would draft, not checked against the
programs; the `stdout` row is the verifier's to write (`verify.with_record`).

**No fixture says which option is correct**, and nothing here is verified. The option
texts are plausible readings of each program, not a claim about what it prints; the
machine decides that, later, in `verify` (AC-7). These files never reach
`bank/questions/`.

| Fixture | What it proves |
|---|---|
| `clean.json` | Three well-formed candidates that fit the default room; all are emitted (AC-3) |
| `one-errored.json` | An errored request is reported with its error and never emitted; the run names the count and topic it missed |
| `one-expired.json` | An expired request is reported and never emitted |
| `over-capacity.json` | Each capacity fact broken once — an 11-line source, a 31-character option, a two-line option — each reported with its fact, none trimmed (D-15) |
| `invalid-shape.json` | Two `does_not_compile` options, and output cut off at `max_tokens`, are reported; the valid third is emitted unchanged |
| `talk.json` | Talk mode: each candidate names a std or core-language concept, which becomes its topic (AC-4) |
| `trace-drafts.json` | Drafted traces: a candidate that runs (q3's bank source) whose offered `stdout` value is discarded and reported, and which the verifier fills on the stub Runner; one that does not compile (q8's bank source) with no `stdout` row; a one-step trace and a step past the last source line, both reported (option b) |

The first paid run (HC-2) is the first time a real batch replaces any of this.

## `claude/` — the claude-cli backend (PQ-69)

`fake_claude.py` stands in for the `claude` executable. A test puts a wrapper for it
first on `PATH`, and it answers each prompt from one of the scripts below. It records
its argv, its cwd, its prompt and the **names** of the environment variables it saw,
never their values.

**These are not recordings either.** Each answer is hand-built in the shape the Claude
Code documentation gives for `claude -p --output-format json` with `--json-schema`:
`type`, `subtype`, `is_error`, `result` or `errors`, `structured_output`,
`duration_ms`, `total_cost_usd` and `usage`. No real `claude` produced any of them.
Two things are **documented, not observed** until HC-2:

- the key names inside `usage`;
- what a usage-limit failure looks like as JSON. The docs give only its message text.

The candidates are `clean.json`'s.

| Script | What it proves |
|---|---|
| `cli-clean.json` | Three clean candidates: emitted, with the CLI's tokens, `duration_ms` and `total_cost_usd` reported as API-equivalent |
| `cli-invalid.json` | A structured output that fails the generator's own validation is reported; the CLI giving up on a schema-valid answer (`error_max_structured_output_retries`) is reported; the run goes on |
| `cli-usage-limit.json` | A session-limit result (exit 1, `is_error`) stops the run at c02; c01 is kept, c03 is never asked for, and a rerun picks up the rest |
| `cli-nonzero-exit.json` | A process that exits 2 with no JSON stops the run at the first candidate |
| `cli-records-argv-and-env.json` | Clean answers; the test reads what the fake recorded: the flags, the empty cwd, the brief on stdin, and no billing variable in the child's environment |
