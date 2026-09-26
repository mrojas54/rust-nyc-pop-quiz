//! The Rust twins of rules that already exist elsewhere, tested against the
//! same data: the receipt (`receipt.receipt_lines`, G-7), the correct-option
//! rule (`bank.correct_index`, G-2), and the copy (`web/shared/copy.js`, §11).

mod common;

use room::answers::{self, receipt_lines, verified_from_json, Record};
use room::copy;
use serde_json::Value;

#[test]
fn the_receipt_matches_every_shared_fixture() {
    let dir = common::repo().join("bank/fixtures/receipts");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("bank/fixtures/receipts")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    // A glob that silently matched nothing would pass every assertion below.
    assert!(files.len() >= 9, "found {} receipt fixtures", files.len());
    for path in files {
        let case: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let verified = verified_from_json(&case["verified"])
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let expected: Option<Vec<String>> = serde_json::from_value(case["expected_lines"].clone()).unwrap();
        assert_eq!(receipt_lines(&verified), expected, "{}", path.display());
    }
}

#[test]
fn no_receipt_line_claims_more_than_a_step() {
    // AC-43, as receipt.py states it beside the strings.
    for line in [
        copy::RECEIPT_COMPILED,
        copy::RECEIPT_RAN_N_TIMES,
        copy::RECEIPT_OUTPUT_NEVER_VARIED,
        copy::RECEIPT_MIRI_CLEAN,
        copy::RECEIPT_MIRI_UB,
        copy::RECEIPT_COMPILER_REFUSED,
        copy::RECEIPT_ERROR_CODES,
        copy::RECEIPT_NOTHING_RAN,
    ] {
        assert!(line.starts_with("✓ "), "{line}");
        assert!(!line.ends_with(['.', '!', '?']), "{line}");
        for word in ["verified", "established", "proves", "always", "guaranteed"] {
            assert!(!line.to_lowercase().contains(word), "{line}");
        }
    }
}

fn correct(v: &Value) -> Result<Option<usize>, answers::LoadError> {
    Record::from_json(&v.to_string())?.correct_index()
}

#[test]
fn the_correct_option_is_derived_the_way_bank_py_derives_it() {
    // q3 ran: the option equal to the verifier's stdout, less one newline.
    let q3 = common::q3_json();
    assert_eq!(correct(&q3).unwrap(), Some(4));
    assert_eq!(
        q3["verified"]["stdout"].as_str().unwrap().strip_suffix('\n'),
        q3["options"][4]["text"].as_str()
    );

    // Exactly one newline, not whitespace in general.
    let mut two_newlines = q3.clone();
    let stdout = format!("{}\n", two_newlines["verified"]["stdout"].as_str().unwrap());
    two_newlines["verified"]["stdout"] = stdout.into();
    assert_eq!(correct(&two_newlines).unwrap(), None);

    // Does not compile: the one does-not-compile option, whatever the texts.
    assert_eq!(correct(&common::planted_dnc()).unwrap(), Some(3));

    // No verified record: no answer.
    let mut unverified = q3.clone();
    unverified.as_object_mut().unwrap().remove("verified");
    assert_eq!(correct(&unverified).unwrap(), None);

    // Two options equal to the output name no single option.
    let mut twice = q3.clone();
    twice["options"][0]["text"] = twice["options"][4]["text"].clone();
    assert!(correct(&twice).is_err());
}

fn refused(v: Value, why: &str) {
    match answers::load(&v.to_string()) {
        Ok(_) => panic!("loaded a record that should be refused: {why}"),
        Err(e) => assert!(e.0.contains(why), "{e} (expected {why:?})"),
    }
}

#[test]
fn a_record_the_room_cannot_run_is_refused_at_load() {
    let q3 = common::q3_json();

    let mut four = q3.clone();
    four["options"].as_array_mut().unwrap().pop();
    refused(four, "has 4 options");

    let mut no_dnc = q3.clone();
    no_dnc["options"][3]["kind"] = "output".into();
    refused(no_dnc, "0 does-not-compile options");

    let mut unverified = q3.clone();
    unverified.as_object_mut().unwrap().remove("verified");
    refused(unverified, "no verified record");

    let mut no_receipt = q3.clone();
    no_receipt["verified"].as_object_mut().unwrap().remove("runs");
    refused(no_receipt, "no receipt");

    let mut drifted = q3.clone();
    drifted["options"][4]["text"] = "[5, 4, 3]".into();
    refused(drifted, "no option matches");

    let mut one_step = q3.clone();
    one_step["trace"]["steps"].as_array_mut().unwrap().truncate(1);
    refused(one_step, "at least two steps");

    let mut no_stdout_step = q3.clone();
    no_stdout_step["trace"]["steps"].as_array_mut().unwrap().pop();
    refused(no_stdout_step, "names no stdout");

    let mut untempting = q3.clone();
    untempting["options"][1].as_object_mut().unwrap().remove("why_tempting");
    refused(untempting, "option B has no why_tempting");

    let mut unknown = q3.clone();
    unknown["correct"] = 4.into();
    refused(unknown, "not a bank record");
}

#[test]
fn the_public_half_holds_nothing_sealed() {
    let q = common::load(&common::planted());
    let public = q.public();
    assert_eq!(public.id(), "planted");
    assert_eq!(public.trace_len().get(), 6);
    assert_eq!(public.walk().len(), 5, "steps 0..=M-2 only");
    let dump = format!("{public:?}");
    for plant in [common::PLANT_RESOLVING_NOTE, common::PLANT_WHAT, common::PLANT_TAKEAWAY, common::PLANT_MIDDLE_STDOUT] {
        assert!(!dump.contains(plant), "{plant} is in the public half");
    }
    for letter in ["A", "B", "C", "D", "E"] {
        assert!(!dump.contains(&common::plant_why(letter)));
    }
    assert!(!dump.contains("\"stdout\"") && !dump.contains("name: \"stdout\""));
    // Options stay in the order they arrived.
    let q3 = common::q3_json();
    for (i, text) in public.options().iter().enumerate() {
        assert_eq!(text, q3["options"][i]["text"].as_str().unwrap());
    }
}

/// `key: "value",` lines between `var COPY = {` and `var COPY_ROWS`.
fn copy_js() -> Vec<(String, String)> {
    let js = std::fs::read_to_string(common::repo().join("web/shared/copy.js")).unwrap();
    let start = js.find("var COPY = {").unwrap();
    let end = js.find("var COPY_ROWS").unwrap();
    js[start..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (key, rest) = line.split_once(": \"")?;
            if !key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                return None;
            }
            let value = rest.trim_end_matches(',').strip_suffix('"')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

#[test]
fn the_copy_module_mirrors_copy_js_in_both_directions() {
    let js = copy_js();
    assert!(js.len() >= 100, "parsed {} keys from copy.js", js.len());
    let rust: Vec<(String, String)> = copy::ALL
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert_eq!(rust, js, "room/src/copy.rs has drifted from web/shared/copy.js");
}

#[test]
fn the_phase_labels_and_host_actions_are_section_11s() {
    use room::phase::{HostAction, Phase};
    let labels: Vec<&str> = Phase::ALL.iter().map(|p| p.host_label()).collect();
    assert_eq!(
        labels,
        ["before the question", "question live", "answers closed", "the split", "walking it through", "the answer", "released"]
    );
    assert_eq!(HostAction::RunItAgain.label(), copy::HOST_ACTION_RUN_AGAIN);
    // The client's own lines, untouched.
    assert_eq!(copy::WALL_IDLE_TITLE, "Time for a pop quiz.");
    assert_eq!(copy::WALL_RELEASED_TITLE, "Let's go to the bar.");
}
