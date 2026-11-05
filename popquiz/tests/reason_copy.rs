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
//! variable (`LoadError`, `RoomError::Refused`, `phase::Refused`,
//! `Ended::host_reason`). In them, a literal containing a letter also fails
//!
//! - in the arguments of any [`SOURCE_SINKS`] call or macro, and, in the
//!   [`FORMAT_SOURCES`], of any `format!`: every `format!` there builds a
//!   refusal, so none may carry its own words (elsewhere one may build a
//!   digest, and is read only inside one of these shapes);
//! - in a struct literal's `reason:` field (`Refused { reason: … }`); or
//! - anywhere in the body of a [`REASON_FNS`] function, which returns a
//!   reason from a `match` or an `Err` rather than through a call.
//!
//! A same-file `const` or `static` is read as its literal (PQ-84): where the
//! scan reads a span, the name of one that holds a letter-bearing literal fails
//! as that literal would. A constant used nowhere a reason is built passes.
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
/// the host routes' `RoomError` (PQ-60 ruling A); the affirmation's refusal,
/// the phase machine's and the ended room's (PQ-84, cleared "all").
const REASON_SOURCES: &[&str] = &[
    "quiz_session/models/answers.rs",
    "quiz_session/state_manager.rs",
    "quiz_session/host/create.rs",
    "quiz_session/models/affirmation.rs",
    "quiz_session/models/phase.rs",
    "quiz_session/lifecycle.rs",
];

/// In [`REASON_SOURCES`], the calls and macros that build a refusal.
const SOURCE_SINKS: &[&str] = &["refuse", "Refused", "LoadError"];

/// The [`REASON_SOURCES`] where every `format!` builds a refusal (PQ-60's
/// three). `affirmation.rs` formats its digest (`sha256:{hex}`), so there a
/// `format!` is read only inside a sink, a `reason:` field or a [`REASON_FNS`]
/// body.
const FORMAT_SOURCES: &[&str] = &[
    "quiz_session/models/answers.rs",
    "quiz_session/state_manager.rs",
    "quiz_session/host/create.rs",
];

/// In [`FORMAT_SOURCES`], the macros that build a refusal.
const FORMAT_SINKS: &[&str] = &["format", "format_args"];

/// In [`REASON_SOURCES`], the functions that return a reason: every literal in
/// their bodies is bound for a person (`Ended::host_reason`, the affirmation's
/// `refusal`, `Review::check_shape`).
const REASON_FNS: &[&str] = &["host_reason", "refusal", "check_shape"];

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
    reason_fns: usize,
}

/// `source` reads the file as a [`REASON_SOURCES`] file: its `reason:` struct
/// fields and [`REASON_FNS`] bodies are read too.
fn scan(src: &str, sinks: &[&str], source: bool) -> Scan {
    let toks = tokens(src);
    let mut out = Scan::default();
    let consts = sentence_consts(&toks);
    // The literal a token stands for: itself, or, for the name of a same-file
    // `const`/`static` that holds a letter-bearing literal, that literal. So
    // `const WHY: &str = "…"; refused(s, WHY)` fails as the literal would.
    let lit = |j: usize| -> Option<(usize, &str)> {
        match &toks[j].1 {
            Tok::Str(s) => Some((j, s.as_str())),
            Tok::Word(w) => consts.get(w.as_str()).map(|&l| match &toks[l].1 {
                Tok::Str(s) => (l, s.as_str()),
                _ => unreachable!("sentence_consts records string tokens only"),
            }),
            Tok::Punct(_) => None,
        }
    };
    let letters = |s: &str| s.chars().any(char::is_alphabetic);
    // A literal inside nested sinks (`refuse(&id, format_args!("…"))`) is one
    // hit, not one per enclosing call.
    let mut seen = vec![false; toks.len()];
    let mut hit = |l: usize, what: String, out: &mut Scan| {
        if let Tok::Str(s) = &toks[l].1 {
            if !std::mem::replace(&mut seen[l], true) {
                out.hits.push((toks[l].0, what, s.clone()));
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
        let (what, read) = match t {
            Tok::Str(key)
                if KEYS.contains(&key.as_str())
                    && next(1) == Some(&Tok::Punct(':'))
                    && next(2) != Some(&Tok::Punct(':')) =>
            {
                out.fields += 1;
                (format!("\"{key}\""), span(&toks, i + 2, true))
            }
            Tok::Word(name)
                if sinks.contains(&name.as_str())
                    && args.is_some()
                    && !(i > 0 && toks[i - 1].1 == Tok::Word("fn".into())) =>
            {
                out.sink_calls += 1;
                (format!("{name}(…)"), span(&toks, args.unwrap_or(i), false))
            }
            // `Refused { reason: "…".into() }`: a struct literal's field, so
            // the word follows `{` or `,` (not a closure's `|reason: String|`).
            Tok::Word(w)
                if source
                    && w == "reason"
                    && next(1) == Some(&Tok::Punct(':'))
                    && next(2) != Some(&Tok::Punct(':'))
                    && i > 0
                    && matches!(toks[i - 1].1, Tok::Punct('{' | ',')) =>
            {
                out.fields += 1;
                ("reason:".into(), span(&toks, i + 2, true))
            }
            // `fn host_reason(…) -> … { … }`: the whole body.
            Tok::Word(f) if source && f == "fn" => {
                let Some(Tok::Word(name)) = next(1) else {
                    continue;
                };
                if !REASON_FNS.contains(&name.as_str()) {
                    continue;
                }
                let Some(open) = (i + 2..toks.len()).find(|&k| toks[k].1 == Tok::Punct('{')) else {
                    continue;
                };
                out.reason_fns += 1;
                (format!("fn {name}"), span(&toks, open + 1, false))
            }
            _ => continue,
        };
        // A `refusal` key's value may be one of the codes; nothing else may.
        let code_ok = matches!(t, Tok::Str(key) if key == "refusal");
        for j in read {
            if let Some((l, s)) = lit(j) {
                if letters(s) && !(code_ok && REFUSAL_CODES.contains(&s)) {
                    hit(l, what.clone(), &mut out);
                }
            }
        }
    }
    out
}

/// Each same-file `const` or `static` whose initializer holds a literal with a
/// letter in it: its name, and the token of the first such literal.
fn sentence_consts(toks: &[(usize, Tok)]) -> std::collections::HashMap<&str, usize> {
    let mut out = std::collections::HashMap::new();
    for (i, (_, t)) in toks.iter().enumerate() {
        if !matches!(t, Tok::Word(w) if w == "const" || w == "static") {
            continue;
        }
        let mut k = i + 1;
        if matches!(toks.get(k), Some((_, Tok::Word(w))) if w == "mut") {
            k += 1;
        }
        // `const NAME:` — not `*const T` or `const fn`.
        let (Some((_, Tok::Word(name))), Some((_, Tok::Punct(':')))) =
            (toks.get(k), toks.get(k + 1))
        else {
            continue;
        };
        let Some(eq) = (k..toks.len()).find(|&m| toks[m].1 == Tok::Punct('=')) else {
            continue;
        };
        let end = (eq..toks.len())
            .find(|&m| toks[m].1 == Tok::Punct(';'))
            .unwrap_or(toks.len());
        let literal = (eq..end)
            .find(|&m| matches!(&toks[m].1, Tok::Str(s) if s.chars().any(char::is_alphabetic)));
        if let Some(l) = literal {
            out.insert(name.as_str(), l);
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

/// The sinks the scan reads in file `name`.
fn sinks_for(name: &str, is_source: bool) -> Vec<&'static str> {
    let mut sinks = REASON_SINKS.to_vec();
    if is_source {
        sinks.extend(SOURCE_SINKS);
    }
    if FORMAT_SOURCES.contains(&name) {
        sinks.extend(FORMAT_SINKS);
    }
    sinks
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
    let mut reason_fns = 0;
    let read = files
        .iter()
        .map(|f| (f, false))
        .chain(srcs.iter().map(|f| (f, true)));
    for ((name, text), is_source) in read {
        let s = scan(text, &sinks_for(name, is_source), is_source);
        fields += s.fields;
        reason_fns += s.reason_fns;
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
        "read only {source_calls} calls to {SOURCE_SINKS:?} or {FORMAT_SINKS:?} in {REASON_SOURCES:?}"
    );
    // PQ-84: five such bodies (host_reason, check_shape, and three `refusal`s).
    assert!(
        reason_fns >= 4,
        "read only {reason_fns} bodies of {REASON_FNS:?} in {REASON_SOURCES:?}"
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
fn every_reason_fn_is_still_a_function_the_scan_can_see() {
    // A renamed reason function would take its body out of the scan.
    let srcs = sources();
    for f in REASON_FNS {
        let decl = format!("fn {f}(");
        assert!(
            srcs.iter().any(|(_, t)| t.contains(&decl)),
            "{decl} is gone from {REASON_SOURCES:?}; update REASON_FNS"
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
    scan(src, REASON_SINKS, false)
        .hits
        .into_iter()
        .map(|(_, _, lit)| lit)
        .collect()
}

/// [`flags`] as the scan reads a [`FORMAT_SOURCES`] file, the strictest.
fn source_flags(src: &str) -> Vec<String> {
    scan(src, &sinks_for(FORMAT_SOURCES[0], true), true)
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
    // The shapes PQ-84's three sources had: a struct literal's `reason`, a
    // reason function's match arms, its `Err`, and its `format!` framing.
    assert_eq!(
        source_flags(r#"Err(Refused { from: None, command, reason: "No room yet.".into() })"#),
        ["No room yet."]
    );
    assert_eq!(
        source_flags(
            r#"pub fn host_reason(self) -> &'static str {
                match self { Ended::Expired => "This room has ended.", Ended::Inactive => copy::X }
            }"#
        ),
        ["This room has ended."]
    );
    assert_eq!(
        source_flags(
            r#"pub(crate) fn check_shape(&self) -> Result<(), &'static str> {
                if bad { return Err("the digest is bad"); } Ok(())
            }"#
        ),
        ["the digest is bad"]
    );
    assert_eq!(
        source_flags(
            r#"pub fn refusal(&self, id: &str) -> Option<String> {
                let why = match self { Affirmation::Stale => copy::S, _ => return None };
                Some(format!("{id}: {why}"))
            }"#
        ),
        ["{id}: {why}"]
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
        pub struct Refused { pub from: Option<Phase>, pub command: Command, pub reason: String }
        let refuse = |reason: String| Refused { from: Some(phase), command, reason };
        Command::Host(_) => Err(Refused { from: None, command, reason: copy::HOST_REFUSED_NO_ROOM_YET.into() }),
        pub fn host_reason(self) -> &'static str {
            match self { Ended::Expired => copy::HOST_ENDED_EXPIRED, Ended::Inactive => copy::HOST_ENDED_INACTIVE }
        }
        pub fn refusal(&self, id: &str) -> Option<String> { Some(copy::load_refused(id, copy::LOAD_AFFIRMATION_NONE)) }
        pub fn state(&self) -> &'static str { match self { Affirmation::Stale => "stale", _ => "none" } }
    "#;
    assert_eq!(source_flags(clean), Vec::<String>::new());
    // affirmation.rs's digest is not a reason: outside the FORMAT_SOURCES a
    // bare `format!` passes, and one inside `refuse(…)` still fails.
    let affirmation = sinks_for("quiz_session/models/affirmation.rs", true);
    let digest =
        r#"let hex: String = bytes.map(|b| format!("{b:02x}")).collect(); format!("sha256:{hex}")"#;
    assert_eq!(scan(digest, &affirmation, true).hits.len(), 0);
    let inside = r#"Err(refuse(format!("{} is not ready.", id)))"#;
    assert_eq!(scan(inside, &affirmation, true).hits.len(), 1);
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
fn the_scan_reads_a_same_file_constant_as_its_literal() {
    // PQ-60's Minor: the sentence one name away from the sink.
    let bad = r#"
        const BAD: &str = "Turn to your neighbour and volunteer one.";
        fn f() -> Response { refused(StatusCode::CONFLICT, BAD) }
    "#;
    assert_eq!(flags(bad), ["Turn to your neighbour and volunteer one."]);
    let json = r#"
        pub(crate) static WHY: &str = "Saved it.";
        Json(serde_json::json!({ "saved": saved, "reason": WHY }))
    "#;
    assert_eq!(flags(json), ["Saved it."]);
    // In a source: a struct field, a sink's argument, a reason function's arm.
    let field = r#"const NO_ROOM: &str = "No room yet.";
        Err(Refused { from: None, command, reason: NO_ROOM.into() })"#;
    assert_eq!(source_flags(field), ["No room yet."]);
    let arm = r#"const ENDED: &str = "This room has ended.";
        pub fn host_reason(self) -> &'static str { match self { Ended::Expired => ENDED, _ => copy::X } }"#;
    assert_eq!(source_flags(arm), ["This room has ended."]);
    // Used twice, reported once.
    let twice = r#"const W: &str = "No."; refused(a, W); refused(b, W);"#;
    assert_eq!(flags(twice), ["No."]);
    // Constants that never reach a reason pass, as do a listed code and copy's.
    let clean = r#"
        pub const VAR: &str = "POPQUIZ_ADMIN_TOKEN";
        const CSS: &str = "text/css; charset=utf-8";
        const HOST_HTML: &str = include_str!("../../../../web/host/index.html");
        const FULL: &str = "full";
        let token = var(VAR);
        asset(HOST_HTML, CSS);
        Json(serde_json::json!({ "refusal": FULL, "reason": copy::ANSWER_NOT_YET_OPEN }));
        let p: *const u8 = std::ptr::null();
        const fn two() -> usize { 2 }
    "#;
    assert_eq!(flags(clean), Vec::<String>::new());
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
    let s = scan(clean, REASON_SINKS, false);
    assert_eq!(s.hits, Vec::<(usize, String, String)>::new());
    assert_eq!((s.fields, s.sink_calls), (5, 1));
}

#[test]
fn room_only_reasons_are_word_for_word() {
    // PQ-84 moved these out of affirmation.rs, phase.rs and lifecycle.rs. Each
    // is frozen copy (AC-98): a one-character change fails here.
    use room::copy;
    for (got, want) in [
        (copy::LOAD_DIGEST_MALFORMED, "review.affirmed_digest is not sha256:<64 hex>"),
        (
            copy::LOAD_DIGEST_WITHOUT_WHO,
            "review.affirmed_digest is recorded without affirmed_by and affirmed_at",
        ),
        (
            copy::LOAD_AFFIRMATION_STALE,
            "its text changed after it was affirmed; affirm it again at review.",
        ),
        (
            copy::LOAD_AFFIRMATION_UNBOUND,
            "its affirmation records who and when but not what was affirmed; affirm it again at review.",
        ),
        (
            copy::LOAD_AFFIRMATION_NONE,
            "is not affirmed; affirm it at review, then schedule it again.",
        ),
        (
            copy::HOST_REFUSED_NO_ROOM_YET,
            "There is no room yet. Create a room comes first.",
        ),
        (
            copy::HOST_REFUSED_ROOM_EXISTS,
            "This room already exists. Create a room makes a separate one.",
        ),
        (
            copy::HOST_REFUSED_RELEASED,
            "The room is released. Run it again starts a new room.",
        ),
        (
            copy::HOST_REFUSED_TRACE_STILL,
            "The trace only steps while walking it through or at the answer.",
        ),
        (copy::HOST_REFUSED_FIRST_STEP, "That's the first step."),
        (
            copy::HOST_REFUSED_LAST_STEP_BEFORE_REVEAL,
            "That's the last step before the reveal.",
        ),
        (copy::HOST_REFUSED_LAST_STEP, "That's the last step."),
        (copy::HOST_ENDED_EXPIRED, "This room has ended."),
        (copy::HOST_ENDED_INACTIVE, "This room closed for inactivity."),
    ] {
        assert_eq!(got, want);
    }
    // The two built ones, as the `format!`s they replaced built them.
    assert_eq!(
        copy::host_refused_not_next("Reveal", "Start"),
        format!("{} isn't next. {} comes next.", "Reveal", "Start")
    );
    let why = copy::LOAD_AFFIRMATION_NONE;
    assert_eq!(copy::load_refused("q3", why), format!("{}: {}", "q3", why));
}
