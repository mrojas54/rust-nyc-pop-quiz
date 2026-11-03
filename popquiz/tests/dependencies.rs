//! AC-1's static half: the room reaches no generator, verifier or LLM code.
//!
//! Generation, verification and every model call live in the offline pipeline
//! (`pipeline/`, the Python package `popquiz`). The room must have no way to
//! call into any of it. This file reads the room's manifest, its lock and its
//! source as text, and refuses:
//!
//! - a dependency in any table (ordinary, optional, feature-gated, dev, build,
//!   per-target, patch) whose crate name is denied, or whose `path` points into
//!   `pipeline/`;
//! - a denied crate anywhere in the lock's subgraph under `room`;
//! - a `use` or `extern crate` in `src/` whose root is a denied name.
//!
//! **The denylist names crates, never folders.** The room's own folder is
//! `popquiz/`, the same word as the Python package, so a check on paths would
//! fail on main or need a carve-out. A dependency carries a name, and names are
//! what is checked; the only path check is the one that resolves a `path =`
//! dependency against the repository's `pipeline/` folder.
//!
//! A scan is a tripwire, not a proof: someone who means to can dodge it. It
//! checks the static half of AC-1 only.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

// --- The denylist -------------------------------------------------------------

/// Words that deny a crate when one of them is a whole `_`-separated part of
/// its name: `async-openai` and `cargo-miri` are denied, `generic-array` is not.
/// `popquiz` is the pipeline's Python package (its import and distribution name
/// in `pipeline/pyproject.toml`); `pipeline` is the folder that holds it.
const DENIED_WORDS: &[&str] = &[
    "anthropic",
    "openai",
    "llm",
    "generator",
    "miri",
    "pipeline",
    "popquiz",
];

/// The compiler's own crates (`rustc_driver`, `rustc_interface`, …), which is
/// how Miri or an in-process verifier would reach rustc. This also refuses
/// crates.io's `rustc-hash` and `rustc_version`. Neither is in the lock today;
/// adding one is a person's call, not a reason to loosen this.
const DENIED_PREFIX: &str = "rustc_";

/// Whether a crate name is denied. Cargo treats `-` and `_` in a name as the
/// same character, so this does too.
fn denied(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase().replace('-', "_");
    name.starts_with(DENIED_PREFIX) || name.split('_').any(|word| DENIED_WORDS.contains(&word))
}

const WHY: &str = "AC-1: the room must not reach generator, verifier or LLM code \
                   (popquiz/tests/dependencies.rs)";

// --- Files --------------------------------------------------------------------

/// The room crate's folder. Read at run time, so the built test can be pointed
/// at a throwaway copy holding a planted dependency that `cargo --locked`
/// would refuse to build. `cargo test` sets it; the compile-time value is the
/// fallback.
fn crate_dir() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// `a/b/../c` as `a/c`, without touching the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

// --- TOML, read as text ---------------------------------------------------------

/// One line of TOML with its comment (a `#` outside a string) removed.
fn toml_code(line: &str) -> &str {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        match quote {
            Some(q) => {
                if escaped {
                    escaped = false;
                } else if c == '\\' && q == '"' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' => quote = Some(c),
                '#' => return &line[..i],
                _ => {}
            },
        }
    }
    line
}

/// How much a line of TOML code opens (`[`, `{`) minus closes, strings excluded.
fn depth_change(code: &str) -> i32 {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut depth = 0;
    for c in code.chars() {
        match quote {
            Some(q) => {
                if escaped {
                    escaped = false;
                } else if c == '\\' && q == '"' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' => quote = Some(c),
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                _ => {}
            },
        }
    }
    depth
}

fn unquote(text: &str) -> String {
    text.trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string()
}

/// The string value of `key = "…"` anywhere in an entry's text: inside an
/// inline table, or on its own line in a `[dependencies.<name>]` table.
fn string_field(text: &str, key: &str) -> Option<String> {
    for (i, _) in text.match_indices(key) {
        let before = text[..i].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            continue;
        }
        let Some(after) = text[i + key.len()..].trim_start().strip_prefix('=') else {
            continue;
        };
        let after = after.trim_start();
        let Some(quote) = after.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            continue;
        };
        let value = &after[1..];
        return value.find(quote).map(|end| value[..end].to_string());
    }
    None
}

// --- The manifest -------------------------------------------------------------

enum Table {
    /// `[dependencies]` and its kin: every key is a dependency.
    List,
    /// `[dependencies.<name>]`: one dependency, its fields one per line.
    One(String),
    Other,
}

/// What kind of table a header opens. Dependency tables are `dependencies`,
/// `dev-dependencies` and `build-dependencies` (Cargo also takes the `_`
/// spellings), each possibly under `target.<cfg>.`, plus `patch.<source>` and
/// `replace`.
fn table_kind(header: &str) -> Table {
    let header = header.trim();
    if header.starts_with("patch.") || header == "replace" {
        return Table::List;
    }
    let Some((before, after)) = header.rsplit_once("dependencies") else {
        return Table::Other;
    };
    let whole_word = before.is_empty()
        || before.ends_with('.')
        || ["dev-", "dev_", "build-", "build_"]
            .iter()
            .any(|kind| before == *kind || before.ends_with(&format!(".{kind}")));
    if !whole_word {
        return Table::Other;
    }
    if after.is_empty() {
        Table::List
    } else if let Some(name) = after.strip_prefix('.') {
        Table::One(unquote(name))
    } else {
        Table::Other
    }
}

/// One dependency as the manifest declares it.
struct Dep {
    /// The header of the table it sits in.
    table: String,
    /// The key it is declared under.
    name: String,
    /// The text that declares it, where `package` and `path` are read from.
    body: String,
}

/// Every dependency in every dependency table, and how many such tables there are.
fn manifest_deps(manifest: &str) -> (Vec<Dep>, usize) {
    let mut deps: Vec<Dep> = Vec::new();
    let mut tables = 0;
    let mut current: Option<(String, Table)> = None;
    let mut depth = 0;
    for raw in manifest.lines() {
        let code = toml_code(raw);
        let trimmed = code.trim();
        if depth == 0 && trimmed.starts_with('[') {
            let header = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            let kind = if trimmed.starts_with("[[") {
                Table::Other
            } else {
                table_kind(header)
            };
            if !matches!(kind, Table::Other) {
                tables += 1;
            }
            if let Table::One(name) = &kind {
                deps.push(Dep {
                    table: header.to_string(),
                    name: name.clone(),
                    body: String::new(),
                });
            }
            current = Some((header.to_string(), kind));
            continue;
        }
        match &current {
            Some((header, Table::List)) => {
                if depth == 0 {
                    if let Some((key, _)) = trimmed.split_once('=') {
                        deps.push(Dep {
                            table: header.clone(),
                            name: unquote(key),
                            body: String::new(),
                        });
                    }
                }
                if let Some(dep) = deps.last_mut().filter(|dep| dep.table == *header) {
                    dep.body.push_str(code);
                    dep.body.push('\n');
                }
            }
            Some((_, Table::One(_))) => {
                let dep = deps
                    .last_mut()
                    .expect("a dotted table pushed its dependency");
                dep.body.push_str(code);
                dep.body.push('\n');
            }
            _ => {}
        }
        depth += depth_change(code);
    }
    (deps, tables)
}

/// What is wrong with a manifest that sits in `crate_dir`.
fn manifest_violations(crate_dir: &Path, manifest: &str) -> Vec<String> {
    let pipeline = normalize(&crate_dir.join("..").join("pipeline"));
    let mut violations = Vec::new();
    for dep in manifest_deps(manifest).0 {
        let at = format!("Cargo.toml [{}] {}", dep.table, dep.name);
        if denied(&dep.name) {
            violations.push(format!("{at}: denied crate name"));
        }
        if let Some(package) = string_field(&dep.body, "package") {
            if denied(&package) {
                violations.push(format!("{at}: renames denied crate `{package}`"));
            }
        }
        if let Some(path) = string_field(&dep.body, "path") {
            if normalize(&crate_dir.join(&path)).starts_with(&pipeline) {
                violations.push(format!("{at}: path `{path}` points into pipeline/"));
            }
        }
    }
    violations
}

// --- The lock -------------------------------------------------------------------

#[derive(Default)]
struct Package {
    name: String,
    version: String,
    /// `None` for a path package: the room itself, or a path dependency.
    source: Option<String>,
    /// As the lock writes them: `name`, `name version` or `name version (source)`.
    deps: Vec<String>,
}

fn lock_packages(lock: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut current: Option<Package> = None;
    let mut in_deps = false;
    for raw in lock.lines() {
        let line = toml_code(raw).trim();
        if in_deps {
            if line.starts_with(']') {
                in_deps = false;
            } else if !line.is_empty() {
                if let Some(package) = current.as_mut() {
                    package.deps.push(unquote(line.trim_end_matches(',')));
                }
            }
            continue;
        }
        if line.starts_with('[') {
            packages.extend(current.take());
            if line == "[[package]]" {
                current = Some(Package::default());
            }
            continue;
        }
        let (Some(package), Some((key, value))) = (current.as_mut(), line.split_once('=')) else {
            continue;
        };
        match key.trim() {
            "name" => package.name = unquote(value),
            "version" => package.version = unquote(value),
            "source" => package.source = Some(unquote(value)),
            "dependencies" => {
                let value = value.trim();
                if value == "[" {
                    in_deps = true;
                } else {
                    package.deps.extend(
                        value
                            .trim_matches(|c| c == '[' || c == ']')
                            .split(',')
                            .filter(|dep| !dep.trim().is_empty())
                            .map(unquote),
                    );
                }
            }
            _ => {}
        }
    }
    packages.extend(current);
    packages
}

/// Every package the room reaches in its lock, the room included.
fn room_subgraph(packages: &[Package]) -> Vec<&Package> {
    let root = packages
        .iter()
        .find(|package| package.name == "room" && package.source.is_none())
        .expect("the lock holds the room crate itself");
    let mut seen = BTreeSet::new();
    let mut stack = vec![root];
    let mut reached = Vec::new();
    while let Some(package) = stack.pop() {
        if !seen.insert((package.name.as_str(), package.version.as_str())) {
            continue;
        }
        reached.push(package);
        for dep in &package.deps {
            let mut words = dep.split_whitespace();
            let name = words.next().unwrap_or_default();
            let version = words.next();
            let matches: Vec<&Package> = packages
                .iter()
                .filter(|candidate| {
                    candidate.name == name
                        && match version {
                            None => true,
                            Some(version) => candidate.version == version,
                        }
                })
                .collect();
            assert!(
                !matches.is_empty(),
                "Cargo.lock lists `{dep}` under {} but holds no such package",
                package.name
            );
            stack.extend(matches);
        }
    }
    reached
}

fn lock_violations(lock: &str) -> Vec<String> {
    let packages = lock_packages(lock);
    let mut violations = Vec::new();
    for package in room_subgraph(&packages) {
        let at = format!("Cargo.lock {} {}", package.name, package.version);
        if denied(&package.name) {
            violations.push(format!("{at}: denied crate in the room's subgraph"));
        }
        if package.source.is_none() && package.name != "room" {
            violations.push(format!("{at}: a path package other than the room"));
        }
    }
    violations
}

// --- The source -----------------------------------------------------------------

/// A line of Rust with any attributes and visibility in front of the item removed:
/// `#[cfg(any())] pub(crate) use x;` reads as `use x;`.
fn item_head(line: &str) -> &str {
    let mut rest = line.trim_start();
    loop {
        if rest.starts_with("#[") {
            let mut depth = 0;
            let mut end = None;
            for (i, c) in rest.char_indices() {
                match c {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            match end {
                Some(end) => rest = rest[end + 1..].trim_start(),
                None => return "",
            }
        } else if rest.starts_with("pub(") {
            match rest.find(')') {
                Some(end) => rest = rest[end + 1..].trim_start(),
                None => return "",
            }
        } else if let Some(after) = rest.strip_prefix("pub ") {
            rest = after.trim_start();
        } else {
            return rest;
        }
    }
}

/// A `use` or `extern crate` statement: its text after the keyword, up to `;`.
struct Import {
    extern_crate: bool,
    tree: String,
}

/// A file's imports and the module names it declares with `mod`.
fn imports(text: &str) -> (Vec<Import>, BTreeSet<String>) {
    let mut found = Vec::new();
    let mut mods = BTreeSet::new();
    let mut open: Option<Import> = None;
    for raw in text.lines() {
        let line = raw.find("//").map_or(raw, |comment| &raw[..comment]);
        if let Some(mut import) = open.take() {
            import.tree.push(' ');
            import.tree.push_str(line);
            match import.tree.find(';') {
                Some(end) => {
                    import.tree.truncate(end);
                    found.push(import);
                }
                None => open = Some(import),
            }
            continue;
        }
        let head = item_head(line);
        let started = if let Some(rest) = head.strip_prefix("use ") {
            Some(Import {
                extern_crate: false,
                tree: rest.to_string(),
            })
        } else {
            head.strip_prefix("extern crate ").map(|rest| Import {
                extern_crate: true,
                tree: rest.to_string(),
            })
        };
        if let Some(mut import) = started {
            match import.tree.find(';') {
                Some(end) => {
                    import.tree.truncate(end);
                    found.push(import);
                }
                None => open = Some(import),
            }
        } else if let Some(rest) = head.strip_prefix("mod ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                mods.insert(name);
            }
        }
    }
    (found, mods)
}

/// The first path segment of each path in a use tree: `{a::b, ::c::{d}}` is
/// `a` and `c`.
fn roots(tree: &str) -> Vec<String> {
    let tree = tree.trim().trim_start_matches("::").trim_start();
    if let Some(group) = tree.strip_prefix('{') {
        let mut out = Vec::new();
        let mut depth = 0;
        let mut item = String::new();
        for c in group.chars() {
            match c {
                '{' => {
                    depth += 1;
                    item.push(c);
                }
                '}' if depth == 0 => break,
                '}' => {
                    depth -= 1;
                    item.push(c);
                }
                ',' if depth == 0 => out.extend(roots(&std::mem::take(&mut item))),
                _ => item.push(c),
            }
        }
        out.extend(roots(&item));
        return out;
    }
    let root: String = tree
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if root.is_empty() {
        Vec::new()
    } else {
        vec![root]
    }
}

/// What is wrong with one source file, and the denied names it was allowed to
/// `use` because the same file declares them as modules.
///
/// A `use` root that the file declares with `mod` is that module, which is how
/// Rust resolves it: `server/routes/mod.rs` declares `mod pipeline;` (the room's
/// admin route for the pipeline's pushes) and then `use pipeline::admin_routes;`.
/// `extern crate` always names a crate, so it gets no such reading.
fn source_violations(file: &str, text: &str) -> (Vec<String>, BTreeSet<String>) {
    let (found, mods) = imports(text);
    let mut violations = Vec::new();
    let mut local = BTreeSet::new();
    for import in found {
        for root in roots(&import.tree) {
            if !denied(&root) {
                continue;
            }
            if !import.extern_crate && mods.contains(&root) {
                local.insert(root);
                continue;
            }
            let keyword = if import.extern_crate {
                "extern crate"
            } else {
                "use"
            };
            violations.push(format!(
                "{file}: `{keyword} {}` names denied crate `{root}`",
                import.tree.trim()
            ));
        }
    }
    (violations, local)
}

struct SourceScan {
    files: usize,
    imports: usize,
    violations: Vec<String>,
    local: BTreeSet<String>,
}

fn scan_source(crate_dir: &Path) -> SourceScan {
    let mut files = Vec::new();
    rust_files(&crate_dir.join("src"), &mut files);
    let mut scan = SourceScan {
        files: files.len(),
        imports: 0,
        violations: Vec::new(),
        local: BTreeSet::new(),
    };
    for path in files {
        let text = read(&path);
        let name = path
            .strip_prefix(crate_dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        scan.imports += imports(&text).0.len();
        let (violations, local) = source_violations(&name, &text);
        scan.violations.extend(violations);
        scan.local.extend(local);
    }
    scan
}

fn refuse(violations: &[String]) {
    assert!(
        violations.is_empty(),
        "{WHY}:\n  {}",
        violations.join("\n  ")
    );
}

// --- The checks -------------------------------------------------------------------

#[test]
fn the_manifest_declares_no_denied_crate_and_no_path_into_the_pipeline() {
    let dir = crate_dir();
    refuse(&manifest_violations(&dir, &read(&dir.join("Cargo.toml"))));
}

#[test]
fn the_lock_reaches_no_denied_crate_from_the_room() {
    refuse(&lock_violations(&read(&crate_dir().join("Cargo.lock"))));
}

#[test]
fn the_room_source_imports_no_denied_crate() {
    refuse(&scan_source(&crate_dir()).violations);
}

/// The three checks above pass vacuously if their readers see nothing. This
/// holds each reader to things known to be there.
#[test]
fn the_readers_see_every_section_they_check() {
    let dir = crate_dir();
    let manifest = read(&dir.join("Cargo.toml"));
    let (deps, tables) = manifest_deps(&manifest);
    let headers = manifest
        .lines()
        .map(|line| toml_code(line).trim())
        .filter(|line| line.starts_with('[') && line.contains("dependencies"))
        .count();
    assert!(
        tables >= 2,
        "found {tables} dependency tables in Cargo.toml"
    );
    assert_eq!(
        tables, headers,
        "a dependency table in Cargo.toml went unread"
    );
    let declared = |table: &str, name: &str| {
        deps.iter()
            .find(|dep| dep.table == table && dep.name == name)
            .unwrap_or_else(|| panic!("Cargo.toml [{table}] {name} was not read"))
    };
    declared("dependencies", "axum");
    assert!(
        declared("dependencies", "tokio-tungstenite")
            .body
            .contains("optional = true"),
        "the optional, feature-gated tokio-tungstenite was not read as such"
    );
    declared("dev-dependencies", "tower");

    let packages = lock_packages(&read(&dir.join("Cargo.lock")));
    let reached = room_subgraph(&packages);
    for name in ["axum", "tokio-tungstenite", "tower", "ring"] {
        assert!(
            reached.iter().any(|package| package.name == name),
            "the room's lock subgraph is missing {name}"
        );
    }
    assert_eq!(
        reached.len(),
        packages.len(),
        "the room is the lock's only root, so it should reach every package"
    );

    let scan = scan_source(&dir);
    assert!(scan.files > 10, "read {} source files", scan.files);
    assert!(scan.imports > 100, "read {} imports", scan.imports);
    assert!(
        scan.local.contains("pipeline"),
        "the scan should see the room's own `mod pipeline` route and its `use`"
    );
}

#[test]
fn the_denylist_matches_crate_names_not_the_rooms_folder() {
    for name in [
        "anthropic",
        "async-openai",
        "openai_api_rust",
        "llm",
        "llm-chain",
        "generator",
        "miri",
        "cargo-miri",
        "pipeline",
        "popquiz",
        "rustc_driver",
        "rustc_interface",
    ] {
        assert!(denied(name), "{name} should be denied");
    }
    for name in [
        "room",
        "getrandom",
        "generic-array",
        "wit-bindgen",
        "axum",
        "tokio",
    ] {
        assert!(!denied(name), "{name} should be allowed");
    }
    // The folder is `popquiz/`; nothing above reads it. The crate is `room`.
    assert!(crate_dir().ends_with("popquiz"));
    assert!(!denied("room"));
}

/// The mutations named on PQ-47, planted in the real text. On the real files
/// M1 and M2 stop `cargo --locked` before any test runs; here they reach the
/// checks, so the checks stay able to refuse them.
#[test]
fn the_planted_mutations_are_refused() {
    let dir = crate_dir();
    let manifest = read(&dir.join("Cargo.toml"));

    // M1: a path dependency on the pipeline.
    let m1 = manifest.replacen(
        "[dependencies]\n",
        "[dependencies]\npipeline = { path = \"../pipeline\" }\n",
        1,
    );
    assert_ne!(m1, manifest, "M1 was not planted");
    let found = manifest_violations(&dir, &m1);
    assert!(
        found.iter().any(|v| v.contains("points into pipeline/")),
        "M1: {found:?}"
    );

    // M2: an optional, feature-gated LLM client.
    let m2 = manifest
        .replacen("[features]\n", "[features]\nllm = [\"dep:anthropic\"]\n", 1)
        .replacen(
            "[dependencies]\n",
            "[dependencies]\nanthropic = { version = \"0.1\", optional = true }\n",
            1,
        );
    assert_ne!(m2, manifest, "M2 was not planted");
    let found = manifest_violations(&dir, &m2);
    assert!(
        found.iter().any(|v| v.contains("[dependencies] anthropic")),
        "M2: {found:?}"
    );

    // M3: an import the compiler never sees.
    let (found, _) = source_violations("m3.rs", "#[cfg(any())]\nuse openai::Client;\n");
    assert_eq!(found.len(), 1, "M3: {found:?}");

    // The same name as the room's own module is local only where it is declared.
    let declared = "mod pipeline;\npub(crate) use pipeline::admin_routes;\n";
    assert!(source_violations("routes.rs", declared).0.is_empty());
    let undeclared = "pub(crate) use pipeline::admin_routes;\n";
    assert_eq!(source_violations("other.rs", undeclared).0.len(), 1);

    for text in [
        "use {std::fmt, ::openai::Client};\n",
        "use std::{\n    fmt,\n};\nuse anthropic\n    ::Messages;\n",
        "extern crate rustc_driver;\n",
        "mod popquiz;\nextern crate popquiz;\n",
    ] {
        assert_eq!(source_violations("planted.rs", text).0.len(), 1, "{text}");
    }
    let renamed = manifest.replacen(
        "[dev-dependencies]\n",
        "[dev-dependencies]\nclient = { package = \"async-openai\", version = \"0.1\" }\n",
        1,
    );
    assert!(
        manifest_violations(&dir, &renamed)
            .iter()
            .any(|v| v.contains("async-openai")),
        "a renamed denied crate"
    );
    let dotted =
        format!("{manifest}\n[target.'cfg(unix)'.build-dependencies.helper]\npackage = \"llm\"\n");
    assert!(
        manifest_violations(&dir, &dotted)
            .iter()
            .any(|v| v.contains("renames denied crate `llm`")),
        "a dotted per-target build dependency"
    );
}
