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
//! One carve-out: a `refusal` value that is a bare slug (`wrong_server`) passes.
//! A slug is what the client branches on, not a sentence anyone reads.
//! Literals elsewhere (keys, slugs in `slug()`, headers) are not this scan's.

mod common;

/// The JSON keys whose string a person may read.
const KEYS: &[&str] = &["reason", "message", "refusal"];

/// Helpers that put their argument in a `reason` field.
const REASON_SINKS: &[&str] = &["refused"];

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

fn is_slug(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

/// The tokens from `from` to the first unmatched closer, or, when `field` is
/// set, to the first `,` at depth 0 if that comes sooner: a JSON field's value,
/// or (unset) a call's whole argument list.
fn span(toks: &[(usize, Tok)], from: usize, field: bool) -> &[(usize, Tok)] {
    let mut depth = 0usize;
    for (j, (_, t)) in toks.iter().enumerate().skip(from) {
        match t {
            Tok::Punct('(' | '[' | '{') => depth += 1,
            Tok::Punct(')' | ']' | '}') if depth == 0 => return &toks[from..j],
            Tok::Punct(')' | ']' | '}') => depth -= 1,
            Tok::Punct(',') if field && depth == 0 => return &toks[from..j],
            _ => {}
        }
    }
    &toks[from..]
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

fn scan(src: &str) -> Scan {
    let toks = tokens(src);
    let mut out = Scan::default();
    for (i, (_, t)) in toks.iter().enumerate() {
        let next = |k: usize| toks.get(i + k).map(|(_, t)| t);
        match t {
            Tok::Str(key)
                if KEYS.contains(&key.as_str())
                    && next(1) == Some(&Tok::Punct(':'))
                    && next(2) != Some(&Tok::Punct(':')) =>
            {
                out.fields += 1;
                for (line, v) in span(&toks, i + 2, true) {
                    if let Tok::Str(s) = v {
                        if s.chars().any(char::is_alphabetic) && !(key == "refusal" && is_slug(s)) {
                            out.hits.push((*line, format!("\"{key}\""), s.clone()));
                        }
                    }
                }
            }
            Tok::Word(name)
                if REASON_SINKS.contains(&name.as_str())
                    && next(1) == Some(&Tok::Punct('('))
                    && !(i > 0 && toks[i - 1].1 == Tok::Word("fn".into())) =>
            {
                out.sink_calls += 1;
                for (line, v) in span(&toks, i + 2, false) {
                    if let Tok::Str(s) = v {
                        if s.chars().any(char::is_alphabetic) {
                            out.hits.push((*line, format!("{name}(…)"), s.clone()));
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
    let (mut hits, mut fields, mut sink_calls) = (Vec::new(), 0, 0);
    for (name, text) in &files {
        let s = scan(text);
        fields += s.fields;
        sink_calls += s.sink_calls;
        hits.extend(
            s.hits
                .into_iter()
                .map(|(line, what, lit)| format!("src/{name}:{line} {what}: {lit:?}")),
        );
    }
    // Floors against a tokenizer that silently matches nothing: PQ-60 left 14
    // such fields and 8 `refused` calls in these files.
    assert!(
        fields >= 10,
        "read only {fields} reason/message/refusal fields"
    );
    assert!(
        sink_calls >= 5,
        "read only {sink_calls} calls to {REASON_SINKS:?}"
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

// The scan against the audit's mutation and the ticket's two plants, as text, so
// the rule's reach is pinned here and not only in a reverted edit.

fn flags(src: &str) -> Vec<String> {
    scan(src).hits.into_iter().map(|(_, _, lit)| lit).collect()
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
    // A slug passes only as a refusal.
    assert_eq!(flags(r#"json!({ "reason": "full" })"#), ["full"]);
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
    let s = scan(clean);
    assert_eq!(s.hits, Vec::<(usize, String, String)>::new());
    assert_eq!((s.fields, s.sink_calls), (5, 1));
}
