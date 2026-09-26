//! SPEC.md §11, the binding strings — mirrored from `web/shared/copy.js`.
//!
//! One constant per `copy.js` key, named as the key upper-cased, with the same
//! string. SPEC §11 is where copy is authored; `copy.js` is its port; this file
//! mirrors the port so the room's payloads carry exactly those strings. T-22's
//! lint compares the two, and `tests/copy_mirror.rs` already walks them against
//! each other in both directions, so a string changed in one and not the other
//! fails `just test`.
//!
//! Do not edit a string here. Edit SPEC §11, then `copy.js`, then this file.
//! Placeholders keep §11's ‹guillemet› form; [`fill`] substitutes them.

pub const WALL_IDLE_TITLE: &str = "Time for a pop quiz.";
pub const WALL_IDLE_JOIN: &str = "join @ ‹link›";
pub const WALL_LIVE_JOIN: &str = "join @ ‹link› · still open";
pub const WALL_LIVE_WELL_HEADER: &str = "What does this program print?";
pub const WALL_TRACE_STEP: &str = "Step ‹N› of ‹M›";
pub const WALL_TRACE_PIVOT: &str = "Pause here.";
pub const BUZZER_JOIN_LABEL: &str = "room code";
pub const BUZZER_JOIN_BUTTON: &str = "join";
pub const BUZZER_JOIN_BENEATH: &str = "or open the link on the screen";
pub const WALL_CLOSED: &str = "answers are closed";
pub const WALL_SPLIT_ANSWERED: &str = "‹answered› of ‹present› in the room answered";
pub const WALL_WORK_LEAD: &str = "Let's walk it.";
pub const WALL_WORK_NO_ANSWER: &str = "Still no answer.";
pub const WALL_WORK_NOBODY: &str = "Nobody has to say anything.";
pub const WALL_REVEAL_MOST_CHOSEN: &str = "‹n› of us said ‹X›";
pub const WALL_RELEASED_TITLE: &str = "Let's go to the bar.";
pub const WALL_RELEASED_LINK: &str = "‹link›";
pub const WALL_RELEASED_LINE: &str = "the question, the walk-through and the why — at your own pace.";
pub const BUZZER_IDLE: &str = "You're in. Everything happens on the screen at the front — look up.";
pub const BUZZER_FOOT: &str = "no account · no name · no score";
pub const BUZZER_SPLIT_LOOKUP: &str = "Look up.";
pub const BUZZER_SPLIT_WHERE: &str = "Where the room landed.";
pub const BUZZER_SPLIT_COUNT: &str = "‹n›";
pub const BUZZER_SPLIT_SAID: &str = "people said ‹X›, including you.";
pub const BUZZER_SPLIT_READINGS: &str = "Five different readings. Nobody knows what anyone picked.";
pub const BUZZER_WORK_LOOKUP: &str = "Look up.";
pub const BUZZER_WORK_WALKING: &str = "We're walking it through.";
pub const BUZZER_WORK_NOTHING: &str = "Nothing to do. Nobody knows the answer yet.";
pub const BUZZER_REVEAL_LOOKUP: &str = "Look up.";
pub const BUZZER_REVEAL_ON_SCREEN: &str = "The answer is on the screen.";
pub const BUZZER_REVEAL_IT_WAS: &str = "✓ It was ‹Y›.";
pub const BUZZER_REVEAL_COMPANY: &str = "You and ‹n−1› other people read it the same way.";
pub const BUZZER_REVEAL_COMPANY_ONE: &str = "You and 1 other person read it the same way.";
pub const BUZZER_REVEAL_HOST_READING: &str = "The host is reading out the why now.";
pub const BUZZER_REVEAL_ONLY_ONE: &str = "You were the only one who read it that way.";
pub const BUZZER_NOANSWER_LOOKUP: &str = "Look up.";
pub const BUZZER_NOANSWER_COUNT: &str = "‹k› people didn't answer, you included.";
pub const BUZZER_NOANSWER_COUNT_ONE: &str = "You didn't answer.";
pub const BUZZER_NOANSWER_SPLIT: &str = "Where the room landed.";
pub const BUZZER_NOANSWER_WORK: &str = "We're walking it through.";
pub const BUZZER_NOANSWER_REVEAL: &str = "✓ It was ‹Y›. The host is reading out the why now.";
pub const JOIN_FAIL_MALFORMED: &str = "That's not a room code — six letters and numbers, never O, 0, I or 1. Try again.";
pub const JOIN_FAIL_UNKNOWN: &str = "No room with that code. Check the screen at the front.";
pub const JOIN_FAIL_NOT_YET_OPEN: &str = "That room isn't open yet. Hold on — the host will put it on the screen.";
pub const JOIN_FAIL_ALREADY_ENDED: &str = "That room has ended. Look for the link on the screen.";
pub const JOIN_FAIL_CLOSED_INACTIVITY: &str = "That room went quiet and closed. If it comes back, the screen at the front will say so.";
pub const JOIN_FAIL_FULL: &str = "That room is full. Watch the screen — you can still play along.";
pub const BUZZER_FOOT_COMPUTED: &str = "computed on this phone · never sent anywhere";
pub const BUZZER_RELEASED: &str = "Nothing about you was recorded.";
pub const BUZZER_HINT_ACTION: &str = "Show me a hint";
pub const BUZZER_HINT_SHOWN: &str = "Only you can see this. Nobody is told you looked.";
pub const BUZZER_SAVING: &str = "saving…";
pub const BUZZER_SAVED: &str = "saved — ‹X›";
pub const BUZZER_SAVE_FAILED: &str = "couldn't save. Your last answer, ‹X›, is safe.";
pub const BUZZER_SAVE_RETRY: &str = "Try again";
pub const BUZZER_NO_ANSWER_YET: &str = "tap a letter";
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
pub const HOST_ACTION_PUT_ON_SCREEN: &str = "Put it on the screen";
pub const HOST_ACTION_CLOSE: &str = "Close answers";
pub const HOST_ACTION_SHOW_SPLIT: &str = "Show the room its split";
pub const HOST_ACTION_WALK: &str = "Let's walk it";
pub const HOST_ACTION_REVEAL: &str = "Reveal";
pub const HOST_ACTION_RELEASE: &str = "Release the room";
pub const HOST_ACTION_RUN_AGAIN: &str = "Run it again";
pub const WALL_REVEAL_NOBODY_ELSE: &str = "Nobody read it another way.";
pub const HOST_REVEAL_NOBODY_ELSE: &str = "Why nobody said anything else";
pub const HOST_REVEAL_READ_ALOUD: &str = "Read it aloud";
pub const HOST_REVEAL_BEAT_WHAT: &str = "What happens";
pub const HOST_REVEAL_BEAT_WHY: &str = "Why ‹n› of us said ‹X›";
pub const HOST_REVEAL_BEAT_REMEMBER: &str = "What to remember";
pub const HOST_FIT_FITS: &str = "fits the room";
pub const HOST_FIT_CLIPPED_Y: &str = "too long for this room — clipped at the bottom";
pub const HOST_FIT_CLIPPED_X: &str = "too wide for this room — clipped at the right";
pub const HOST_FIT_CLIPPED_XY: &str = "too long and too wide for this room";
pub const HOST_FIRST_RESUME: &str = "If you lose this phone, open this on another one: ‹resume link›";
pub const NOT_A_GUARANTEE_OPTIONS_PUBLIC: &str = "Option text is public — the correct answer is always one of the five visible options.";
pub const NOT_A_GUARANTEE_HOST_HONEST: &str = "A host who reads Rust can work out the answer from the source; the host's not being shown it keeps the host honest, it is not a security guarantee.";
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
pub const HOME_MACHINE_ONLY: &str = "The machine checked the answer only. An organizer approved the explanation.";
pub const HOME_NOT_RECORDED: &str = "not recorded";

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
    ("buzzer_join_beneath", BUZZER_JOIN_BENEATH),
    ("wall_closed", WALL_CLOSED),
    ("wall_split_answered", WALL_SPLIT_ANSWERED),
    ("wall_work_lead", WALL_WORK_LEAD),
    ("wall_work_no_answer", WALL_WORK_NO_ANSWER),
    ("wall_work_nobody", WALL_WORK_NOBODY),
    ("wall_reveal_most_chosen", WALL_REVEAL_MOST_CHOSEN),
    ("wall_released_title", WALL_RELEASED_TITLE),
    ("wall_released_link", WALL_RELEASED_LINK),
    ("wall_released_line", WALL_RELEASED_LINE),
    ("buzzer_idle", BUZZER_IDLE),
    ("buzzer_foot", BUZZER_FOOT),
    ("buzzer_split_lookup", BUZZER_SPLIT_LOOKUP),
    ("buzzer_split_where", BUZZER_SPLIT_WHERE),
    ("buzzer_split_count", BUZZER_SPLIT_COUNT),
    ("buzzer_split_said", BUZZER_SPLIT_SAID),
    ("buzzer_split_readings", BUZZER_SPLIT_READINGS),
    ("buzzer_work_lookup", BUZZER_WORK_LOOKUP),
    ("buzzer_work_walking", BUZZER_WORK_WALKING),
    ("buzzer_work_nothing", BUZZER_WORK_NOTHING),
    ("buzzer_reveal_lookup", BUZZER_REVEAL_LOOKUP),
    ("buzzer_reveal_on_screen", BUZZER_REVEAL_ON_SCREEN),
    ("buzzer_reveal_it_was", BUZZER_REVEAL_IT_WAS),
    ("buzzer_reveal_company", BUZZER_REVEAL_COMPANY),
    ("buzzer_reveal_company_one", BUZZER_REVEAL_COMPANY_ONE),
    ("buzzer_reveal_host_reading", BUZZER_REVEAL_HOST_READING),
    ("buzzer_reveal_only_one", BUZZER_REVEAL_ONLY_ONE),
    ("buzzer_noanswer_lookup", BUZZER_NOANSWER_LOOKUP),
    ("buzzer_noanswer_count", BUZZER_NOANSWER_COUNT),
    ("buzzer_noanswer_count_one", BUZZER_NOANSWER_COUNT_ONE),
    ("buzzer_noanswer_split", BUZZER_NOANSWER_SPLIT),
    ("buzzer_noanswer_work", BUZZER_NOANSWER_WORK),
    ("buzzer_noanswer_reveal", BUZZER_NOANSWER_REVEAL),
    ("join_fail_malformed", JOIN_FAIL_MALFORMED),
    ("join_fail_unknown", JOIN_FAIL_UNKNOWN),
    ("join_fail_not_yet_open", JOIN_FAIL_NOT_YET_OPEN),
    ("join_fail_already_ended", JOIN_FAIL_ALREADY_ENDED),
    ("join_fail_closed_inactivity", JOIN_FAIL_CLOSED_INACTIVITY),
    ("join_fail_full", JOIN_FAIL_FULL),
    ("buzzer_foot_computed", BUZZER_FOOT_COMPUTED),
    ("buzzer_released", BUZZER_RELEASED),
    ("buzzer_hint_action", BUZZER_HINT_ACTION),
    ("buzzer_hint_shown", BUZZER_HINT_SHOWN),
    ("buzzer_saving", BUZZER_SAVING),
    ("buzzer_saved", BUZZER_SAVED),
    ("buzzer_save_failed", BUZZER_SAVE_FAILED),
    ("buzzer_save_retry", BUZZER_SAVE_RETRY),
    ("buzzer_no_answer_yet", BUZZER_NO_ANSWER_YET),
    ("buzzer_reconnecting_with_answer", BUZZER_RECONNECTING_WITH_ANSWER),
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
    ("wall_reveal_nobody_else", WALL_REVEAL_NOBODY_ELSE),
    ("host_reveal_nobody_else", HOST_REVEAL_NOBODY_ELSE),
    ("host_reveal_read_aloud", HOST_REVEAL_READ_ALOUD),
    ("host_reveal_beat_what", HOST_REVEAL_BEAT_WHAT),
    ("host_reveal_beat_why", HOST_REVEAL_BEAT_WHY),
    ("host_reveal_beat_remember", HOST_REVEAL_BEAT_REMEMBER),
    ("host_fit_fits", HOST_FIT_FITS),
    ("host_fit_clipped_y", HOST_FIT_CLIPPED_Y),
    ("host_fit_clipped_x", HOST_FIT_CLIPPED_X),
    ("host_fit_clipped_xy", HOST_FIT_CLIPPED_XY),
    ("host_first_resume", HOST_FIRST_RESUME),
    ("not_a_guarantee_options_public", NOT_A_GUARANTEE_OPTIONS_PUBLIC),
    ("not_a_guarantee_host_honest", NOT_A_GUARANTEE_HOST_HONEST),
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
];

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
        assert_eq!(fill(WALL_REVEAL_MOST_CHOSEN, &[("n", "24"), ("X", "A")]), "24 of us said A");
        assert_eq!(fill(WALL_IDLE_TITLE, &[]), "Time for a pop quiz.");
    }

    #[test]
    #[should_panic(expected = "no value for ‹n›")]
    fn fill_refuses_a_missing_value() {
        fill(WALL_REVEAL_MOST_CHOSEN, &[("X", "A")]);
    }
}
