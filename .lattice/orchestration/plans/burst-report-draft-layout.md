# Burst-report split — earlier draft layout (reference only, out of PQ-71)

Pasted by the client 2026-10-12. Ruled out of PQ-71 the same day (client: "Keep it out of PQ-71"), as the live-room plan says: the burst-report split was premature. No ticket minted; mint one only on the client's word.

The full draft parent (335 lines) is saved beside this note as `burst_report.rs`, moved from the untracked `discord_burst_refactor/` folder on 2026-10-12. It does not compile alone.

Only the parent `room/src/cli/burst_report.rs` body was pasted; the draft bodies of `burst_report/statistics.rs` and `burst_report/verdict.rs` and the Discord draft were not.

## Proposed structure (as pasted)

```text
room/src/host/
├── discord.rs              # Sign-in, sessions, token refresh, role checks
└── discord/
    ├── config.rs           # Settings, secrets, retry policy, logging
    └── http.rs             # HTTP/TLS requests, responses, form encoding

room/src/cli/
├── burst_report.rs         # Measurements and JSON report assembly
└── burst_report/
    ├── statistics.rs       # Percentiles and confidence intervals
    └── verdict.rs          # Thresholds and pass/miss/invalid decisions
```

- Compatibility: retain existing public names through re-exports.
- Visibility: child modules stay private; transport operations use `pub(super)`.
- Boundaries: transport knows nothing about sessions; statistics and verdict logic remain independent of report assembly.
- Validation: review for behavior changes, run the room tests, and compile/test feature-gated binaries.

## Draft parent module head (as pasted)

The draft keeps `burst_report.rs` as the report assembler and includes the two children by explicit `#[path]`, because `burst.rs` and `tests/burst_report.rs` include `burst_report.rs` itself with `#[path]`:

```rust
#[path = "burst_report/statistics.rs"]
mod statistics;
#[path = "burst_report/verdict.rs"]
mod verdict;

pub use statistics::{ci_ranks, nearest_rank, r2, Stats};
pub use verdict::{
    verdict, Exit, VerdictInput, MIN_NOFILE, REVEAL_P95_MS, SEND_LAG_P95_INVALID_MS,
    WRITE_P95_MS,
};
```

The rest of the pasted file (`Cycle`, `Measured`, `substrate_note`, `report`) is the report assembly as it stands; on `main` it lives in `room/src/cli/burst_report.rs` together with the statistics and verdict code.
