use organizer_preview::{capture, App};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const BUSY: [&str; 4] = [
    "Loading the review…",
    "Recording…",
    "Preparing the edit…",
    "Saving the edit…",
];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

// A temp bank holding a copy of the review fixture, never bank/questions/.
fn temp_bank() -> PathBuf {
    let bank = std::env::temp_dir().join(format!("organizer-review-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&bank);
    fs::create_dir_all(bank.join("questions")).unwrap();
    fs::copy(
        repo().join("pipeline/tests/fixtures/review/questions/q4.json"),
        bank.join("questions/q4.json"),
    )
    .unwrap();
    bank
}

fn screen(app: &mut App) -> String {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let text = capture(app, 120, 400).unwrap();
        if !BUSY.iter().any(|busy| text.contains(busy)) {
            return text.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        assert!(Instant::now() < deadline, "review CLI did not finish");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn keys(app: &mut App, keys: &str) -> String {
    for key in keys.chars() {
        app.handle(key);
    }
    screen(app)
}

fn record(bank: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(bank.join("questions/q4.json")).unwrap()).unwrap()
}

#[test]
fn review_walks_accept_affirm_edit_and_reaffirm_through_the_cli() {
    std::env::remove_var("POPQUIZ_ORGANIZER");
    let bank = temp_bank();
    assert!(!bank.starts_with(repo().join("bank")));
    let mut app = App::load(&bank, Some("q4")).unwrap();
    let verified = record(&bank)["verified"].clone();

    let notice = keys(&mut app, "v");
    assert!(notice.contains("Review shows the answer"));
    assert!(!notice.contains("[correct, machine record]"));
    let shown = keys(&mut app, "\n");
    assert!(shown.contains("[correct, machine record]"));
    assert!(shown.contains("Affirmation: not affirmed"));

    assert!(keys(&mut app, "ay").contains("q4 accepted."));
    assert_eq!(record(&bank)["review"]["status"], "accepted");

    keys(&mut app, "\n");
    assert!(keys(&mut app, "d4y").contains("q4 judged difficulty 4"));
    assert_eq!(record(&bank)["review"]["difficulty_judged"], 4);

    // No name, no affirmation: the CLI refuses and the record is unchanged.
    keys(&mut app, "\n");
    let refused = keys(&mut app, "Ay");
    assert!(refused.contains("Not recorded."));
    assert!(refused.contains("Affirm needs your name"));
    assert!(record(&bank)["review"].get("affirmed_by").is_none());

    // The organizer sets a name, reopens review, and affirms.
    std::env::set_var("POPQUIZ_ORGANIZER", "Test Organizer");
    keys(&mut app, "\n");
    keys(&mut app, "\u{1b}");
    keys(&mut app, "v\n");
    let confirm = keys(&mut app, "A");
    assert!(confirm.contains("Organizer: Test Organizer"));
    assert!(keys(&mut app, "y").contains("q4 affirmed by Test Organizer"));
    assert!(keys(&mut app, "\n").contains("Affirmation: affirmed by Test Organizer"));

    // A prose edit, made where the organizer's editor would make it, clears it.
    std::env::set_var("VISUAL", "unused-by-this-test");
    keys(&mut app, "\u{1b}");
    keys(&mut app, "v\n");
    keys(&mut app, "e");
    let (_, path) = app.take_editor_request().expect("editor requested");
    let mut edit: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    edit["explains"]["what"] =
        format!("{} Edited.", edit["explains"]["what"].as_str().unwrap()).into();
    fs::write(&path, edit.to_string()).unwrap();
    app.editor_finished(true);
    assert!(screen(&mut app).contains("Save this edit?"));
    assert!(keys(&mut app, "y").contains("its affirmation is cleared"));
    assert!(!path.exists());
    let after = keys(&mut app, "\n");
    assert!(after.contains("Status: edited"));
    assert!(after.contains("Affirmation: not affirmed"));

    assert!(keys(&mut app, "Ay").contains("q4 affirmed by Test Organizer"));
    assert_eq!(record(&bank)["verified"], verified);

    keys(&mut app, "\n");
    assert!(keys(&mut app, "xToo close\ny").contains("q4 rejected: Too close"));
    assert_eq!(record(&bank)["review"]["status"], "rejected");
    std::env::remove_var("POPQUIZ_ORGANIZER");
    std::env::remove_var("VISUAL");
    fs::remove_dir_all(&bank).unwrap();

    let status = std::process::Command::new("git")
        .args(["status", "--porcelain", "bank/questions/"])
        .current_dir(repo())
        .output()
        .unwrap();
    assert!(status.stdout.is_empty(), "the real bank changed");
}

#[test]
fn the_tui_source_writes_no_file() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in fs::read_dir(src).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        for writer in [
            "fs::write",
            "File::create",
            "OpenOptions",
            "fs::copy",
            "fs::rename",
            "write_all",
        ] {
            assert!(!text.contains(writer), "{} uses {writer}", path.display());
        }
    }
}
