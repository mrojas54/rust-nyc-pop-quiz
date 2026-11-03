//! AC-1's static half: the room reaches no generator, verifier or LLM code.
//!
//! Generation, verification and every model call live in the offline pipeline
//! (`pipeline/`, the Python package `popquiz`). The room must have no way to
//! call into any of it. This file reads the room's manifest, its lock and its
//! source, and refuses:
//!
//! - a dependency in any table (ordinary, optional, feature-gated, dev, build,
//!   per-target, patch, replace) whose crate name is denied, or whose `path`
//!   points into `pipeline/`;
//! - a denied crate anywhere in the lock's subgraph under `room`;
//! - a `use` or `extern crate` in `src/` whose root is a denied name.
//!
//! **The denylist names crates, never folders.** The room's own folder is
//! `popquiz/`, the same word as the Python package, so a check on paths would
//! fail on main or need a carve-out. A dependency carries a name, and names are
//! what is checked; the only path check resolves a `path` dependency against
//! the repository's `pipeline/` folder.
//!
//! The manifest and the lock are read by a small TOML parser and the source by
//! a small Rust tokenizer, both below, so layout (dotted or quoted keys, escaped
//! strings, comments, imports split across lines) does not change what is seen.
//! No crate is added for either.
//!
//! A scan is a tripwire, not a proof: someone who means to can dodge it. Imports
//! a macro writes, and paths used with no `use` at all (`openai::Client::new()`),
//! are left to the manifest and lock checks: a crate the source can name must be
//! declared there first. This is the static half of AC-1 only.

use std::collections::{BTreeMap, BTreeSet};
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

// --- TOML -----------------------------------------------------------------------
//
// Enough of TOML 1.0 for Cargo.toml and Cargo.lock: bare, quoted and dotted
// keys; `[table]` and `[[array]]` headers; the four string kinds with their
// escapes; arrays and inline tables. Numbers, booleans and dates are kept as
// `Other`, since nothing here reads them. Malformed input panics with a line.

#[derive(Debug)]
enum Toml {
    Str(String),
    Table(Vec<(String, Toml)>),
    Array(Vec<Toml>),
    Other,
}

impl Toml {
    fn get(&self, key: &str) -> Option<&Toml> {
        self.entries()
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    fn str(&self) -> Option<&str> {
        match self {
            Toml::Str(text) => Some(text),
            _ => None,
        }
    }

    fn entries(&self) -> &[(String, Toml)] {
        match self {
            Toml::Table(entries) => entries,
            _ => &[],
        }
    }

    fn items(&self) -> &[Toml] {
        match self {
            Toml::Array(items) => items,
            _ => &[],
        }
    }
}

/// The table a key path names, created where missing. A path through an array
/// of tables goes into its last element, as a TOML header does.
fn table_at<'a>(
    table: &'a mut Vec<(String, Toml)>,
    path: &[String],
) -> &'a mut Vec<(String, Toml)> {
    let Some((first, rest)) = path.split_first() else {
        return table;
    };
    let index = match table.iter().position(|(name, _)| name == first) {
        Some(index) => index,
        None => {
            table.push((first.clone(), Toml::Table(Vec::new())));
            table.len() - 1
        }
    };
    let child = match &mut table[index].1 {
        Toml::Table(entries) => entries,
        Toml::Array(items) => match items.last_mut() {
            Some(Toml::Table(entries)) => entries,
            _ => panic!("TOML key `{first}` is an array, not an array of tables"),
        },
        _ => panic!("TOML key `{first}` is a value, not a table"),
    };
    table_at(child, rest)
}

fn insert(table: &mut Vec<(String, Toml)>, key: &[String], value: Toml) {
    let (last, parents) = key.split_last().expect("a key has at least one part");
    table_at(table, parents).push((last.clone(), value));
}

struct TomlParser {
    text: Vec<char>,
    at: usize,
}

impl TomlParser {
    fn peek(&self) -> Option<char> {
        self.text.get(self.at).copied()
    }

    fn looking_at(&self, word: &str) -> bool {
        word.chars()
            .enumerate()
            .all(|(k, c)| self.text.get(self.at + k) == Some(&c))
    }

    fn fail(&self, what: &str) -> ! {
        let end = self.at.min(self.text.len());
        let line = self.text[..end].iter().filter(|c| **c == '\n').count() + 1;
        panic!("TOML line {line}: {what}")
    }

    fn expect(&mut self, c: char) {
        if self.peek() != Some(c) {
            self.fail(&format!("expected `{c}`"));
        }
        self.at += 1;
    }

    fn spaces(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.at += 1;
        }
    }

    /// Spaces, line breaks and comments.
    fn blank(&mut self) {
        loop {
            self.spaces();
            match self.peek() {
                Some('\n' | '\r') => self.at += 1,
                Some('#') => {
                    while !matches!(self.peek(), None | Some('\n')) {
                        self.at += 1;
                    }
                }
                _ => return,
            }
        }
    }

    fn key(&mut self) -> Vec<String> {
        let mut parts = Vec::new();
        loop {
            self.spaces();
            let part = match self.peek() {
                Some('"') => self.basic_string(),
                Some('\'') => self.literal_string(),
                _ => {
                    let start = self.at;
                    while self
                        .peek()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    {
                        self.at += 1;
                    }
                    if start == self.at {
                        self.fail("expected a key");
                    }
                    self.text[start..self.at].iter().collect()
                }
            };
            parts.push(part);
            self.spaces();
            if self.peek() != Some('.') {
                return parts;
            }
            self.at += 1;
        }
    }

    /// The end of a multi-line string: three quotes, and up to two more that
    /// belong to the content.
    fn closes_multiline(&mut self, quote: char, out: &mut String) -> bool {
        let mut run = 0;
        while self.text.get(self.at + run) == Some(&quote) {
            run += 1;
        }
        if run < 3 {
            return false;
        }
        let extra = (run - 3).min(2);
        out.extend(std::iter::repeat_n(quote, extra));
        self.at += 3 + extra;
        true
    }

    fn skip_first_newline(&mut self) {
        if self.looking_at("\r\n") {
            self.at += 2;
        } else if self.looking_at("\n") {
            self.at += 1;
        }
    }

    fn basic_string(&mut self) -> String {
        let multi = self.looking_at("\"\"\"");
        self.at += if multi { 3 } else { 1 };
        if multi {
            self.skip_first_newline();
        }
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                self.fail("unterminated string");
            };
            if multi && c == '"' && self.closes_multiline('"', &mut out) {
                return out;
            }
            if !multi && c == '"' {
                self.at += 1;
                return out;
            }
            if !multi && c == '\n' {
                self.fail("line break in a one-line string");
            }
            self.at += 1;
            if c != '\\' {
                out.push(c);
                continue;
            }
            let Some(escape) = self.peek() else {
                self.fail("unterminated escape");
            };
            self.at += 1;
            match escape {
                'b' => out.push('\u{8}'),
                't' => out.push('\t'),
                'n' => out.push('\n'),
                'f' => out.push('\u{c}'),
                'r' => out.push('\r'),
                'e' => out.push('\u{1b}'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'u' | 'U' | 'x' => {
                    let width = match escape {
                        'u' => 4,
                        'U' => 8,
                        _ => 2,
                    };
                    let Some(hex) = self.text.get(self.at..self.at + width) else {
                        self.fail("short unicode escape");
                    };
                    let hex: String = hex.iter().collect();
                    let Some(decoded) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
                    else {
                        self.fail("bad unicode escape");
                    };
                    out.push(decoded);
                    self.at += width;
                }
                ' ' | '\t' | '\n' | '\r' if multi => {
                    // A line-ending backslash: the break and the next line's indent go.
                    while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
                        self.at += 1;
                    }
                }
                _ => self.fail("bad escape"),
            }
        }
    }

    fn literal_string(&mut self) -> String {
        let multi = self.looking_at("'''");
        self.at += if multi { 3 } else { 1 };
        if multi {
            self.skip_first_newline();
        }
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                self.fail("unterminated string");
            };
            if multi && c == '\'' && self.closes_multiline('\'', &mut out) {
                return out;
            }
            if !multi && c == '\'' {
                self.at += 1;
                return out;
            }
            if !multi && c == '\n' {
                self.fail("line break in a one-line string");
            }
            out.push(c);
            self.at += 1;
        }
    }

    fn value(&mut self) -> Toml {
        match self.peek() {
            Some('"') => Toml::Str(self.basic_string()),
            Some('\'') => Toml::Str(self.literal_string()),
            Some('[') => {
                self.at += 1;
                let mut items = Vec::new();
                loop {
                    self.blank();
                    if self.peek() == Some(']') {
                        self.at += 1;
                        return Toml::Array(items);
                    }
                    items.push(self.value());
                    self.blank();
                    match self.peek() {
                        Some(',') => self.at += 1,
                        Some(']') => {}
                        _ => self.fail("expected `,` or `]`"),
                    }
                }
            }
            Some('{') => {
                self.at += 1;
                let mut table = Vec::new();
                loop {
                    self.blank();
                    if self.peek() == Some('}') {
                        self.at += 1;
                        return Toml::Table(table);
                    }
                    let key = self.key();
                    self.expect('=');
                    self.spaces();
                    let value = self.value();
                    insert(&mut table, &key, value);
                    self.blank();
                    match self.peek() {
                        Some(',') => self.at += 1,
                        Some('}') => {}
                        _ => self.fail("expected `,` or `}`"),
                    }
                }
            }
            Some(_) => {
                let start = self.at;
                while !matches!(
                    self.peek(),
                    None | Some(',' | ']' | '}' | '\n' | '\r' | '#')
                ) {
                    self.at += 1;
                }
                if start == self.at {
                    self.fail("expected a value");
                }
                Toml::Other
            }
            None => self.fail("expected a value"),
        }
    }
}

fn parse_toml(text: &str) -> Toml {
    let mut parser = TomlParser {
        text: text.chars().collect(),
        at: 0,
    };
    let mut root = Vec::new();
    let mut current: Vec<String> = Vec::new();
    loop {
        parser.blank();
        match parser.peek() {
            None => return Toml::Table(root),
            Some('[') => {
                parser.at += 1;
                let array = parser.peek() == Some('[');
                if array {
                    parser.at += 1;
                }
                let key = parser.key();
                parser.expect(']');
                if array {
                    parser.expect(']');
                    let (last, parents) = key.split_last().expect("a header has a key");
                    let parent = table_at(&mut root, parents);
                    match parent.iter_mut().find(|(name, _)| name == last) {
                        Some((_, Toml::Array(items))) => items.push(Toml::Table(Vec::new())),
                        Some(_) => parser.fail("`[[…]]` over a key that is not an array"),
                        None => {
                            parent.push((last.clone(), Toml::Array(vec![Toml::Table(Vec::new())])))
                        }
                    }
                } else {
                    table_at(&mut root, &key);
                }
                current = key;
            }
            Some(_) => {
                let key = parser.key();
                parser.expect('=');
                parser.spaces();
                let value = parser.value();
                insert(table_at(&mut root, &current), &key, value);
            }
        }
        parser.spaces();
        if !matches!(parser.peek(), None | Some('\n' | '\r' | '#')) {
            parser.fail("expected the end of the line");
        }
    }
}

// --- The manifest -------------------------------------------------------------

/// The tables a dependency can be declared in, at the root or under
/// `target.<cfg>`. Cargo also takes the `_` spellings.
const DEPENDENCY_KINDS: &[&str] = &[
    "dependencies",
    "dev-dependencies",
    "dev_dependencies",
    "build-dependencies",
    "build_dependencies",
];

/// Every root key Cargo.toml may hold, so a section this file does not read
/// fails `the_readers_see_every_section_they_check` instead of passing unread.
const KNOWN_ROOT_KEYS: &[&str] = &[
    "cargo-features",
    "package",
    "lib",
    "bin",
    "example",
    "test",
    "bench",
    "features",
    "dependencies",
    "dev-dependencies",
    "dev_dependencies",
    "build-dependencies",
    "build_dependencies",
    "target",
    "patch",
    "replace",
    "badges",
    "lints",
    "profile",
    "workspace",
];

/// One dependency as the manifest declares it.
struct Dep {
    /// Where it is declared, as a dotted table name.
    table: String,
    /// The key it is declared under (for `[replace]`, the name before `:`).
    name: String,
    /// `package = "…"`: the crate it really is, when renamed.
    package: Option<String>,
    path: Option<String>,
    /// `workspace = true`: the real spec is the workspace's entry of this name.
    /// (Cargo refuses `workspace = false`, so the key alone says it.)
    inherited: bool,
}

/// The crate a `[replace]` key names. Cargo takes a package ID spec there:
/// `name`, `name:version` or `name@version`, optionally after a source URL and
/// `#`; a URL whose fragment is only a version (`url#1.0.0`), or that has no
/// fragment, names the last segment of its path.
fn spec_name(spec: &str) -> String {
    let (url, rest) = match spec.split_once('#') {
        Some((url, rest)) => (Some(url), rest),
        None if spec.contains("://") => (Some(spec), ""),
        None => (None, spec),
    };
    let name = rest.split(['@', ':']).next().unwrap_or_default();
    match url {
        Some(url) if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) => {
            let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');
            let last = path.rsplit('/').next().unwrap_or(path);
            last.strip_suffix(".git").unwrap_or(last).to_string()
        }
        _ => name.to_string(),
    }
}

/// Where inherited dependency specs live. The room is its own workspace root,
/// so this is in its own manifest.
const WORKSPACE_DEPENDENCIES: &str = "workspace.dependencies";

/// Every dependency in every dependency table, the workspace's included.
fn manifest_deps(manifest: &Toml) -> Vec<Dep> {
    let mut deps = Vec::new();
    let mut read_table = |table: String, entries: &[(String, Toml)]| {
        for (key, spec) in entries {
            let field = |name: &str| spec.get(name).and_then(Toml::str).map(String::from);
            deps.push(Dep {
                table: table.clone(),
                name: if table == "replace" {
                    spec_name(key)
                } else {
                    key.clone()
                },
                package: field("package"),
                path: field("path"),
                inherited: spec.get("workspace").is_some(),
            });
        }
    };
    if let Some(table) = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
    {
        read_table(WORKSPACE_DEPENDENCIES.to_string(), table.entries());
    }
    for kind in DEPENDENCY_KINDS {
        if let Some(table) = manifest.get(kind) {
            read_table(kind.to_string(), table.entries());
        }
    }
    for (cfg, target) in manifest.get("target").map_or(&[][..], Toml::entries) {
        for kind in DEPENDENCY_KINDS {
            if let Some(table) = target.get(kind) {
                read_table(format!("target.{cfg}.{kind}"), table.entries());
            }
        }
    }
    for (source, table) in manifest.get("patch").map_or(&[][..], Toml::entries) {
        read_table(format!("patch.{source}"), table.entries());
    }
    if let Some(table) = manifest.get("replace") {
        read_table("replace".to_string(), table.entries());
    }
    deps
}

/// What is wrong with a manifest that sits in `crate_dir`.
fn manifest_violations(crate_dir: &Path, manifest: &str) -> Vec<String> {
    let pipeline = normalize(&crate_dir.join("..").join("pipeline"));
    let mut violations = Vec::new();
    let deps = manifest_deps(&parse_toml(manifest));
    let in_workspace: BTreeSet<&str> = deps
        .iter()
        .filter(|dep| dep.table == WORKSPACE_DEPENDENCIES)
        .map(|dep| dep.name.as_str())
        .collect();
    for dep in &deps {
        let at = format!("Cargo.toml [{}] {}", dep.table, dep.name);
        // Its spec is checked where it is written. One this manifest does not
        // write comes from a workspace above the room that this test cannot read.
        if dep.inherited && !in_workspace.contains(dep.name.as_str()) {
            violations.push(format!(
                "{at}: inherits from a workspace outside the room's manifest"
            ));
        }
    }
    for dep in deps {
        let at = format!("Cargo.toml [{}] {}", dep.table, dep.name);
        if denied(&dep.name) {
            violations.push(format!("{at}: denied crate name"));
        }
        if let Some(package) = dep.package.filter(|package| denied(package)) {
            violations.push(format!("{at}: renames denied crate `{package}`"));
        }
        if let Some(path) = dep.path {
            if normalize(&crate_dir.join(&path)).starts_with(&pipeline) {
                violations.push(format!("{at}: path `{path}` points into pipeline/"));
            }
        }
    }
    violations
}

// --- The lock -------------------------------------------------------------------

struct Package {
    name: String,
    version: String,
    /// `None` for a path package: the room itself, or a path dependency.
    source: Option<String>,
    /// As the lock writes them: `name`, `name version` or `name version (source)`.
    deps: Vec<String>,
}

fn lock_packages(lock: &str) -> Vec<Package> {
    let lock = parse_toml(lock);
    let text = |package: &Toml, key: &str| package.get(key).and_then(Toml::str).map(String::from);
    lock.get("package")
        .map_or(&[][..], Toml::items)
        .iter()
        .map(|package| Package {
            name: text(package, "name").unwrap_or_default(),
            version: text(package, "version").unwrap_or_default(),
            source: text(package, "source"),
            deps: package
                .get("dependencies")
                .map_or(&[][..], Toml::items)
                .iter()
                .filter_map(Toml::str)
                .map(String::from)
                .collect(),
        })
        .collect()
}

/// A lock source without the commit a git source is pinned to. Cargo writes a
/// package's source as `git+URL?query#commit` and a dependency line's as
/// `(git+URL?query)`; the URL and its `?branch=`/`?tag=`/`?rev=` still tell
/// two git sources apart.
fn unpinned(source: &str) -> &str {
    source.split_once('#').map_or(source, |(source, _)| source)
}

/// The packages a lock entry's dependency line can mean. The lock adds the
/// version, then the source, only when the shorter form would be ambiguous.
fn resolve<'a>(packages: &'a [Package], dep: &str) -> Vec<usize> {
    let (spec, source) = match dep.split_once(" (") {
        Some((spec, source)) => (spec, Some(source.trim_end_matches(')'))),
        None => (dep, None),
    };
    let mut words = spec.split_whitespace();
    let name = words.next().unwrap_or_default();
    let version = words.next();
    (0..packages.len())
        .filter(|&index| {
            let candidate: &'a Package = &packages[index];
            candidate.name == name
                && version.is_none_or(|version| candidate.version == version)
                && source.is_none_or(|source| {
                    candidate.source.as_deref().map(unpinned) == Some(unpinned(source))
                })
        })
        .collect()
}

/// Every package the room reaches in its lock, the room included.
fn room_subgraph(packages: &[Package]) -> Vec<&Package> {
    let root = packages
        .iter()
        .position(|package| package.name == "room" && package.source.is_none())
        .expect("the lock holds the room crate itself");
    let mut seen = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(index) = stack.pop() {
        if !seen.insert(index) {
            continue;
        }
        let package = &packages[index];
        for dep in &package.deps {
            let found = resolve(packages, dep);
            assert!(
                !found.is_empty(),
                "Cargo.lock lists `{dep}` under {} but holds no such package",
                package.name
            );
            stack.extend(found);
        }
    }
    seen.into_iter().map(|index| &packages[index]).collect()
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
//
// A tokenizer that knows what Rust does not read as code: line and nested block
// comments, string, byte and C string literals (raw ones too), character
// literals and lifetimes. What is left is identifiers (raw ones unwrapped),
// `::`, braces, `;`, `,` and single punctuation, which is all the import reader
// needs.

#[derive(Debug, PartialEq)]
enum Tok {
    /// An identifier or keyword; `true` when written raw (`r#name`), which is
    /// never a keyword.
    Ident(String, bool),
    PathSep,
    Open,
    Close,
    Semi,
    Comma,
    /// Any other single character: `#`, `!`, `[`, `]`, `(`, `<`, …
    Punct(char),
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Past a `"…"` body whose opening quote is just before `at`.
fn skip_quoted(text: &[char], mut at: usize) -> usize {
    while at < text.len() {
        match text[at] {
            '\\' => at += 2,
            '"' => return at + 1,
            _ => at += 1,
        }
    }
    at
}

/// Past a raw string body whose opening `"` is just before `at`.
fn skip_raw(text: &[char], mut at: usize, hashes: usize) -> usize {
    while at < text.len() {
        if text[at] == '"' && (1..=hashes).all(|k| text.get(at + k) == Some(&'#')) {
            return at + 1 + hashes;
        }
        at += 1;
    }
    at
}

/// Past a character literal or a lifetime that starts with the `'` at `at`.
fn skip_tick(text: &[char], at: usize) -> usize {
    if text.get(at + 1) == Some(&'\\') {
        let mut end = at + 3;
        while end < text.len() && text[end] != '\'' {
            end += 1;
        }
        return end + 1;
    }
    if text.get(at + 2) == Some(&'\'') {
        return at + 3;
    }
    let mut end = at + 1;
    while end < text.len() && is_ident_char(text[end]) {
        end += 1;
    }
    end
}

fn tokens(source: &str) -> Vec<Tok> {
    let text: Vec<char> = source.chars().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let c = text[at];
        let next = text.get(at + 1).copied();
        if c.is_whitespace() {
            at += 1;
        } else if c == '/' && next == Some('/') {
            while at < text.len() && text[at] != '\n' {
                at += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0;
            while at < text.len() {
                if text[at] == '/' && text.get(at + 1) == Some(&'*') {
                    depth += 1;
                    at += 2;
                } else if text[at] == '*' && text.get(at + 1) == Some(&'/') {
                    depth -= 1;
                    at += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    at += 1;
                }
            }
        } else if c == '"' {
            at = skip_quoted(&text, at + 1);
        } else if c == '\'' {
            at = skip_tick(&text, at);
        } else if c.is_alphabetic() || c == '_' {
            let start = at;
            while at < text.len() && is_ident_char(text[at]) {
                at += 1;
            }
            let word: String = text[start..at].iter().collect();
            let after = text.get(at).copied();
            if matches!(word.as_str(), "b" | "c") && after == Some('"') {
                at = skip_quoted(&text, at + 1);
                continue;
            }
            if word == "b" && after == Some('\'') {
                at = skip_tick(&text, at);
                continue;
            }
            if matches!(word.as_str(), "r" | "br" | "cr") && matches!(after, Some('"' | '#')) {
                let mut end = at;
                while text.get(end) == Some(&'#') {
                    end += 1;
                }
                let hashes = end - at;
                if text.get(end) == Some(&'"') {
                    at = skip_raw(&text, end + 1, hashes);
                    continue;
                }
                if word == "r"
                    && hashes == 1
                    && text
                        .get(end)
                        .is_some_and(|c| c.is_alphabetic() || *c == '_')
                {
                    let start = end;
                    while end < text.len() && is_ident_char(text[end]) {
                        end += 1;
                    }
                    out.push(Tok::Ident(text[start..end].iter().collect(), true));
                    at = end;
                    continue;
                }
            }
            out.push(Tok::Ident(word, false));
        } else if c.is_ascii_digit() {
            while at < text.len() && is_ident_char(text[at]) {
                at += 1;
            }
        } else {
            at += 1;
            out.push(match c {
                ':' if next == Some(':') => {
                    at += 1;
                    Tok::PathSep
                }
                '{' => Tok::Open,
                '}' => Tok::Close,
                ';' => Tok::Semi,
                ',' => Tok::Comma,
                _ => Tok::Punct(c),
            });
        }
    }
    out
}

fn keyword(token: Option<&Tok>, word: &str) -> bool {
    matches!(token, Some(Tok::Ident(name, false)) if name == word)
}

/// The root of each path in a use tree, and whether it was written absolute
/// (`::name`, which in Rust 2018 and later is always an external crate).
fn use_roots(tree: &[Tok], absolute: bool) -> Vec<(String, bool)> {
    match tree.first() {
        Some(Tok::PathSep) => use_roots(&tree[1..], true),
        Some(Tok::Open) => {
            let mut out = Vec::new();
            let mut depth = 0;
            let mut start = 1;
            for (index, token) in tree.iter().enumerate().skip(1) {
                match token {
                    Tok::Open => depth += 1,
                    Tok::Close if depth == 0 => {
                        out.extend(use_roots(&tree[start..index], absolute));
                        break;
                    }
                    Tok::Close => depth -= 1,
                    Tok::Comma if depth == 0 => {
                        out.extend(use_roots(&tree[start..index], absolute));
                        start = index + 1;
                    }
                    _ => {}
                }
            }
            out
        }
        Some(Tok::Ident(name, _)) => vec![(name.clone(), absolute)],
        _ => Vec::new(),
    }
}

/// One crate-or-module root a file imports.
struct Import {
    /// The scope the import sits in (see [`FileImports::parent`]).
    scope: usize,
    root: String,
    /// Written `::root`, or by `extern crate`: always an external crate.
    external: bool,
    keyword: &'static str,
}

/// A file's imports and the module names it declares, by scope.
struct FileImports {
    found: Vec<Import>,
    /// `(scope, name)` for each `mod name;` or `mod name { … }`.
    declared: BTreeSet<(usize, String)>,
    /// Each scope's enclosing scope. Scope 0 is the file; every brace opens
    /// one more. A block sees the names its enclosing blocks declare, up to its
    /// module, so a block's parent is the scope around it. A module (the file,
    /// or an inline `mod name { … }`) has none: Rust does not look past it
    /// without `super::`.
    parent: Vec<Option<usize>>,
    /// Each inline module's scope, and the `(scope, name)` that declares it.
    modules: BTreeMap<usize, (usize, String)>,
    /// The file starts `#![cfg(…)]` (or `cfg_attr`, `path`), so the module it
    /// holds may be absent.
    conditional: bool,
}

impl FileImports {
    /// Whether `name` is a module that `scope` can see: declared in it, or in a
    /// scope around it within the same module.
    fn declares(&self, scope: usize, name: &str) -> bool {
        let mut at = Some(scope);
        while let Some(scope) = at {
            if self.declared.contains(&(scope, name.to_string())) {
                return true;
            }
            at = self.parent[scope];
        }
        false
    }
}

fn imports(source: &str) -> FileImports {
    let tokens = tokens(source);
    let mut file = FileImports {
        found: Vec::new(),
        declared: BTreeSet::new(),
        parent: vec![None],
        modules: BTreeMap::new(),
        conditional: false,
    };
    let mut scopes = vec![0];
    // The next item carries a `#[cfg…]`, `#[cfg_attr…]` or `#[path…]`. Only an
    // unconditional `mod` makes a `use` of its name local: a conditional one
    // may be absent exactly when the `use` is compiled.
    let mut conditional = false;
    let mut at = 0;
    while at < tokens.len() {
        let here = *scopes.last().expect("the file scope is never popped");
        if let Some((end, inner, gated)) = attribute(&tokens, at) {
            if gated && !inner {
                conditional = true;
            } else if gated && here == 0 {
                // An inner `#![cfg…]` gates the module it sits in: this file's,
                file.conditional = true;
            } else if let Some(declaration) = file.modules.get(&here).filter(|_| gated) {
                // or an inline one's.
                file.declared.remove(declaration);
            }
            at = end;
            continue;
        }
        if keyword(tokens.get(at), "mod") {
            let counts = !conditional;
            conditional = false;
            if let Some(Tok::Ident(name, _)) = tokens.get(at + 1) {
                let declaration = (here, name.clone());
                match tokens.get(at + 2) {
                    Some(Tok::Semi) => {
                        if counts {
                            file.declared.insert(declaration);
                        }
                        at += 3;
                        continue;
                    }
                    Some(Tok::Open) => {
                        let scope = file.parent.len();
                        if counts {
                            file.declared.insert(declaration.clone());
                            file.modules.insert(scope, declaration);
                        }
                        scopes.push(scope);
                        file.parent.push(None);
                        at += 3;
                        continue;
                    }
                    _ => {}
                }
            }
        } else if keyword(tokens.get(at), "use")
            && !matches!(tokens.get(at + 1), Some(Tok::Punct(_)))
        {
            conditional = false;
            // `use<'a>` in `impl Trait + use<'a>` is not an import, and is the
            // only `use` followed by punctuation.
            let end = tokens[at + 1..]
                .iter()
                .position(|token| *token == Tok::Semi)
                .map_or(tokens.len(), |offset| at + 1 + offset);
            for (root, external) in use_roots(&tokens[at + 1..end], false) {
                file.found.push(Import {
                    scope: here,
                    root,
                    external,
                    keyword: "use",
                });
            }
            at = end + 1;
            continue;
        } else if keyword(tokens.get(at), "extern") && keyword(tokens.get(at + 1), "crate") {
            conditional = false;
            if let Some(Tok::Ident(name, _)) = tokens.get(at + 2) {
                file.found.push(Import {
                    scope: here,
                    root: name.clone(),
                    external: true,
                    keyword: "extern crate",
                });
                at += 3;
                continue;
            }
        } else if tokens[at] == Tok::Open {
            conditional = false;
            scopes.push(file.parent.len());
            file.parent.push(Some(here));
        } else if tokens[at] == Tok::Close {
            conditional = false;
            if scopes.len() > 1 {
                scopes.pop();
            }
        } else if matches!(tokens[at], Tok::Semi | Tok::Comma) {
            conditional = false;
        }
        at += 1;
    }
    file
}

/// The attribute starting at `at` (`#[…]`, or inner `#![…]`): where it ends,
/// whether it is inner, and whether it can make its item absent or come from
/// another file (`cfg`, `cfg_attr`, `path`).
fn attribute(tokens: &[Tok], at: usize) -> Option<(usize, bool, bool)> {
    if tokens.get(at) != Some(&Tok::Punct('#')) {
        return None;
    }
    let inner = tokens.get(at + 1) == Some(&Tok::Punct('!'));
    let open = at + 1 + usize::from(inner);
    if tokens.get(open) != Some(&Tok::Punct('[')) {
        return None;
    }
    let mut depth = 0;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Tok::Punct('[') => depth += 1,
            Tok::Punct(']') => {
                depth -= 1;
                if depth == 0 {
                    let conditional = tokens[open..index].iter().any(|token| {
                        matches!(token, Tok::Ident(name, false)
                            if matches!(name.as_str(), "cfg" | "cfg_attr" | "path"))
                    });
                    return Some((index + 1, inner, conditional));
                }
            }
            _ => {}
        }
    }
    None
}

/// What is wrong with one source file, and the denied names it was allowed to
/// `use` because its own module declares them.
///
/// A `use` root that its scope can see declared with `mod` is that module,
/// which is how Rust resolves it: `server/routes/mod.rs` declares
/// `mod pipeline;` (the room's admin route for the pipeline's pushes) and then
/// `use pipeline::admin_routes;`. A `mod` inside a block is seen only inside
/// that block, and a conditional `mod` (see [`imports`]) is not counted, nor
/// is a `mod name;` whose file, in `conditional`, gates itself. An absolute
/// `::name` and `extern crate` always name a crate, so they get no such reading.
fn file_violations(
    file: &str,
    scanned: &FileImports,
    conditional: &BTreeSet<String>,
) -> (Vec<String>, BTreeSet<String>) {
    let mut violations = Vec::new();
    let mut local = BTreeSet::new();
    for import in &scanned.found {
        if !denied(&import.root) {
            continue;
        }
        if !import.external
            && !conditional.contains(&import.root)
            && scanned.declares(import.scope, &import.root)
        {
            local.insert(import.root.clone());
            continue;
        }
        violations.push(format!(
            "{file}: `{}` of denied crate `{}`",
            import.keyword, import.root
        ));
    }
    (violations, local)
}

/// One file on its own, as the planted cases use it.
fn source_violations(file: &str, source: &str) -> (Vec<String>, BTreeSet<String>) {
    file_violations(file, &imports(source), &BTreeSet::new())
}

struct SourceScan {
    files: usize,
    imports: usize,
    violations: Vec<String>,
    local: BTreeSet<String>,
}

/// The module a file holds when `mod name;` loads it: `name.rs` or
/// `name/mod.rs`.
fn module_name(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    if stem == "mod" {
        Some(path.parent()?.file_name()?.to_str()?.to_string())
    } else {
        Some(stem.to_string())
    }
}

fn scan_source(crate_dir: &Path) -> SourceScan {
    let mut paths = Vec::new();
    rust_files(&crate_dir.join("src"), &mut paths);
    let files: Vec<(PathBuf, String)> = paths
        .into_iter()
        .map(|path| {
            let source = read(&path);
            (
                path.strip_prefix(crate_dir).unwrap_or(&path).to_path_buf(),
                source,
            )
        })
        .collect();
    scan_files(&files)
}

/// Source files as paths within the crate and their text.
fn scan_files(files: &[(PathBuf, String)]) -> SourceScan {
    let scanned: Vec<FileImports> = files.iter().map(|(_, source)| imports(source)).collect();
    // A module file that gates itself makes every `mod` of its name conditional.
    let conditional: BTreeSet<String> = files
        .iter()
        .zip(&scanned)
        .filter(|(_, file)| file.conditional)
        .filter_map(|((path, _), _)| module_name(path))
        .collect();
    let mut scan = SourceScan {
        files: files.len(),
        imports: 0,
        violations: Vec::new(),
        local: BTreeSet::new(),
    };
    for ((path, _), file) in files.iter().zip(&scanned) {
        scan.imports += file.found.len();
        let (violations, local) = file_violations(&path.display().to_string(), file, &conditional);
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
    let manifest = parse_toml(&read(&dir.join("Cargo.toml")));
    for (key, _) in manifest.entries() {
        assert!(
            KNOWN_ROOT_KEYS.contains(&key.as_str()),
            "Cargo.toml has a `{key}` section this test does not know; \
             read it here if it can declare a dependency"
        );
    }
    for (cfg, target) in manifest.get("target").map_or(&[][..], Toml::entries) {
        for (key, _) in target.entries() {
            assert!(
                DEPENDENCY_KINDS.contains(&key.as_str()),
                "Cargo.toml [target.{cfg}] has a `{key}` table this test does not read"
            );
        }
    }
    let deps = manifest_deps(&manifest);
    let declared = |table: &str, name: &str| {
        deps.iter()
            .any(|dep| dep.table == table && dep.name == name)
    };
    assert!(
        declared("dependencies", "axum"),
        "[dependencies] axum was not read"
    );
    assert!(
        declared("dependencies", "tokio-tungstenite"),
        "the optional, feature-gated tokio-tungstenite was not read"
    );
    assert!(
        declared("dev-dependencies", "tower"),
        "[dev-dependencies] tower was not read"
    );

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

/// The real manifest with `extra` added straight after its first `header` line.
fn planted(manifest: &str, header: &str, extra: &str) -> String {
    let line = format!("{header}\n");
    let out = manifest.replacen(&line, &format!("{line}{extra}\n"), 1);
    assert_ne!(out, manifest, "{header} is not in Cargo.toml");
    out
}

/// The mutations named on PQ-47, and the variants a review found, planted in
/// the real text. On the real files M1 and M2 stop `cargo --locked` before any
/// test runs; here they reach the checks, so the checks stay able to refuse them.
#[test]
fn the_planted_mutations_are_refused() {
    let dir = crate_dir();
    let manifest = read(&dir.join("Cargo.toml"));
    let refused = |text: &str, expected: &str| {
        let found = manifest_violations(&dir, text);
        assert!(
            found.iter().any(|violation| violation.contains(expected)),
            "expected a violation containing `{expected}`, got {found:?}"
        );
    };

    // M1: a path dependency on the pipeline.
    let m1 = planted(
        &manifest,
        "[dependencies]",
        "pipeline = { path = \"../pipeline\" }",
    );
    refused(&m1, "path `../pipeline` points into pipeline/");
    refused(&m1, "[dependencies] pipeline: denied crate name");

    // M2: an optional, feature-gated LLM client.
    let m2 = planted(
        &planted(&manifest, "[features]", "llm = [\"dep:anthropic\"]"),
        "[dependencies]",
        "anthropic = { version = \"0.1\", optional = true }",
    );
    refused(&m2, "[dependencies] anthropic: denied crate name");

    // Manifest variants: dotted keys, quoted keys, escaped and multi-line
    // strings, dotted and quoted headers, patch and replace.
    for (extra_header, extra, expected) in [
        (
            "[dependencies]",
            "anthropic.version = \"0.1\"\nanthropic.optional = true",
            "] anthropic: denied",
        ),
        (
            "[dependencies]",
            "client = { \"package\" = \"openai\", version = \"0.1\" }",
            "`openai`",
        ),
        (
            "[dependencies]",
            "helper = { \"path\" = \"../pipeline\" }",
            "points into pipeline/",
        ),
        (
            "[dependencies]",
            "helper = { path = '../pipeline/src' }",
            "points into pipeline/",
        ),
        (
            "[dependencies]",
            "client = { package = \"op\\u0065nai\", version = \"0.1\" }",
            "`openai`",
        ),
        (
            "[dependencies]",
            "client = { package = \"\"\"openai\"\"\", version = \"0.1\" }",
            "`openai`",
        ),
        (
            "[dev-dependencies]",
            "client = { package = \"async-openai\", version = \"0.1\" }",
            "`async-openai`",
        ),
    ] {
        refused(&planted(&manifest, extra_header, extra), expected);
    }
    for (appended, expected) in [
        (
            "[target.'cfg(unix)'.build-dependencies.helper]\npackage = \"llm\"",
            "`llm`",
        ),
        (
            "[target.'cfg(any())'.\"build-dependencies\"]\nllm = \"0.1\"",
            "] llm: denied",
        ),
        (
            "[target . 'cfg(any())' . build-dependencies]\nllm = \"0.1\"",
            "] llm: denied",
        ),
        (
            "[patch.crates-io]\nopenai = { git = \"https://example.com/x\" }",
            "[patch.crates-io] openai",
        ),
        (
            "[replace]\n\"miri:0.1.0\" = { git = \"https://example.com/x\" }",
            "[replace] miri",
        ),
        // The other package ID spec forms a `[replace]` key can take.
        (
            "[replace]\n\"openai@0.1.0\" = { git = \"https://example.com/x\" }",
            "[replace] openai",
        ),
        (
            "[replace]\n\"https://github.com/rust-lang/crates.io-index#openai@0.1.0\" = \
             { git = \"https://example.com/x\" }",
            "[replace] openai",
        ),
        (
            "[replace]\n\"registry+https://github.com/rust-lang/crates.io-index#openai:0.1.0\" = \
             { git = \"https://example.com/x\" }",
            "[replace] openai",
        ),
        (
            "[replace]\n\"https://example.com/openai#0.1.0\" = { git = \"https://example.com/x\" }",
            "[replace] openai",
        ),
        (
            "[replace]\n\"git+https://example.com/x/openai.git?branch=main\" = \
             { git = \"https://example.com/x\" }",
            "[replace] openai",
        ),
    ] {
        refused(&format!("{manifest}\n{appended}\n"), expected);
    }

    // Workspace inheritance: the spec is in `[workspace.dependencies]`, or in a
    // workspace above the room that this test cannot read.
    let inheriting = planted(
        &manifest,
        "[dependencies]",
        "client = { workspace = true, optional = true }",
    );
    refused(
        &format!(
            "{inheriting}\n[workspace.dependencies]\n\
             client = {{ package = \"openai\", version = \"0.1\" }}\n"
        ),
        "[workspace.dependencies] client: renames denied crate `openai`",
    );
    refused(
        &format!("{manifest}\n[workspace.dependencies]\nhelper = {{ path = \"../pipeline\" }}\n"),
        "[workspace.dependencies] helper: path `../pipeline` points into pipeline/",
    );
    // A name of its own, so a manifest that already inherits `client` from its
    // own workspace still shows this case.
    refused(
        &planted(
            &manifest,
            "[dependencies]",
            "pq47-inherited = { workspace = true }",
        ),
        "[dependencies] pq47-inherited: inherits from a workspace outside",
    );

    // M3: an import the compiler never sees.
    let source = |text: &str| source_violations("planted.rs", text).0;
    assert_eq!(
        source("#[cfg(any())]\nuse openai::Client;\n").len(),
        1,
        "M3"
    );

    // Source variants: layout, raw identifiers, absolute paths, nested modules.
    for text in [
        "#[cfg(any())]\nuse\n    openai::Client;\n",
        "#[cfg(any())]\nuse{openai::Client};\n",
        "fn f() {} #[cfg(any())] use openai::Client;\n",
        "use {std::fmt, ::openai::Client};\n",
        "use std::{\n    fmt,\n};\nuse anthropic\n    ::Messages;\n",
        "#[cfg(any())]\nuse r#openai::Client;\n",
        "extern crate rustc_driver;\n",
        "extern crate r#popquiz;\n",
        "mod popquiz;\nextern crate popquiz;\n",
        "mod pipeline;\n#[cfg(any())]\nuse ::pipeline::Client;\n",
        "mod inner {\n    mod openai;\n}\n#[cfg(any())]\nuse openai::Client;\n",
        "fn f() {\n    use generator::Gen;\n}\n",
        // A `mod` in a block is not seen outside it.
        "fn f() { mod openai {} }\n#[cfg(any())]\nuse openai::Client;\n",
        "fn f() {\n    { mod openai {} }\n    use openai::Client;\n}\n",
        "const _: () = { mod openai {} };\n#[cfg(any())]\nuse openai::Client;\n",
        // Nor is a module's `mod` seen inside an inline module, without `super::`.
        "mod openai;\nmod inner {\n    use openai::Client;\n}\n",
        // A conditional `mod` may be absent exactly when the `use` is compiled.
        "#[cfg(not(feature = \"llm\"))]\nmod openai {}\n#[cfg(feature = \"llm\")]\nuse openai::Client;\n",
        "#[cfg(not(feature = \"llm\"))]\npub(crate) mod openai;\nuse openai::Client;\n",
        "#[allow(dead_code)]\n#[cfg_attr(any(), cfg(any()))]\nmod openai;\nuse openai::Client;\n",
        "#[path = \"elsewhere.rs\"]\nmod openai;\nuse openai::Client;\n",
        "mod openai {\n    #![cfg(not(feature = \"llm\"))]\n}\n#[cfg(feature = \"llm\")]\nuse openai::Client;\n",
    ] {
        assert_eq!(source(text).len(), 1, "{text}");
    }

    // The room's own `mod pipeline` is local where it is seen, and only there.
    for text in [
        "mod pipeline;\npub(crate) use pipeline::admin_routes;\n",
        "mod pipeline;\nfn f() {\n    use pipeline::admin_routes;\n}\n",
        "fn f() {\n    mod pipeline {}\n    use pipeline::admin_routes;\n}\n",
        "fn f() {\n    mod pipeline {}\n    {\n        use pipeline::admin_routes;\n    }\n}\n",
        // Attributes that cannot remove it, and a gate on the item before it.
        "#[allow(dead_code)]\npub(crate) mod pipeline;\nuse pipeline::admin_routes;\n",
        "#[cfg(test)]\nmod tests {}\nmod pipeline;\nuse pipeline::admin_routes;\n",
        "#[cfg(test)]\nconst A: u8 = 1;\nmod pipeline;\nuse pipeline::admin_routes;\n",
        "#![cfg_attr(docsrs, feature(doc_cfg))]\nmod pipeline;\nuse pipeline::admin_routes;\n",
    ] {
        assert!(source(text).is_empty(), "{text}");
    }
    assert_eq!(source("pub(crate) use pipeline::admin_routes;\n").len(), 1);

    // A `mod name;` whose file gates itself, as `name.rs` or `name/mod.rs`.
    let scan = |files: &[(&str, &str)]| {
        let files: Vec<(PathBuf, String)> = files
            .iter()
            .map(|(path, text)| (PathBuf::from(path), text.to_string()))
            .collect();
        scan_files(&files).violations
    };
    let routes = "mod pipeline;\nuse pipeline::admin_routes;\n";
    for module in ["src/pipeline.rs", "src/pipeline/mod.rs"] {
        assert_eq!(
            scan(&[
                ("src/lib.rs", routes),
                (module, "#![cfg(not(feature = \"llm\"))]\n")
            ])
            .len(),
            1,
            "{module}"
        );
        assert!(
            scan(&[("src/lib.rs", routes), (module, "#![allow(dead_code)]\n")]).is_empty(),
            "{module}"
        );
    }
}

/// Text that only looks like a dependency or an import is not one.
#[test]
fn the_readers_do_not_mistake_text_for_code() {
    let dir = crate_dir();
    let manifest = read(&dir.join("Cargo.toml"));
    // Measured against the manifest's own result, so a real dependency planted
    // in it fails the manifest check alone, not this one too.
    let baseline = manifest_violations(&dir, &manifest);
    for (what, text) in [
        (
            "a multi-line string",
            planted(
                &manifest,
                "[package]",
                "metadata.example = \"\"\"\n[dependencies]\nopenai = \"0.1\"\n\"\"\"",
            ),
        ),
        (
            "a multi-line literal string",
            planted(
                &manifest,
                "[package]",
                "metadata.example = '''\n[dependencies]\nopenai = \"0.1\"\n'''",
            ),
        ),
        (
            "a comment",
            planted(&manifest, "[dependencies]", "# openai = \"0.1\""),
        ),
        (
            "an empty patch table",
            format!("{manifest}\n[patch.crates-io]\n"),
        ),
    ] {
        assert_eq!(manifest_violations(&dir, &text), baseline, "{what}");
    }

    for text in [
        "/* an example:\nuse openai::Client;\n*/\n",
        "/* nested /* comments */\nuse openai::Client;\n*/\n",
        "// use openai::Client;\n",
        "const EXAMPLE: &str = r#\"\nuse openai::Client;\n\"#;\n",
        "const EXAMPLE: &str = \"\\\"\nuse openai::Client;\n\";\n",
        "const QUOTE: char = '\"';\nconst EXAMPLE: &str = \"use openai::Client;\";\n",
        "fn f<'a>(x: &'a str) -> impl Sized + use<'a> { x }\n",
        "fn pipeline_name() -> &'static str { \"pipeline\" }\n",
    ] {
        assert!(source_violations("text.rs", text).0.is_empty(), "{text}");
    }
}

/// Cargo can lock two packages of one name and version from different sources,
/// and then names the source in each dependency line.
#[test]
fn the_lock_walk_tells_sources_apart() {
    let lock = r#"
version = 4

[[package]]
name = "room"
version = "0.1.0"
dependencies = [
 "shim 1.0.0 (git+https://example.com/shim)",
]

[[package]]
name = "shim"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "shim"
version = "1.0.0"
source = "git+https://example.com/shim"
dependencies = [
 "openai",
]

[[package]]
name = "openai"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
    let found = lock_violations(lock);
    assert!(
        found.iter().any(|violation| violation.contains("openai")),
        "the git `shim` reaches openai: {found:?}"
    );
    let packages = lock_packages(lock);
    let reached: Vec<_> = room_subgraph(&packages)
        .iter()
        .map(|package| (package.name.clone(), package.source.clone()))
        .collect();
    assert_eq!(
        reached.len(),
        3,
        "room, the git shim and openai: {reached:?}"
    );
}

/// The shape `cargo generate-lockfile` writes for one crate from two git
/// repositories (one on a branch): each package's source carries the commit it
/// is pinned to, and each dependency line names the source without it.
#[test]
fn the_lock_walk_matches_git_sources_without_their_commit() {
    let lock = r#"
version = 4

[[package]]
name = "room"
version = "0.1.0"
dependencies = [
 "shim 1.0.0 (git+https://example.com/b?branch=dev)",
 "shim 1.0.0 (git+https://example.com/a)",
]

[[package]]
name = "shim"
version = "1.0.0"
source = "git+https://example.com/b?branch=dev#84b81c3f13100567babfe14dea141f96d2d8f1f4"
dependencies = [
 "openai",
]

[[package]]
name = "shim"
version = "1.0.0"
source = "git+https://example.com/a#2e4a07543fbfb2fbda07356922c0fa81f18b9bbb"

[[package]]
name = "openai"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
    let packages = lock_packages(lock);
    let reached: Vec<_> = room_subgraph(&packages)
        .iter()
        .map(|package| (package.name.clone(), package.source.clone()))
        .collect();
    assert_eq!(reached.len(), 4, "room, both shims and openai: {reached:?}");
    assert!(
        lock_violations(lock)
            .iter()
            .any(|violation| violation.contains("openai")),
        "the branch `shim` reaches openai"
    );
    // The branch still tells the two repositories' sources apart.
    assert_eq!(
        resolve(
            &packages,
            "shim 1.0.0 (git+https://example.com/b?branch=dev)"
        ),
        vec![1]
    );
    assert_eq!(
        resolve(&packages, "shim 1.0.0 (git+https://example.com/a)"),
        vec![2]
    );
}
