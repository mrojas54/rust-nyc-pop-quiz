//! T-12: take it home (SPEC.md §13) — the additive witnessed read, the
//! completed snapshot, and `GET /last`.
//!
//! - `answers::Revealed` now lends every option's `why_tempting` and the
//!   verified record's detail rows, through the same witness as everything else
//!   it lends: nothing new is reachable before `reveal`.
//! - `used::TakeHome` carries them: a middle beat for **every** incorrect
//!   option, and *How we know*'s machine rows. Still no count (AC-56, D-12).
//! - `GET /last` serves the page with the snapshot written into it.
//!
//! Two files are GENERATED here and never edited by hand; regenerate both with
//! `UPDATE_TAKE_HOME_FIXTURES=1 cargo test --test take_home`:
//!
//! - `room/tests/fixtures/take_home.shape.json` — the snapshot's shape for q3.
//! - `web/home/fixtures/take-home.json` — the snapshots `web/test/home.test.js`
//!   renders: q3 (a legacy record that ran), and two SYNTHETIC records built
//!   from q3's prose — a complete record (`bank/fixtures/receipts/complete-ran.json`'s
//!   verified record, whose stdout is a placeholder that says no program
//!   produced it) and a legacy does-not-compile record that no compiler
//!   produced and that says so. Nothing here writes down what a program prints.

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use common::*;
use room::phase::{Command, HostAction, Phase};
use room::question::Letter;
use room::rooms::{AppState, Urls};
use room::used::TakeHome;
use serde_json::{json, Value};
use tower::ServiceExt;

// --------------------------------------------------------------------------
// The questions.
// --------------------------------------------------------------------------

/// The SYNTHETIC placeholder `complete-ran.json` carries as its stdout, less
/// the one trailing newline `println!` would add — read from the file, so
/// nothing is typed here.
fn complete_verified() -> Value {
    let text = std::fs::read_to_string(repo().join("bank/fixtures/receipts/complete-ran.json")).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    v["verified"].clone()
}

/// q3's prose on a COMPLETE verified record (every SPEC §3.2 field). The
/// record describes no program; its correct option is set to the record's own
/// placeholder output so that `load` can derive it (G-2).
fn synthetic_complete() -> Value {
    let mut v = q3_json();
    v["id"] = "synthetic-complete".into();
    let verified = complete_verified();
    let stdout = verified["stdout"].as_str().unwrap();
    v["options"][4]["text"] = stdout.strip_suffix('\n').unwrap_or(stdout).into();
    v["verified"] = verified;
    v
}

/// q3's prose on a SYNTHETIC legacy does-not-compile record: no compiler ran,
/// and its `rustc` and its one error code say so. D is then the answer, and E
/// needs its own middle beat.
fn synthetic_dnc() -> Value {
    let mut v = q3_json();
    v["id"] = "synthetic-dnc".into();
    v["verified"] = json!({
        "rustc": "SYNTHETIC - no compiler ran",
        "edition": "2021",
        "legacy": true,
        "compile_error_code": ["ESYNTHETIC"],
    });
    v["options"][4]["why_tempting"] = v["options"][3]["why_tempting"].clone();
    v["options"][3].as_object_mut().unwrap().remove("why_tempting");
    v
}

/// On [`meetup_evening`]'s clock, so the snapshot's `meetup_date` — and the
/// generated fixtures — are the same on every day the suite runs.
fn state_with(questions: &[Value]) -> Arc<AppState> {
    let clock = room::lifecycle::ManualClock::new(meetup_evening());
    Arc::new(
        AppState::new(Arc::new(TestAuth), questions.iter().map(load).collect(), Urls::default()).with_clock(clock),
    )
}

/// Create a room on `question`, walk it to `reveal`, then release it.
fn release(state: &AppState, question: &str) -> String {
    let now = state.now();
    let room = state.create_room(Some(ORGANIZER), question, now).unwrap();
    for a in TO_REVEAL.iter().chain([&HostAction::ReleaseRoom]) {
        state.act(&room.id, Some(&room.host_session), Command::Host(*a), now).unwrap();
    }
    room.id
}

fn snapshot_after(question: &Value) -> TakeHome {
    let state = state_with(std::slice::from_ref(question));
    release(&state, question["id"].as_str().unwrap());
    state.take_home().unwrap()
}

// --------------------------------------------------------------------------
// The witnessed read.
// --------------------------------------------------------------------------

#[test]
fn the_new_reads_come_only_with_the_witness() {
    let state = state_with(&[q3_json()]);
    let now = state.now();
    let room = state.create_room(Some(ORGANIZER), "q3", now).unwrap();
    for phase in Phase::ALL {
        let lent = state
            .with_room(&room.id, |r| {
                r.open().map(|o| {
                    let why: Vec<bool> = o.revealed.why_tempting.iter().map(Option::is_some).collect();
                    (why, o.revealed.how_we_know.legacy, o.revealed.how_we_know.rustc.clone())
                })
            })
            .unwrap();
        if phase == Phase::Reveal {
            let (why, legacy, rustc) = lent.expect("reveal lends the reads");
            let correct = state.with_room(&room.id, |r| r.open().unwrap().revealed.correct).unwrap();
            for letter in Letter::ALL {
                assert_eq!(why[letter.index()], letter != correct, "{letter:?}");
            }
            assert!(legacy, "q3 is a legacy record (D-16)");
            assert_eq!(rustc, q3_json()["verified"]["rustc"].as_str().unwrap());
        } else {
            assert!(lent.is_none(), "{phase:?} lends nothing");
        }
        if phase == Phase::Released {
            break;
        }
        state
            .act(&room.id, Some(&room.host_session), Command::Host(phase.next_action()), now)
            .unwrap();
    }
}

// --------------------------------------------------------------------------
// The snapshot.
// --------------------------------------------------------------------------

#[test]
fn every_incorrect_option_has_its_beat_and_none_carries_a_count() {
    let q = q3_json();
    let home = snapshot_after(&q);
    let letters: Vec<Letter> = home.why.iter().map(|w| w.letter).collect();
    let want: Vec<Letter> = Letter::ALL.into_iter().filter(|l| *l != home.correct).collect();
    assert_eq!(letters, want, "one beat per incorrect option, in letter order (§13)");
    for w in &home.why {
        let i = w.letter.index();
        assert_eq!(w.text, q["options"][i]["text"].as_str().unwrap());
        assert_eq!(w.why_tempting, q["options"][i]["why_tempting"].as_str().unwrap());
    }
    // AC-56, D-12: nothing about the room, and no number outside the trace.
    let mut v = serde_json::to_value(&home).unwrap();
    v.as_object_mut().unwrap().remove("trace");
    let mut all = Vec::new();
    walk(&v, &mut all);
    for (k, value) in all {
        assert!(!["count", "counts", "totals", "split", "middle", "most_chosen", "percent"].contains(&k), "{k}");
        assert!(!value.is_number(), "a number under {k}");
    }
}

#[test]
fn a_legacy_record_holds_only_what_it_recorded() {
    let home = snapshot_after(&q3_json());
    let m = &home.machine;
    let verified = &q3_json()["verified"];
    assert!(m.legacy);
    assert_eq!(m.compiler, verified["rustc"].as_str().unwrap());
    assert_eq!(m.edition, verified["edition"].as_str().unwrap());
    assert_eq!(m.target, None, "never back-filled (D-16): /last reads *not recorded*");
    assert_eq!(m.flags, None);
    let miri = m.miri.as_ref().expect("the MVP's Miri pass is recorded");
    assert_eq!((&miri.version, &miri.configs, &miri.seeds), (&None, &None, &None), "run separately (D-16)");
}

#[test]
fn a_complete_record_carries_the_machine_verbatim() {
    let home = snapshot_after(&synthetic_complete());
    let v = complete_verified();
    let m = serde_json::to_value(&home.machine).unwrap();
    assert_eq!(m["legacy"], false);
    assert_eq!(m["compiler"], v["rustc"], "the full -Vv");
    assert_eq!(m["edition"], v["edition"]);
    assert_eq!(m["target"], v["target_triple"]);
    assert_eq!(m["flags"], v["flags"]);
    assert_eq!(m["miri"]["version"], v["miri"]["version"]);
    assert_eq!(m["miri"]["configs"], v["miri"]["configs"]);
    let seeds: Vec<String> = v["miri"]["seeds"].as_array().unwrap().iter().map(|s| s.to_string()).collect();
    assert_eq!(m["miri"]["seeds"], json!(seeds));
    assert_eq!(home.receipt.lines.len(), 4, "the wall's four-line list");
}

#[test]
fn a_does_not_compile_record_has_no_miri() {
    let home = snapshot_after(&synthetic_dnc());
    assert!(home.machine.legacy);
    assert_eq!(home.machine.miri, None, "nothing ran");
    assert_eq!(home.machine.target, None);
    assert_eq!(home.correct, Letter::D);
    assert_eq!(home.why.iter().map(|w| w.letter).collect::<Vec<_>>(), [Letter::A, Letter::B, Letter::C, Letter::E]);
    assert_eq!(home.receipt.lines.len(), 3, "the does-not-compile list");
}

// --------------------------------------------------------------------------
// The generated fixtures.
// --------------------------------------------------------------------------

/// The JSON type of every value; a list by its elements, merged.
fn shape(v: &Value) -> Value {
    match v {
        Value::Null => json!("null"),
        Value::Bool(_) => json!("boolean"),
        Value::Number(_) => json!("number"),
        Value::String(_) => json!("string"),
        Value::Array(items) => match items.iter().map(shape).reduce(|a, b| merge(&a, &b)) {
            None => json!([]),
            Some(s) => json!([s]),
        },
        Value::Object(map) => Value::Object(map.iter().map(|(k, v)| (k.clone(), shape(v))).collect()),
    }
}

fn merge(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Array(x), Value::Array(y)) if x.is_empty() => Value::Array(y.clone()),
        (Value::Array(x), Value::Array(_)) if b.as_array().unwrap().is_empty() => Value::Array(x.clone()),
        (Value::Array(x), Value::Array(y)) => json!([merge(&x[0], &y[0])]),
        (Value::Object(x), Value::Object(y)) => Value::Object(x.iter().map(|(k, v)| (k.clone(), merge(v, &y[k]))).collect()),
        (x, _) => x.clone(),
    }
}

fn check_generated(rel: &str, text: &str) {
    let path = repo().join(rel);
    if std::env::var_os("UPDATE_TAKE_HOME_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        on_disk == text,
        "{rel} is not what the room builds; regenerate with UPDATE_TAKE_HOME_FIXTURES=1 cargo test --test take_home"
    );
}

#[test]
fn the_fixtures_are_what_the_room_builds() {
    let q3 = serde_json::to_value(snapshot_after(&q3_json())).unwrap();

    let mut shape_doc = serde_json::Map::new();
    shape_doc.insert(
        "_note".into(),
        json!("GENERATED by room/tests/take_home.rs from AppState::take_home() for bank/questions/q3.json - never edit by hand; regenerate with UPDATE_TAKE_HOME_FIXTURES=1 cargo test --test take_home. Every key, and each value's JSON type; types only, no value from any program. q3 is a legacy record (D-16), so machine.target, machine.flags and machine.miri's three fields are null here; on a complete record they are a string, an object of {opt_level: string, overflow_checks: boolean, debug_assertions: boolean}, and a string, a list of strings and a list of strings. T-12 renders this at /last (SPEC 13). Lists show one element's shape."),
    );
    shape_doc.extend(shape(&q3).as_object().unwrap().clone());
    check_generated(
        "room/tests/fixtures/take_home.shape.json",
        &(serde_json::to_string_pretty(&Value::Object(shape_doc)).unwrap() + "\n"),
    );

    let web = json!({
        "_note": "GENERATED by room/tests/take_home.rs from AppState::take_home() - never edit by hand; regenerate with UPDATE_TAKE_HOME_FIXTURES=1 cargo test --test take_home. q3 is bank/questions/q3.json, a legacy record that ran. complete and dnc are SYNTHETIC: q3's prose on a complete verified record (bank/fixtures/receipts/complete-ran.json, whose output is a placeholder no program produced) and on a legacy does-not-compile record no compiler produced.",
        "q3": q3,
        "complete": serde_json::to_value(snapshot_after(&synthetic_complete())).unwrap(),
        "dnc": serde_json::to_value(snapshot_after(&synthetic_dnc())).unwrap(),
    });
    check_generated("web/home/fixtures/take-home.json", &(serde_json::to_string_pretty(&web).unwrap() + "\n"));
}

// --------------------------------------------------------------------------
// GET /last.
// --------------------------------------------------------------------------

async fn get(app: &Router, uri: &str) -> (StatusCode, String, String) {
    let res = app
        .clone()
        .oneshot(Request::builder().method(Method::GET).uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let ty = res
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 22).await.unwrap();
    (status, ty, String::from_utf8(bytes.to_vec()).unwrap())
}

/// The JSON the room wrote into the page's snapshot slot.
fn embedded(page: &str) -> Value {
    let open = "<script type=\"application/json\" id=\"take-home\">";
    let start = page.find(open).expect("the snapshot slot") + open.len();
    let end = start + page[start..].find("</script>").unwrap();
    serde_json::from_str(&page[start..end]).unwrap()
}

#[tokio::test]
async fn last_says_so_before_the_first_release() {
    let app = room::router_with(state_with(&[q3_json()]));
    let (status, ty, page) = get(&app, "/last").await;
    assert_eq!(status, StatusCode::OK);
    assert!(ty.starts_with("text/html"), "{ty}");
    assert_eq!(embedded(&page), Value::Null);
    assert!(page.contains("/home/home.js") && page.contains("/home/home.css"));
    for (uri, want) in [("/home/home.js", "text/javascript"), ("/home/home.css", "text/css")] {
        let (status, ty, body) = get(&app, uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert!(ty.starts_with(want), "{uri}: {ty}");
        assert!(!body.is_empty());
    }
}

#[tokio::test]
async fn last_is_the_snapshot_and_is_rebuilt_at_every_release() {
    let mut again = q3_json();
    again["id"] = "q3-again".into();
    let state = state_with(&[q3_json(), again]);
    let app = room::router_with(state.clone());

    release(&state, "q3");
    let (_, _, page) = get(&app, "/last").await;
    assert_eq!(embedded(&page), serde_json::to_value(state.take_home().unwrap()).unwrap());

    // A second room: while it runs, /last is still the first question…
    let now = state.now();
    let room = state.create_room(Some(ORGANIZER), "q3-again", now).unwrap();
    for a in TO_REVEAL {
        state.act(&room.id, Some(&room.host_session), Command::Host(a), now).unwrap();
        let (_, _, page) = get(&app, "/last").await;
        assert_eq!(embedded(&page)["question_id"], "q3", "{a:?}");
    }
    // …and at its release, the page is the new one.
    state.act(&room.id, Some(&room.host_session), Command::Host(HostAction::ReleaseRoom), now).unwrap();
    let (_, _, page) = get(&app, "/last").await;
    assert_eq!(embedded(&page)["question_id"], "q3-again");
}

#[tokio::test]
async fn the_snapshot_cannot_close_its_script_element() {
    let mut q = q3_json();
    q["options"][0]["why_tempting"] = "</script><script>alert(1)</script><!-- & \u{2028}".into();
    let state = state_with(std::slice::from_ref(&q));
    let app = room::router_with(state.clone());
    release(&state, "q3");
    let (_, _, page) = get(&app, "/last").await;
    assert_eq!(page.matches("</script>").count(), 8, "one close per script element in index.html");
    assert!(!page.contains("<script>alert") && !page.contains("<!--  &"));
    assert_eq!(embedded(&page), serde_json::to_value(state.take_home().unwrap()).unwrap(), "and it still round-trips");
}
