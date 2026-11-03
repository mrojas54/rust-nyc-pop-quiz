//! G-5, GAP-14 (PQ-60): no route writes its own sentence.
//!
//! Every string the room sends a person in a `reason`, `message` or `refusal`
//! field comes from `src/copy.rs`. The audit's M5b3 put *Turn to your neighbour
//! and volunteer one.* into the answer route's reason as a literal and every
//! test passed; this scan is what fails it now.
//!
//! The rule, kept as narrow as M5b3 needs: under `src/server/` and
//! `src/connections/`, a string literal containing a letter fails when it is
//!
//! - in the value of a `"reason"`, `"message"` or `"refusal"` JSON key, read
//!   from the key's `:` to the next `,` or closing bracket at the same depth; or
//! - in the arguments of a call to a [`REASON_SINKS`] helper, which puts its
//!   argument in a `reason` field (admin's `refused`).
//!
//! The [`REASON_SOURCES`] build the refusals those routes forward through a
//! variable (`LoadError`, `RoomError::Refused`). In them, a literal containing
//! a letter also fails in the arguments of any [`SOURCE_SINKS`] call or macro;
//! every `format!` there builds a refusal, so none may carry its own words.
//!
//! One carve-out: a `refusal` value that is one of [`REFUSAL_CODES`] passes.
//! Those are the codes a client branches on, not sentences anyone reads; any
//! other word there fails, so a new code is added to the list on purpose.
//! Literals elsewhere (keys, slugs in `slug()`, headers) are not this scan's.

mod common;

/// The JSON keys whose string a person may read.
const KEYS: &[&str] = &["reason", "message", "refusal"];

/// The refusal codes a client branches on: the host denials (`routes/mod.rs`)
/// and `JoinRefusal::slug()` (`connections/sessions.rs`).
const REFUSAL_CODES: &[&str] = &[
    "wrong_server",
    "wrong_role",
    "malformed",
    "unknown",
    "not_yet_open",
    "already_ended",
    "closed_for_inactivity",
    "full",
];

/// Helpers that put their argument in a `reason` field.
const REASON_SINKS: &[&str] = &["refused"];

/// Files outside `src/server/` and `src/connections/` whose refusals a route
/// sends as its `reason`: the admin channel's `LoadError` and `RoomError`, and
/// the host routes' `RoomError` (PQ-60 ruling A, these three files only).
const REASON_SOURCES: &[&str] = &[
    "quiz_session/models/answers.rs",
    "quiz_session/state_manager.rs",
    "quiz_session/host/create.rs",
];

/// In [`REASON_SOURCES`], the calls and macros that build a refusal.
const SOURCE_SINKS: &[&str] = &["refuse", "Refused", "LoadError", "format", "format_args"];

#[derive(Debug, PartialEq)]
enum Tok {
    Str(String),
    Word(String),
    Punct(char),
}

/// The source as tokens with their line numbers. Comments are dropped; string,
/// raw-string and char literals are read whole, so a quote inside one does not
/// open a string; a lifetime is not a char literal.
fn tokens(src: &str) -> Vec<(usize, Tok)> {
    let c: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let (mut i, mut line) = (0, 1);
    let at = |i: usize| c.get(i).copied().unwrap_or('\0');
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if ch == '/' && at(i + 1) == '/' {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '/' && at(i + 1) == '*' {
            let mut depth = 0;
            loop {
                if i >= c.len() {
                    break;
                }
                if c[i] == '/' && at(i + 1) == '*' {
                    depth += 1;
                    i += 2;
                } else if c[i] == '*' && at(i + 1) == '/' {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    line += usize::from(c[i] == '\n');
                    i += 1;
                }
            }
        } else if ch == '"' {
            let start = line;
            let (text, next) = string(&c, i + 1, &mut line);
            out.push((start, Tok::Str(text)));
            i = next;
        } else if ch == '\'' {
            if at(i + 1) == '\\' {
                i += 2;
                while i < c.len() && c[i] != '\'' {
                    i += 1;
                }
                i += 1;
            } else if at(i + 2) == '\'' {
                i += 3;
            } else {
                i += 1; // a lifetime; its name follows as a word
            }
        } else if ch.is_alphanumeric() || ch == '_' {
            let s = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            let word: String = c[s..i].iter().collect();
            if matches!(word.as_str(), "r" | "br") && matches!(at(i), '"' | '#') {
                let hashes = c[i..].iter().take_while(|&&h| h == '#').count();
                let mut j = i + hashes + 1;
                let start = line;
                let body = j;
                while j < c.len() && !(c[j] == '"' && (1..=hashes).all(|k| at(j + k) == '#')) {
                    line += usize::from(c[j] == '\n');
                    j += 1;
                }
                out.push((start, Tok::Str(c[body..j].iter().collect())));
                i = j + 1 + hashes;
            } else if word == "b" && at(i) == '"' {
                let start = line;
                let (text, next) = string(&c, i + 1, &mut line);
                out.push((start, Tok::Str(text)));
                i = next;
            } else {
                out.push((line, Tok::Word(word)));
            }
        } else {
            out.push((line, Tok::Punct(ch)));
            i += 1;
        }
    }
    out
}

/// A `"…"` body from just after its opening quote: (text, index past the close).
fn string(c: &[char], mut i: usize, line: &mut usize) -> (String, usize) {
    let mut text = String::new();
    while i < c.len() && c[i] != '"' {
        if c[i] == '\\' {
            text.push(c[i]);
            i += 1;
        }
        if i < c.len() {
            *line += usize::from(c[i] == '\n');
            text.push(c[i]);
            i += 1;
        }
    }
    (text, i + 1)
}

/// The tokens from `from` to the first unmatched closer, or, when `field` is
/// set, to the first `,` at depth 0 if that comes sooner: a JSON field's value,
/// or (unset) a call's whole argument list.
fn span(toks: &[(usize, Tok)], from: usize, field: bool) -> std::ops::Range<usize> {
    let mut depth = 0usize;
    for (j, (_, t)) in toks.iter().enumerate().skip(from) {
        match t {
            Tok::Punct('(' | '[' | '{') => depth += 1,
            Tok::Punct(')' | ']' | '}') if depth == 0 => return from..j,
            Tok::Punct(')' | ']' | '}') => depth -= 1,
            Tok::Punct(',') if field && depth == 0 => return from..j,
            _ => {}
        }
    }
    from..toks.len()
}

/// A literal sentence bound for a person: (line, what it is in, the literal).
/// Also counts the fields and sink calls it read, so a scan that reads nothing
/// cannot pass by accident.
#[derive(Default)]
struct Scan {
    hits: Vec<(usize, String, String)>,
    fields: usize,
    sink_calls: usize,
}

fn scan(src: &str, sinks: &[&str]) -> Scan {
    let toks = tokens(src);
    let mut out = Scan::default();
    // A literal inside nested sinks (`refuse(&id, format_args!("…"))`) is one
    // hit, not one per enclosing call.
    let mut seen = vec![false; toks.len()];
    let mut hit = |j: usize, what: String, out: &mut Scan| {
        if let Tok::Str(s) = &toks[j].1 {
            if !std::mem::replace(&mut seen[j], true) {
                out.hits.push((toks[j].0, what, s.clone()));
            }
        }
    };
    for (i, (_, t)) in toks.iter().enumerate() {
        let next = |k: usize| toks.get(i + k).map(|(_, t)| t);
        // `name(` or the macro `name!(`: where the arguments open.
        let args = match (next(1), next(2)) {
            (Some(Tok::Punct('(')), _) => Some(i + 2),
            (Some(Tok::Punct('!')), Some(Tok::Punct('('))) => Some(i + 3),
            _ => None,
        };
        match t {
            Tok::Str(key)
                if KEYS.contains(&key.as_str())
                    && next(1) == Some(&Tok::Punct(':'))
                    && next(2) != Some(&Tok::Punct(':')) =>
            {
                out.fields += 1;
                for j in span(&toks, i + 2, true) {
                    if let Tok::Str(s) = &toks[j].1 {
                        if s.chars().any(char::is_alphabetic)
                            && !(key == "refusal" && REFUSAL_CODES.contains(&s.as_str()))
                        {
                            hit(j, format!("\"{key}\""), &mut out);
                        }
                    }
                }
            }
            Tok::Word(name)
                if sinks.contains(&name.as_str())
                    && args.is_some()
                    && !(i > 0 && toks[i - 1].1 == Tok::Word("fn".into())) =>
            {
                out.sink_calls += 1;
                for j in span(&toks, args.unwrap_or(i), false) {
                    if let Tok::Str(s) = &toks[j].1 {
                        if s.chars().any(char::is_alphabetic) {
                            hit(j, format!("{name}(…)"), &mut out);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// The files the rule covers: every `.rs` under `src/server/` and `src/connections/`.
fn covered() -> Vec<(String, String)> {
    common::room_sources(&[])
        .into_iter()
        .filter(|(name, _)| name.starts_with("server/") || name.starts_with("connections/"))
        .collect()
}

/// The [`REASON_SOURCES`], read.
fn sources() -> Vec<(String, String)> {
    common::room_sources(&[])
        .into_iter()
        .filter(|(name, _)| REASON_SOURCES.contains(&name.as_str()))
        .collect()
}

#[test]
fn no_route_sends_a_sentence_that_is_not_in_the_copy_module() {
    let files = covered();
    for must in [
        "server/routes/connections.rs",
        "server/admin.rs",
        "connections/sessions.rs",
    ] {
        assert!(
            files.iter().any(|(n, _)| n == must),
            "the scan did not read {must}"
        );
    }
    let srcs = sources();
    for must in REASON_SOURCES {
        assert!(
            srcs.iter().any(|(n, _)| n == must),
            "the scan did not read {must}; update REASON_SOURCES"
        );
    }
    let (mut hits, mut fields, mut sink_calls, mut source_calls) = (Vec::new(), 0, 0, 0);
    let all_sinks: Vec<&str> = REASON_SINKS.iter().chain(SOURCE_SINKS).copied().collect();
    let read = files
        .iter()
        .map(|f| (f, false))
        .chain(srcs.iter().map(|f| (f, true)));
    for ((name, text), is_source) in read {
        let s = scan(text, if is_source { &all_sinks } else { REASON_SINKS });
        fields += s.fields;
        if is_source {
            source_calls += s.sink_calls;
        } else {
            sink_calls += s.sink_calls;
        }
        hits.extend(
            s.hits
                .into_iter()
                .map(|(line, what, lit)| format!("src/{name}:{line} {what}: {lit:?}")),
        );
    }
    // Floors against a tokenizer that silently matches nothing: PQ-60 left 14
    // such fields and 8 `refused` calls in server/ and connections/, and over
    // 20 refusal calls in the three sources.
    assert!(
        fields >= 10,
        "read only {fields} reason/message/refusal fields"
    );
    assert!(
        sink_calls >= 5,
        "read only {sink_calls} calls to {REASON_SINKS:?}"
    );
    assert!(
        source_calls >= 15,
        "read only {source_calls} calls to {SOURCE_SINKS:?} in {REASON_SOURCES:?}"
    );
    assert!(
        hits.is_empty(),
        "a client-bound sentence is written in a route; move it into src/copy.rs \
         word for word (AC-98) and send the constant:\n  {}",
        hits.join("\n  ")
    );
}

#[test]
fn every_reason_sink_is_still_a_function_the_scan_can_see() {
    // A renamed helper would take its calls out of the scan without a failure.
    let files = covered();
    for sink in REASON_SINKS {
        let decl = format!("fn {sink}(");
        assert!(
            files.iter().any(|(_, t)| t.contains(&decl)),
            "{decl} is gone from src/server and src/connections; update REASON_SINKS"
        );
    }
}

#[test]
fn every_refusal_code_is_still_sent() {
    // A code no route sends any more is a hole in the carve-out; drop it.
    let files = covered();
    for code in REFUSAL_CODES {
        let lit = format!("\"{code}\"");
        assert!(
            files.iter().any(|(_, t)| t.contains(&lit)),
            "{lit} is gone from src/server and src/connections; update REFUSAL_CODES"
        );
    }
}

// The scan against the audit's mutation and the ticket's two plants, as text, so
// the rule's reach is pinned here and not only in a reverted edit.

fn flags(src: &str) -> Vec<String> {
    scan(src, REASON_SINKS)
        .hits
        .into_iter()
        .map(|(_, _, lit)| lit)
        .collect()
}

/// [`flags`] as the scan reads a [`REASON_SOURCES`] file.
fn source_flags(src: &str) -> Vec<String> {
    let sinks: Vec<&str> = REASON_SINKS.iter().chain(SOURCE_SINKS).copied().collect();
    scan(src, &sinks)
        .hits
        .into_iter()
        .map(|(_, _, lit)| lit)
        .collect()
}

#[test]
fn the_scan_fails_a_sentence_written_in_a_reason_source() {
    // The shapes the three sources had before PQ-60 moved their words.
    assert_eq!(
        source_flags(r#"return refuse(&id, "has no verified record, so it has no answer");"#),
        ["has no verified record, so it has no answer"]
    );
    assert_eq!(
        source_flags(r#"refuse(&id, format_args!("has {dnc} options; it needs one"))"#),
        ["has {dnc} options; it needs one"]
    );
    assert_eq!(
        source_flags(r#".ok_or_else(|| RoomError::Refused("No question.".into()))?;"#),
        ["No question."]
    );
    assert_eq!(
        source_flags(r#"Err(RoomError::Refused(format!("A room is running {id}.")))"#),
        ["A room is running {id}."]
    );
    assert_eq!(
        source_flags(r#"map_err(|e| LoadError(format!("not a bank record: {e}")))"#),
        ["not a bank record: {e}"]
    );
    assert_eq!(
        source_flags(r#"then(|| format!("{id}: is affirmed"))"#),
        ["{id}: is affirmed"]
    );
    // The moved forms pass.
    let clean = r#"
        pub struct LoadError(pub String);
        fn refuse<T>(id: &str, why: impl std::fmt::Display) -> Result<T, LoadError> {
            Err(LoadError(copy::load_refused(id, &why.to_string())))
        }
        return refuse(&id, copy::LOAD_NO_VERIFIED);
        return Err(RoomError::Refused(copy::ROOM_NOTHING_SCHEDULED.into()));
        Err(RoomError::Refused(why)) => refused(StatusCode::CONFLICT, why),
    "#;
    assert_eq!(source_flags(clean), Vec::<String>::new());
}

#[test]
fn the_scan_fails_m5b3_and_both_plants() {
    // M5b3: the sentence as the answer route's reason, in its match.
    let m5b3 = r#"Json(serde_json::json!({
        "reason": match phase {
            crate::phase::Phase::Idle => "Turn to your neighbour and volunteer one.",
            _ => crate::copy::BUZZER_CLOSED,
        },
        "phase": phase,
    }))"#;
    assert_eq!(flags(m5b3), ["Turn to your neighbour and volunteer one."]);
    // A new `reason` literal in a connections route.
    let route = r#"Json(serde_json::json!({ "saved": saved, "reason": "Saved it." }))"#;
    assert_eq!(flags(route), ["Saved it."]);
    // One in admin's response, and one through its helper, `format!` or not.
    let admin =
        r#"Json(serde_json::json!({ "id": id, "scheduled": "new", "reason": "Scheduled." }))"#;
    assert_eq!(flags(admin), ["Scheduled."]);
    assert_eq!(flags(r#"refused(StatusCode::BAD_REQUEST, "No.")"#), ["No."]);
    assert_eq!(
        flags(r#"refused(s, format!("{id}: bad order."))"#),
        ["{id}: bad order."]
    );
    assert_eq!(flags(r#"json!({ "message": "hello" })"#), ["hello"]);
    // A code passes only as a refusal, and only a listed one.
    assert_eq!(flags(r#"json!({ "reason": "full" })"#), ["full"]);
    assert_eq!(flags(r#"json!({ "refusal": "volunteer" })"#), ["volunteer"]);
}

#[test]
fn the_scan_passes_constants_slugs_comments_and_the_helper_itself() {
    let clean = r#"
        // { "reason": "a comment is not a route" }
        /* "reason": "nor is a block comment" */
        fn refused(status: StatusCode, reason: impl Into<String>) -> Response {
            (status, Json(serde_json::json!({ "reason": reason.into() }))).into_response()
        }
        let quote = '"';
        fn f<'a>(x: &'a str) {}
        Json(serde_json::json!({ "refusal": "wrong_server", "reason": crate::copy::HOST_DENIED_WRONG_SERVER }));
        Json(serde_json::json!({ "refusal": refusal.slug(), "reason": refusal.message() }));
        refused(StatusCode::BAD_REQUEST, copy::admin_bank_order_not_indexes(id));
        Json(serde_json::json!({ "id": id, "scheduled": "new" }));
    "#;
    let s = scan(clean, REASON_SINKS);
    assert_eq!(s.hits, Vec::<(usize, String, String)>::new());
    assert_eq!((s.fields, s.sink_calls), (5, 1));
}
