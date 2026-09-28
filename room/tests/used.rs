//! T-11: the used-question ledger and the take-it-home snapshot (G-10, AC-92,
//! SPEC.md §3.1 `used`, §13).
//!
//! A question is *used* when its room is released — not at build, not at
//! reveal, not when a room expires or goes quiet. The release transition is
//! the ledger's only writer, and a used question is never run twice.

mod common;

use std::time::Duration;

use common::*;
use room::lifecycle::Clock;
use room::phase::{HostAction, Phase};
use room::question::Letter;
use room::rooms::{Fit, RoomError, Urls};
use room::used;
use room::view;
use serde_json::{json, Value};

const USED_REFUSAL: &str = "That question has already been run. Pick another.";

// --------------------------------------------------------------------------
// AC-92 / G-10: written at release, and only then.
// --------------------------------------------------------------------------

#[test]
fn ac92_used_is_written_at_release() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    // The wall refits during the night; the last verdict is the one in reveal.
    c.state.record_fit(&room.id, Fit::Fits).unwrap();
    c.state.record_fit(&room.id, Fit::ClippedY).unwrap();
    c.advance(Duration::from_secs(4 * 60));
    let released = c.clock.now();
    c.walk(&room, &[HostAction::ReleaseRoom]);

    let all = c.state.used().all();
    assert_eq!(all.len(), 1);
    assert_eq!(
        serde_json::to_value(&all[0]).unwrap(),
        json!({
            "question_id": "q3",
            "used": {
                "meetup_date": MEETUP_DATE,
                "room_id": room.id,
                "released_at": used::rfc3339(released),
                "fit": "clipped_y",
            }
        })
    );
    assert!(c.state.used().contains("q3"));
    // A fit reported after release changes nothing: the record is written.
    c.state.record_fit(&room.id, Fit::ClippedXy).unwrap();
    assert_eq!(c.state.used().all()[0].used.fit, Some(Fit::ClippedY));
}

#[test]
fn ac92_a_room_the_wall_never_measured_records_no_fit() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    let v = serde_json::to_value(c.state.used().all()).unwrap();
    assert_eq!(v[0]["used"]["fit"], Value::Null, "nothing observed it, so nothing is written");
}

#[test]
fn ac92_nothing_is_written_before_release() {
    let c = Clocked::new();
    let room = c.create("q3");
    assert!(c.state.used().all().is_empty(), "creating a room is not using its question");
    for action in TO_REVEAL {
        c.walk(&room, &[action]);
        assert!(c.state.used().all().is_empty(), "{action:?}");
        assert!(!c.state.used().contains("q3"), "{action:?}");
        assert!(c.state.take_home().is_none(), "{action:?}: the snapshot is built at release only");
    }
    assert_eq!(c.state.with_room(&room.id, |r| r.phase()).unwrap(), Phase::Reveal);
}

#[test]
fn ac92_an_expired_or_quiet_room_records_nothing() {
    let c = Clocked::new();
    let quiet = c.create("q3");
    let expired = c.create("q3-again");
    c.walk(&expired, &TO_REVEAL);
    // Thirty quiet minutes: past idle's bound and reveal's.
    c.advance(room::lifecycle::IDLE_QUIET);
    assert_eq!(c.act(&expired, HostAction::ReleaseRoom), Err(RoomError::Ended(room::lifecycle::Ended::Inactive)));
    let mut gone = c.state.sweep(c.clock.now());
    gone.sort();
    let mut want = vec![quiet.id.clone(), expired.id.clone()];
    want.sort();
    assert_eq!(gone, want);
    assert!(c.state.used().all().is_empty());
    assert!(c.state.take_home().is_none());
    // Both questions are still in the reserve: a new room on either is fine.
    c.create("q3");
    c.create("q3-again");
}

#[test]
fn ac92_scheduling_questions_writes_nothing() {
    // Handing the room its questions — the room's side of a build — records
    // no use of any of them.
    let state = room::rooms::AppState::new(std::sync::Arc::new(TestAuth), vec![q3(), q3_again()], Urls::default());
    assert!(state.used().all().is_empty());
    assert!(state.take_home().is_none());
}

#[test]
fn g10_the_release_transition_is_the_only_writer() {
    let src = common::repo().join("room/src");
    let mut writers = Vec::new();
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap();
            if code.contains(".append(") || code.contains("UsedRecord {") && !code.contains("struct") && !code.contains("|UsedRecord") {
                writers.push((name.clone(), n + 1, code.trim().to_string()));
            }
        }
    }
    // One `append` and the one `UsedRecord` it appends, both in rooms.rs.
    assert_eq!(writers.len(), 2, "{writers:#?}");
    assert!(writers.iter().all(|(f, _, _)| f == "rooms.rs"), "{writers:#?}");
    assert!(writers.iter().any(|(_, _, c)| c.starts_with("self.used.append(")), "{writers:#?}");
    // And that call sits in `AppState::act`, behind the release transition's
    // parting, which only `Room::act`'s `Released` arm builds.
    let rooms = std::fs::read_to_string(src.join("rooms.rs")).unwrap();
    let act = rooms.rfind("    pub fn act(").expect("AppState::act");
    let append = rooms.find("self.used.append(").unwrap();
    let parting = rooms[act..].find("if let Some(parting) = room.take_parting()").map(|i| i + act);
    assert!(parting.is_some_and(|p| p < append), "the append is not behind take_parting");
    assert_eq!(rooms.matches("self.parting = Some(").count(), 1);
    let released_arm = rooms.find("Phase::Released => {").unwrap();
    let set = rooms.find("self.parting = Some(").unwrap();
    assert!(released_arm < set && set - released_arm < 1200, "the parting is built outside the Released arm");
    // The ledger's `append` is crate-private.
    let used = std::fs::read_to_string(src.join("used.rs")).unwrap();
    assert!(used.contains("pub(crate) fn append("));
}

#[test]
fn ac92_the_pipeline_makes_a_used_record_only_when_reading_the_bank() {
    // The laptop side writes no `used` of its own: the one construction in
    // `pipeline/src` is `bank.py`'s reader, which parses what the room wrote.
    let mut found = Vec::new();
    let mut stack = vec![common::repo().join("pipeline/src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("py") {
                let text = std::fs::read_to_string(&path).unwrap();
                // `Used(` as a whole word: not `_MostUsed(`.
                let whole = |l: &str| {
                    l.match_indices("Used(").any(|(i, _)| {
                        !l[..i].ends_with(|ch: char| ch.is_ascii_alphanumeric() || ch == '_')
                    })
                };
                for line in text.lines().filter(|l| whole(l)) {
                    found.push((path.file_name().unwrap().to_string_lossy().to_string(), line.trim().to_string()));
                }
            }
        }
    }
    assert_eq!(found, [("bank.py".to_string(), "else Used(".to_string())]);
}

#[test]
fn ac92_the_record_has_the_pipelines_shape() {
    let bank = std::fs::read_to_string(common::repo().join("pipeline/src/popquiz/bank.py")).unwrap();
    let class = &bank[bank.find("class Used:").expect("class Used")..];
    let body_end = class[1..].find("\n\n\n").map_or(class.len(), |i| i + 1);
    let mut fields: Vec<String> = class[..body_end]
        .lines()
        .skip(1)
        .filter_map(|l| {
            let l = l.strip_prefix("    ")?;
            let (name, _) = l.split_once(": ")?;
            name.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_').then(|| name.to_string())
        })
        .collect();
    fields.sort();
    assert_eq!(fields, ["fit", "meetup_date", "released_at", "room_id"], "bank.py's Used changed");

    let c = Clocked::new();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    c.state.record_fit(&room.id, Fit::ClippedX).unwrap();
    c.walk(&room, &[HostAction::ReleaseRoom]);
    let record = serde_json::to_value(&c.state.used().all()[0].used).unwrap();
    let mut keys: Vec<&String> = record.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, fields.iter().collect::<Vec<_>>());

    // `fit` serializes to one of bank.py's `Fit` literals.
    let literal = bank.lines().find(|l| l.starts_with("Fit = Literal[")).unwrap();
    for fit in [Fit::Fits, Fit::ClippedX, Fit::ClippedY, Fit::ClippedXy] {
        let name = serde_json::to_value(fit).unwrap();
        assert!(literal.contains(&format!("\"{}\"", name.as_str().unwrap())), "{name} not in {literal}");
    }
}

// --------------------------------------------------------------------------
// G-10: never twice.
// --------------------------------------------------------------------------

#[test]
fn g10_run_it_again_refuses_the_used_question() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    assert_eq!(
        c.state.run_again(&room.id, Some(ORGANIZER), "q3", c.clock.now()).err(),
        Some(RoomError::Refused(USED_REFUSAL.into()))
    );
    assert_eq!(
        c.state.create_room(Some(ORGANIZER), "q3", c.clock.now()).err(),
        Some(RoomError::Refused(USED_REFUSAL.into()))
    );
    assert_eq!(c.state.room_count(), 1, "no room was made");
}

#[test]
fn g10_run_it_again_on_an_unused_question_makes_a_new_room() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    let again = c.state.run_again(&room.id, Some(ORGANIZER), "q3-again", c.clock.now()).unwrap();
    assert_ne!(again.id, room.id);
    assert_ne!(again.code, room.code);
    assert_eq!(c.state.with_room(&again.id, |r| r.phase()).unwrap(), Phase::Idle);
    assert_eq!(c.state.with_room(&room.id, |r| r.phase()).unwrap(), Phase::Released);
    assert_eq!(c.state.used().all().len(), 1, "making a room uses nothing");
}

#[test]
fn g10_a_used_question_stays_used_after_its_room_is_gone() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    c.advance(room::rooms::ROOM_LIFETIME);
    c.state.sweep(c.clock.now());
    assert_eq!(c.state.room_count(), 0);
    assert_eq!(
        c.state.create_room(Some(ORGANIZER), "q3", c.clock.now()).err(),
        Some(RoomError::Refused(USED_REFUSAL.into()))
    );
}

// --------------------------------------------------------------------------
// §13: the take-it-home snapshot, rebuilt at every release.
// --------------------------------------------------------------------------

/// The JSON type of every value, arrays by their first element; each element
/// of a list must have that shape too.
fn shape(v: &Value) -> Value {
    match v {
        Value::Null => json!("null"),
        Value::Bool(_) => json!("boolean"),
        Value::Number(_) => json!("number"),
        Value::String(_) => json!("string"),
        Value::Array(items) => {
            let shapes: Vec<Value> = items.iter().map(shape).collect();
            match shapes.first() {
                None => json!([]),
                Some(first) => {
                    // Merge: an empty list inside an element takes a sibling's shape.
                    let mut merged = first.clone();
                    for s in &shapes[1..] {
                        merged = merge(&merged, s);
                    }
                    json!([merged])
                }
            }
        }
        Value::Object(map) => Value::Object(map.iter().map(|(k, v)| (k.clone(), shape(v))).collect()),
    }
}

fn merge(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Array(x), Value::Array(y)) if x.is_empty() => Value::Array(y.clone()),
        (Value::Array(x), Value::Array(y)) if y.is_empty() => Value::Array(x.clone()),
        (Value::Array(x), Value::Array(y)) => json!([merge(&x[0], &y[0])]),
        (Value::Object(x), Value::Object(y)) => {
            assert_eq!(x.keys().collect::<Vec<_>>(), y.keys().collect::<Vec<_>>(), "list elements differ in shape");
            Value::Object(x.iter().map(|(k, v)| (k.clone(), merge(v, &y[k]))).collect())
        }
        (x, y) => {
            assert_eq!(x, y, "list elements differ in type");
            x.clone()
        }
    }
}

#[test]
fn take_home_matches_the_fixture_shape() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    let snapshot = serde_json::to_value(c.state.take_home().unwrap()).unwrap();
    let text = std::fs::read_to_string(common::repo().join("room/tests/fixtures/take_home.shape.json")).unwrap();
    let mut fixture: Value = serde_json::from_str(&text).unwrap();
    fixture.as_object_mut().unwrap().remove("_note");
    assert_eq!(shape(&snapshot), fixture);
}

#[test]
fn take_home_is_the_question_as_the_reveal_showed_it() {
    let c = Clocked::new();
    let room = c.create("q3");
    let token = c.state.join(&room.code).unwrap().token;
    c.walk(&room, &[HostAction::PutOnScreen]);
    c.state.answer(&room.id, token.as_str(), Letter::A).unwrap();
    c.walk(&room, &TO_REVEAL[1..]);
    // What the room showed at reveal — the machine's reads, not ours.
    let urls = c.state.urls().clone();
    let wall = c.state.with_room(&room.id, |r| view::wall(r, &urls)).unwrap();
    let host = c.state.with_room(&room.id, view::host).unwrap();
    let reveal = wall.reveal.clone().unwrap();
    c.walk(&room, &[HostAction::ReleaseRoom]);

    let home = c.state.take_home().unwrap();
    assert_eq!(home.question_id, "q3");
    assert_eq!(home.meetup_date, MEETUP_DATE);
    assert_eq!(home.correct, reveal.correct);
    assert_eq!(home.mark, "✓");
    assert!(home.colour);
    assert_eq!(Some(&home.source), wall.source.as_ref());
    let options = wall.options.unwrap();
    assert_eq!(home.options.len(), 5);
    for (h, w) in home.options.iter().zip(&options) {
        assert_eq!((h.letter, &h.text), (w.letter, &w.text));
        assert_eq!(h.correct, h.letter == reveal.correct);
    }
    assert_eq!(home.options.iter().filter(|o| o.correct).count(), 1);
    assert_eq!(home.receipt.lines, reveal.receipt.lines);
    assert_eq!(home.receipt.heading, reveal.receipt.heading);
    let beats = host.read_aloud.unwrap();
    assert_eq!(Some(&home.what), beats.what.text.as_ref());
    assert_eq!(Some(&home.takeaway), beats.takeaway.text.as_ref());
    // The whole trace, resolving step included.
    let m = usize::from(wall.trace.unwrap().m);
    assert_eq!(home.trace.len(), m);
    assert!(home.trace[m - 1].values.iter().any(|v| v.name == "stdout"));
}

#[test]
fn take_home_is_rebuilt_at_each_release() {
    let c = Clocked::new();
    assert!(c.state.take_home().is_none(), "nothing before the first release");
    let first = c.create("q3");
    c.walk(&first, &TO_REVEAL);
    c.walk(&first, &[HostAction::ReleaseRoom]);
    assert_eq!(c.state.take_home().unwrap().question_id, "q3");
    let second = c.state.run_again(&first.id, Some(ORGANIZER), "q3-again", c.clock.now()).unwrap();
    assert_eq!(c.state.take_home().unwrap().question_id, "q3", "a new room changes nothing");
    c.walk(&second, &TO_REVEAL);
    assert_eq!(c.state.take_home().unwrap().question_id, "q3", "neither does its reveal");
    c.walk(&second, &[HostAction::ReleaseRoom]);
    assert_eq!(c.state.take_home().unwrap().question_id, "q3-again");
    assert_eq!(c.state.used().all().len(), 2);
}
