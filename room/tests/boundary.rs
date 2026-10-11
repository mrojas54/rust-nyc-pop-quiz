//! AC-61's in-crate half: the seal, as the source states it.
//!
//! The `compile_fail` doctests in `src/phase.rs`, `src/answers.rs` and
//! `src/rooms.rs` prove the boundary as another crate sees it — each paired
//! with a twin that compiles and differs only in the forbidden line, so each
//! can fail for one reason only. What a doctest cannot see is a hole opened
//! *inside* the crate: a `pub(crate)` constructor for the witness, a second
//! function in the vault that reads a field without one, a `Debug` derive that
//! prints the answer. This file reads the source and refuses those.
//!
//! It is a complement, not the proof: a scan can be dodged by someone who
//! means to. The compiler checks the part that matters — field privacy — and
//! this checks that nobody has quietly built a way around it.

use std::path::PathBuf;

fn src(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn all_sources() -> Vec<(String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![dir];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let name = path.strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"))
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                out.push((name, std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    assert!(out.len() >= 8, "found {} source files", out.len());
    out
}

/// The code of a file with comment lines removed (doc comments hold the
/// `compile_fail` snippets, which name forbidden things on purpose).
fn code(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The body of the first `{ … }` block after `opener`.
fn block<'a>(text: &'a str, opener: &str) -> &'a str {
    let start = text.find(opener).unwrap_or_else(|| panic!("no {opener:?}"));
    let open = start + text[start..].find('{').unwrap();
    let mut depth = 0;
    for (i, c) in text[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open + 1..open + i];
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced block after {opener:?}");
}

/// `(name, signature up to the opening brace)` of every `fn` in `text`.
fn fns(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("fn ") {
        let before = rest[..i].chars().last();
        let after = &rest[i + 3..];
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            rest = after;
            continue;
        }
        let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        let sig_end = after.find('{').unwrap_or(after.len());
        out.push((name, after[..sig_end].split_whitespace().collect::<Vec<_>>().join(" ")));
        rest = after;
    }
    out
}

#[test]
fn unsafe_is_forbidden_crate_wide() {
    assert!(src("lib.rs").contains("#![forbid(unsafe_code)]"));
}

#[test]
fn the_witness_is_built_in_one_place() {
    for (name, text) in all_sources() {
        let hits = code(&text).matches("RevealWitness {").count();
        if name == "phase.rs" {
            // The struct definition, and the one construction in `revealed()`.
            assert_eq!(hits, 1, "phase.rs builds RevealWitness {hits} times");
            assert_eq!(code(&text).matches("RevealWitness<'m> {").count(), 1);
        } else {
            assert_eq!(hits, 0, "{name} builds a RevealWitness");
        }
    }
    // And the one construction is inside `revealed`, under `Phase::Reveal`.
    let phase = code(&src("phase.rs"));
    let revealed = block(&phase, "pub fn revealed(&self)");
    assert!(revealed.contains("Phase::Reveal => Some(RevealWitness {"));
}

#[test]
fn the_vault_holds_only_the_allowed_functions_and_every_read_takes_the_witness() {
    let answers = code(&src("answers.rs"));
    let vault = block(&answers, "mod vault {");
    let found = fns(vault);
    let names: Vec<&str> = found.iter().map(|(n, _)| n.as_str()).collect();
    // `seal` puts the secrets in; `judge` returns an opaque Verdict; `open`
    // (the vault's, and the Verdict's) is the only way anything comes out.
    assert_eq!(names, ["seal", "judge", "open", "open"], "the vault's functions changed");
    for (name, sig) in &found {
        match name.as_str() {
            "seal" => assert!(sig.starts_with("seal(") && sig.contains("-> Vault"), "{sig}"),
            "judge" => assert!(sig.ends_with("-> Verdict"), "judge must return the sealed Verdict: {sig}"),
            "open" => assert!(sig.contains("&RevealWitness<'_>"), "open without a witness: {sig}"),
            other => panic!("unexpected fn {other} in the vault"),
        }
    }
    assert!(vault.contains("pub(super) fn seal("), "seal stays private to answers");
    // The vault's fields are declared private — no `pub` field anywhere in it.
    for s in ["pub correct", "pub why_tempting", "pub explains", "pub receipt", "pub trace", "pub how_we_know", "pub middle", "pub(crate)", "pub(super) correct"] {
        assert!(!vault.contains(s), "vault exposes {s:?}");
    }
    // Declared once, privately.
    let decls: usize = all_sources().iter().map(|(_, t)| code(t).matches("mod vault").count()).sum();
    assert_eq!(decls, 1);
    assert!(!answers.contains("pub mod vault"));
    // T-12: take-it-home's reads (every why_tempting, the machine) leave the
    // vault only through `open`, beside the rest of what it lends.
    let open = block(vault, "pub fn open<'v>(&'v self, _proof: &RevealWitness<'_>)");
    assert!(open.contains("why_tempting: &self.why_tempting") && open.contains("how_we_know: &self.how_we_know"));
}

#[test]
fn nothing_sealed_can_be_printed_copied_or_serialized() {
    let sealed_types = [
        ("answers.rs", "pub struct Vault"),
        ("answers.rs", "pub struct Verdict"),
        ("answers.rs", "pub struct Scheduled"),
        ("rooms.rs", "pub struct Room "),
        ("rooms.rs", "struct Frozen"),
        ("rooms.rs", "pub struct Opened"),
    ];
    for (file, decl) in sealed_types {
        let text = src(file);
        let at = text.find(decl).unwrap_or_else(|| panic!("{decl} not in {file}"));
        // The attributes directly above the declaration.
        // Cut back to the start of the declaration's line: the vault's types
        // are indented, and the indentation is not an attribute.
        let line_start = text[..at].rfind('\n').map_or(0, |i| i + 1);
        let above: Vec<&str> = text[..line_start]
            .lines()
            .rev()
            .take_while(|l| l.trim_start().starts_with("#[") || l.trim_start().starts_with("//"))
            .collect();
        for line in above.iter().filter(|l| l.trim_start().starts_with("#[")) {
            assert!(!line.contains("derive"), "{decl} in {file} derives: {line}");
        }
        let ty = decl.split_whitespace().last().unwrap();
        for all in all_sources() {
            for tr in ["Debug", "Clone", "Copy", "Serialize"] {
                assert!(
                    !code(&all.1).contains(&format!("{tr} for {ty}")),
                    "{} implements {tr} for {ty}",
                    all.0
                );
            }
        }
    }
}

#[test]
fn the_vault_is_touched_only_by_answers_and_the_phase_only_by_act() {
    for (name, text) in all_sources() {
        let code = code(&text);
        if name != "answers.rs" {
            assert!(!code.contains(".vault"), "{name} reaches into the vault");
            assert!(!code.contains("Vault::"), "{name} names the vault");
        }
        if name != "rooms.rs" {
            assert!(!code.contains(".question.open("), "{name} opens a question");
        }
        // phase.rs binds a local `machine` inside `apply`, which is the machine.
        if name != "rooms.rs" && name != "phase.rs" {
            assert!(!code.contains("machine ="), "{name} assigns a machine");
        }
    }
    // In rooms.rs, `machine` is assigned once — in `act`, from `apply`.
    let rooms = code(&src("rooms.rs"));
    assert_eq!(rooms.matches("self.machine =").count(), 1);
    let act = block(&rooms, "pub fn act(");
    assert!(act.contains("apply(State::Room(self.machine.clone()), command)?"));
    assert!(act.contains("self.machine = next;"));
    // The only other `machine` value a room holds is the one `create` got
    // from `apply(State::Unmade(..), CreateRoom)`.
    let create = block(&rooms, "pub fn create(");
    assert!(create.contains("apply(State::Unmade(len), Command::Host(HostAction::CreateRoom))?"));
}

#[test]
fn the_pre_reveal_builders_never_see_the_opened_room() {
    let view = code(&src("view.rs"));
    let sealed = block(&view, "mod sealed {");
    for forbidden in ["Opened", ".open(", "Middle", "answers", "Room", "revealed"] {
        assert!(!sealed.contains(forbidden), "the pre-reveal builders mention {forbidden:?}");
    }
    for (name, sig) in fns(sealed) {
        assert!(sig.contains("view: PublicView<'_>"), "{name} takes something other than a PublicView: {sig}");
    }
    // And each projection branches on the witness, not on the phase.
    for p in ["pub fn wall(", "pub fn buzzer(", "pub fn host("] {
        let body = block(&view, p);
        assert!(body.contains("match room.open()"), "{p} does not branch on room.open()");
        assert!(body.contains("None => sealed::"), "{p}");
    }
}
