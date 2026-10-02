/* ===========================================================================
   The binding strings — SPEC §11, transcribed verbatim.

   "Authored here and only here." SPEC §11 is where copy is authored; this file
   is its port, and it is the module T-22's forbidden-copy lint (G-5, AC-98) and
   trope check (§11.1) run over. Every string gets its own key and a comment
   naming the §11 row it came from, so the lint has something stable to point at
   and a dropped string is visible.

   RULES FOR EDITING THIS FILE:

     - Do not paraphrase, and do not improve. These are written to be said by a
       host standing in front of people. Where the client wrote the line
       herself ("Time for a pop quiz.", "Let's go to the bar.") it beats
       anything written for her.
     - Do not author a new participant-facing string here. Author it in
       SPEC.md §11 first; this file follows.
     - Placeholders keep SPEC's own ‹guillemet› form so a diff against §11 is
       readable. fill() substitutes them.
     - Counts, not verdicts. No ✗, no score, no "wrong" (AC-94, and the
       Forbidden row). Correct gets a ✓ because that is a fact about the
       answer, not about a person.

   Casing is as §11 writes it, including the lower-case room lines ("answers
   are closed"), which is a voice decision (DESIGN.md), not an oversight.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  var COPY = {

    /* --- Wall, idle ----------------------------------------------------- */
    wall_idle_title: "Time for a pop quiz.",
    wall_idle_join: "join @ ‹link›",

    /* --- Wall, live ----------------------------------------------------- */
    wall_live_join: "join @ ‹link›",
    wall_live_well_header: "What does this program print?",

    /* --- Wall, trace (work, reveal) ------------------------------------- */
    wall_trace_step: "Step ‹N› of ‹M›",
    wall_trace_pivot: "Pause here.",

    /* --- Buzzer, join form ---------------------------------------------- */
    buzzer_join_label: "room code",
    buzzer_join_button: "join",

    /* --- Wall, closed --------------------------------------------------- */
    wall_closed: "answers are closed",

    /* --- Wall, split on ------------------------------------------------- */
    wall_split_answered: "‹answered› of ‹present› in the room answered",

    /* --- Wall, reveal --------------------------------------------------- */
    wall_reveal_most_chosen: "‹n› of us said ‹X›",

    /* --- Wall, released ------------------------------------------------- */
    wall_released_title: "Let's go to the bar.",
    wall_released_link: "‹link›",

    /* --- Buzzer, idle --------------------------------------------------- */
    buzzer_idle: "You're in.",

    /* --- Buzzer, reveal ------------------------------------------------- */
    buzzer_reveal_it_was: "✓ It was ‹Y›.",

    /* --- Buzzer, reveal, no answer given -------------------------------- */
    buzzer_noanswer_count_one: "You didn't answer.",

    /* --- Buzzer, join failures (AC-29) — six states, six next steps ------ */
    join_fail_malformed: "That's not a room code — six letters and numbers, never O, 0, I or 1. Try again.",
    join_fail_unknown: "No room with that code. Check the screen at the front.",
    join_fail_not_yet_open: "That room isn't open yet. Hold on — the host will put it on the screen.",
    join_fail_already_ended: "That room has ended. Look for the link on the screen.",
    join_fail_closed_inactivity: "That room went quiet and closed. If it comes back, the screen at the front will say so.",
    join_fail_full: "That room is full. Watch the screen — you can still play along.",

    /* --- Buzzer, hint --------------------------------------------------- */
    buzzer_hint_action: "Show me a hint",

    /* --- Buzzer, live, submission (AC-35/36) ---------------------------- */
    buzzer_saving: "saving…",
    buzzer_saved: "saved — ‹X›",
    buzzer_save_failed: "couldn't save. Your last answer, ‹X›, is safe.",
    buzzer_save_retry: "Try again",
    buzzer_no_answer_yet: "Vote",

    /* --- Buzzer, reconnecting (AC-37) ----------------------------------- */
    buzzer_reconnecting_with_answer: "paused — reconnecting… your answer ‹X› is safe",
    buzzer_reconnecting: "paused — reconnecting…",

    /* --- Buzzer, closed ------------------------------------------------- */
    buzzer_closed: "answers are closed",
    buzzer_closed_you_said: "you said ‹X›",
    buzzer_closed_you_didnt: "you didn't answer",

    /* --- Live region (AC-83), verbatim — the ten announcements ---------- */
    live_question_on_screen: "The question is on the screen.",
    live_saving: "Saving.",
    live_saved: "Saved, ‹X›.",
    live_save_failed: "Couldn't save; your last answer is safe.",
    live_answers_closed: "Answers are closed.",
    live_split_on_screen: "The room's split is on the screen.",
    live_walking_through: "Walking it through on the screen.",
    live_revealed: "Revealed: it was ‹Y›.",
    live_released: "The room is released.",
    live_hint_shown: "Hint shown, only to you.",

    /* --- Host, phase labels (AC-49) — announced, not shown --------------- */
    /* The host screen's heading is the wordmark on every phase; the label is
       what the host's live region announces on each phase change. */
    host_phase_idle: "before the question",
    host_phase_live: "question live",
    host_phase_closed: "answers closed",
    host_phase_split: "the split",
    host_phase_work: "walking it through",
    host_phase_reveal: "the answer",
    host_phase_released: "released",

    /* --- Take it home, headings in order -------------------------------- */
    home_heading_question: "‹date›'s question",
    home_heading_what: "What happens",
    home_heading_why: "Why you might have read it as ‹X›",
    home_heading_remember: "What to remember",
    home_heading_walk: "Walk it yourself",
    home_heading_how_we_know: "How we know",

    /* --- Host, actions -------------------------------------------------- */
    host_action_create: "Create a room",
    host_action_put_on_screen: "Start",
    host_action_close: "Close answers",
    host_action_show_split: "Show the room its split",
    host_action_walk: "Trace",
    host_action_reveal: "Reveal",
    host_action_release: "End Pop Quiz",
    host_action_run_again: "Run it again",

    /* --- Host, sign-in and denials (T-10, AC-70) ------------------------ */
    /* SPEC §11 has no row for these yet (F-33, routed upstream); the wording
       is the client's. The denials name the condition that failed and never
       use the Forbidden row's word for it. */
    host_action_sign_in: "Sign in with Discord",
    host_denied_wrong_server: "That Discord account isn't in the Rust NYC server.",
    host_denied_wrong_role: "That Discord account doesn't have the organizer role.",

    /* --- Wall + host, reveal, no incorrect votes ------------------------ */
    wall_reveal_nobody_else: "Nobody read it another way.",
    host_reveal_nobody_else: "Why nobody said anything else",

    /* --- Host, reveal — the three-beat script --------------------------- */
    host_reveal_read_aloud: "Read it aloud",
    host_reveal_beat_what: "What happens",
    host_reveal_beat_why: "Why ‹n› of us said ‹X›",
    host_reveal_beat_remember: "What to remember",

    /* --- Host, fit line (AC-100) — in the payload, not shown ----------- */
    /* One per fitVerdict() value. There is deliberately no entry for an
       unmeasured well — see typemodel.js fitVerdict(). */
    host_fit_fits: "fits the room",
    host_fit_clipped_y: "too long for this room — clipped at the bottom",
    host_fit_clipped_x: "too wide for this room — clipped at the right",
    host_fit_clipped_xy: "too long and too wide for this room",

    /* --- Title — the wordmark; on a phone, the surface's role beneath -- */
    title_wordmark: "Rust NYC Pop Quiz",
    host_title_role: "Host",
    buzzer_title_role: "Guest",

    /* --- Host + wall, count (AC-46) — Joined, then Answered ------------- */
    count_joined: "Joined: ‹n›",
    count_answered: "Answered: ‹n›",

    /* --- Static fallback (AC-102) --------------------------------------- */
    /* SPEC writes the key names in markdown code fencing; the backticks are
       its formatting, not punctuation the organizer reads off a screen — the
       same way `‹answered› of ‹present›…` drops its fencing above. */
    static_next_phase: "Space next phase",
    static_step_trace: "← → step the trace",
    static_back_phase: "Esc back a phase",

    /* --- Uniqueness (organizer-facing, AC-18) — never "original" -------- */
    uniqueness: "no exact or normalized duplicate found",

    /* --- The receipt — SPEC §7.5 ----------------------------------------
       A step list, not sentences. Each line carries a ✓ that marks the step as
       DONE, not passed: "Nothing ran" carries one too. A line renders only if
       the record holds the step it names (G-2), so the list can never claim
       more than was done.

       Two lists, and precedence is does-not-compile first, so every record
       renders exactly one of them and there is no third.

       No line of the wall receipt ends in terminal punctuation or contains
       "verified", "established", "proves", "always" or "guaranteed" (AC-43). */
    receipt_heading: "How we know",
    /* For a record that ran — complete or `legacy`. */
    receipt_compiled: "✓ Compiled",
    receipt_ran_n_times: "✓ Ran ‹N› times",
    receipt_output_never_varied: "✓ Output never varied",
    receipt_miri_clean: "✓ Miri ran clean",
    receipt_miri_ub: "✓ Miri flagged undefined behavior",
    /* For a does-not-compile record — nothing ran (D-22). */
    receipt_compiler_refused: "✓ Compiler refused it",
    receipt_error_codes: "✓ Error ‹codes›",
    receipt_nothing_ran: "✓ Nothing ran",

    /* --- Take it home, beyond the wall's list (§13, AC-71, AC-87) ------- */
    home_machine_only: "The machine checked the answer only. An organizer approved the explanation.",
    home_not_recorded: "not recorded",

    /* === PROPOSED-§11 ====================================================
       Strings a page already shows that SPEC §11 does not author. They were
       literals in the pages; T-22 moved them here WORDS UNCHANGED so that
       every participant-facing string is a key and both lints cover it. Each
       is routed upstream for §11 to adopt or replace — F-34 (take it home),
       F-38 (the step buttons), and three found by T-22's inventory — and the
       day §11 authors one, it leaves this block for its row. Add nothing here
       that a page does not already say; a new sentence goes to §11 first. */

    /* F-34: take it home, the page before the first release. */
    proposed_home_nothing_yet: "Welcome to the POP QUIZ",
    /* F-34: §13's "the Miri row saying the check was run separately". */
    proposed_home_miri_separately: "run separately",
    /* F-34: How we know, the machine's rows — the prototype's own labels. */
    proposed_home_row_compiler: "compiler",
    proposed_home_row_edition: "edition",
    proposed_home_row_target: "target",
    proposed_home_row_flags: "flags",
    proposed_home_row_miri: "miri",
    proposed_home_miri_seeds: "seeds",
    /* F-38: the trace's step buttons, as a screen reader names them. */
    proposed_trace_previous_step: "previous step",
    proposed_trace_next_step: "next step",
    /* T-22: a trace step's value change, spoken after the note (AC-83). */
    proposed_trace_value_now: "‹name› is now ‹now›",
    /* T-22: the screen-reader word beside a ✓ (check.js, AC-40). */
    proposed_check_correct: "Correct",
    /* T-22: a source well's region label when its caller names none. */
    proposed_well_label: "Source code"
  };

  /* Which §11 row each key came from. The completeness test walks this in both
     directions: every row has its keys, and every key belongs to a row. A
     string that appears here and not in COPY is a dropped transcription; one
     that appears in COPY and not here was authored somewhere other than §11,
     which is what §11's "authored here and only here" forbids. */
  var COPY_ROWS = {
    "Wall, idle": ["wall_idle_title", "wall_idle_join"],
    "Wall, live": ["wall_live_join", "wall_live_well_header"],
    "Wall, trace (work, reveal)": ["wall_trace_step", "wall_trace_pivot"],
    "Buzzer, join form": ["buzzer_join_label", "buzzer_join_button"],
    "Wall, closed": ["wall_closed"],
    "Wall, split on": ["wall_split_answered"],
    "Wall, reveal": ["wall_reveal_most_chosen"],
    "Wall, released": ["wall_released_title", "wall_released_link"],
    "Buzzer, idle": ["buzzer_idle"],
    "Buzzer, reveal": ["buzzer_reveal_it_was"],
    "Buzzer, reveal, no answer given": ["buzzer_noanswer_count_one"],
    "Buzzer, join failures (AC-29)": ["join_fail_malformed", "join_fail_unknown",
                       "join_fail_not_yet_open", "join_fail_already_ended",
                       "join_fail_closed_inactivity", "join_fail_full"],
    "Buzzer, hint": ["buzzer_hint_action"],
    "Buzzer, live, submission (AC-35/36)": ["buzzer_saving", "buzzer_saved", "buzzer_save_failed",
                       "buzzer_save_retry", "buzzer_no_answer_yet"],
    "Buzzer, reconnecting (AC-37)": ["buzzer_reconnecting_with_answer", "buzzer_reconnecting"],
    "Buzzer, closed": ["buzzer_closed", "buzzer_closed_you_said", "buzzer_closed_you_didnt"],
    "Live region (AC-83), verbatim": ["live_question_on_screen", "live_saving", "live_saved",
                       "live_save_failed", "live_answers_closed", "live_split_on_screen",
                       "live_walking_through", "live_revealed", "live_released", "live_hint_shown"],
    "Host, phase labels (AC-49)": ["host_phase_idle", "host_phase_live", "host_phase_closed",
                       "host_phase_split", "host_phase_work", "host_phase_reveal",
                       "host_phase_released"],
    "Take it home, headings in order": ["home_heading_question", "home_heading_what",
                       "home_heading_why", "home_heading_remember", "home_heading_walk",
                       "home_heading_how_we_know"],
    "Host, actions": ["host_action_create", "host_action_put_on_screen", "host_action_close",
                       "host_action_show_split", "host_action_walk", "host_action_reveal",
                       "host_action_release", "host_action_run_again"],
    "Host, sign-in and denials (T-10, F-33)": ["host_action_sign_in", "host_denied_wrong_server",
                       "host_denied_wrong_role"],
    "Wall + host, reveal, no incorrect votes": ["wall_reveal_nobody_else", "host_reveal_nobody_else"],
    "Host, reveal": ["host_reveal_read_aloud", "host_reveal_beat_what", "host_reveal_beat_why",
                       "host_reveal_beat_remember"],
    "Host, fit line (AC-100)": ["host_fit_fits", "host_fit_clipped_y", "host_fit_clipped_x",
                       "host_fit_clipped_xy"],
    "Title (host phone, buzzer, wall)": ["title_wordmark", "host_title_role", "buzzer_title_role"],
    "Host + wall, count (AC-46)": ["count_joined", "count_answered"],
    "Static fallback": ["static_next_phase", "static_step_trace", "static_back_phase"],
    "Uniqueness (organizer-facing, AC-18)": ["uniqueness"],
    "Receipt (§7.5)": ["receipt_heading", "receipt_compiled", "receipt_ran_n_times",
                       "receipt_output_never_varied", "receipt_miri_clean", "receipt_miri_ub",
                       "receipt_compiler_refused", "receipt_error_codes", "receipt_nothing_ran"],
    "Take it home (§13, AC-71, AC-87)": ["home_machine_only", "home_not_recorded"],
    /* Not a §11 row: the block above that is waiting for one. */
    "PROPOSED-§11 (not yet in SPEC)": ["proposed_home_nothing_yet", "proposed_home_miri_separately",
                       "proposed_home_row_compiler", "proposed_home_row_edition", "proposed_home_row_target",
                       "proposed_home_row_flags", "proposed_home_row_miri", "proposed_home_miri_seeds",
                       "proposed_trace_previous_step", "proposed_trace_next_step", "proposed_trace_value_now",
                       "proposed_check_correct", "proposed_well_label"]
  };

  /* The seven phase labels, in G-6 order, for a host screen that indexes by
     phase rather than by name. */
  var HOST_PHASE_LABEL = {
    idle: COPY.host_phase_idle,
    live: COPY.host_phase_live,
    closed: COPY.host_phase_closed,
    split: COPY.host_phase_split,
    work: COPY.host_phase_work,
    reveal: COPY.host_phase_reveal,
    released: COPY.host_phase_released
  };

  /* The host's fit line for a verdict from typemodel.js. Returns null for an
     unmeasured well rather than inventing a reassuring line for one. */
  var HOST_FIT_LINE = {
    fits: COPY.host_fit_fits,
    clipped_x: COPY.host_fit_clipped_x,
    clipped_y: COPY.host_fit_clipped_y,
    clipped_xy: COPY.host_fit_clipped_xy
  };
  function hostFitLine(verdict) {
    return Object.prototype.hasOwnProperty.call(HOST_FIT_LINE, verdict)
      ? HOST_FIT_LINE[verdict] : null;
  }

  /* Substitute ‹placeholders›. fill("‹n› of us said ‹X›", {n: 24, X: "A"}).
     Throws on a placeholder nobody supplied: a wall reading "‹n› of us said A"
     in front of the room is worse than a loud failure at the call site. */
  function fill(template, values) {
    values = values || {};
    return String(template).replace(/‹([^›]+)›/g, function (_, name) {
      if (!Object.prototype.hasOwnProperty.call(values, name)) {
        throw new Error('copy.fill: no value for ‹' + name + '› in "' + template + '"');
      }
      return String(values[name]);
    });
  }

  /* A key's string, or a loud failure. Never returns the key as a fallback —
     a surface rendering "buzzer_released" is a defect that should stop a test,
     not something the room reads. */
  function t(key, values) {
    if (!Object.prototype.hasOwnProperty.call(COPY, key)) {
      throw new Error("copy: no entry for " + JSON.stringify(key) + " (SPEC §11)");
    }
    return values === undefined ? COPY[key] : fill(COPY[key], values);
  }

  PQ.COPY = COPY;
  PQ.COPY_ROWS = COPY_ROWS;
  PQ.HOST_PHASE_LABEL = HOST_PHASE_LABEL;
  PQ.hostFitLine = hostFitLine;
  PQ.fill = fill;
  PQ.t = t;
})(typeof window !== "undefined" ? window : globalThis);
