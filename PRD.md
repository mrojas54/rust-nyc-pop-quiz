# Rust NYC Fresh Quiz

**Status:** Draft - blocked on feasibility evidence\
**Platform:** Val Town application with Pydantic AI generation and Modal Rust
verification

## Product Summary

Rust NYC Fresh Quiz is a host-led meetup quiz that generates original Rust
program-output questions for every session. Participants join anonymously from
their phones. Hosts sign in through Discord and must currently hold the
`nyc-organizers` role in the Rust East Coast Discord server.

Every accepted question is checked by a pinned Rust compiler and Miri
configuration before the room opens. Correct-choice identity, explanation, and
verification outcome are withheld from all pre-reveal client payloads. The
correct answer text necessarily appears as one public choice. A Rust-expert host
may still infer an answer by reading the source; host ignorance is not a
security guarantee.

## Problem

Existing Rust quizzes use a finite, publicly available question bank. Repeat
attendees can recognize questions and answers, reducing both the challenge and
the value of the resulting technical discussion.

Rust NYC needs a live quiz that:

- Produces original questions for every session.
- Checks each answer with a pinned toolchain instead of trusting generated
  content.
- Rejects exact duplicates and reduces likely semantic repetition.
- Supports a live room without participant accounts.
- Keeps answer records out of pre-reveal browser and API state.
- Uses the existing Discord organizer role as its authorization source.
- Matches Rust NYC's nostalgic, technical visual identity.

## Goals

- Produce an original, verified quiz for every room.
- Prevent exact duplicates and measurably reduce superficial variants of
  previous questions.
- Authorize organizers from the existing Discord role rather than a separate
  allowlist.
- Support up to 200 anonymous participants with synchronized timers and reveals.
- Match the self-contained Rust NYC brand and accessibility contract below.

## Non-Goals

- Participant profiles, nicknames, or leaderboards.
- Manual question authoring or editing.
- Question exports or reusable question banks.
- Self-paced practice mode.
- External Rust crates.
- Dark mode in version one.

## Users

### Organizer

An authenticated Rust East Coast Discord member who currently holds the
`nyc-organizers` role and can create and control rooms.

### Participant

An anonymous meetup attendee who joins with a room code and submits answers from
a personal device.

### Platform Owner

The maintainer responsible for the Val Town project, Discord application, Modal
verifier, Pydantic AI Gateway project, secrets, and pinned Rust toolchain.

## Core Experience

1. The organizer selects **Sign in with Discord**.
2. The application confirms that the organizer belongs to the configured Rust
   East Coast guild and holds the configured `nyc-organizers` role ID.
3. The organizer selects question count, timer, difficulty, Rust topics, and
   whether the quiz should relate to tonight's talk.
4. The application generates and verifies the complete quiz while displaying
   answer-free progress.
5. The lobby becomes available only after every question passes verification.
6. Participants join using a six-character room code and receive anonymous,
   signed sessions.
7. The host starts each question and may publish a hint to everyone without a
   private preview.
8. Participants choose an answer and may change it until the server deadline or
   an early close.
9. The host reveals the correct answer, explanation, and anonymous response
   totals.
10. The host advances through the quiz and receives a final aggregate summary.

## Discord Authorization

- Use Discord's server-side OAuth2 authorization-code flow with only the
  `identify` and `guilds.members.read` scopes.
- After exchanging the authorization code, call `GET /users/@me` and
  `GET /users/@me/guilds/{guild.id}/member`.
- Authorize only when the member response's `roles` array contains the exact
  `DISCORD_ORGANIZER_ROLE_ID` value. Discord documents that the current-member
  endpoint requires `guilds.members.read` and returns role IDs in the member
  object: <https://docs.discord.com/developers/resources/user>.
- Do not grant exceptions for guild ownership, administrator permission, or a
  similarly named role.
- Store immutable guild and role IDs in `DISCORD_GUILD_ID` and
  `DISCORD_ORGANIZER_ROLE_ID`. Use `nyc-organizers` only as display text.
- Protect OAuth initiation with a single-use state value stored in a secure
  cookie and expiring after ten minutes.
- Require an exact registered callback URL.
- Store Discord access and refresh tokens encrypted with AES-256-GCM in the
  server-side session record. Never expose or log them.
- Issue an eight-hour application session using a `Secure`, `HttpOnly`,
  `SameSite=Lax` cookie.
- Before every privileged mutation, revalidate the Discord role when the last
  successful check is more than 60 seconds old.
- Refresh expired Discord OAuth tokens when possible.
- Invalidate the organizer session when the user leaves the guild or loses the
  required role.
- Treat a verified missing role, guild departure, revoked grant, or invalid
  refresh token as a hard denial and invalidate the organizer session
  immediately.
- For Discord timeouts, connection failures, `429`, and `5xx` responses only,
  allow the existing room owner to continue `start`, `show_hint`, `close`,
  `reveal`, `next`, and `end` actions on an already opened room for at most 15
  minutes after the last successful role check. Show an outage warning, retry
  verification in the background, prohibit new-room creation and ownership
  changes, and fail closed when the grace period expires.
- Participants do not authenticate with Discord.

## Quiz Configuration

Each room must support:

- 1 to 5 questions, defaulting to 3.
- 30 to 180 seconds per question in 15-second increments, defaulting to 90.
- Difficulty 1, 2, 3, or mixed.
- Quiz focus **General Rust** (default) or **Tonight's talk**.
- One or more of the following topics:
  - Ownership.
  - Borrowing and lifetimes.
  - Traits and generics.
  - Patterns and enums.
  - Iterators and closures.
  - Async and concurrency.
  - Unsafe memory.
- **Tonight's talk** requires a public talk title of at most 120 characters and
  either a public abstract or organizer-written key takeaways of at most 4 KiB
  plain text. Do not accept slide, code, or file uploads.
- In talk mode, use the selected Rust topics as guardrails and require every
  question to connect an idea from the talk context to an underlying Rust
  language or standard-library concept. A question must remain understandable
  from its displayed source and must not require remembering an exact quote from
  the speaker.
- If the talk centers on an external crate, translate it to a compatible core
  Rust concept without using the crate API. Fail before generation with an
  actionable message when the supplied context has no compatible Rust concept.
- For a fixed difficulty, assign that value to every slot. For mixed difficulty,
  allocate levels 1, 2, and 3 as evenly as the question count permits, then
  shuffle their slot order.
- Shuffle selected topics and allocate them round-robin. When there are fewer
  slots than selected topics, choose a subset without replacement. Do not
  correlate topic or difficulty allocation with the target answer category.

## Functional Requirements

### Room Lifecycle

- Use the lifecycle `generating`, `ready`, `lobby`, `asking`, `closed`,
  `revealed`, and `complete`, plus terminal `failed` and `abandoned` states.
- Define `ready` as the state in which every question has passed verification
  but joining is not yet open. The host action `open` transitions the room from
  `ready` to `lobby`, which makes the room code joinable.
- Support the host actions `open`, `start`, `show_hint`, `close`, `reveal`,
  `next`, and `end`.
- Make answer deadlines server-authoritative.
- Permit late joiners to answer the current question while it remains open.
- Never open a partially generated room.
- Make the creating organizer's immutable Discord user ID the room owner. Only
  that Discord user may mutate the room in version one; signing in again as the
  same user restores control. Other organizers may not access its host view or
  take over the room.
- Generate room codes from `ABCDEFGHJKMNPQRSTVWXYZ23456789`, normalize input to
  uppercase, and compare case-insensitively. Codes are six characters, unique
  among retained rooms, generated with a cryptographically secure RNG, and
  retried up to 20 times on a database uniqueness conflict.
- Expire joining 12 hours after room creation or immediately when the room
  completes. Mark nonterminal rooms abandoned after six hours without host
  activity and make their codes unjoinable.

### Participant Responses

- Present three plausible output choices plus **does not compile** and
  **exhibits undefined behavior**.
- Permit one response per participant and question.
- Allow a participant to replace that response until the question closes.
- Store no participant name, email address, or Discord identity.
- Display only aggregate response totals after reveal.
- Admit at most 200 active participant sessions using a transactional capacity
  check. Return `409 room_full` to the 201st new session. Reconnecting with the
  same signed session does not consume another place.
- Define uniqueness per signed participant session, not per human. Clearing
  cookies or changing devices creates a new participant and may consume another
  place; preventing that would require participant identity and remains a
  non-goal.

### Pre-Reveal Answer Isolation

- Before reveal, show the host source code, timer, participant count, aggregate
  response counts, topic, difficulty, and a category-neutral **Verified** badge.
- Do not include the correct choice ID or index, explanation, unpublished hint,
  private verification evidence, or any relationship that identifies the correct
  public choice in HTML, client JavaScript, API responses, polling payloads,
  browser-accessible state, or application logs before reveal.
- Do not claim that a Rust-expert host is unable to derive the answer from the
  source. The enforceable property is that secret answer records are not sent to
  any client before reveal.
- Let the host publish a hint without a private preview. The successful
  `show_hint` transition publishes the same hint in the next organizer and
  participant room views.
- Make the pre-reveal verification-status subdocument, badge markup, and
  accessible label byte-identical across deterministic-output, compiler-error,
  and undefined-behavior questions. Show the detailed compiler and Miri receipt
  only after reveal.
- Publish the answer, explanation, and aggregate correctness to every connected
  client only after the reveal transition succeeds.

### Failure Handling

- Allow at most five total candidate attempts per slot, including the initial
  attempt. Schema, verification, uniqueness, distribution, and quality
  rejections each consume one attempt; a Modal retry of the same interrupted
  invocation does not.
- Show the host only coarse `generating`, `retrying`, and `failed` progress plus
  completed-slot counts. Keep target category, verifier path, diagnostics, and
  detailed rejection reasons in platform-owner telemetry only.
- When retries are exhausted, enter the `failed` state and offer a retry or
  configuration change.
- Never substitute an unverified question.
- Enforce a five-minute room-generation deadline. Do not begin another candidate
  attempt when its remaining stage budget cannot fit before the deadline; fail
  with the exhausted stage and attempt counts.

## Question Requirements

- Use Rust 2024 edition and pin the complete verifier manifest: `rustc -Vv`,
  target triple, `rustfmt` version, Miri build, verifier Image digest, Miri seed
  range, and exact `MIRIFLAGS`. Miri documents the seed and optional Tree
  Borrows controls at <https://github.com/rust-lang/miri>.
- Use the Rust standard library only.
- Limit programs to 35 lines and 4 KiB of UTF-8 source.
- Prohibit input, filesystem access, network access, environment access,
  subprocesses, time, randomness, and external dependencies.
- Permit `unsafe` only for questions whose intended answer is undefined
  behavior.
- Assign each question slot a private target answer independently with a
  cryptographically secure RNG before generation, sampling from a fixed weight
  vector that does not depend on question count, topic, difficulty, or any
  participant-observable state. The version-one weights are 79.99% deterministic
  output, 0.01% compiler error (`0.0001` probability), and 20% undefined
  behavior.
- These weights intentionally make the semantic answer labels nonuniform. Before
  solving the program, each of the three output choices has an approximate
  26.663% prior, **does not compile** has a 0.01% prior, and **exhibits
  undefined behavior** has a 20% prior. Secure shuffling still makes each final
  screen position 20% likely to contain the correct choice.
- Accept as an explicit product tradeoff that a repeat attendee can rationally
  almost always ignore **does not compile**. The application must not describe
  the answer distribution as uniform or resistant to that strategy.
- Sample every slot independently. Undefined-behavior questions must be possible
  in rooms of every size; no room size may force a category to zero.
- Generate each slot to its assigned target and accept a candidate only when its
  verified answer type matches the assignment. A mismatch — for example, a slot
  targeted at deterministic output whose program Miri reports as undefined
  behavior — is a rejection that consumes a generation retry.
- Version the 79.99/0.01/20 vector as an immutable version-one product constant;
  it is not host or deployment configuration. Any deviation requires a PRD
  change, a new distribution version, and rerunning the distribution acceptance
  suite.
- Populate the three output choices by answer category. A deterministic-output
  question presents its verified output plus two distinct plausible output
  distractors. A compiler-error or undefined-behavior question presents three
  distinct plausible output distractors, each an output that a reader who
  misses the trap could accept as the program's result. Then shuffle all five
  choices with a cryptographically secure RNG. Apart from the deliberately
  nonuniform semantic-label priors above, no position, markup, wording variant,
  distractor style, receipt field, or timing behavior may reveal additional
  information about the correct option or target category.

### Deterministic Output

- The program must compile successfully.
- Miri must complete with `MIRIFLAGS="-Zmiri-many-seeds=0..4"` under the default
  Stacked Borrows model without finding undefined behavior.
- Three fresh, bounded native processes must produce byte-identical standard
  output. This is a bounded nondeterminism detector, not a proof; three is the
  initial latency/coverage tradeoff and may only change through a versioned
  verifier decision backed by the generation benchmark.
- The verified output must match exactly one answer choice.
- Limit each native run to two seconds and 4 KiB of stdout. Treat timeout,
  output truncation, non-UTF-8 output, or nonempty stderr as rejection.
- Reject output that can depend on `HashMap` or `HashSet` iteration order,
  `RandomState`, pointer/address formatting such as `{:p}`, system entropy,
  thread scheduling, weak-memory outcomes, unspecified debug ordering, or any
  other implementation-dependent source unless the program canonicalizes that
  value before printing.

### Compiler Error

- Compilation must fail.
- The compiler output must contain the intended Rust diagnostic code.
- The failure must result from the behavior described by the explanation.

### Undefined Behavior

- The program must compile successfully.
- Run Miri once with `MIRIFLAGS="-Zmiri-many-seeds=0..4"` and once with
  `MIRIFLAGS="-Zmiri-tree-borrows -Zmiri-many-seeds=0..4"`. Both configurations
  must report the same normalized undefined-behavior category; reject
  model-dependent examples.
- Normal program execution must not be used to infer undefined behavior.

### Uniqueness

- Format accepted source with `rustfmt`.
- Store a cryptographic hash of the formatted source and a versioned AST
  fingerprint. The AST pass may remove comments and alpha-rename resolved local
  bindings, but must not normalize literals, operators, types, control flow, or
  any token that can change the verified answer.
- Build a versioned semantic descriptor containing the primary concept, trap
  mechanism, outcome category, diagnostic family or output shape, and
  control/data-flow features. Embed the source plus descriptor and use indexed
  locality-sensitive buckets to retrieve at most 20 historical candidates with
  cosine similarity at or above an initially configured `0.82` threshold.
- Reject exact source and AST fingerprint matches. Submit high-similarity top-K
  matches to a separate semantic judge and reject candidates judged to teach the
  same concept through the same trap, even when syntax differs.
- Store the source, descriptor, embedding, fingerprint algorithm version, and
  judge decision permanently. During an algorithm migration, compute both old
  and new versions and compare against both until historical records are
  reindexed.
- Treat semantic uniqueness as a measured risk reduction, not proof of program
  inequivalence. Maintain curated equivalent and distinct fixture pairs and
  publish false-accept and false-reject rates with every algorithm version.

### Quality And Difficulty

- Define difficulty 1 as one primary concept with short local reasoning;
  difficulty 2 as an interaction between two concepts or multiple state changes;
  and difficulty 3 as multi-step reasoning involving a subtle language rule.
  Difficulty may not rely on trivia, illegible formatting, or ambiguity.
- After toolchain verification and uniqueness checks, run a separate Pydantic AI
  quality judge using a configured model distinct from the generation model.
- Give the judge the source, choices, verified outcome, explanation, requested
  topic, requested difficulty, quiz focus, and optional talk context. Require
  structured scores for unambiguity, distractor plausibility, explanation
  accuracy, topical relevance, talk relevance when applicable, discussion value,
  and difficulty fit.
- Require `unambiguous=true`, `explanation_consistent=true`,
  `topic_relevant=true`, `talk_relevant=true` in talk mode, every output
  distractor judged plausible (two for a deterministic-output question, three
  for a compiler-error or undefined-behavior question), discussion value of at
  least 3 on a 5-point rubric, and an estimated difficulty within one level of
  the request. A judge rejection consumes a candidate attempt.
- Treat the judge as a quality-control heuristic, never as verification of the
  Rust answer. Calibrate its rubric against a blind organizer-rated sample
  before release and after material prompt or model changes.

## Platform Architecture

### Val Town Application

- Host the participant and organizer application using React and Hono in a Val
  Town HTTP val.
- Host the HTTP API, Discord OAuth callback, val-scoped SQLite database, Blob
  assets, and retention cron on Val Town.
- Use versioned conditional polling every second during active play and every
  three seconds in the lobby, generation, and completed states.
- Treat that polling design as provisional until the required 200-client Val
  Town spike passes. If it fails, revise the transport or participant-facing
  host before changing the capacity or propagation requirements.
- Return `304 Not Modified` when the client's room version is current.
- Return the latest room view directly after every successful host action or
  answer submission.

### Data Separation

- Store public question data and private answer data in separate tables.
- Make every organizer and participant state query read only public question
  tables, `question_hints`, and `question_reveals`. State-query code must never
  query or join `question_secrets`.
- On a successful version-checked `show_hint` transition, use one transaction to
  copy only the current question's hint into `question_hints`. Public state
  queries may return it after that transaction commits.
- On a successful version-checked reveal transition, use one transaction to read
  the current question's secret and copy only its releasable fields into
  `question_reveals`. Public state queries may return that copied record after
  the transaction commits.
- Limit `question_secrets` writes to candidate acceptance and reads to the hint
  and reveal transition modules so a view-assembly defect cannot leak a secret
  that was never loaded.
- Protect room transitions with conditional versions, uniqueness constraints,
  and transactional SQLite batches.
- Retain generated questions and fingerprints indefinitely.
- Delete anonymous sessions, responses, completed or abandoned room state, and
  analytics after 30 days using a daily cron. Retain only question artifacts,
  uniqueness metadata, and non-participant verifier evidence indefinitely.
- Delete raw talk titles, abstracts, and key takeaways with room state after 30
  days. Permanent uniqueness records may retain derived Rust concept tags but
  not the supplied talk text.

### External Generation And Verification

- Run generation and Rust verification in a deployed Modal application. Val Town
  remains the participant-facing application and durable system of record.
- Assign each quiz a persistent generation ID and a monotonically increasing
  job-attempt number, then submit it through a proxy-token-protected Modal Web
  Function using a timestamped HMAC-signed request over HTTPS. The endpoint
  must validate the request, spawn a background generation Function, and
  immediately return its Modal Function Call ID. See Modal's Web Function and job-processing
  documentation: <https://modal.com/docs/guide/webhooks> and
  <https://modal.com/docs/guide/job-queue>.
- Deduplicate only the same `(generation ID, job attempt)` using a durable Modal
  Dict entry written with `skip_if_exists`. Store the returned Function Call ID,
  attempt number, and last heartbeat with the Val Town room.
- Generate candidates in the Modal Function with a Pydantic AI `Agent` whose
  `output_type` is the versioned `QuestionCandidate` Pydantic model. Reject any
  response that does not pass Pydantic validation before Rust verification.
- Pass the optional talk context to generation and quality judging, but not into
  the Rust Sandbox. The Sandbox receives only the candidate source and verifier
  manifest.
- Generate semantic vectors through Pydantic AI Gateway using the configured
  embedding model. Use the configured judge model for both near-duplicate and
  quality decisions, but keep separate prompts, schemas, and retry budgets.
- Pass each slot's assigned target answer type into the generation request, and
  treat the verifier's determined answer type as authoritative. Reject any
  candidate whose verified type differs from its assignment, regardless of
  Pydantic validity, so the sampled distribution is realized only from verified
  outcomes.
- Route every model request through Pydantic AI Gateway, managed in Logfire,
  using a project-scoped Gateway key, a `gateway/<api-format>:<model>` model
  string, and an explicitly configured provider or routing-group route. See
  <https://pydantic.dev/docs/ai/overview/gateway/>.
- Configure Gateway spending limits and, when needed, a routing group limited to
  approved models that satisfy the same structured-output contract. Gateway
  routing or Pydantic validation never substitutes for compiler and Miri
  verification.
- Give only the Modal orchestration Function access to the Pydantic AI Gateway
  key and the Val Town callback URL. Do not pass either value into a Rust
  Sandbox.
- Keep prompts, completions, tool arguments, generated answers, and source code
  out of Logfire telemetry. Disable Gateway conversation-content telemetry for
  this project. If Pydantic AI instrumentation is enabled, require
  `include_content=False`; retain only timing, token usage, cost, model, status,
  room ID, generation ID, and attempt metadata. See
  <https://pydantic.dev/docs/ai/integrations/logfire/>.
- Verify each candidate in a fresh Modal Sandbox built from a versioned named
  Image containing the pinned Rust compiler, Miri, and verifier harness. See
  <https://modal.com/docs/guide/sandboxes>.
- Create Rust Sandboxes with `block_network=True`, no secrets, no mounted
  Volumes, and explicit CPU, memory, disk, wall-clock, and output limits. Always
  terminate the Sandbox after verification. See
  <https://modal.com/docs/guide/sandbox-networking>.
- Begin with one physical CPU core, 2 GiB memory, 2 GiB writable disk, a
  180-second Sandbox lifetime, and 64 KiB combined diagnostic-output capture.
  Treat every limit breach as a rejected candidate. Changes require a versioned
  verifier manifest and benchmark evidence.
- Val Town cannot perform Rust verification because it prohibits general
  filesystem and subprocess execution:
  <https://docs.val.town/troubleshooting/permission-errors/>.
- Configure Modal Function retries and timeouts for transient infrastructure
  failures. Make every generation attempt and callback idempotent by room ID,
  slot number, attempt number, and event sequence so a retry cannot duplicate an
  accepted question.
- Require a signed heartbeat at least every 15 seconds while a job is active. If
  no heartbeat arrives for 90 seconds or Modal reports a terminal failed call, a
  Val Town transaction may mark that job attempt stale and increment the job
  attempt, up to three job attempts and within the room deadline. A resubmission
  uses a new Modal Dict key; callbacks from older attempts are rejected.
- Exchange progress, candidates, completion, and failure events through
  timestamped HMAC-signed callbacks sent only over HTTPS. HMAC signing provides
  integrity and authenticity, not confidentiality; TLS is required because
  candidate callbacks carry the accepted answer and explanation.
- Reject callbacks with an invalid signature, stale timestamp, duplicate event
  sequence, stale job attempt, or mismatched generation and room IDs.
- Define a callback timestamp as stale when it is more than five minutes old or
  more than 30 seconds in the future. Require a strictly increasing event
  sequence within each job attempt.
- Return no generated question, correct answer, or explanation in the Modal
  Function result, and never include those values in Modal application logs.
- Give the Modal application no Discord credentials, participant sessions, or
  direct database access.

## Public Interfaces

### Shared Types

- `QuizConfig`: question count, answer duration, difficulty, topics, and focus.
- `TalkContext`: public title and public abstract or key takeaways, present only
  when focus is `tonights_talk`.
- `RoomPhase`: the supported lifecycle states.
- `QuestionPublic`: source, shuffled choices, topic, difficulty, published hint
  if any, and uniform verification badge.
- `QuestionSecret`: correct choice ID, unpublished hint, explanation, and
  verification evidence.
- `QuestionReveal`: the releasable answer, explanation, detailed verification
  receipt, and aggregate totals copied after reveal.
- `RoomView`: versioned, phase-specific state safe for organizer and participant
  clients, including the optional public talk title.

### HTTP Routes

- `GET /auth/discord` starts Discord authorization.
- `GET /auth/discord/callback` completes authorization and role verification.
- `POST /auth/logout` revokes the application session.
- `POST /api/host/rooms` creates a room and starts generation.
- `POST /api/host/rooms/:id/actions` applies a version-checked host action.
- `POST /api/rooms/:code/join` creates an anonymous room membership.
- `GET /api/rooms/:code/state?version=N` returns a sanitized room view.
- `PUT /api/rooms/:code/responses/:questionId` inserts or replaces an answer.
- `POST /internal/generation-events` records idempotent progress or failure
  events.
- `POST /internal/question-candidates` atomically accepts a verified candidate
  or returns `409 Conflict` with a non-secret rejection category for a duplicate
  or semantic near-match.

## Brand And Accessibility

- Make this section the normative brand contract; implementation must not depend
  on an absent repository style-guide or badge asset.
- Use a maximum-width 480px, single-column interface and a 4px spacing scale.
  Use 4px radii and 1px borders. Reduce panel padding from 24px to 16px at
  viewports 360px wide or narrower.
- Use Instrument Serif for headings at 24px or larger. Use a system monospace
  stack for 14px body text, controls, code, and data; reserve 12px text for
  nonessential metadata.
- Use `#f8f9fa` for the page, `#e9ecef` for panels, `#dee2e6` for inputs,
  `#2d3748` for primary text, `#4a5568` for secondary text, and `#cbd5e0` for
  borders.
- Use `#d69e2e` with `#2d3748` text for primary actions (`5.02:1`). The supplied
  `#b7791f` hover pairing fails AA for normal text; use `#945c0a` with white
  text for hover/active states (`5.53:1`) instead. Do not use `#718096` for
  essential 14px text because it fails AA on the supplied light surfaces.
- Use muted green `#38a169`, red `#e53e3e`, orange `#dd6b20`, and blue `#3182ce`
  only with independently tested foreground/background pairs and never as the
  sole status signal.
- Show the same category-neutral **Verified** badge before reveal. After reveal,
  replace or expand it with the detailed compiler/Miri receipt. No pre-reveal
  visual or accessibility label may identify the verification path.
- Meet WCAG 2.2 AA expectations for contrast, focus, keyboard navigation,
  semantic structure, reduced motion, and screen-reader announcements.
- Give interactive controls at least a 44px touch target. Wrap choice and button
  labels. Put source code in a labelled, keyboard-focusable horizontal scroll
  region so it never widens the page or overlaps timers and controls at 320px.
- Test all palette pairs automatically and capture Playwright screenshots at
  320px, 480px, and desktop widths. A future raster badge may be added to scoped
  Blob storage only after the asset is supplied and its contrast and accessible
  name are approved.

## Security And Correctness Invariants

These are release-blocking properties enforced by schemas, database boundaries,
and tests. They are not production metrics and do not require logging answers.

- Every accepted question has verifier evidence matching the pinned manifest and
  target answer category before the room can enter `ready`.
- Pre-reveal public tables, DTOs, HTML, JavaScript state, and network payloads
  contain no secret-only field or relationship that identifies which public
  choice is correct. Public choice text is expected to include the answer text.
- Public state-query code never reads `question_secrets`; only a successful
  reveal transaction copies approved fields into `question_reveals`.
- No accepted formatted-source hash or same-version AST fingerprint is reused.
- Every room transition, response replacement, participant admission, and
  candidate acceptance is transactional and version-checked.

## Success Metrics

- Across a 1,000,000-draw sampler-only simulation, compiler-error targets occur
  between 0.007% and 0.013%, undefined-behavior targets remain within 0.25
  percentage points of 20%, and deterministic-output targets make up the
  remainder.
- Across the same simulation, secure shuffling makes each final choice position
  correct within 0.25 percentage points of 20%, overall and grouped by room
  size. Topic and difficulty must not shift the configured target-category
  distribution.
- At least 95% of default three-question rooms become ready within five minutes.
- Answer submission latency remains below 500ms at p95.
- Reveal propagation reaches connected participants within two seconds at p95.
- A 200-participant room completes without lost or duplicated per-session
  responses and with fewer than 0.1% failed polling requests.
- The semantic-uniqueness fixture suite rejects at least 95% of known
  superficial variants while rejecting no more than 10% of intentionally
  distinct questions.
- In a blind sample of at least 30 generated questions rated independently by
  two Rust organizers, at least 90% are judged unambiguous and
  discussion-worthy, and the median judged difficulty differs from the requested
  level by no more than one.
- In a separate sample of at least 20 talk-mode questions, at least 90% are
  judged to have a clear, nontrivial connection to the supplied public talk
  context without requiring unavailable talk details.

## Pre-Implementation Readiness Gates

The PRD remains **Draft** until both spikes pass with recorded environment,
configuration, raw results, and p50/p95/p99 measurements.

### Val Town Capacity Spike

- Use a minimal representative polling endpoint and the intended val-scoped
  SQLite schema in a production-like Val Town preview.
- Run 200 participant clients for at least 15 minutes, polling every second
  during active play, replacing answers near deadlines, and exercising all host
  transitions.
- Require answer writes below 500ms at p95, reveal visibility within two seconds
  at p95, fewer than 0.1% failed polls, no persistent SQLite lock failures, and
  no lost or duplicated per-session response.
- If the spike fails, revise the transport or participant-facing host and rerun
  it. Do not lower the capacity target or mark the PRD ready without an explicit
  product decision.

### Generation Latency Spike

- Generate at least 100 default three-question rooms using representative
  release-candidate Pydantic models, Modal Image, Miri flags, cold/warm start
  mix, and a seeded history of at least 10,000 uniqueness records.
- Run slots concurrently with a maximum of five slot workers per room and a
  documented global concurrency cap.
- Measure queue/dispatch, model generation, Pydantic validation, Sandbox start,
  static policy checks, compilation, each Miri configuration, native runs,
  uniqueness, quality judging, callbacks, retries, and total room time.
- Use initial p95 stage budgets of 5 seconds for queue/dispatch, 30 seconds per
  model request, 5 seconds for Sandbox start, 2 seconds for static checks, 10
  seconds for compilation, 60 seconds for each Miri configuration, and 15
  seconds for uniqueness, quality, and callback processing.
- Require at least 95 rooms to reach `ready` within five minutes and zero rooms
  to accept an unverified question. If the gate fails, revise concurrency,
  verifier scope, retry budgets, or the five-minute target through an explicit
  product decision.

## Acceptance Tests

### Authentication And Security

- Successful Discord authorization with the configured guild and role.
- Rejection for wrong guild, wrong role, removed role, expired token, invalid
  state, reused state, callback CSRF, and Discord API failure.
- OAuth refresh, organizer-session expiry, immediate hard-denial invalidation,
  transient-outage grace actions, grace expiry, and prohibition of room creation
  during grace.
- HMAC callback validation, replay rejection, and event idempotency.
- Modal same-attempt submission idempotency, Function retry recovery, heartbeat
  timeout, stale-attempt replacement, three-job-attempt exhaustion, and
  rejection of late or mismatched callbacks.
- Pydantic model validation, malformed structured output, Gateway timeout,
  approved-route failover, spending-limit rejection, and exhausted model retry
  handling.
- Proof that Logfire traces and Gateway observability contain no prompts,
  completions, Rust source, answer choices, correct answers, or explanations.
- Proof that every Rust Sandbox blocks outbound networking, receives no secret
  or Volume mount, enforces resource limits, and is terminated after use.
- Canary-secret tests proving that pre-reveal public tables, HTML, JavaScript
  state, API responses, and polling payloads omit secret fields, correct-choice
  identity, explanation, unpublished hint, and private evidence. The public
  choice text itself is not treated as secret.
- Static and integration tests proving the public state-query module cannot read
  `question_secrets`, `show_hint` copies only the hint, and reveal copies only
  releasable fields after each transaction commits.

### Quiz Behavior

- Every room transition and invalid transition.
- Server deadlines, early close, hint publication, reveal, next question, and
  completion.
- Joining, reconnecting, late joining, duplicate submissions, and response
  replacement.
- Concurrent host actions using stale and current room versions.
- Case-insensitive room-code entry, collision retry, expiry, abandonment, owner
  reauthentication, non-owner rejection, participant cap, `room_full`, and
  same-session reconnection without another capacity charge.
- Hint publication without private preview and identical post-transition hint
  content for organizer and participant clients.
- General-Rust and talk-focus configuration, missing or oversized talk context,
  public title display, raw-context retention expiry, and actionable rejection
  when no standard-library-compatible concept can be derived.

### Verification

- Valid deterministic output, compiler error, and Miri-confirmed undefined
  behavior.
- Timeout, nondeterministic, malformed, oversized-output, mismatched-answer, and
  duplicate candidates.
- Rejection of hash-order, pointer-address, scheduling, weak-memory, and
  unspecified-debug-order fixtures; execution of the pinned Miri seed range.
- Stacked/Tree Borrows agreement for accepted undefined-behavior fixtures and
  rejection when the models disagree.
- Enforcement of the answer distribution: per-slot independence, room-size
  independence, the 79.99/0.01/20 semantic-label priors, uniform secure position
  shuffling, absence of additional category-correlated cues, and the defined
  empirical tolerances.
- Exact/AST duplicate rejection, semantic fixture thresholds, bounded top-K
  lookup, and dual-version fingerprint migration.
- Quality rubric, separate judge model, malformed judge output, difficulty
  mismatch, ambiguous-answer rejection, talk-relevance failure, and exhausted
  quality retries.
- Enforcement of line limits and prohibited capabilities.

### Experience And Capacity

- Separate Playwright organizer and participant contexts across mobile and
  desktop viewports.
- Keyboard, screen-reader, reduced-motion, contrast, and responsive-layout
  checks.
- Byte-identical pre-reveal verification-status subdocuments, badge markup, and
  accessible labels for every answer category.
- Automated contrast assertions for every token pair and screenshot checks at
  320px, 480px, and desktop widths, including focused code scrolling and wrapped
  longest-choice fixtures.
- A 200-participant load test using the production polling schedule.

## Delivery Requirements

- Develop each vertical slice with separate RED, GREEN, and DOCS commits.
- Require formatting, linting, type checking, unit tests, database tests,
  verifier tests, accessibility tests, Playwright tests, and load tests before
  release.
- Deploy Val Town and verifier preview environments for a facilitator rehearsal.
- Promote both components only after security and load acceptance criteria pass.
- Open a feature pull request containing deployment instructions, environment
  variables, the versioned Modal Image runbook, and rollback steps.

## Required Configuration

- `DISCORD_CLIENT_ID`
- `DISCORD_CLIENT_SECRET`
- `DISCORD_REDIRECT_URI`
- `DISCORD_GUILD_ID`
- `DISCORD_ORGANIZER_ROLE_ID`
- `SESSION_ENCRYPTION_KEY`
- `MODAL_GENERATION_URL`
- `MODAL_PROXY_KEY`
- `MODAL_PROXY_SECRET`
- `GENERATION_HMAC_SECRET`
- `VAL_TOWN_GENERATION_CALLBACK_URL` (Modal only)
- `PYDANTIC_AI_GATEWAY_API_KEY` (Modal only)
- `PYDANTIC_AI_GENERATOR_MODEL` (Modal only)
- `PYDANTIC_AI_JUDGE_MODEL` (Modal only)
- `PYDANTIC_AI_EMBEDDING_MODEL` (Modal only)
- `MODAL_RUST_IMAGE_NAME` (Modal only)
- `FINGERPRINT_ALGORITHM_VERSION`
- `SEMANTIC_TOP_K` (default `20`)
- `SEMANTIC_SIMILARITY_THRESHOLD` (default `0.82`)

## Assumptions

- Val Town Pro is available for private source, custom domains, and increased
  storage.
- Val Town is the participant-facing host; Modal is used only for candidate
  generation and isolated Rust verification.
- A Modal workspace and deployment credentials are available through Modal's
  Rust NYC sponsor relationship. The application still enforces explicit
  concurrency and resource limits.
- A Logfire organization and Pydantic AI Gateway project key are available
  through Pydantic's Rust NYC sponsor relationship. The Gateway project has
  explicit model allowlists and spending limits.
- The Rust East Coast guild ID and `nyc-organizers` role ID will be supplied as
  deployment configuration.
- Platform owners can inspect private infrastructure data. Quiz hosts receive no
  answer record before reveal but may infer the answer by reading the source.
- Exact and same-version AST duplicates are prohibited. Semantic repetition is
  reduced by measured heuristics and cannot be guaranteed absent.
- The 0.01% compiler-error target is an intentional, learnable answer prior and
  weakens the original repeat-attendee anti-gaming objective.
