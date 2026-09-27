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

// --------------------------------------------------------------------------
// Answers by kind (the F-19 ruling): the same cases as `test_bank.py`'s.
//
// Every `verified` block below is the verifier's own output, pasted verbatim:
// `popquiz.verify.verify` replayed on `StubRunner` over the named recording in
// `pipeline/tests/fixtures/verify/recordings/` (the accepted case in
// `cases.toml`), with `now` fixed as `test_verify.py` fixes it. Nothing here
// types a `stdout`, an `exit_code` or a Miri verdict; the options are authored.
// `recorded_blocks_are_the_pipelines_recordings` pins each block to its file.
// --------------------------------------------------------------------------

/// `q3.json`, verified with expect = "ran".
const Q3: &str = r#"{"rustc": "rustc 1.98.1 (48a229cea 2026-09-01)\nbinary: rustc\ncommit-hash: 48a229ceaefd4985c50990b14116b6d856af0985\ncommit-date: 2026-09-01\nhost: aarch64-unknown-linux-gnu\nrelease: 1.98.1\nLLVM version: 22.1.8", "edition": "2021", "target_triple": "aarch64-unknown-linux-gnu", "flags": {"opt_level": "0", "overflow_checks": true, "debug_assertions": true}, "runs": {"count": 5, "byte_identical": true}, "stdout": "[1, 2, 3, 2, 1]\n", "exit_code": 0, "miri": {"clean": true, "output_matched": true, "version": "miri 0.1.0 (420ed2a0c3 2026-09-18)", "configs": ["stacked_borrows"], "seeds": [0]}, "verified_at": "2026-09-26T00:00:00Z", "verifier_version": "popquiz-verify 7b1bc351e4dd"}"#;
/// `q8.json`, verified with expect = "does_not_compile".
const Q8: &str = r#"{"rustc": "rustc 1.98.1 (48a229cea 2026-09-01)\nbinary: rustc\ncommit-hash: 48a229ceaefd4985c50990b14116b6d856af0985\ncommit-date: 2026-09-01\nhost: aarch64-unknown-linux-gnu\nrelease: 1.98.1\nLLVM version: 22.1.8", "edition": "2021", "target_triple": "aarch64-unknown-linux-gnu", "flags": {"opt_level": "0", "overflow_checks": true, "debug_assertions": true}, "compile_error_code": ["E0502"], "verified_at": "2026-09-26T00:00:00Z", "verifier_version": "popquiz-verify 7b1bc351e4dd"}"#;
/// `panics.json`, verified with expect = "ran".
const PANICS: &str = r#"{"rustc": "rustc 1.98.1 (48a229cea 2026-09-01)\nbinary: rustc\ncommit-hash: 48a229ceaefd4985c50990b14116b6d856af0985\ncommit-date: 2026-09-01\nhost: aarch64-unknown-linux-gnu\nrelease: 1.98.1\nLLVM version: 22.1.8", "edition": "2021", "target_triple": "aarch64-unknown-linux-gnu", "flags": {"opt_level": "0", "overflow_checks": true, "debug_assertions": true}, "runs": {"count": 5, "byte_identical": true}, "stdout": "counting\n", "exit_code": 101, "miri": {"clean": true, "output_matched": true, "version": "miri 0.1.0 (420ed2a0c3 2026-09-18)", "configs": ["stacked_borrows"], "seeds": [0]}, "verified_at": "2026-09-26T00:00:00Z", "verifier_version": "popquiz-verify 7b1bc351e4dd"}"#;
/// `ub-both.json`, verified with expect = "ub".
const UB_BOTH: &str = r#"{"rustc": "rustc 1.98.1 (48a229cea 2026-09-01)\nbinary: rustc\ncommit-hash: 48a229ceaefd4985c50990b14116b6d856af0985\ncommit-date: 2026-09-01\nhost: aarch64-unknown-linux-gnu\nrelease: 1.98.1\nLLVM version: 22.1.8", "edition": "2021", "target_triple": "aarch64-unknown-linux-gnu", "flags": {"opt_level": "0", "overflow_checks": true, "debug_assertions": true}, "runs": {"count": 5, "byte_identical": true}, "stdout": "before\n", "exit_code": 0, "miri": {"clean": false, "output_matched": true, "version": "miri 0.1.0 (420ed2a0c3 2026-09-18)", "configs": ["stacked_borrows", "tree_borrows"], "seeds": [0]}, "verified_at": "2026-09-26T00:00:00Z", "verifier_version": "popquiz-verify 7b1bc351e4dd"}"#;

fn block(json: &str) -> Value {
    serde_json::from_str(json).expect("a verified block")
}

/// A record with one option of every kind (bank's `_KINDED`) and `verified`.
fn kinded(verified: Value) -> Value {
    let mut v = common::q3_json();
    v["options"] = serde_json::json!([
        { "text": "one", "kind": "output", "why_tempting": "authored" },
        { "text": "two", "kind": "output", "why_tempting": "authored" },
        { "text": "it panics", "kind": "panic", "why_tempting": "authored" },
        { "text": "undefined behaviour", "kind": "ub", "why_tempting": "authored" },
        { "text": "does not compile", "kind": "does_not_compile", "why_tempting": "authored" },
    ]);
    v["verified"] = verified;
    v
}

/// A block's `stdout` as an option spells it, read from the block.
fn printed(verified: &Value) -> String {
    let stdout = verified["stdout"].as_str().expect("a record that ran");
    stdout.strip_suffix('\n').unwrap_or(stdout).to_string()
}

#[test]
fn a_record_that_does_not_compile_derives_that_option_by_kind() {
    // q8's replay; the option set puts does-not-compile last, as bank's does.
    assert_eq!(correct(&kinded(block(Q8))).unwrap(), Some(4));
}

#[test]
fn a_panic_derives_the_panic_option_by_kind() {
    assert_eq!(correct(&kinded(block(PANICS))).unwrap(), Some(2));
}

#[test]
fn a_panic_is_never_the_output_option_even_when_the_text_matches() {
    // What the program printed before it panicked, as an output option.
    let panics = block(PANICS);
    let mut q = kinded(panics.clone());
    q["options"][1]["text"] = printed(&panics).into();
    assert_eq!(correct(&q).unwrap(), Some(2));
}

#[test]
fn a_panic_with_no_panic_option_derives_nothing_rather_than_the_output() {
    // A rule that fires is final: it does not fall through to text equality.
    let panics = block(PANICS);
    let mut q = common::q3_json();
    q["options"][0]["text"] = printed(&panics).into();
    q["verified"] = panics;
    assert_eq!(correct(&q).unwrap(), None);
}

#[test]
fn ub_under_both_borrow_models_derives_the_ub_option() {
    assert_eq!(correct(&kinded(block(UB_BOTH))).unwrap(), Some(3));
}

#[test]
fn ub_outranks_a_nonzero_exit() {
    // ub-both's record with the panics record's exit code: copied, not typed.
    let mut ub = block(UB_BOTH);
    ub["exit_code"] = block(PANICS)["exit_code"].clone();
    assert_eq!(correct(&kinded(ub)).unwrap(), Some(3));
}

#[test]
fn ub_under_one_borrow_model_is_not_a_ub_answer() {
    // The verifier never accepts this shape (ub-sb-only.json is rejected as
    // `borrow_models_disagree`), so no machine record of it exists: this is
    // ub-both's record with `tree_borrows` taken out of `configs`. It is not
    // UB by kind, it exited zero, so it falls through to rule 4, the output
    // text: nothing when no option prints it (bank's case) ...
    let mut one = block(UB_BOTH);
    one["miri"]["configs"] = serde_json::json!(["stacked_borrows"]);
    assert_eq!(correct(&kinded(one.clone())).unwrap(), None);
    // ... and the output option when one does.
    let mut q = kinded(one.clone());
    q["options"][1]["text"] = printed(&one).into();
    assert_eq!(correct(&q).unwrap(), Some(1));
}

#[test]
fn an_unclean_miri_pass_with_no_configs_is_not_read_as_confirmed_ub() {
    // A legacy record names no models, so which agreed is unknown and nothing
    // is inferred (D-16): ub-both's record with `configs` removed.
    let mut legacy = block(UB_BOTH);
    legacy["miri"].as_object_mut().unwrap().remove("configs");
    assert_eq!(correct(&kinded(legacy)).unwrap(), None);
}

#[test]
fn two_options_of_the_derived_kind_is_an_error() {
    let mut q = kinded(block(PANICS));
    q["options"][1] = serde_json::json!({ "text": "panics", "kind": "panic", "why_tempting": "authored" });
    let err = correct(&q).unwrap_err();
    assert!(err.0.contains("options match"), "{err}");

    let mut q = kinded(block(UB_BOTH));
    q["options"][0]["kind"] = "ub".into();
    assert!(correct(&q).is_err());
}

fn recording(name: &str) -> Value {
    let path = common::repo().join(format!("pipeline/tests/fixtures/verify/recordings/{name}.json"));
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

#[test]
fn recorded_blocks_are_the_pipelines_recordings() {
    // The cross-language pin. The blocks above are the pipeline's fixture set,
    // replayed; re-record one on the pipeline side and this fails until the
    // block is replayed again, so neither suite can move alone.
    for (name, verified) in [("q3", Q3), ("panics", PANICS), ("ub-both", UB_BOTH)] {
        let (rec, v) = (recording(name), block(verified));
        let runs: Vec<&Value> = rec
            .as_object()
            .unwrap()
            .iter()
            .filter(|(k, _)| k.starts_with("run:"))
            .map(|(_, r)| r)
            .collect();
        assert_eq!(runs.len() as u64, v["runs"]["count"].as_u64().unwrap(), "{name}");
        for run in runs {
            assert_eq!(run["stdout"], v["stdout"], "{name}");
            assert_eq!(run["exit_code"], v["exit_code"], "{name}");
        }
        let mut models: Vec<&str> = rec
            .as_object()
            .unwrap()
            .keys()
            .filter_map(|k| k.strip_prefix("miri:")?.split(':').next())
            .collect();
        let mut configs: Vec<&str> = v["miri"]["configs"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
        models.sort();
        models.dedup();
        configs.sort();
        assert_eq!(models, configs, "{name}");
    }
    let (rec, v) = (recording("q8"), block(Q8));
    assert_ne!(rec["compile"]["exit_code"], 0);
    assert!(rec.get("run:1").is_none(), "q8 never ran");
    for code in v["compile_error_code"].as_array().unwrap() {
        let wanted = format!("error[{}]", code.as_str().unwrap());
        assert!(rec["compile"]["stderr"].as_str().unwrap().contains(&wanted), "{wanted}");
    }
    // And the bank both sides read: every question in it names one option.
    let mut files: Vec<_> = std::fs::read_dir(common::repo().join("bank/questions"))
        .expect("bank/questions")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    assert!(files.len() >= 4, "found {} bank questions", files.len());
    for path in files {
        let q: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(correct(&q).unwrap().is_some(), "{}", path.display());
    }
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
    // A floor against a parser that silently matches nothing; PQ-34 took the
    // module from 120 keys to 90.
    assert!(js.len() >= 80, "parsed {} keys from copy.js", js.len());
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
