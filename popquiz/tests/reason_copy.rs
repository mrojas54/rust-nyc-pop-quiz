//! G-5, GAP-14 (PQ-60): no route writes its own sentence.
//!
//! Every string the room sends a person in a `reason`, `message` or `refusal`
//! field comes from `src/copy.rs`. The audit's M5b3 put *Turn to your neighbour
//! and volunteer one.* into the answer route's reason as a literal and every
//! test passed; this scan is what fails it now.
//!
//! The scan parses each file with `syn` and walks its syntax tree (PQ-85), so
//! an attribute, a closure or a comment can neither hide one of the shapes
//! below nor fake one. Under `src/server/` and `src/connections/`, a string
//! literal containing a letter fails when it is
//!
//! - in the value of a `"reason"`, `"message"` or `"refusal"` key in a macro
//!   body (`json!`), read from the key's `:` to the next `,` at the same depth;
//!   or
//! - in the arguments of a call to a [`REASON_SINKS`] helper, which puts its
//!   argument in a `reason` field (admin's `refused`).
//!
//! The [`REASON_SOURCES`] build the refusals those routes forward through a
//! variable (`LoadError`, `RoomError::Refused`, `phase::Refused`,
//! `Ended::host_reason`). In them, a literal containing a letter also fails
//!
//! - in the arguments of any [`SOURCE_SINKS`] call, method call or macro, and,
//!   in the [`FORMAT_SOURCES`], of any `format!`: every `format!` there builds a
//!   refusal, so none may carry its own words (elsewhere one may build a
//!   digest, and is read only inside one of these shapes);
//! - in a struct literal's `reason` field (`Refused { reason: … }`, or the
//!   shorthand `Refused { reason }`); or
//! - anywhere in the body of a function named in [`REASON_FNS`], which returns
//!   a reason from a `match` or an `Err` rather than through a call.
//!
//! Where the scan reads an expression, a name stands for what it was bound to
//! (PQ-84, PQ-85). A local binding (`let`, or a `match`, `if let` or `for`
//! pattern) stands for the literals the expression it was bound from can
//! evaluate to (a literal, a `format!`'s text, through `.into()`, `Some(…)`,
//! a block's or branch's result), until its block ends or another binding
//! shadows it; a parameter stands for nothing. Any other name is a same-file `const` or `static`, matched by its
//! last path segment, so `WHY`, `Self::WHY` and `Phase::WHY` all read this
//! file's `WHY`; a path through `copy` is copy's, even where this file has a
//! constant of the same name. A binding or constant used nowhere a reason is
//! built passes.
//!
//! One carve-out: a `refusal` value that is one of [`REFUSAL_CODES`] passes.
//! Those are the codes a client branches on, not sentences anyone reads; any
//! other word there fails, so a new code is added to the list on purpose.
//! Literals elsewhere (keys, slugs in `slug()`, headers, attributes) are not
//! this scan's.

mod common;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use proc_macro2::{Delimiter, Spacing, Span, TokenStream, TokenTree};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Expr, Lit, Member, Pat, Token};

/// The JSON keys whose string a person may read.
const KEYS: &[&str] = &["reason", "message", "refusal"];

/// The refusal codes a client branches on: the host denials (`routes/mod.rs`)
/// and `JoinRefusal::slug()` (`connections/sessions.rs`).
const REFUSAL_CODES: &[&str] = &[
    "wrong_server",
    "wrong_role",
    "malformed",
    "unknown",
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
/// `format!` is read only inside a sink, a `reason` field or a [`REASON_FNS`]
/// body.
const FORMAT_SOURCES: &[&str] = &[
    "quiz_session/models/answers.rs",
    "quiz_session/state_manager.rs",
    "quiz_session/host/create.rs",
];

/// In [`FORMAT_SOURCES`], the macros that build a refusal.
const FORMAT_SINKS: &[&str] = &["format", "format_args"];

/// In [`REASON_SOURCES`], the functions that return a reason, each pinned to
/// the file it lives in (PQ-85): every literal in their bodies is bound for a
/// person (`Ended::host_reason`, `Review::check_shape`, the affirmation's and
/// the answers' `refusal`). A function of one of these names is read in every
/// source; the pin is what fails when one is renamed, so a namesake elsewhere
/// cannot stand in for it.
const REASON_FNS: &[(&str, &str)] = &[
    ("quiz_session/lifecycle.rs", "host_reason"),
    ("quiz_session/models/affirmation.rs", "check_shape"),
    ("quiz_session/models/affirmation.rs", "refusal"),
    ("quiz_session/models/answers.rs", "refusal"),
];

/// A literal sentence bound for a person: (line, what it is in, the literal).
/// Also counts the fields and sink calls it read, and names the reason
/// functions it read, so a scan that reads nothing cannot pass by accident.
#[derive(Default)]
struct Scan {
    hits: Vec<(usize, String, String)>,
    fields: usize,
    sink_calls: usize,
    reason_fns: Vec<String>,
}

/// A string literal and where it is written.
#[derive(Clone)]
struct Found {
    line: usize,
    column: usize,
    text: String,
}

/// An expression the scan is reading: what it is in, and whether a
/// [`REFUSAL_CODES`] code may stand there.
struct Reading {
    what: String,
    code_ok: bool,
}

/// `source` reads the file as a [`REASON_SOURCES`] file: its `reason` struct
/// fields and [`REASON_FNS`] bodies are read too. A fragment that is not a
/// whole file (a test's statement) is read as a function body.
fn scan(src: &str, sinks: &[&str], source: bool) -> Scan {
    let file = syn::parse_file(src)
        .or_else(|_| syn::parse_file(&format!("fn fragment() {{ {src} }}")))
        .unwrap_or_else(|e| panic!("the scan cannot parse this as a file or a body: {e}"));
    let mut consts = Consts::default();
    consts.visit_file(&file);
    let mut scanner = Scanner {
        sinks,
        source,
        consts: Rc::new(consts.0),
        scopes: vec![HashMap::new()],
        reading: Vec::new(),
        captures: Vec::new(),
        in_const: 0,
        seen: HashSet::new(),
        out: Scan::default(),
    };
    scanner.visit_file(&file);
    scanner.out
}

/// A string, byte-string or C-string literal, read, with where it is written.
fn found(lit: &Lit) -> Option<Found> {
    let text = match lit {
        Lit::Str(s) => s.value(),
        Lit::ByteStr(b) => String::from_utf8_lossy(&b.value()).into_owned(),
        Lit::CStr(c) => c.value().to_string_lossy().into_owned(),
        _ => return None,
    };
    let at = lit.span().start();
    Some(Found {
        line: at.line,
        column: at.column,
        text,
    })
}

/// Every `const` and `static` in the file, wherever it is declared: its name
/// and initializer.
#[derive(Default)]
struct Consts(HashMap<String, Vec<Expr>>);

impl Consts {
    fn add(&mut self, name: &syn::Ident, init: &Expr) {
        self.0
            .entry(name.to_string())
            .or_default()
            .push(init.clone());
    }

    /// The constants a macro's tokens declare
    /// (`macro_rules! m { () => { const WHY: &str = "…"; } }`): as items where
    /// they parse, else `const NAME: … = init;` token by token, at any depth.
    fn in_tokens(&mut self, ts: TokenStream) {
        if let Ok(file) = syn::parse2::<syn::File>(ts.clone()) {
            self.visit_file(&file);
            return;
        }
        let toks: Vec<TokenTree> = ts.into_iter().collect();
        let punct = |t: Option<&TokenTree>, c: char| matches!(t, Some(TokenTree::Punct(p)) if p.as_char() == c);
        for (i, t) in toks.iter().enumerate() {
            match t {
                TokenTree::Group(g) => self.in_tokens(g.stream()),
                TokenTree::Ident(kw) if kw == "const" || kw == "static" => {
                    let mut k = i + 1;
                    if matches!(toks.get(k), Some(TokenTree::Ident(m)) if m == "mut") {
                        k += 1;
                    }
                    let Some(TokenTree::Ident(name)) = toks.get(k) else {
                        continue;
                    };
                    if !punct(toks.get(k + 1), ':') {
                        continue;
                    }
                    let Some(eq) = (k + 2..toks.len()).find(|&j| punct(toks.get(j), '=')) else {
                        continue;
                    };
                    let end = (eq + 1..toks.len())
                        .find(|&j| punct(toks.get(j), ';'))
                        .unwrap_or(toks.len());
                    self.add(
                        name,
                        &token_expr(toks[eq + 1..end].iter().cloned().collect()),
                    );
                }
                _ => {}
            }
        }
    }
}

/// Tokens as an expression: parsed where they are Rust, else held as a macro
/// call the scan reads token by token (a `$x` in a macro template).
fn token_expr(ts: TokenStream) -> Expr {
    syn::parse2(ts.clone()).unwrap_or_else(|_| {
        Expr::Macro(syn::ExprMacro {
            attrs: Vec::new(),
            mac: syn::Macro {
                path: syn::Ident::new("tokens", Span::call_site()).into(),
                bang_token: Default::default(),
                delimiter: syn::MacroDelimiter::Paren(Default::default()),
                tokens: ts,
            },
        })
    })
}

impl<'ast> Visit<'ast> for Consts {
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        self.add(&c.ident, &c.expr);
        visit::visit_item_const(self, c);
    }
    fn visit_impl_item_const(&mut self, c: &'ast syn::ImplItemConst) {
        self.add(&c.ident, &c.expr);
        visit::visit_impl_item_const(self, c);
    }
    fn visit_trait_item_const(&mut self, c: &'ast syn::TraitItemConst) {
        if let Some((_, init)) = &c.default {
            self.add(&c.ident, init);
        }
        visit::visit_trait_item_const(self, c);
    }
    fn visit_item_static(&mut self, s: &'ast syn::ItemStatic) {
        self.add(&s.ident, &s.expr);
        visit::visit_item_static(self, s);
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        self.in_tokens(m.tokens.clone());
    }
}

struct Scanner<'s> {
    sinks: &'s [&'s str],
    source: bool,
    consts: Rc<HashMap<String, Vec<Expr>>>,
    /// The local bindings in scope, innermost last: each name and the literals
    /// of the expression it was bound from.
    scopes: Vec<HashMap<String, Vec<Found>>>,
    /// The expressions being read, innermost last.
    reading: Vec<Reading>,
    /// The open [`literals_in`](Self::literals_in) collections, innermost
    /// last: every literal met while one is open is added to each.
    captures: Vec<Vec<Found>>,
    /// How deep the scan is in constants' initializers: there nothing is
    /// counted, since the file's own walk counts it, and a constant that names
    /// itself stops.
    in_const: usize,
    /// A literal inside nested sinks (`refuse(&id, format_args!("…"))`) is one
    /// hit, not one per enclosing call.
    seen: HashSet<(usize, usize)>,
    out: Scan,
}

impl Scanner<'_> {
    fn counting(&self) -> bool {
        self.in_const == 0
    }

    fn read(&mut self, what: String, code_ok: bool, f: impl FnOnce(&mut Self)) {
        self.reading.push(Reading { what, code_ok });
        f(self);
        self.reading.pop();
    }

    fn sink(&mut self, name: &str) -> bool {
        let sink = self.sinks.contains(&name);
        if sink && self.counting() {
            self.out.sink_calls += 1;
        }
        sink
    }

    fn literal(&mut self, found: Found) {
        for capture in &mut self.captures {
            capture.push(found.clone());
        }
        let Some(r) = self.reading.last() else {
            return;
        };
        let letters = found.text.chars().any(char::is_alphabetic);
        let code = r.code_ok && REFUSAL_CODES.contains(&found.text.as_str());
        if letters && !code && self.seen.insert((found.line, found.column)) {
            let what = r.what.clone();
            self.out.hits.push((found.line, what, found.text));
        }
    }

    /// A name met where a reason is built or a binding is captured: `bare` is
    /// a one-segment path, which a local binding may answer for.
    fn name(&mut self, name: &str, bare: bool) {
        if self.reading.is_empty() && self.captures.is_empty() {
            return;
        }
        if bare {
            if let Some(bound) = self.scopes.iter().rev().find_map(|s| s.get(name)) {
                for found in bound.clone() {
                    self.literal(found);
                }
                return;
            }
        }
        let consts = Rc::clone(&self.consts);
        let Some(inits) = consts.get(name) else {
            return;
        };
        if self.in_const > 8 {
            return;
        }
        self.in_const += 1;
        // A constant's initializer sees no local binding.
        let scopes = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        for init in inits {
            self.visit_expr(init);
        }
        self.scopes = scopes;
        self.in_const -= 1;
    }

    /// Binds every name in `pat` to `found`; `_` binds nothing, and a bare
    /// name that is one of this file's constants is that constant, not a new
    /// binding.
    fn bind(&mut self, pat: &Pat, found: &[Found]) {
        match pat {
            Pat::Ident(p) => {
                let name = p.ident.to_string();
                let constant = p.by_ref.is_none()
                    && p.mutability.is_none()
                    && p.subpat.is_none()
                    && self.consts.contains_key(&name);
                if !constant {
                    let scope = self
                        .scopes
                        .last_mut()
                        .expect("the scan is always in a scope");
                    scope.insert(name, found.to_vec());
                }
                if let Some((_, sub)) = &p.subpat {
                    self.bind(sub, found);
                }
            }
            Pat::Or(p) => p.cases.iter().for_each(|c| self.bind(c, found)),
            Pat::Paren(p) => self.bind(&p.pat, found),
            Pat::Reference(p) => self.bind(&p.pat, found),
            Pat::Slice(p) => p.elems.iter().for_each(|e| self.bind(e, found)),
            Pat::Struct(p) => p.fields.iter().for_each(|f| self.bind(&f.pat, found)),
            Pat::Tuple(p) => p.elems.iter().for_each(|e| self.bind(e, found)),
            Pat::TupleStruct(p) => p.elems.iter().for_each(|e| self.bind(e, found)),
            Pat::Type(p) => self.bind(&p.pat, found),
            _ => {}
        }
    }

    /// Every literal `f` meets, counting nothing and reporting nothing.
    fn literals_in(&mut self, f: impl FnOnce(&mut Self)) -> Vec<Found> {
        let reading = std::mem::take(&mut self.reading);
        self.in_const += 1;
        self.captures.push(Vec::new());
        f(self);
        let got = self.captures.pop().unwrap_or_default();
        self.in_const -= 1;
        self.reading = reading;
        got
    }

    /// The literals `e`'s value can be, which is what a binding of it stands
    /// for: a literal; every literal of a macro (`format!`'s text); what a
    /// name stands for; and through a conversion's receiver (`.into()`), a
    /// call's arguments (`Some(…)`), a block's or branch's result, `+`, an
    /// array or tuple, `&`, `?` and casts. Not a method's arguments
    /// (`map.remove("key")`), a closure, or a `match` scrutinee, which pick a
    /// value rather than become one.
    fn value(&mut self, e: &Expr) -> Vec<Found> {
        match e {
            Expr::Lit(l) => found(&l.lit).into_iter().collect(),
            Expr::Macro(m) => self.literals_in(|s| s.tokens(m.mac.tokens.clone())),
            Expr::Path(_) => self.literals_in(|s| s.visit_expr(e)),
            Expr::MethodCall(m) => self.value(&m.receiver),
            Expr::Call(c) => c.args.iter().flat_map(|a| self.value(a)).collect(),
            Expr::Block(b) => self.block_value(&b.block),
            Expr::Unsafe(b) => self.block_value(&b.block),
            Expr::Const(b) => self.block_value(&b.block),
            Expr::If(i) => {
                let mut out = self.block_value(&i.then_branch);
                if let Some((_, other)) = &i.else_branch {
                    out.extend(self.value(other));
                }
                out
            }
            Expr::Match(m) => {
                let scrutinee = self.value(&m.expr);
                let mut out = Vec::new();
                for arm in &m.arms {
                    self.scopes.push(HashMap::new());
                    self.bind(&arm.pat, &scrutinee);
                    out.extend(self.value(&arm.body));
                    self.scopes.pop();
                }
                out
            }
            Expr::Binary(b) if matches!(b.op, syn::BinOp::Add(_)) => {
                let mut out = self.value(&b.left);
                out.extend(self.value(&b.right));
                out
            }
            Expr::Array(a) => a.elems.iter().flat_map(|x| self.value(x)).collect(),
            Expr::Tuple(t) => t.elems.iter().flat_map(|x| self.value(x)).collect(),
            Expr::Paren(p) => self.value(&p.expr),
            Expr::Group(g) => self.value(&g.expr),
            Expr::Reference(r) => self.value(&r.expr),
            Expr::Cast(c) => self.value(&c.expr),
            Expr::Try(t) => self.value(&t.expr),
            Expr::Await(a) => self.value(&a.base),
            Expr::Unary(u) => self.value(&u.expr),
            _ => Vec::new(),
        }
    }

    /// A block's [`value`](Self::value): its last expression's, with the
    /// block's own `let`s bound.
    fn block_value(&mut self, b: &syn::Block) -> Vec<Found> {
        self.scopes.push(HashMap::new());
        let mut out = Vec::new();
        for stmt in &b.stmts {
            match stmt {
                syn::Stmt::Local(l) => {
                    let bound = match &l.init {
                        Some(init) => self.value(&init.expr),
                        None => Vec::new(),
                    };
                    self.bind(&l.pat, &bound);
                }
                syn::Stmt::Expr(e, None) => out = self.value(e),
                _ => {}
            }
        }
        self.scopes.pop();
        out
    }

    fn function(&mut self, sig: &syn::Signature, body: &syn::Block) {
        // A function sees no local binding of the body it is declared in.
        let scopes = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        for input in &sig.inputs {
            if let syn::FnArg::Typed(t) = input {
                self.bind(&t.pat, &[]);
            }
        }
        let name = sig.ident.to_string();
        if self.source && REASON_FNS.iter().any(|(_, f)| *f == name) {
            if self.counting() {
                self.out.reason_fns.push(name.clone());
            }
            self.read(format!("fn {name}"), false, |s| s.visit_block(body));
        } else {
            self.visit_block(body);
        }
        self.scopes = scopes;
    }

    /// A macro's tokens: as comma-separated expressions where they parse
    /// (`format!`, `vec!`, `assert!`), else as statements and items (a macro
    /// body's `let`s and `fn`s), else piece by piece between commas.
    fn tokens(&mut self, ts: TokenStream) {
        if let Ok(args) = Punctuated::<Expr, Token![,]>::parse_terminated.parse2(ts.clone()) {
            for arg in &args {
                self.visit_expr(arg);
            }
            return;
        }
        if let Ok(stmts) = syn::Block::parse_within.parse2(ts.clone()) {
            self.scopes.push(HashMap::new());
            for stmt in &stmts {
                self.visit_stmt(stmt);
            }
            self.scopes.pop();
            return;
        }
        let toks: Vec<TokenTree> = ts.into_iter().collect();
        for piece in toks.split(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ',')) {
            self.piece(piece);
        }
    }

    /// A `json!` key the scan reads (`"reason": value`), if `key` is one and
    /// a `:` follows it.
    fn key(key: &proc_macro2::Literal, colon: Option<&TokenTree>) -> Option<String> {
        let colon = matches!(colon, Some(TokenTree::Punct(p)) if p.as_char() == ':' && p.spacing() == Spacing::Alone);
        match syn::parse2::<Lit>(TokenTree::Literal(key.clone()).into()) {
            Ok(Lit::Str(k)) if colon && KEYS.contains(&k.value().as_str()) => Some(k.value()),
            _ => None,
        }
    }

    /// Tokens that may be one expression; otherwise their groups, literals
    /// and names one by one.
    fn piece(&mut self, toks: &[TokenTree]) {
        if let Ok(e) = syn::parse2::<Expr>(toks.iter().cloned().collect()) {
            self.visit_expr(&e);
            return;
        }
        let punct = |k: Option<usize>, c: char| matches!(k.and_then(|k| toks.get(k)), Some(TokenTree::Punct(p)) if p.as_char() == c);
        let mut skip = 0;
        for (i, t) in toks.iter().enumerate() {
            if i < skip {
                continue;
            }
            // Where the tokens are not Rust (a `$x` in a macro template, a
            // `select!` arm, `vec![x; n]`), a sink call (`refuse(…)`,
            // `.refuse(…)`, `format!(…)`) and a `reason:` field are still read.
            if let TokenTree::Ident(id) = t {
                let name = id.to_string();
                // A [`REASON_FNS`] function a macro template defines, whose
                // body (`if $ready { "…" } …`) is not Rust: its body is read.
                if let (true, "fn", Some(TokenTree::Ident(f))) =
                    (self.source, name.as_str(), toks.get(i + 1))
                {
                    let f = f.to_string();
                    let body = toks
                        .iter()
                        .enumerate()
                        .skip(i + 2)
                        .find_map(|(j, t)| match t {
                            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                                Some((j, g))
                            }
                            _ => None,
                        });
                    if let (true, Some((j, body))) = (REASON_FNS.iter().any(|(_, n)| *n == f), body)
                    {
                        if self.counting() {
                            self.out.reason_fns.push(f.clone());
                        }
                        self.read(format!("fn {f}"), false, |s| s.tokens(body.stream()));
                        skip = j + 1;
                        continue;
                    }
                }
                let member = punct(i.checked_sub(1), '.');
                let bang = punct(Some(i + 1), '!');
                if let Some(TokenTree::Group(g)) = toks.get(i + 1 + usize::from(bang)) {
                    let called = bang || g.delimiter() == Delimiter::Parenthesis;
                    if called && self.sink(&name) {
                        let what = if member {
                            format!(".{name}(…)")
                        } else if bang {
                            format!("{name}!(…)")
                        } else {
                            format!("{name}(…)")
                        };
                        self.read(what, false, |s| s.tokens(g.stream()));
                        skip = i + 2 + usize::from(bang);
                        continue;
                    }
                }
                let field = matches!(toks.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':' && p.spacing() == Spacing::Alone);
                let metavariable = punct(i.checked_sub(1), '$');
                if self.source && name == "reason" && field && !member && !metavariable {
                    if self.counting() {
                        self.out.fields += 1;
                    }
                    let value = &toks[i + 2..];
                    self.read("reason:".into(), false, |s| s.piece(value));
                    return;
                }
            }
            match t {
                TokenTree::Group(g) => self.tokens(g.stream()),
                TokenTree::Literal(l) => {
                    if let Ok(lit) = syn::parse2::<Lit>(TokenTree::Literal(l.clone()).into()) {
                        self.visit_lit(&lit);
                    }
                    // A `json!` body's `"reason": value`, wherever it sits in
                    // the piece (after a `$(…)*` repetition, say).
                    if let Some(k) = Self::key(l, toks.get(i + 1)) {
                        if self.counting() {
                            self.out.fields += 1;
                        }
                        let code_ok = k == "refusal";
                        let value = &toks[i + 2..];
                        self.read(format!("\"{k}\""), code_ok, |s| s.piece(value));
                        return;
                    }
                }
                TokenTree::Ident(id) => {
                    // A member (`x.why`) or a path's head (`Phase::`) is not a name.
                    let member = punct(i.checked_sub(1), '.');
                    let head = punct(Some(i + 1), ':') && punct(Some(i + 2), ':');
                    if member || head {
                        continue;
                    }
                    let mut path = vec![id.to_string()];
                    let mut k = i;
                    while k >= 3 && punct(Some(k - 1), ':') && punct(Some(k - 2), ':') {
                        let TokenTree::Ident(seg) = &toks[k - 3] else {
                            break;
                        };
                        path.push(seg.to_string());
                        k -= 3;
                    }
                    if !path.iter().any(|s| s == "copy") {
                        self.name(&id.to_string(), path.len() == 1);
                    }
                }
                TokenTree::Punct(_) => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Scanner<'_> {
    // Doc comments and `#[allow(…)]` are not sentences anyone is sent.
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {}

    fn visit_lit(&mut self, lit: &'ast Lit) {
        if let Some(f) = found(lit) {
            self.literal(f);
        }
    }

    fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
        let segments = &p.path.segments;
        if !segments.iter().any(|s| s.ident == "copy") {
            if let Some(last) = segments.last() {
                let bare =
                    segments.len() == 1 && p.path.leading_colon.is_none() && p.qself.is_none();
                self.name(&last.ident.to_string(), bare);
            }
        }
        visit::visit_expr_path(self, p);
    }

    fn visit_block(&mut self, b: &'ast syn::Block) {
        self.scopes.push(HashMap::new());
        visit::visit_block(self, b);
        self.scopes.pop();
    }

    fn visit_local(&mut self, l: &'ast syn::Local) {
        let mut found = Vec::new();
        if let Some(init) = &l.init {
            self.visit_expr(&init.expr);
            found = self.value(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        self.bind(&l.pat, &found);
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.function(&f.sig, &f.block);
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.function(&f.sig, &f.block);
    }

    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        if let Some(body) = &f.default {
            self.function(&f.sig, body);
        }
    }

    fn visit_expr_closure(&mut self, c: &'ast syn::ExprClosure) {
        self.scopes.push(HashMap::new());
        for input in &c.inputs {
            self.bind(input, &[]);
        }
        self.visit_expr(&c.body);
        self.scopes.pop();
    }

    fn visit_expr_match(&mut self, m: &'ast syn::ExprMatch) {
        self.visit_expr(&m.expr);
        let found = self.value(&m.expr);
        for arm in &m.arms {
            self.scopes.push(HashMap::new());
            self.bind(&arm.pat, &found);
            self.visit_pat(&arm.pat);
            if let Some((_, guard)) = &arm.guard {
                self.visit_expr(guard);
            }
            self.visit_expr(&arm.body);
            self.scopes.pop();
        }
    }

    fn visit_expr_if(&mut self, i: &'ast syn::ExprIf) {
        // An `if let` binds for the condition's rest and the `then` block.
        self.scopes.push(HashMap::new());
        self.visit_expr(&i.cond);
        self.visit_block(&i.then_branch);
        self.scopes.pop();
        if let Some((_, other)) = &i.else_branch {
            self.visit_expr(other);
        }
    }

    fn visit_expr_while(&mut self, w: &'ast syn::ExprWhile) {
        self.scopes.push(HashMap::new());
        self.visit_expr(&w.cond);
        self.visit_block(&w.body);
        self.scopes.pop();
    }

    fn visit_expr_let(&mut self, l: &'ast syn::ExprLet) {
        self.visit_expr(&l.expr);
        let found = self.value(&l.expr);
        self.bind(&l.pat, &found);
        self.visit_pat(&l.pat);
    }

    fn visit_expr_for_loop(&mut self, f: &'ast syn::ExprForLoop) {
        self.visit_expr(&f.expr);
        let found = self.value(&f.expr);
        self.scopes.push(HashMap::new());
        self.bind(&f.pat, &found);
        self.visit_block(&f.body);
        self.scopes.pop();
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        self.visit_expr(&c.func);
        let name = match &*c.func {
            Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        };
        match name {
            Some(name) if self.sink(&name) => self.read(format!("{name}(…)"), false, |s| {
                c.args.iter().for_each(|a| s.visit_expr(a));
            }),
            _ => c.args.iter().for_each(|a| self.visit_expr(a)),
        }
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        self.visit_expr(&m.receiver);
        let name = m.method.to_string();
        if self.sink(&name) {
            self.read(format!(".{name}(…)"), false, |s| {
                m.args.iter().for_each(|a| s.visit_expr(a));
            });
        } else {
            m.args.iter().for_each(|a| self.visit_expr(a));
        }
    }

    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        for field in &s.fields {
            let reason = matches!(&field.member, Member::Named(n) if n == "reason");
            if self.source && reason {
                if self.counting() {
                    self.out.fields += 1;
                }
                self.read("reason:".into(), false, |sc| sc.visit_expr(&field.expr));
            } else {
                self.visit_expr(&field.expr);
            }
        }
        if let Some(rest) = &s.rest {
            self.visit_expr(rest);
        }
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if self.sink(&name) {
            self.read(format!("{name}!(…)"), false, |s| {
                s.tokens(m.tokens.clone())
            });
        } else {
            self.tokens(m.tokens.clone());
        }
    }
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
        reason_fns += s.reason_fns.len();
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
    // Floors against a scan that silently matches nothing: PQ-60 left 14
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
        "read only {reason_fns} bodies of {REASON_FNS:?}"
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
    // A renamed reason function would take its body out of the scan, and a
    // namesake in another file must not stand in for it (PQ-85).
    let srcs = sources();
    for (file, f) in REASON_FNS {
        let read = srcs.iter().filter(|(n, _)| n == file).any(|(n, t)| {
            scan(t, &sinks_for(n, true), true)
                .reason_fns
                .iter()
                .any(|g| g == f)
        });
        assert!(read, "fn {f} is gone from src/{file}; update REASON_FNS");
    }
}

/// Inside a `macro_rules!` template (`$x` placeholders) the scan reads only as
/// far as the tokens allow, and four review passes on the syn rewrite (PQ-85)
/// each found another shape it misses there. No file it reads defines one, so
/// none of those shapes can occur; this keeps it so.
#[test]
fn no_scanned_file_defines_a_macro() {
    #[derive(Default)]
    struct Macros(usize);
    impl<'ast> Visit<'ast> for Macros {
        fn visit_item_macro(&mut self, m: &'ast syn::ItemMacro) {
            if m.mac.path.is_ident("macro_rules") {
                self.0 += 1;
            }
            visit::visit_item_macro(self, m);
        }
    }
    let mut files = covered();
    files.extend(sources());
    assert!(files.len() > REASON_SOURCES.len(), "the scan read no files");
    let defining: Vec<String> = files
        .into_iter()
        .filter(|(_, src)| {
            let mut macros = Macros::default();
            macros.visit_file(&syn::parse_file(src).expect("a room source parses"));
            macros.0 > 0
        })
        .map(|(name, _)| name)
        .collect();
    assert!(
        defining.is_empty(),
        "{defining:?} define a macro_rules! macro; the reason scan cannot read its templates in full: write the code without one, or teach the scan the shape first"
    );
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

/// [`flags`] as the scan reads `phase.rs`: a source, but not a
/// [`FORMAT_SOURCES`] file, so a bare `format!` there is not a sink.
fn phase_flags(src: &str) -> Vec<String> {
    scan(src, &sinks_for("quiz_session/models/phase.rs", true), true)
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
        source_flags(r#"q.ok_or_else(|| RoomError::Refused("No question.".into()))?;"#),
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
        match r { Err(RoomError::Refused(why)) => refused(StatusCode::CONFLICT, why), _ => todo!() };
        pub struct Refused { pub from: Option<Phase>, pub command: Command, pub reason: String }
        let refuse = |reason: String| Refused { from: Some(phase), command, reason };
        match command {
            Command::Host(_) => Err(Refused { from: None, command, reason: copy::HOST_REFUSED_NO_ROOM_YET.into() }),
            _ => todo!(),
        };
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
    // A block initializer: its inner `;` does not end the constant.
    let block = r#"const WHY: &str = { let _ = (); "Not allowed." };
        fn f() -> Response { refused(StatusCode::CONFLICT, WHY) }"#;
    assert_eq!(flags(block), ["Not allowed."]);
    let block_field = r#"const WHY: &str = { let _ = (); "Not allowed." };
        Err(Refused { from: None, command, reason: WHY.into() })"#;
    assert_eq!(source_flags(block_field), ["Not allowed."]);
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
fn the_scan_reads_the_shapes_the_pq84_review_found() {
    // Gap 1: an attribute on a `reason` field does not hide it.
    assert_eq!(
        source_flags(
            r#"Err(Refused { from: None, command, #[allow(unused)] reason: "No room yet.".into() })"#
        ),
        ["No room yet."]
    );
    assert_eq!(
        source_flags(r#"Refused { #[cfg(all())] #[allow(x::y)] reason: "No.".into(), command }"#),
        ["No."]
    );
    // Gap 2: a closure's parameter named `reason` is not a field, and its body
    // is not a field's value, however it is written or wherever it sits; a
    // field whose value holds a closure is still read, whatever the fields
    // before it hold. Every shape here is one a token scan got wrong
    // (PQ-85's review passes 1 to 5).
    for field in [
        r#"Refused { from: None, reason: why.map_or_else(|| "No room yet.".into(), |w| w), command }"#,
        r#"Refused { from: None, reason: if a || b { "No room yet.".into() } else { copy::X.into() }, command }"#,
        r#"Refused { from: None, reason: if a | b { "No room yet.".into() } else { copy::X.into() }, command }"#,
        r#"Refused { from: None, reason: match flags | mask { 0 => "No room yet.".into(), _ => copy::X.into() } }"#,
        r#"Refused { from: a | b, command: x || y, reason: "No room yet.".into() }"#,
        r#"Refused { from: f.map(|x| x), note: |x| x, reason: "No room yet.".into() }"#,
        r#"Refused { from: |x: u8,| x, reason: "No room yet.".into(), note: |x: u8| x }"#,
        r#"Refused { from: |x: u8,| x, reason: if a | b { "No room yet.".into() } else { c || d } }"#,
        r#"Refused { from: x? | y, note: f() | g, reason: "No room yet.".into() }"#,
        r#"Refused { from: || x, note: |a| |b| a, reason: "No room yet.".into() }"#,
        r#"Refused { from: 'x' | Mask, reason: "No room yet.".into() }"#,
        r#"Refused { from: '\'' | Mask, note: b'x' | m, reason: "No room yet.".into() }"#,
        r#"Refused { from: 1. | Mask, note: v[0] | m, reason: "No room yet.".into() }"#,
        r#"Refused { from: &mut |x: u8,| x, reason: "No room yet.".into() }"#,
        r#"Refused { from: match (|x: u8,| x) { _ => 0 }, reason: "No room yet.".into() }"#,
    ] {
        assert_eq!(phase_flags(field), ["No room yet."], "{field}");
    }
    for closure in [
        r#"let note = |from: Phase, reason: &str| format!("{from:?}: {reason}");"#,
        r#"let n = f(|from: Phase, reason: &str| format!("{from:?}: {reason}"));"#,
        r#"Holder { f: |from: Phase, reason: &str| format!("{from:?}: {reason}") }"#,
        r#"let n = move |from: Phase, reason: &str| format!("{from:?}: {reason}");"#,
        r#"let n = &|from: u8, #[allow(unused)] reason: &str| format!("{from}: {reason}");"#,
        r#"let n = &mut |from: u8, reason: &str| format!("{from}: {reason}");"#,
        r#"let n = async move |from: u8, reason: &str| format!("{from}: {reason}");"#,
        r#"let n = |a: u8| |from: u8, reason: &str| format!("{a}{from}: {reason}");"#,
        r#"let n = f(|x: u8,| x, |from: u8, reason: &str| format!("{from}: {reason}"));"#,
        r#"let n = 'l: { break 'l |from: u8, #[allow(unused)] reason: &str| format!("{from}: {reason}"); };"#,
    ] {
        assert_eq!(phase_flags(closure), Vec::<String>::new(), "{closure}");
    }
    // Gap 3: `copy::NAME` is copy's, even where this file has its own `NAME`;
    // a bare name or any other path (`self::`, `Self::`, `Phase::`) is still
    // this file's.
    let shadow = r#"const HOST_ENDED_EXPIRED: &str = "expired";
        pub fn host_reason(self) -> &'static str {
            match self { Ended::Expired => copy::HOST_ENDED_EXPIRED, _ => crate::copy::X }
        }"#;
    assert_eq!(source_flags(shadow), Vec::<String>::new());
    assert_eq!(
        flags(r#"const WHY: &str = "No."; refused(a, self::WHY)"#),
        ["No."]
    );
    assert_eq!(
        source_flags(
            r#"impl Ended { const WHY: &str = "Ended."; pub fn host_reason(self) -> &'static str { Self::WHY } }"#
        ),
        ["Ended."]
    );
    assert_eq!(
        phase_flags(
            r#"impl Phase { const WHY: &str = "No room yet."; } Refused { from: None, command, reason: Phase::WHY.into() }"#
        ),
        ["No room yet."]
    );
    // A range bound is the name, not a member: `..WHY` is this file's `WHY`.
    for range in [
        r#"const WHY: &str = "No room yet."; Refused { reason: (..WHY).end.into() }"#,
        r#"const WHY: &str = "No room yet."; Refused { reason: (""..WHY).end.into() }"#,
    ] {
        assert_eq!(phase_flags(range), ["No room yet."], "{range}");
    }
    // Gap 5: a local binding is read as the literals it was bound from, where
    // the scan reads it: a `let`, or a pattern that destructures a literal.
    assert_eq!(
        phase_flags(
            r#"let why = format!("{phase:?} does not step the trace."); return Err(refuse(why));"#
        ),
        ["{phase:?} does not step the trace."]
    );
    assert_eq!(
        flags(r#"let mut why: &str = "No."; refused(s, why)"#),
        ["No."]
    );
    assert_eq!(
        phase_flags(r#"let why = "No room yet."; Refused { from: None, command, reason: why }"#),
        ["No room yet."]
    );
    assert_eq!(
        phase_flags(
            r#"let reason = "No room yet.".to_string(); Refused { from: None, command, reason }"#
        ),
        ["No room yet."]
    );
    assert_eq!(flags(r#"for why in ["No."] { refused(s, why); }"#), ["No."]);
    // Inside a macro whose tokens are not Rust, a sink call, a `reason:` field
    // and a constant the macro declares are still read; so is a C string. Each
    // shape is one the syn rewrite first missed (its review pass 1).
    for (src, want) in [
        (
            r#"macro_rules! m { ($prefix:expr) => { refuse($prefix.to_owned() + "No trace.") } }"#,
            "No trace.",
        ),
        (
            r#"macro_rules! m { ($c:expr) => { Refused { from: None, command: $c, reason: "No room yet.".into() } } }"#,
            "No room yet.",
        ),
        (
            r#"tokio::select! { _ = task() => refused(s, "No."), }"#,
            "No.",
        ),
        (r#"let v = vec![refused(s, "No."); 2];"#, "No."),
        (
            r#"macro_rules! define_reason { () => { const WHY: &str = "No trace."; }; }
            define_reason!(); fn f() { refused(s, WHY); }"#,
            "No trace.",
        ),
        (
            r#"macro_rules! d { ($p:expr) => { const WHY: &str = concat!($p, "No trace."); }; }
            fn f() { refused(s, WHY); }"#,
            "No trace.",
        ),
        (
            r#"refused(s, c"No trace.".to_string_lossy().into_owned())"#,
            "No trace.",
        ),
        // Its review pass 2: a `json!` key after a repetition, a reason
        // function a macro defines, and a macro body's `let`.
        (
            r#"macro_rules! m { ($($key:tt: $val:expr,)*) => { json!({ $($key: $val,)* "reason": "No room." }) } }"#,
            "No room.",
        ),
        (
            r#"macro_rules! m { () => { fn refusal() -> &'static str { "No room." } } }"#,
            "No room.",
        ),
        (
            r#"macro_rules! m { () => {{ let why = format!("No trace."); refuse(why) }} }"#,
            "No trace.",
        ),
        // Its review pass 3: a reason function in a template that is not Rust.
        (
            r#"macro_rules! m { ($ready:expr) => { fn refusal() -> &'static str { if $ready { "No room." } else { copy::HOST_ENDED_EXPIRED } } } }"#,
            "No room.",
        ),
        (
            r#"macro_rules! m { ($($i:item)*) => { $($i)* fn refusal() -> &'static str { "No room." } } }"#,
            "No room.",
        ),
    ] {
        assert_eq!(phase_flags(src), [want], "{src}");
    }
    assert_eq!(
        phase_flags(r#"macro_rules! m { ($reason:expr) => { refuse($reason) } }"#),
        Vec::<String>::new()
    );
    // Only in its block and until another binding shadows it, whether a
    // `let` or a pattern's; a field, member or label of that name is not the
    // binding; `_` binds nothing; one with no letter, or used nowhere a reason
    // is built, passes.
    let clean = r#"
        { let why = "No."; }
        refused(s, why);
        let why = "No."; let why = copy::NO; refused(s, why);
        let why = "No."; for why in [copy::A] { refused(s, why); }
        let why = "No."; match r { Err(why) => refused(s, why), _ => todo!() };
        let why = "No."; if let Some(why) = opt { refused(s, why); }
        let why = "No."; let f = |why: &str| refused(s, why);
        let why = "No."; refused(s, Note { why: copy::A });
        let label = "Reveal"; refused(s, copy::not_next(x.label));
        let _ = "No."; refused(s, match x { Some(_) => copy::A, _ => copy::B });
        let n = format!("{}", 3); refused(s, n);
        let hex = format!("sha256:{hex}"); digest(hex);
    "#;
    assert_eq!(flags(clean), Vec::<String>::new());
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
