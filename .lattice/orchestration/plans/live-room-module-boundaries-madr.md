# Separate live-room responsibilities while retaining shared state ownership

- Status: Proposed
- Scope: Live-room authentication, hosting, participant operations, and lifecycle management
- Implementation plan: [Live-room hosting and management refactor](live-room-refactor-plan.md)

## Context and problem statement

The Discord backend combines configuration, HTTP transport, and authorization.
`AppState` combines host management with participant/session operations. These
responsibilities are distinguishable, but their placement makes changes harder
to locate and review.

How should the code be organized so readers can follow operations while
preserving authorization, lifecycle behavior, and atomic updates?

## Decision drivers

- Make operations easy to locate and trace.
- Keep related checks and state changes together.
- Preserve public APIs and observable behavior.
- Avoid exposing internals to enable file separation.
- Keep architectural complexity proportional to current requirements.
- Preserve existing concurrency guarantees.

## Considered options

1. Keep the current modules and improve documentation only.
2. Extract cohesive private modules with existing state ownership.
3. Introduce service and repository layers.
4. Introduce one async actor per room.
5. Split each operation into a separate file.

## Decision outcome

Proposed choice: **extract cohesive private modules with existing state
ownership**, because the demonstrated problem is mixed responsibilities and
navigation. This option addresses that problem without introducing a new
execution model.

| Explicit decision | Rationale | Accepted tradeoff |
|---|---|---|
| Separate Discord configuration and HTTP transport | Each has a distinct responsibility and a small interface | Understanding a request may involve several files |
| Keep OAuth, sessions, refresh coordination, authorization, and retry orchestration together | Their ordering and concurrency rules are closely connected | The backend remains relatively large |
| Group participant operations in one private module | They share session handling and count-update invariants | `AppState` implementations span files |
| Keep host transitions and release effects together | Their atomicity should remain locally visible | Management code retains several related responsibilities |
| Retain existing locks and state ownership | Module extraction does not require concurrency changes | Possible shared-lock contention remains |
| Preserve public paths through re-exports | Callers should remain independent of implementation layout | Re-exports require ongoing maintenance |
| Retain existing cohesive creation, room, lifecycle, and route modules | File length alone does not justify extraction | Module sizes remain uneven |
| Keep new modules private and restrict cross-module access | Implementation layout should not enlarge the public API | Sibling modules still share selected internals |

### Required invariants

- Participant session changes and room counts update under the same lock.
- Release, session clearing, used-record writing, and take-home updates remain
  atomic.
- Creation and scheduling preserve lock ordering.
- Host authorization precedes disclosure of ended-room state.
- Discord refresh remains serialized and survives caller cancellation.
- Existing credential boundaries and pre-reveal answer isolation remain intact.

### Consequences

- Good, because code becomes easier to navigate by responsibility.
- Good, because public callers need no migration.
- Good, because critical transitions remain reviewable together.
- Good, because existing behavioral tests remain applicable.
- Bad, because this decision does not improve runtime scalability.
- Bad, because sibling modules still depend on shared state through restricted
  access.
- Bad, because boundaries require documentation and discipline to prevent
  future scattering.

### Confirmation

Confirm through the normal room suite, the lifecycle scenario covering Discord
unavailability, compilation of all feature-gated callers, and specialist review
of the exact diff. Commands are recorded in the implementation plan.

Passing tests alone is insufficient: review must verify visibility, lock
boundaries, authorization ordering, and the completeness of each extraction.
The decision remains proposed until accepted; an earlier successful test run
does not establish that the planned refactor is complete.

## Pros and cons of the options

### Keep the current modules and improve documentation only

- Good, because it has minimal migration risk and verification cost.
- Good, because all current operation bodies stay where readers expect them.
- Bad, because mixed responsibilities remain and files continue to grow across
  unrelated concerns.
- Not chosen, because documentation alone does not establish the identified
  configuration, transport, and participant boundaries.

### Extract cohesive private modules with existing state ownership

- Good, because navigation improves without requiring caller changes.
- Good, because behavior, lock scopes, and state ownership can remain unchanged.
- Good, because small interfaces can use restricted visibility.
- Bad, because `AppState` methods span files and shared-state coupling remains.
- Chosen, because it addresses the observed problem with limited architectural
  and verification cost.

### Introduce service and repository layers

- Good, because dependency interfaces become explicit.
- Good, because persistence concerns could eventually have separate ownership.
- Bad, because it introduces traits, forwarding, and wiring without a
  demonstrated persistence requirement for this change.
- Bad, because following an operation may require more indirection.
- Not chosen, because the current refactor does not justify these abstractions.

### Introduce one async actor per room

- Good, because each room has one state owner and commands are serialized.
- Good, because it offers an alternative to shared mutex-based mutation.
- Bad, because mailbox capacity, backpressure, cancellation, replies, and
  shutdown all need explicit semantics and tests.
- Bad, because routes and WebSocket callers require substantial changes, and
  coordination with scheduling and cross-room state still needs a design.
- Not chosen, because no measured contention or lifecycle requirement has been
  established that warrants this migration. An actor change needs its own MADR.

### Split each operation into a separate file

- Good, because individual operations are easy to locate by filename.
- Bad, because related checks, helpers, and invariants become scattered.
- Bad, because the result can be many tightly coupled fragments rather than
  cohesive modules.
- Not chosen, because local readability and responsibility boundaries matter
  more than uniform or minimal file length.

## When to revisit

Reconsider state ownership when measured contention, persistence requirements,
or independently managed room tasks justify a different model.

Reconsider module boundaries when routine changes repeatedly cross them, rather
than merely when a file passes a line-count threshold. Any actor migration
should receive its own MADR, evidence, and validation plan.

## Related decisions

Existing architecture records must be inspected and linked before assigning this
proposal a permanent ADR identifier or promoting it to an accepted ADR location.
Do not invent identifiers or imply that this proposal supersedes an existing
decision without checking it.

Module documentation describes where code lives; this MADR records why the
boundaries were chosen.
