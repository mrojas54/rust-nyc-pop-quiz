use organizer_preview::snapshot;

#[test]
fn selecting_a_step_waits_for_enter_before_changing_the_trace() {
    let screen = snapshot(&['t', 'j'], 120, 40);
    assert!(screen.contains("Steps / FOCUS"));
    assert!(screen.contains("> 2 Compare neighbours"));
    assert!(screen.contains("Step 1 of 6"));
    let jumped = snapshot(&['t', 'j', '\n'], 120, 40);
    assert!(jumped.contains("Step 2 of 6"));
    assert!(jumped.contains("Only neighbours"));
}

#[test]
fn tab_changes_pane_and_escape_goes_back_without_quitting() {
    let screen = snapshot(&['t', '\t'], 120, 40);
    assert!(screen.contains("Source (Rust) / FOCUS"));
    let screen = snapshot(&['t', '\t', '\u{1b}'], 120, 40);
    assert!(screen.contains("Steps / FOCUS"));
    let screen = snapshot(&['t', '\u{1b}'], 120, 40);
    assert!(screen.contains("What happens when this program runs?"));
    assert!(!screen.contains("Step 1 of 6"));
}

#[test]
fn list_navigation_cannot_enter_reveal() {
    let mut keys = vec!['t'];
    keys.extend(std::iter::repeat_n('j', 40));
    keys.push('\n');
    let screen = snapshot(&keys, 120, 40);
    assert!(screen.contains("Step 5 of 6"));
    assert!(screen.contains("6 Reveal / press r"));
    assert!(!screen.contains("Correct answer"));
}

#[test]
fn help_is_dismissible_and_does_not_navigate_underneath() {
    let screen = snapshot(&['t', '?', 'r'], 120, 40);
    assert!(screen.contains("Keyboard help"));
    assert!(!screen.contains("Correct answer"));
    let screen = snapshot(&['t', '?', '\u{1b}'], 120, 40);
    assert!(!screen.contains("Keyboard help"));
    assert!(screen.contains("Steps / FOCUS"));
}

#[test]
fn question_has_source_and_five_choices_without_reveal() {
    let screen = snapshot(&[], 100, 36);
    assert!(screen.contains("fn main()"));
    for letter in ['A', 'B', 'C', 'D', 'E'] {
        assert!(screen.contains(&format!("{letter}  ")));
    }
    assert!(!screen.contains("Correct answer"));
    assert!(!screen.contains("How we know"));
}

#[test]
fn teaching_trace_stops_before_the_resolving_step() {
    let mut keys = vec!['t'];
    keys.extend(std::iter::repeat_n('>', 30));
    let screen = snapshot(&keys, 100, 36);
    assert!(screen.contains("Step 5 of 6"));
    assert!(screen.contains("Pause here"));
    assert!(!screen.contains("Correct answer"));
    assert!(!screen.contains("stdout:"));
    keys.push('r');
    let screen = snapshot(&keys, 100, 60);
    assert!(screen.contains("Step 6 of 6"));
    assert!(screen.contains("Correct answer"));
    assert!(screen.contains("How we know"));
}

#[test]
fn trace_highlights_the_authored_step_and_shows_narration() {
    let screen = snapshot(&['t', '>', '>'], 100, 36);
    assert!(screen.contains(">  3"));
    assert!(screen.contains("Step 3 of 6"));
    assert!(screen.contains("Same value, side by side."));
}

#[test]
fn return_from_reveal_and_candidate_switch_reset_the_boundary() {
    let screen = snapshot(&['r', 't'], 100, 36);
    assert!(screen.contains("Step 5 of 6"));
    assert!(!screen.contains("Correct answer"));
    let screen = snapshot(&['r', ']'], 100, 36);
    assert!(screen.contains("q4"));
    assert!(!screen.contains("Correct answer"));
    assert!(!screen.contains("Step 6 of 6"));
}

#[test]
fn narrow_terminal_still_has_navigation() {
    let screen = snapshot(&['t'], 40, 14);
    assert!(screen.contains("q Quit"));
    assert!(screen.contains("Question"));
}

#[test]
fn preview_leaves_all_bank_files_unchanged() {
    let paths: Vec<_> = std::fs::read_dir(organizer_preview::default_bank().join("questions"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let before: Vec<_> = paths
        .iter()
        .map(|path| std::fs::read(path).unwrap())
        .collect();
    snapshot(&['t', '>', 'r', ']', '[', '1'], 100, 36);
    for (path, expected) in paths.iter().zip(before) {
        assert_eq!(std::fs::read(path).unwrap(), expected);
    }
}

#[test]
fn cli_reports_an_unknown_candidate_without_entering_terminal_mode() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_organizer-preview"))
        .args(["--question", "missing-candidate", "--snapshot", "trace"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("was not found"));
    assert!(!output.stdout.contains(&0x1b));
}
