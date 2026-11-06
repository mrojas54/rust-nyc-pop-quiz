//! SPEC.md §11, the binding strings — mirrored from `web/shared/copy.js`.
//!
//! One constant per `copy.js` key, named as the key upper-cased, with the same
//! string. SPEC §11 is where copy is authored; `copy.js` is its port; this file
//! mirrors the port so the room's payloads carry exactly those strings.
//! `tests/twins.rs` walks the two against each other in both directions, so a
//! string changed in one and not the other fails `just test`; the lints (SPEC
//! §11's Forbidden row and §11.1) run over `copy.js` in `web/test/copylint.test.js`,
//! which makes them run over this file too.
//!
//! Do not edit a string here. Edit SPEC §11, then `copy.js`, then this file.
//! Placeholders keep §11's ‹guillemet› form; [`fill`] substitutes them.
//!
//! The one exception is the ROOM-ONLY block below [`ALL`]: reasons the room's
//! routes send that §11 does not author, kept here so no client-bound sentence
//! is written anywhere else (`tests/reason_copy.rs`).

pub const WALL_IDLE_TITLE: &str = "Time for a pop quiz.";
pub const WALL_IDLE_JOIN: &str = "join @ ‹link›";
pub const WALL_LIVE_JOIN: &str = "join @ ‹link›";
pub const WALL_LIVE_WELL_HEADER: &str = "What does this program print?";
pub const WALL_TRACE_STEP: &str = "Step ‹N› of ‹M›";
pub const WALL_TRACE_PIVOT: &str = "Pause here.";
pub const BUZZER_JOIN_LABEL: &str = "room code";
pub const BUZZER_JOIN_BUTTON: &str = "join";
pub const WALL_CLOSED: &str = "answers are closed";
pub const WALL_SPLIT_ANSWERED: &str = "‹answered› of ‹present› in the room answered";
pub const WALL_REVEAL_MOST_CHOSEN: &str = "‹n› of us said ‹X›";
pub const WALL_RELEASED_TITLE: &str = "Let's go to the bar.";
pub const WALL_RELEASED_LINK: &str = "‹link›";
pub const BUZZER_IDLE: &str = "You're in.";
pub const BUZZER_REVEAL_IT_WAS: &str = "✓ It was ‹Y›.";
pub const BUZZER_NOANSWER_COUNT_ONE: &str = "You didn't answer.";
pub const JOIN_FAIL_MALFORMED: &str =
    "That's not a room code — six letters and numbers, never O, 0, I or 1. Try again.";
pub const JOIN_FAIL_UNKNOWN: &str = "No room with that code. Check the screen at the front.";
pub const JOIN_FAIL_ALREADY_ENDED: &str = "That room has ended. Look for the link on the screen.";
pub const JOIN_FAIL_CLOSED_INACTIVITY: &str =
    "That room went quiet and closed. If it comes back, the screen at the front will say so.";
pub const JOIN_TRANSPORT_FAILED: &str =
    "Couldn't finish joining. Wait a moment. Try again — or watch the screen and play along.";
pub const JOIN_FAIL_FULL: &str = "That room is full. Watch the screen — you can still play along.";
pub const BUZZER_HINT_ACTION: &str = "Show me a hint";
pub const BUZZER_SAVING: &str = "saving…";
pub const BUZZER_SAVED: &str = "saved — ‹X›";
pub const BUZZER_SAVE_FAILED: &str = "couldn't save. Your last answer, ‹X›, is safe.";
pub const BUZZER_SAVE_RETRY: &str = "Try again";
pub const BUZZER_NO_ANSWER_YET: &str = "Vote";
pub const BUZZER_RECONNECTING_WITH_ANSWER: &str = "paused — reconnecting… your answer ‹X› is safe";
pub const BUZZER_RECONNECTING: &str = "paused — reconnecting…";
pub const BUZZER_CLOSED: &str = "answers are closed";
pub const BUZZER_CLOSED_YOU_SAID: &str = "you said ‹X›";
pub const BUZZER_CLOSED_YOU_DIDNT: &str = "you didn't answer";
pub const LIVE_QUESTION_ON_SCREEN: &str = "The question is on the screen.";
pub const LIVE_SAVING: &str = "Saving.";
pub const LIVE_SAVED: &str = "Saved, ‹X›.";
pub const LIVE_SAVE_FAILED: &str = "Couldn't save; your last answer is safe.";
pub const LIVE_ANSWERS_CLOSED: &str = "Answers are closed.";
pub const LIVE_SPLIT_ON_SCREEN: &str = "The room's split is on the screen.";
pub const LIVE_WALKING_THROUGH: &str = "Walking it through on the screen.";
pub const LIVE_REVEALED: &str = "Revealed: it was ‹Y›.";
pub const LIVE_RELEASED: &str = "The room is released.";
pub const LIVE_HINT_SHOWN: &str = "Hint shown, only to you.";
pub const HOST_PHASE_IDLE: &str = "before the question";
pub const HOST_PHASE_LIVE: &str = "question live";
pub const HOST_PHASE_CLOSED: &str = "answers closed";
pub const HOST_PHASE_SPLIT: &str = "the split";
pub const HOST_PHASE_WORK: &str = "walking it through";
pub const HOST_PHASE_REVEAL: &str = "the answer";
pub const HOST_PHASE_RELEASED: &str = "released";
pub const HOME_HEADING_QUESTION: &str = "‹date›'s question";
pub const HOME_HEADING_WHAT: &str = "What happens";
pub const HOME_HEADING_WHY: &str = "Why you might have read it as ‹X›";
pub const HOME_HEADING_REMEMBER: &str = "What to remember";
pub const HOME_HEADING_WALK: &str = "Walk it yourself";
pub const HOME_HEADING_HOW_WE_KNOW: &str = "How we know";
pub const HOST_ACTION_CREATE: &str = "Create a room";
pub const HOST_ACTION_PUT_ON_SCREEN: &str = "Start";
pub const HOST_ACTION_CLOSE: &str = "Close answers";
pub const HOST_ACTION_SHOW_SPLIT: &str = "Show the room its split";
pub const HOST_ACTION_WALK: &str = "Trace";
pub const HOST_ACTION_REVEAL: &str = "Reveal";
pub const HOST_ACTION_RELEASE: &str = "End Pop Quiz";
pub const HOST_ACTION_RUN_AGAIN: &str = "Run it again";
pub const HOST_ACTION_SIGN_IN: &str = "Sign in with Discord";
pub const HOST_DENIED_WRONG_SERVER: &str = "That Discord account isn't in the Rust NYC server.";
pub const HOST_DENIED_WRONG_ROLE: &str = "That Discord account doesn't have the organizer role.";
pub const WALL_REVEAL_NOBODY_ELSE: &str = "Nobody read it another way.";
pub const HOST_REVEAL_NOBODY_ELSE: &str = "Why nobody said anything else";
pub const HOST_REVEAL_READ_ALOUD: &str = "Read it aloud";
pub const HOST_REVEAL_BEAT_WHAT: &str = "What happens";
pub const HOST_REVEAL_BEAT_WHY: &str = "Why ‹n› of us said ‹X›";
pub const HOST_REVEAL_BEAT_REMEMBER: &str = "What to remember";
pub const HOST_REVEAL_PROVENANCE: &str = "An organizer approved this explanation.";
pub const HOST_FIT_FITS: &str = "fits the room";
pub const HOST_FIT_CLIPPED_Y: &str = "too long for this room — clipped at the bottom";
pub const HOST_FIT_CLIPPED_X: &str = "too wide for this room — clipped at the right";
pub const HOST_FIT_CLIPPED_XY: &str = "too long and too wide for this room";
pub const TITLE_WORDMARK: &str = "Rust NYC Pop Quiz";
pub const HOST_TITLE_ROLE: &str = "Host";
pub const BUZZER_TITLE_ROLE: &str = "Guest";
pub const COUNT_JOINED: &str = "Joined: ‹n›";
pub const COUNT_ANSWERED: &str = "Answered: ‹n›";
pub const STATIC_NEXT_PHASE: &str = "Space next phase";
pub const STATIC_STEP_TRACE: &str = "← → step the trace";
pub const STATIC_BACK_PHASE: &str = "Esc back a phase";
pub const UNIQUENESS: &str = "no exact or normalized duplicate found";
pub const RECEIPT_HEADING: &str = "How we know";
pub const RECEIPT_COMPILED: &str = "✓ Compiled";
pub const RECEIPT_RAN_N_TIMES: &str = "✓ Ran ‹N› times";
pub const RECEIPT_OUTPUT_NEVER_VARIED: &str = "✓ Output never varied";
pub const RECEIPT_MIRI_CLEAN: &str = "✓ Miri ran clean";
pub const RECEIPT_MIRI_UB: &str = "✓ Miri flagged undefined behavior";
pub const RECEIPT_COMPILER_REFUSED: &str = "✓ Compiler refused it";
pub const RECEIPT_ERROR_CODES: &str = "✓ Error ‹codes›";
pub const RECEIPT_NOTHING_RAN: &str = "✓ Nothing ran";
pub const HOME_MACHINE_ONLY: &str =
    "The machine checked the answer only. An organizer approved the explanation.";
pub const HOME_NOT_RECORDED: &str = "not recorded";
pub const LANDING_JOIN: &str = "Join a room";
pub const LANDING_LAST: &str = "Last meetup's question";
pub const LANDING_HOST: &str = "Are you a host? Click here";
pub const LANDING_KOAN: &str = "What is the sound of one value dropping?";

// PROPOSED-§11: strings the pages already showed that §11 does not author yet,
// moved into `copy.js` words unchanged by T-22 (F-34, F-38, and three more).
// Mirrored so the two files stay one list; the room sends none of them.
pub const PROPOSED_HOME_NOTHING_YET: &str = "Welcome to the POP QUIZ";
pub const PROPOSED_HOME_MIRI_SEPARATELY: &str = "run separately";
pub const PROPOSED_HOME_ROW_COMPILER: &str = "compiler";
pub const PROPOSED_HOME_ROW_EDITION: &str = "edition";
pub const PROPOSED_HOME_ROW_TARGET: &str = "target";
pub const PROPOSED_HOME_ROW_FLAGS: &str = "flags";
pub const PROPOSED_HOME_ROW_MIRI: &str = "miri";
pub const PROPOSED_HOME_MIRI_SEEDS: &str = "seeds";
pub const PROPOSED_TRACE_PREVIOUS_STEP: &str = "previous step";
pub const PROPOSED_TRACE_NEXT_STEP: &str = "next step";
pub const PROPOSED_TRACE_VALUE_NOW: &str = "‹name› is now ‹now›";
pub const PROPOSED_CHECK_CORRECT: &str = "Correct";
pub const PROPOSED_WELL_LABEL: &str = "Source code";

/// Every constant above with its `copy.js` key, in file order.
pub const ALL: &[(&str, &str)] = &[
    ("wall_idle_title", WALL_IDLE_TITLE),
    ("wall_idle_join", WALL_IDLE_JOIN),
    ("wall_live_join", WALL_LIVE_JOIN),
    ("wall_live_well_header", WALL_LIVE_WELL_HEADER),
    ("wall_trace_step", WALL_TRACE_STEP),
    ("wall_trace_pivot", WALL_TRACE_PIVOT),
    ("buzzer_join_label", BUZZER_JOIN_LABEL),
    ("buzzer_join_button", BUZZER_JOIN_BUTTON),
    ("wall_closed", WALL_CLOSED),
    ("wall_split_answered", WALL_SPLIT_ANSWERED),
    ("wall_reveal_most_chosen", WALL_REVEAL_MOST_CHOSEN),
    ("wall_released_title", WALL_RELEASED_TITLE),
    ("wall_released_link", WALL_RELEASED_LINK),
    ("buzzer_idle", BUZZER_IDLE),
    ("buzzer_reveal_it_was", BUZZER_REVEAL_IT_WAS),
    ("buzzer_noanswer_count_one", BUZZER_NOANSWER_COUNT_ONE),
    ("join_fail_malformed", JOIN_FAIL_MALFORMED),
    ("join_fail_unknown", JOIN_FAIL_UNKNOWN),
    ("join_fail_already_ended", JOIN_FAIL_ALREADY_ENDED),
    ("join_fail_closed_inactivity", JOIN_FAIL_CLOSED_INACTIVITY),
    ("join_transport_failed", JOIN_TRANSPORT_FAILED),
    ("join_fail_full", JOIN_FAIL_FULL),
    ("buzzer_hint_action", BUZZER_HINT_ACTION),
    ("buzzer_saving", BUZZER_SAVING),
    ("buzzer_saved", BUZZER_SAVED),
    ("buzzer_save_failed", BUZZER_SAVE_FAILED),
    ("buzzer_save_retry", BUZZER_SAVE_RETRY),
    ("buzzer_no_answer_yet", BUZZER_NO_ANSWER_YET),
    (
        "buzzer_reconnecting_with_answer",
        BUZZER_RECONNECTING_WITH_ANSWER,
    ),
    ("buzzer_reconnecting", BUZZER_RECONNECTING),
    ("buzzer_closed", BUZZER_CLOSED),
    ("buzzer_closed_you_said", BUZZER_CLOSED_YOU_SAID),
    ("buzzer_closed_you_didnt", BUZZER_CLOSED_YOU_DIDNT),
    ("live_question_on_screen", LIVE_QUESTION_ON_SCREEN),
    ("live_saving", LIVE_SAVING),
    ("live_saved", LIVE_SAVED),
    ("live_save_failed", LIVE_SAVE_FAILED),
    ("live_answers_closed", LIVE_ANSWERS_CLOSED),
    ("live_split_on_screen", LIVE_SPLIT_ON_SCREEN),
    ("live_walking_through", LIVE_WALKING_THROUGH),
    ("live_revealed", LIVE_REVEALED),
    ("live_released", LIVE_RELEASED),
    ("live_hint_shown", LIVE_HINT_SHOWN),
    ("host_phase_idle", HOST_PHASE_IDLE),
    ("host_phase_live", HOST_PHASE_LIVE),
    ("host_phase_closed", HOST_PHASE_CLOSED),
    ("host_phase_split", HOST_PHASE_SPLIT),
    ("host_phase_work", HOST_PHASE_WORK),
    ("host_phase_reveal", HOST_PHASE_REVEAL),
    ("host_phase_released", HOST_PHASE_RELEASED),
    ("home_heading_question", HOME_HEADING_QUESTION),
    ("home_heading_what", HOME_HEADING_WHAT),
    ("home_heading_why", HOME_HEADING_WHY),
    ("home_heading_remember", HOME_HEADING_REMEMBER),
    ("home_heading_walk", HOME_HEADING_WALK),
    ("home_heading_how_we_know", HOME_HEADING_HOW_WE_KNOW),
    ("host_action_create", HOST_ACTION_CREATE),
    ("host_action_put_on_screen", HOST_ACTION_PUT_ON_SCREEN),
    ("host_action_close", HOST_ACTION_CLOSE),
    ("host_action_show_split", HOST_ACTION_SHOW_SPLIT),
    ("host_action_walk", HOST_ACTION_WALK),
    ("host_action_reveal", HOST_ACTION_REVEAL),
    ("host_action_release", HOST_ACTION_RELEASE),
    ("host_action_run_again", HOST_ACTION_RUN_AGAIN),
    ("host_action_sign_in", HOST_ACTION_SIGN_IN),
    ("host_denied_wrong_server", HOST_DENIED_WRONG_SERVER),
    ("host_denied_wrong_role", HOST_DENIED_WRONG_ROLE),
    ("wall_reveal_nobody_else", WALL_REVEAL_NOBODY_ELSE),
    ("host_reveal_nobody_else", HOST_REVEAL_NOBODY_ELSE),
    ("host_reveal_read_aloud", HOST_REVEAL_READ_ALOUD),
    ("host_reveal_beat_what", HOST_REVEAL_BEAT_WHAT),
    ("host_reveal_beat_why", HOST_REVEAL_BEAT_WHY),
    ("host_reveal_beat_remember", HOST_REVEAL_BEAT_REMEMBER),
    ("host_reveal_provenance", HOST_REVEAL_PROVENANCE),
    ("host_fit_fits", HOST_FIT_FITS),
    ("host_fit_clipped_y", HOST_FIT_CLIPPED_Y),
    ("host_fit_clipped_x", HOST_FIT_CLIPPED_X),
    ("host_fit_clipped_xy", HOST_FIT_CLIPPED_XY),
    ("title_wordmark", TITLE_WORDMARK),
    ("host_title_role", HOST_TITLE_ROLE),
    ("buzzer_title_role", BUZZER_TITLE_ROLE),
    ("count_joined", COUNT_JOINED),
    ("count_answered", COUNT_ANSWERED),
    ("static_next_phase", STATIC_NEXT_PHASE),
    ("static_step_trace", STATIC_STEP_TRACE),
    ("static_back_phase", STATIC_BACK_PHASE),
    ("uniqueness", UNIQUENESS),
    ("receipt_heading", RECEIPT_HEADING),
    ("receipt_compiled", RECEIPT_COMPILED),
    ("receipt_ran_n_times", RECEIPT_RAN_N_TIMES),
    ("receipt_output_never_varied", RECEIPT_OUTPUT_NEVER_VARIED),
    ("receipt_miri_clean", RECEIPT_MIRI_CLEAN),
    ("receipt_miri_ub", RECEIPT_MIRI_UB),
    ("receipt_compiler_refused", RECEIPT_COMPILER_REFUSED),
    ("receipt_error_codes", RECEIPT_ERROR_CODES),
    ("receipt_nothing_ran", RECEIPT_NOTHING_RAN),
    ("home_machine_only", HOME_MACHINE_ONLY),
    ("home_not_recorded", HOME_NOT_RECORDED),
    ("landing_join", LANDING_JOIN),
    ("landing_last", LANDING_LAST),
    ("landing_host", LANDING_HOST),
    ("landing_koan", LANDING_KOAN),
    ("proposed_home_nothing_yet", PROPOSED_HOME_NOTHING_YET),
    (
        "proposed_home_miri_separately",
        PROPOSED_HOME_MIRI_SEPARATELY,
    ),
    ("proposed_home_row_compiler", PROPOSED_HOME_ROW_COMPILER),
    ("proposed_home_row_edition", PROPOSED_HOME_ROW_EDITION),
    ("proposed_home_row_target", PROPOSED_HOME_ROW_TARGET),
    ("proposed_home_row_flags", PROPOSED_HOME_ROW_FLAGS),
    ("proposed_home_row_miri", PROPOSED_HOME_ROW_MIRI),
    ("proposed_home_miri_seeds", PROPOSED_HOME_MIRI_SEEDS),
    ("proposed_trace_previous_step", PROPOSED_TRACE_PREVIOUS_STEP),
    ("proposed_trace_next_step", PROPOSED_TRACE_NEXT_STEP),
    ("proposed_trace_value_now", PROPOSED_TRACE_VALUE_NOW),
    ("proposed_check_correct", PROPOSED_CHECK_CORRECT),
    ("proposed_well_label", PROPOSED_WELL_LABEL),
];

// ROOM-ONLY: the `reason` sentences the room's routes send that §11 does not
// author (PQ-60, GAP-14). Moved out of the routes word for word; changing one
// is still a copy change (AC-98). Not in [`ALL`], so not mirrored in `copy.js`:
// no page renders them (the buzzer renders a refused answer from `phase` and
// `saved`, never `reason`), and a `copy.js` key must belong to a §11 row. The
// admin reasons reach the organizer through `popquiz schedule`'s error.
// `tests/reason_copy.rs` fails a sentence left as a literal in a route.

/// `PUT /rooms/{id}/answer`, 400: the letter is not one of the five.
pub const ANSWER_BAD_LETTER: &str = "An answer is one of A, B, C, D or E.";
/// `PUT /rooms/{id}/answer`, 409 before the question is on the screen.
pub const ANSWER_NOT_YET_OPEN: &str = "Answers open when the question is on the screen.";
/// The admin channel's refusals of a pushed record.
pub const ADMIN_NOT_A_CLUB: &str = "That is not a club name.";
pub const ADMIN_RECORD_TOO_LARGE: &str = "The record is larger than a question record can be.";
pub const ADMIN_RECORD_NOT_UTF8: &str = "The record is not UTF-8 JSON.";
pub const ADMIN_RECORD_NOT_OBJECT: &str = "The record is not a JSON object.";
pub const ADMIN_BANK_ORDER_NOT_INDEXES: &str = "‹id›: bank_order is not a list of option indexes.";
pub const ADMIN_PATH_NAMES_ANOTHER: &str = "The path names ‹id›; the record is ‹record›.";

/// [`ADMIN_BANK_ORDER_NOT_INDEXES`] for the record pushed as `id`.
pub fn admin_bank_order_not_indexes(id: &str) -> String {
    fill(ADMIN_BANK_ORDER_NOT_INDEXES, &[("id", id)])
}

/// [`ADMIN_PATH_NAMES_ANOTHER`]: the path's id, then the record's own.
pub fn admin_path_names_another(path_id: &str, record_id: &str) -> String {
    fill(
        ADMIN_PATH_NAMES_ANOTHER,
        &[("id", path_id), ("record", record_id)],
    )
}

// Why a bank record will not load (`answers::LoadError`). The admin channel
// sends it as its `reason`, and the organizer reads it in `popquiz schedule`'s
// error. Every one but the two parse errors is prefixed by [`LOAD_REFUSED`].

pub const LOAD_REFUSED: &str = "‹id›: ‹why›";
pub const LOAD_NOT_A_BANK_RECORD: &str = "not a bank record: ‹error›";
pub const LOAD_NOT_A_VERIFIED_RECORD: &str = "not a verified record: ‹error›";
pub const LOAD_SEVERAL_CORRECT: &str = "‹n› options match the verified answer (indexes ‹indexes›), so no single option is the correct one";
pub const LOAD_BANK_ORDER_NOT_ARRANGEMENT: &str =
    "bank_order is not an arrangement of the five options.";
pub const LOAD_NOT_ACCEPTED: &str = "‹id›: is affirmed but was not accepted at review; accept it at review, then schedule it again.";
pub const LOAD_OPTION_COUNT: &str = "has ‹n› options; a room needs five";
pub const LOAD_DNC_COUNT: &str = "has ‹n› does-not-compile options; it needs exactly one";
pub const LOAD_NO_VERIFIED: &str = "has no verified record, so it has no answer";
pub const LOAD_NO_RECEIPT: &str = "renders no receipt, so it cannot be scheduled";
pub const LOAD_NO_MATCH: &str = "no option matches the verified answer";
pub const LOAD_TRACE_TOO_SHORT: &str = "the trace needs at least two steps to walk";
pub const LOAD_TRACE_NO_STDOUT: &str = "the trace's final step names no stdout";
pub const LOAD_TRACE_EARLY_STDOUT: &str =
    "trace step ‹n› names stdout, but only the last step may, and only the verifier writes it";
pub const LOAD_NO_WHY_TEMPTING: &str = "option ‹letter› has no why_tempting text";
// `Review::check_shape` (affirmation.rs): the review block's digest is malformed.
pub const LOAD_DIGEST_MALFORMED: &str = "review.affirmed_digest is not sha256:<64 hex>";
pub const LOAD_DIGEST_WITHOUT_WHO: &str =
    "review.affirmed_digest is recorded without affirmed_by and affirmed_at";
// `Affirmation::refusal` (affirmation.rs): why an unaffirmed record will not
// run. The admin channel sends it to the organizer and the host route to the
// host page, both after [`LOAD_REFUSED`].
pub const LOAD_AFFIRMATION_STALE: &str =
    "its text changed after it was affirmed; affirm it again at review.";
pub const LOAD_AFFIRMATION_UNBOUND: &str =
    "its affirmation records who and when but not what was affirmed; affirm it again at review.";
pub const LOAD_AFFIRMATION_NONE: &str =
    "is not affirmed; affirm it at review, then schedule it again.";

/// [`LOAD_REFUSED`]: record `id`, then why.
pub fn load_refused(id: &str, why: &str) -> String {
    fill(LOAD_REFUSED, &[("id", id), ("why", why)])
}

/// [`LOAD_NOT_A_BANK_RECORD`] with the parser's error.
pub fn load_not_a_bank_record(error: &str) -> String {
    fill(LOAD_NOT_A_BANK_RECORD, &[("error", error)])
}

/// [`LOAD_NOT_A_VERIFIED_RECORD`] with the parser's error.
pub fn load_not_a_verified_record(error: &str) -> String {
    fill(LOAD_NOT_A_VERIFIED_RECORD, &[("error", error)])
}

/// [`LOAD_SEVERAL_CORRECT`] for the option indexes that all match.
pub fn load_several_correct(matches: &[usize]) -> String {
    fill(
        LOAD_SEVERAL_CORRECT,
        &[
            ("n", &matches.len().to_string()),
            ("indexes", &format!("{matches:?}")),
        ],
    )
}

/// [`LOAD_NOT_ACCEPTED`] for record `id`.
pub fn load_not_accepted(id: &str) -> String {
    fill(LOAD_NOT_ACCEPTED, &[("id", id)])
}

/// [`LOAD_OPTION_COUNT`] for a record with `n` options.
pub fn load_option_count(n: usize) -> String {
    fill(LOAD_OPTION_COUNT, &[("n", &n.to_string())])
}

/// [`LOAD_DNC_COUNT`] for a record with `n` does-not-compile options.
pub fn load_dnc_count(n: usize) -> String {
    fill(LOAD_DNC_COUNT, &[("n", &n.to_string())])
}

/// [`LOAD_TRACE_EARLY_STDOUT`] for trace step `n`, counted from one.
pub fn load_trace_early_stdout(n: usize) -> String {
    fill(LOAD_TRACE_EARLY_STDOUT, &[("n", &n.to_string())])
}

/// [`LOAD_NO_WHY_TEMPTING`] for the option shown as `letter`.
pub fn load_no_why_tempting(letter: &str) -> String {
    fill(LOAD_NO_WHY_TEMPTING, &[("letter", letter)])
}

// Why the room will not schedule a record or open a room (`RoomError::Refused`).
// The admin channel sends the first to the organizer; the host page shows the
// rest as the host route's `reason`.

pub const ROOM_RUNNING_CANNOT_REPLACE: &str =
    "A room is running ‹id›; it can be replaced once that room is gone.";
pub const ROOM_RUN_AGAIN_NEEDS_ORGANIZER: &str =
    "Run it again needs the organizer's credential and a question.";
pub const ROOM_QUESTION_OPEN_ELSEWHERE: &str =
    "That question is open in another room. Pick another.";
pub const ROOM_QUESTION_ALREADY_RUN: &str = "That question has already been run. Pick another.";
pub const ROOM_NOTHING_SCHEDULED: &str = "No question is scheduled with that id.";
/// SPEC §8.5, R1-a: the record was arranged for another meetup date.
pub const ROOM_QUESTION_OTHER_DATE: &str = "That question was scheduled for another date.";

/// [`ROOM_RUNNING_CANNOT_REPLACE`] for question `id`.
pub fn room_running_cannot_replace(id: &str) -> String {
    fill(ROOM_RUNNING_CANNOT_REPLACE, &[("id", id)])
}

// Why the phase machine refuses a host command (`phase::Refused`), and what a
// host command is told once the room has ended (`Ended::host_reason`). The host
// page shows each as the host route's `reason`.

pub const HOST_REFUSED_NO_ROOM_YET: &str = "There is no room yet. Create a room comes first.";
pub const HOST_REFUSED_ROOM_EXISTS: &str =
    "This room already exists. Create a room makes a separate one.";
pub const HOST_REFUSED_RELEASED: &str = "The room is released. Run it again starts a new room.";
pub const HOST_REFUSED_NOT_NEXT: &str = "‹action› isn't next. ‹next› comes next.";
pub const HOST_REFUSED_TRACE_STILL: &str =
    "The trace only steps while walking it through or at the answer.";
pub const HOST_REFUSED_FIRST_STEP: &str = "That's the first step.";
pub const HOST_REFUSED_LAST_STEP_BEFORE_REVEAL: &str = "That's the last step before the reveal.";
pub const HOST_REFUSED_LAST_STEP: &str = "That's the last step.";
pub const HOST_ENDED_EXPIRED: &str = "This room has ended.";
pub const HOST_ENDED_INACTIVE: &str = "This room closed for inactivity.";

/// [`HOST_REFUSED_NOT_NEXT`]: the refused action's label, then the next one's.
pub fn host_refused_not_next(action: &str, next: &str) -> String {
    fill(HOST_REFUSED_NOT_NEXT, &[("action", action), ("next", next)])
}

/// Substitute ‹placeholders›. Panics on a placeholder nobody supplied — the
/// same rule as `copy.js`'s `fill()`: a wall reading "‹n› of us said A" in front
/// of the room is worse than a loud failure at the call site.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('‹') {
        out.push_str(&rest[..open]);
        let after = &rest[open + '‹'.len_utf8()..];
        let close = after
            .find('›')
            .unwrap_or_else(|| panic!("copy::fill: unclosed placeholder in {template:?}"));
        let name = &after[..close];
        let value = values
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| *v)
            .unwrap_or_else(|| panic!("copy::fill: no value for ‹{name}› in {template:?}"));
        out.push_str(value);
        rest = &after[close + '›'.len_utf8()..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_substitutes_every_placeholder() {
        assert_eq!(
            fill(WALL_REVEAL_MOST_CHOSEN, &[("n", "24"), ("X", "A")]),
            "24 of us said A"
        );
        assert_eq!(fill(WALL_IDLE_TITLE, &[]), "Time for a pop quiz.");
    }

    #[test]
    fn the_admin_templates_fill_to_the_sentences_the_routes_sent() {
        // The text `format!` produced before PQ-60 moved it here.
        assert_eq!(
            admin_bank_order_not_indexes("q7"),
            "q7: bank_order is not a list of option indexes."
        );
        assert_eq!(
            admin_path_names_another("q7", "q3"),
            "The path names q7; the record is q3."
        );
    }

    #[test]
    fn the_load_and_room_templates_fill_to_the_sentences_they_replaced() {
        // Each against the `format!` it replaced, with the old literal, so a
        // character changed in the move fails here (AC-98).
        let (id, e, m) = ("q7", "expected value at line 1 column 1", vec![1, 3]);
        assert_eq!(load_refused(id, "x"), format!("{id}: {}", "x"));
        assert_eq!(load_not_a_bank_record(e), format!("not a bank record: {e}"));
        assert_eq!(
            load_not_a_verified_record(e),
            format!("not a verified record: {e}")
        );
        assert_eq!(
            load_several_correct(&m),
            format!(
                "{} options match the verified answer (indexes {:?}), so no single option is the correct one",
                m.len(),
                m
            )
        );
        assert_eq!(
            load_not_accepted(id),
            format!("{id}: is affirmed but was not accepted at review; accept it at review, then schedule it again.")
        );
        assert_eq!(
            load_option_count(4),
            format!("has {} options; a room needs five", 4)
        );
        let dnc = 2;
        assert_eq!(
            load_dnc_count(dnc),
            format!("has {dnc} does-not-compile options; it needs exactly one")
        );
        assert_eq!(
            load_trace_early_stdout(2),
            format!(
                "trace step {} names stdout, but only the last step may, and only the verifier writes it",
                2
            )
        );
        assert_eq!(
            load_no_why_tempting("B"),
            format!("option {} has no why_tempting text", "B")
        );
        assert_eq!(
            room_running_cannot_replace(id),
            format!("A room is running {id}; it can be replaced once that room is gone.")
        );
    }

    #[test]
    #[should_panic(expected = "no value for ‹n›")]
    fn fill_refuses_a_missing_value() {
        fill(WALL_REVEAL_MOST_CHOSEN, &[("X", "A")]);
    }
}
