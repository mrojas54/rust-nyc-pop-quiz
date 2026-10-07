"""Dedupe (SPEC 7.3): exact, normalized and near duplicates, the history, the wording.

AC-14 to AC-18. Every Rust program in this file is source text for dedupe to read,
and nothing here says what any of them prints: dedupe never runs a program, and
the candidates built below carry no `verified` record, so they make no claim about
output at all.
"""

from __future__ import annotations

import dataclasses
import json
import pathlib
import re
import shutil

import pytest

from popquiz import dedupe
from popquiz.bank import (
    History,
    Question,
    Review,
    Trace,
    load_bank,
    load_history,
    load_question,
    question_to_dict,
    save_history,
    save_question,
)
from popquiz.dedupe import (
    NEAR_DUPLICATE_THRESHOLD,
    UNIQUENESS_STATEMENT,
    DedupeError,
    check,
    entry_for,
    history_size,
    normalized_tokens,
    record,
    run,
    similarity,
    status,
    sync_with_bank,
    tokenize,
)

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
BANK = REPO / "bank"
MODULE = REPO / "pipeline" / "src" / "popquiz" / "dedupe.py"


@pytest.fixture(scope="module")
def bank() -> dict[str, Question]:
    return {q.id: q for q in load_bank(BANK)}


@pytest.fixture(scope="module")
def history(bank: dict[str, Question]) -> History:
    synced, _, _ = sync_with_bank(History(), bank.values())
    return synced


@pytest.fixture
def bank_copy(tmp_path: pathlib.Path) -> pathlib.Path:
    """A private copy of the committed bank, so a run can write to it."""
    target = tmp_path / "bank"
    shutil.copytree(BANK / "questions", target / "questions")
    shutil.copy(BANK / "history.json", target / "history.json")
    return target


def candidate(base: Question, qid: str, source: str) -> Question:
    """A candidate carrying a bank question's authored options and beats around a
    different program. No `verified`, no trace, no review: it claims nothing about
    what the program prints, and it arrives unjudged."""
    return dataclasses.replace(
        base, id=qid, source=source, verified=None, review=None, trace=Trace()
    )


def write_candidate(path: pathlib.Path, question: Question) -> pathlib.Path:
    path.write_text(json.dumps(question_to_dict(question), indent=2), encoding="utf-8")
    return path


def edit(source: str, old: str, new: str) -> str:
    assert old in source, f"fixture drift: {old!r} is not in the program"
    return source.replace(old, new)


@dataclasses.dataclass(frozen=True)
class Ran:
    returncode: int
    stdout: str
    stderr: str


@pytest.fixture
def cli(monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]):
    """The command line's entry point, run from a working directory of the test's
    choosing.

    In-process, because `just test` may not start a process (the guard in
    `test_runner.py`). That costs nothing here: `main` reads the bank and the
    history from disk on every call and the module keeps no state between calls, so
    a second run can only learn about a first one through the files.
    """

    def invoke(*args: str | pathlib.Path, cwd: pathlib.Path) -> Ran:
        monkeypatch.chdir(cwd)
        capsys.readouterr()
        try:
            code = dedupe.main([str(a) for a in args])
        except SystemExit as exit:  # argparse: --help, or a malformed flag
            code = exit.code if isinstance(exit.code, int) else 1
        out = capsys.readouterr()
        return Ran(code, out.out, out.err)

    return invoke


# A program with more structure than the bank's four: a struct, a method, a for
# loop over a tuple pattern, a match with a guard and a block arm, a closure, a
# type annotation. Its renamed and reformatted twins below are the AC-15 fixtures.
RICH = """\
struct Counter {
    ticks: u32,
}

impl Counter {
    fn bump(&mut self, by: u32) -> u32 {
        self.ticks += by;
        self.ticks
    }
}

fn main() {
    let mut c = Counter { ticks: 0 };
    let steps = [1, 2, 3];
    for (i, s) in steps.iter().enumerate() {
        let total = c.bump(*s);
        match total {
            n if n > 3 => {
                println!("{} big {}", i, n);
            }
            n => println!("{} {}", i, n),
        }
    }
    let doubled: Vec<u32> = steps.iter().map(|&x| x * 2).collect();
    println!("{:?}", doubled);
}
"""

# Every name RICH declares, renamed: the type, the field, the method, the
# parameter, and every binding. Nothing else changes.
RICH_RENAMED = """\
struct Tally {
    value: u32,
}

impl Tally {
    fn advance(&mut self, amount: u32) -> u32 {
        self.value += amount;
        self.value
    }
}

fn main() {
    let mut t = Tally { value: 0 };
    let deltas = [1, 2, 3];
    for (k, d) in deltas.iter().enumerate() {
        let sum = t.advance(*d);
        match sum {
            m if m > 3 => {
                println!("{} big {}", k, m);
            }
            m => println!("{} {}", k, m),
        }
    }
    let twice: Vec<u32> = deltas.iter().map(|&y| y * 2).collect();
    println!("{:?}", twice);
}
"""

# RICH reformatted: other line breaks and spacing, comments, the trailing commas
# rustfmt adds to vertical lists, and the comma after a block arm it removes.
RICH_REFORMATTED = """\
// A counter that says when it gets big.
struct Counter { ticks: u32 }

impl Counter {
    /// Adds `by` and returns the new total.
    fn bump( &mut self , by : u32 ) -> u32 { self.ticks += by; self.ticks }
}

fn main()
{
    let mut c = Counter {
        ticks: 0,
    };
    let steps = [
        1,
        2,
        3,
    ];
    for ( i , s ) in steps.iter().enumerate() {
        let total = c.bump( *s ); /* the step */
        match total {
            n if n > 3 => { println!("{} big {}", i, n); },
            n => println!(
                "{} {}",
                i,
                n,
            ),
        }
    }
    let doubled : Vec<u32> = steps
        .iter()
        .map(|&x| x * 2)
        .collect();
    println!("{:?}", doubled);
}
"""


# --------------------------------------------------------------------------- #
# The lexer
# --------------------------------------------------------------------------- #


def texts(source: str) -> list[str]:
    return [t.text for t in tokenize(source)]


def test_comments_and_layout_vanish() -> None:
    source = "/* a /* nested */ b */ x // line\n/// doc\n//! inner\n  y\n\t z"
    assert texts(source) == ["x", "y", "z"]


def test_lifetimes_are_not_char_literals() -> None:
    tokens = tokenize("fn f<'a>(x: &'a str) -> char { 'a' } 'outer: loop {}")
    kinds = {t.text: t.kind for t in tokens if t.text.startswith("'")}
    assert kinds == {"'a": "lifetime", "'a'": "char", "'outer": "lifetime"}


def test_literals_are_single_tokens() -> None:
    source = r"""r#"a "quoted" b"# b"by\"tes" br"raw" c"cstr" '\'' '\u{1F600}' b'\n' "x\"y" """
    assert tokenize(source) == [
        dedupe.Token("string", 'r#"a "quoted" b"#'),
        dedupe.Token("string", 'b"by\\"tes"'),
        dedupe.Token("string", 'br"raw"'),
        dedupe.Token("string", 'c"cstr"'),
        dedupe.Token("char", "'\\''"),
        dedupe.Token("char", "'\\u{1F600}'"),
        dedupe.Token("char", "b'\\n'"),
        dedupe.Token("string", '"x\\"y"'),
    ]


@pytest.mark.parametrize(
    "source, expected",
    [
        ("1..2", ["1", ".", ".", "2"]),
        ("0..=n", ["0", ".", ".", "=", "n"]),
        ("t.0", ["t", ".", "0"]),
        ("1_000u32 0xffu8 0b1010 1.5e3f64 2. 3.0", ["1_000u32", "0xffu8", "0b1010", "1.5e3f64", "2.", "3.0"]),
        ("1.max(2)", ["1", ".", "max", "(", "2", ")"]),
        ("r#match", ["r#match"]),
        ("a::b >> c", ["a", ":", ":", "b", ">", ">", "c"]),
    ],
)
def test_numbers_ranges_paths_and_punctuation(source: str, expected: list[str]) -> None:
    assert texts(source) == expected


@pytest.mark.parametrize("source", ['let s = "open', "/* never closed", "r#\"raw", "let c = '\\"])
def test_an_unterminated_literal_or_comment_is_an_error(source: str) -> None:
    with pytest.raises(DedupeError):
        tokenize(source)


def test_every_bank_question_lexes(bank: dict[str, Question]) -> None:
    for question in bank.values():
        assert tokenize(question.source), question.id


def test_keywords_and_the_wildcard_are_never_renamed(bank: dict[str, Question]) -> None:
    """q7's closure is `|&(_, k)| k`: `_` is the wildcard and stays, `k` is renamed."""
    tokens = normalized_tokens(bank["q7"].source)
    closure = tokens[tokens.index("sort_by_key") + 2 :][:8]
    assert closure == ["|", "&", "(", "_", ",", "$2", ")", "|"]
    assert "k" not in tokens
    assert {"fn", "let", "mut"} <= set(tokens)


# --------------------------------------------------------------------------- #
# AC-14 - an exact duplicate is rejected
# --------------------------------------------------------------------------- #


def test_a_byte_identical_resubmission_of_any_bank_question_is_rejected(
    bank: dict[str, Question], history: History
) -> None:
    for qid, question in bank.items():
        verdict = check("resubmitted", question.source, history)
        assert verdict.kind == "exact_duplicate", qid
        assert verdict.duplicate_of == qid
        assert not verdict.admitted
        assert verdict.statement is None


def test_one_changed_byte_is_not_an_exact_duplicate(
    bank: dict[str, Question], history: History
) -> None:
    """A trailing newline changes the bytes and nothing else, so the exact check
    lets it through and the normalized check - the next rule - catches it."""
    verdict = check("padded", bank["q3"].source + "\n", history)
    assert verdict.kind == "normalized_duplicate"
    assert verdict.duplicate_of == "q3"


def test_an_exact_resubmission_is_rejected_by_a_run_and_written_nowhere(
    bank: dict[str, Question], bank_copy: pathlib.Path
) -> None:
    before = sorted(p.name for p in (bank_copy / "questions").iterdir())
    report = run(bank_copy, [candidate(bank["q3"], "t14", bank["q3"].source)])
    assert [v.kind for v in report.verdicts] == ["exact_duplicate"]
    assert sorted(p.name for p in (bank_copy / "questions").iterdir()) == before
    assert "t14" not in load_history(bank_copy).exact_hashes


# --------------------------------------------------------------------------- #
# AC-15 - a normalized duplicate is rejected
# --------------------------------------------------------------------------- #


def rich_history() -> History:
    return record(History(), "rich", RICH)


@pytest.mark.parametrize(
    "label, source",
    [
        ("renamed", RICH_RENAMED),
        ("reformatted", RICH_REFORMATTED),
        (
            "renamed and reformatted",
            RICH_REFORMATTED.replace("Counter", "Tally")
            .replace("ticks", "value")
            .replace("bump", "advance")
            .replace("steps", "deltas")
            .replace("total", "sum")
            .replace("doubled", "twice"),
        ),
    ],
)
def test_renamed_bindings_and_reformatting_are_normalized_duplicates(
    label: str, source: str
) -> None:
    verdict = check(label, source, rich_history())
    assert verdict.kind == "normalized_duplicate", label
    assert verdict.duplicate_of == "rich"


def test_a_renamed_bank_question_is_rejected(bank: dict[str, Question], history: History) -> None:
    for qid, old, new in (
        ("q3", "v", "numbers"),
        ("q4", "b", "divisor"),
        ("q7", "k", "key"),
        ("q8", "first", "head"),
    ):
        source = re.sub(rf"\b{old}\b", new, bank[qid].source)
        assert source != bank[qid].source
        verdict = check("renamed", source, history)
        assert (verdict.kind, verdict.duplicate_of) == ("normalized_duplicate", qid)


def test_inline_format_arguments_are_renamed_with_their_binding() -> None:
    a = 'fn main() { let v = vec![3, 1]; println!("{v:?} {:>w$}", 7, w = 4); }'
    b = 'fn main() { let xs = vec![3, 1]; println!("{xs:?} {:>w$}", 7, w = 4); }'
    assert normalized_tokens(a) == normalized_tokens(b)


def test_a_string_that_is_printed_rather_than_formatted_is_kept_verbatim() -> None:
    """Only the format string's `{name}` is an argument. A second literal is printed
    as it stands, so renaming inside it would equate programs that print different
    text."""
    a = 'fn main() { let v = 1; println!("{} {}", v, "{v}"); }'
    b = 'fn main() { let w = 1; println!("{} {}", w, "{w}"); }'
    assert normalized_tokens(a) != normalized_tokens(b)


@pytest.mark.parametrize(
    "label, before, after",
    [
        # A different standard-library method: never renamed, because not declared.
        ("different std method", "v.dedup();", "v.sort();"),
        ("undeclared name differs", "v.dedup();", "v.reverse();"),
        ("a changed literal", "vec![1, 2, 2, 3, 2, 1, 1]", "vec![1, 2, 2, 3, 2, 1, 2]"),
    ],
)
def test_a_different_program_is_never_a_normalized_duplicate(
    bank: dict[str, Question], history: History, label: str, before: str, after: str
) -> None:
    verdict = check(label, edit(bank["q3"].source, before, after), history)
    assert verdict.admitted, f"{label}: {verdict}"


# Pairs where one program compiles and the other does not, or where the two mean
# different things. Every normalization rule is a claim that two programs are the
# same; each of these would be a false rejection, with nobody in the loop, of a
# question whose answer may well be "does not compile".
NEVER_THE_SAME = [
    (
        "a block-local declaration does not bind a call outside its scope",
        "fn main() { let x = 1; { let drop = 2; } drop(x); }",
        "fn main() { let x = 1; { let nope = 2; } nope(x); }",
    ),
    (
        "a binding does not shadow a name in its own initializer",
        "fn main() { let drop = drop(1); }",
        "fn main() { let nope = nope(1); }",
    ),
    (
        "derived Debug observes type and field names",
        '#[derive(Debug)] struct A { x: i32 } fn main() { println!("{:?}", A { x: 1 }); }',
        '#[derive(Debug)] struct B { y: i32 } fn main() { println!("{:?}", B { y: 1 }); }',
    ),
    (
        "doc comments cannot document function parameters",
        "fn f(x: u8) {} fn main() {}",
        "fn f(\n/// bad\nx: u8) {} fn main() {}",
    ),
    ("the binary entry point is not an arbitrary function", "fn main() {}", "fn other() {}"),
    ("an empty call cannot contain a comma", "fn f() {} fn main() { f(); }", "fn f() {} fn main() { f(,); }"),
    ("an empty array cannot contain a comma", "fn main() { let _: [u8; 0] = []; }", "fn main() { let _: [u8; 0] = [,]; }"),
    ("an empty struct cannot contain a comma", "struct S {} fn main() {}", "struct S {,} fn main() {}"),
    ("an empty enum cannot contain a comma", "enum E {} fn main() {}", "enum E {,} fn main() {}"),
    ("a repeated trailing comma is not layout", "fn main() { let a = [1,]; }", "fn main() { let a = [1,,]; }"),
    (
        "a block-valued field needs its comma",
        'fn main() { let p = P { a: Q { n: 1 }, b: 2 }; println!("{}", p.b); }',
        'fn main() { let p = P { a: Q { n: 1 } b: 2 }; println!("{}", p.b); }',
    ),
    (
        "a match-valued field needs its comma",
        "fn main() {\n    let p = P {\n        a: match 1 { _ => 2 },\n        b: 3,\n    };\n}\n",
        "fn main() {\n    let p = P {\n        a: match 1 { _ => 2 }\n        b: 3,\n    };\n}\n",
    ),
    ("a comma ending a block", "fn main() { f(), }", "fn main() { f() }"),
    ("a comma in an index", "fn main() { let v = vec![1]; let x = v[0,]; }", "fn main() { let v = vec![1]; let x = v[0]; }"),
    ("a comma in a repeat", "fn main() { let a = [0; 3,]; }", "fn main() { let a = [0; 3]; }"),
    ("a comma after struct update", "fn main() { let s = S { a: 1, ..b, }; }", "fn main() { let s = S { a: 1, ..b }; }"),
    ("a tuple of a closure", "fn main() { let t = (|a, b| a + b,); }", "fn main() { let t = (|a, b| a + b); }"),
    ("a tuple of a generic call", "fn main() { let t = (f::<u8, u8>(),); }", "fn main() { let t = (f::<u8, u8>()); }"),
    ("== is not = =", "fn main() { let a = 1 == 1; }", "fn main() { let a = 1 = = 1; }"),
    ("-> is not - >", "fn f() -> u8 { 1 }", "fn f() - > u8 { 1 }"),
    ("a doc comment that documents nothing", "fn main() {\n    let x = 1;\n    /// dangling\n}\n", "fn main() {\n    let x = 1;\n}\n"),
    ("an inner doc comment", "fn main() {\n    //! inner\n    let x = 1;\n}\n", "fn main() {\n    let x = 1;\n}\n"),
    (
        "a trailing comma a macro of the program's own may refuse",
        "macro_rules! m { ($a:expr, $b:expr) => { $a + $b }; }\nfn main() { m!(1, 2,); }",
        "macro_rules! m { ($a:expr, $b:expr) => { $a + $b }; }\nfn main() { m!(1, 2); }",
    ),
    (
        "a method named like the library's, renamed along with the library's call",
        "struct S { items: Vec<u8> }\nimpl S { fn len(&self) -> usize { self.items.len() } }",
        "struct S { items: Vec<u8> }\nimpl S { fn size(&self) -> usize { self.items.size() } }",
    ),
    # PQ-30 C6. The ticket's pair has no `main` and does not compile as a binary
    # (E0601), so `fn main() {}` is added: A exits 0, B 1 (a syntax error).
    (
        "a lifetime and a name do not share a placeholder",
        "fn f<'a>(x: &'a i32) {} fn main() {}",
        "fn f<a>(x: &a i32) {} fn main() {}",
    ),
    # Both compile; they call different macros. A derive keeps every declared
    # name, so a call to a kept macro must be kept too, or `m!` and `k!` merge.
    (
        "a kept macro's calls are kept",
        '#[derive(Clone)] struct S; macro_rules! m { () => { 1 } } macro_rules! k { () => { 2 } } fn main() { let _ = S; println!("{}", m!()); }',
        '#[derive(Clone)] struct S; macro_rules! m { () => { 1 } } macro_rules! k { () => { 2 } } fn main() { let _ = S; println!("{}", k!()); }',
    ),
]


@pytest.mark.parametrize("label, a, b", NEVER_THE_SAME, ids=[case[0] for case in NEVER_THE_SAME])
def test_normalization_never_equates_two_different_programs(label: str, a: str, b: str) -> None:
    assert normalized_tokens(a) != normalized_tokens(b), label
    assert check(label, b, record(History(), "a", a)).kind != "normalized_duplicate"


# Review round 2's open Critical, verbatim: a parameter's spelling was renamed
# program-wide, so a call to the library's `drop` in another function became the
# same token as an unresolved `nope`. The first program compiles; the second does not.
def test_a_function_parameter_does_not_bind_a_separate_function_call():
    a = 'fn f(drop: i32) {} fn main() { drop(1); }'
    b = 'fn f(nope: i32) {} fn main() { nope(1); }'
    assert check('new', b, record(History(), 'old', a)).kind != 'normalized_duplicate'


# The same defect for every other kind of declaration with a scope. In each pair the
# first program compiles and the second does not (rustc exit codes are in the
# ticket's validation note): the declared spelling is used once outside the scope
# that declared it, where it means the library's name, or a field it cannot rename.
SCOPE_NEVER_THE_SAME = [
    (
        "a closure parameter does not bind a call outside the closure",
        "fn main() { let f = |drop: i32| drop; drop(f(1)); }",
        "fn main() { let f = |nope: i32| nope; nope(f(1)); }",
    ),
    (
        "a closure passed as an argument ends with the call",
        "fn main() { let v: Vec<i32> = vec![1].into_iter().map(|drop| drop).collect(); drop(v); }",
        "fn main() { let v: Vec<i32> = vec![1].into_iter().map(|nope| nope).collect(); nope(v); }",
    ),
    (
        "a match arm's binding does not bind past its arm",
        "fn main() { match 1 { drop => { let _ = drop; } } drop(1); }",
        "fn main() { match 1 { nope => { let _ = nope; } } nope(1); }",
    ),
    (
        "a match arm without a comma ends where the next arm starts",
        "fn main() { let n = match 1 { 0 => 0, drop => if drop > 0 { 1 } else { 2 } _ => { drop(3); 4 } }; let _ = n; }",
        "fn main() { let n = match 1 { 0 => 0, nope => if nope > 0 { 1 } else { 2 } _ => { nope(3); 4 } }; let _ = n; }",
    ),
    (
        "an if-let binding does not bind past its block",
        "fn main() { if let Some(drop) = Some(1) { let _ = drop; } drop(2); }",
        "fn main() { if let Some(nope) = Some(1) { let _ = nope; } nope(2); }",
    ),
    (
        "an if-let binding does not bind the statements after it",
        "fn main() { if let Some(drop) = Some(1) { let _ = drop; } let _ = 0; drop(2); }",
        "fn main() { if let Some(nope) = Some(1) { let _ = nope; } let _ = 0; nope(2); }",
    ),
    (
        "a while-let binding does not bind past its loop",
        "fn main() { let mut o = Some(1); while let Some(drop) = o { o = None; let _ = drop; } drop(2); }",
        "fn main() { let mut o = Some(1); while let Some(nope) = o { o = None; let _ = nope; } nope(2); }",
    ),
    (
        "a for-loop pattern does not bind past its loop",
        "fn main() { for drop in 0..1 { let _ = drop; } drop(2); }",
        "fn main() { for nope in 0..1 { let _ = nope; } nope(2); }",
    ),
    (
        "a parameter pattern does not bind past its function",
        "fn f((drop, _): (i32, i32)) -> i32 { drop } fn main() { drop(f((1, 2))); }",
        "fn f((nope, _): (i32, i32)) -> i32 { nope } fn main() { nope(f((1, 2))); }",
    ),
    (
        "a generic parameter does not bind past its item",
        "fn f<String>(_: String) {} fn main() { let _ = String::new(); }",
        "fn f<Nope>(_: Nope) {} fn main() { let _ = Nope::new(); }",
    ),
    (
        "a method is not a free function",
        "struct S; impl S { fn drop(&self) {} } fn main() { drop(1); }",
        "struct S; impl S { fn nope(&self) {} } fn main() { nope(1); }",
    ),
    (
        "a function declared in a block does not bind outside it",
        "fn main() { { fn drop(_: i32) {} } drop(1); }",
        "fn main() { { fn nope(_: i32) {} } nope(1); }",
    ),
    (
        "a function in a module does not bind outside it",
        "mod m { pub fn drop(_: i32) {} } fn main() { drop(1); }",
        "mod m { pub fn nope(_: i32) {} } fn main() { nope(1); }",
    ),
    (
        "a field is not a free function",
        "struct S { drop: i32 } fn main() { drop(1); }",
        "struct S { nope: i32 } fn main() { nope(1); }",
    ),
    (
        "a macro is not a function",
        "macro_rules! drop { () => {}; } fn main() { drop(1); }",
        "macro_rules! nope { () => {}; } fn main() { nope(1); }",
    ),
    (
        "a parameter named like a library field, in that struct's shorthand",
        "use std::ops::Range; fn f(start: i32) -> Range<i32> { Range { start, end: 5 } } fn main() { f(1); }",
        "use std::ops::Range; fn f(begin: i32) -> Range<i32> { Range { begin, end: 5 } } fn main() { f(1); }",
    ),
    (
        "a parameter named like a library field, beside that field",
        "use std::ops::Range; fn f(start: i32) -> Range<i32> { Range { start: start, end: 5 } } fn main() { f(1); }",
        "use std::ops::Range; fn f(begin: i32) -> Range<i32> { Range { begin: begin, end: 5 } } fn main() { f(1); }",
    ),
    (
        "a binding named like a library field, in that struct's pattern",
        "fn main() { let std::ops::Range { start, end: _ } = 0..1; let _ = start; }",
        "fn main() { let std::ops::Range { begin, end: _ } = 0..1; let _ = begin; }",
    ),
    # PQ-30. The pairs below were each probed with rustc 1.96.1 (`--edition 2021
    # --crate-type bin`): the first program exits 0, the second 1.
    (
        "a free function is not a member a method call reaches",
        "fn count_ones(x: u32) -> u32 { x } fn main(){ let _ = 5u32.count_ones(); count_ones(1); }",
        "fn zzz(x: u32) -> u32 { x } fn main(){ let _ = 5u32.zzz(); zzz(1); }",
    ),
    (
        "a closure in an if-let head ends where the block opens",
        "fn main() { if let f = |drop: i32| drop { drop(f); } }",
        "fn main() { if let f = |nope: i32| nope { nope(f); } }",
    ),
    (
        "a top-level function does not reach into a nested module",
        "fn drop(_: i32) {} mod m { pub fn g() { drop(1); } } fn main() { m::g(); }",
        "fn nope(_: i32) {} mod m { pub fn g() { nope(1); } } fn main() { m::g(); }",
    ),
    (
        "a macro call before its macro_rules! is the library's",
        "fn main() { vec![1]; macro_rules! vec { ($($t:tt)*) => { () } } }",
        "fn main() { zz![1]; macro_rules! zz { ($($t:tt)*) => { () } } }",
    ),
    (
        "a comma-less while arm is not the next arm's pattern",
        "fn main(){ let (a,b)=(1,2); let v:Option<i32>=None; match v { None => while a == b { let _f: fn(i32) = drop; break; } Some(_z) => {} } }",
        "fn main(){ let (a,b)=(1,2); let v:Option<i32>=None; match v { None => while a == b { let _f: fn(i32) = nope; break; } Some(_z) => {} } }",
    ),
    (
        "a comma-less match-on-a-field arm is not the next arm's pattern",
        "struct S { a: i32 } fn main(){ let s = S { a: 1 }; let v: Option<i32> = None; match v { None => match s.a { _ => { let _f: fn(i32) = drop; } } Some(_z) => {} } }",
        "struct S { a: i32 } fn main(){ let s = S { a: 1 }; let v: Option<i32> = None; match v { None => match s.a { _ => { let _f: fn(i32) = nope; } } Some(_z) => {} } }",
    ),
    (
        "a comma-less for arm is not the next arm's pattern",
        "fn main(){ let xs = [1]; let v: Option<i32> = None; match v { None => for x in xs { let _f: fn(i32) = drop; let _ = x; } Some(_z) => {} } }",
        "fn main(){ let xs = [1]; let v: Option<i32> = None; match v { None => for x in xs { let _f: fn(i32) = nope; let _ = x; } Some(_z) => {} } }",
    ),
    (
        "an associated type of a generic is not the program's type",
        "type Item = u8; fn f<I: Iterator>(mut i: I) -> Option<I::Item> { i.next() } fn main() { let _: Item = 1; f(0..1); }",
        "type Zz = u8; fn f<I: Iterator>(mut i: I) -> Option<I::Zz> { i.next() } fn main() { let _: Zz = 1; f(0..1); }",
    ),
    (
        "a generic's associated function is not the program's function",
        "fn default() -> i32 { 0 } fn f<T: Default>() -> T { T::default() } fn main() { default(); f::<i32>(); }",
        "fn zz() -> i32 { 0 } fn f<T: Default>() -> T { T::zz() } fn main() { zz(); f::<i32>(); }",
    ),
    (
        "a module's re-export of the library's is not a parameter",
        "mod m { pub use std::mem::drop; } fn f(drop: i32) -> i32 { drop } fn main() { m::drop(f(1)); }",
        "mod m { pub use std::mem::drop; } fn f(nope: i32) -> i32 { nope } fn main() { m::nope(f(1)); }",
    ),
    (
        "a module's glob re-export is not a top-level function",
        "fn drop(_: i32) {} mod m { pub use std::mem::*; } fn main() { m::drop(1); drop(2); }",
        "fn zz(_: i32) {} mod m { pub use std::mem::*; } fn main() { m::zz(1); zz(2); }",
    ),
    (
        "a crate-root import is not a parameter",
        "use std::mem::drop; fn f(drop: i32) -> i32 { drop } fn main() { f(1); crate::drop(1); }",
        "use std::mem::drop; fn f(nope: i32) -> i32 { nope } fn main() { f(1); crate::nope(1); }",
    ),
    (
        "Self's library function is not a free function",
        "fn default() -> i32 { 1 } trait T { fn t() -> Self; } impl T for i32 { fn t() -> i32 { Self::default() } } fn main() { default(); i32::t(); }",
        "fn zz() -> i32 { 1 } trait T { fn t() -> Self; } impl T for i32 { fn t() -> i32 { Self::zz() } } fn main() { zz(); i32::t(); }",
    ),
    (
        "a generic's associated function is not tied to one impl of its trait",
        "struct S; impl Default for S { fn default() -> S { S } } fn f<T: Default>() -> T { T::default() } fn main() { let _: S = f(); }",
        "struct S; impl Default for S { fn zz() -> S { S } } fn f<T: Default>() -> T { T::zz() } fn main() { let _: S = f(); }",
    ),
    (
        "a kept path tail keeps its declaration",
        "trait Tr { type Out; } fn f<T: Tr>(_: T::Out) {} fn main() {}",
        "trait Tr { type Zz; } fn f<T: Tr>(_: T::Out) {} fn main() {}",
    ),
    # PQ-30, eighth class (client ruling 2026-10-07).
    (
        "an item in a library trait's impl keeps the trait's name",
        "struct S; impl Iterator for S { type Item = u8; fn next(&mut self) -> Option<Self::Item> { None } } fn main() { let _ = S.next(); }",
        "struct S; impl Iterator for S { type Zz = u8; fn next(&mut self) -> Option<Self::Zz> { None } } fn main() { let _ = S.next(); }",
    ),
]


@pytest.mark.parametrize(
    "label, a, b", SCOPE_NEVER_THE_SAME, ids=[case[0] for case in SCOPE_NEVER_THE_SAME]
)
def test_a_declared_name_binds_only_within_its_scope(label: str, a: str, b: str) -> None:
    assert normalized_tokens(a) != normalized_tokens(b), label
    assert check(label, b, record(History(), "a", a)).kind != "normalized_duplicate"


# And the renames scoping must keep: each name used only where it is declared.
SCOPED_THE_SAME = [
    (
        "a parameter used in its body",
        "fn f(a: i32) -> i32 { a + 1 } fn main() { f(2); }",
        "fn f(b: i32) -> i32 { b + 1 } fn main() { f(2); }",
    ),
    (
        "a closure parameter used in its body",
        "fn main() { let g = |a: i32| a * 2; g(3); }",
        "fn main() { let g = |b: i32| b * 2; g(3); }",
    ),
    (
        "match, if-let and for bindings used in their scopes",
        "fn main() { for a in 0..2 { match a { b => { if let Some(c) = Some(b) { let _ = c; } } } } }",
        "fn main() { for x in 0..2 { match x { y => { if let Some(z) = Some(y) { let _ = z; } } } } }",
    ),
    (
        "a generic parameter and a method",
        "struct W<T>(T); impl<T> W<T> { fn get(&self) -> &T { &self.0 } }",
        "struct W<U>(U); impl<U> W<U> { fn get(&self) -> &U { &self.0 } }",
    ),
    (
        "a struct with its fields, built and destructured",
        "struct P { x: i32 } fn main() { let p = P { x: 1 }; let P { x } = p; let _ = x; }",
        "struct Q { y: i32 } fn main() { let p = Q { y: 1 }; let Q { y } = p; let _ = y; }",
    ),
    (
        "a renamed lifetime",
        "fn f<'a>(x: &'a i32) -> &'a i32 { x } fn main() {}",
        "fn f<'b>(x: &'b i32) -> &'b i32 { x } fn main() {}",
    ),
    (
        "a method called after a dot",
        "struct S; impl S { fn get_n(&self) -> u8 { 1 } } fn main() { S.get_n(); }",
        "struct S; impl S { fn fetch_n(&self) -> u8 { 1 } } fn main() { S.fetch_n(); }",
    ),
    (
        "a free function called by name",
        "fn helper(x: u32) -> u32 { x } fn main() { helper(1); }",
        "fn aid(x: u32) -> u32 { x } fn main() { aid(1); }",
    ),
    (
        "a closure in an if-let head, used in the block",
        "fn main() { if let f = |a: i32| a { f(1); } }",
        "fn main() { if let f = |b: i32| b { f(1); } }",
    ),
    (
        "a top-level function reached from a module through super",
        "fn helper(_: i32) {} mod m { pub fn g() { super::helper(1); } } fn main() { m::g(); helper(2); }",
        "fn aid(_: i32) {} mod m { pub fn g() { super::aid(1); } } fn main() { m::g(); aid(2); }",
    ),
    (
        "a macro called after its macro_rules!",
        "fn main() { macro_rules! m { () => { 1 } } let _ = m!(); }",
        "fn main() { macro_rules! n { () => { 1 } } let _ = n!(); }",
    ),
    (
        "a struct pattern after a comma-less arm",
        "struct P { x: i32 } fn main() { match (P { x: 1 }) { P { x: 0 } => while false {} P { x } => { let _ = x; } } }",
        "struct Q { y: i32 } fn main() { match (Q { y: 1 }) { Q { y: 0 } => while false {} Q { y } => { let _ = y; } } }",
    ),
    (
        "a generic parameter as a path's head",
        "fn f<T: Default>() -> T { T::default() } fn main() { f::<i32>(); }",
        "fn f<U: Default>() -> U { U::default() } fn main() { f::<i32>(); }",
    ),
    (
        "a module's own function through its path",
        "mod m { pub fn g() {} } fn main() { m::g(); }",
        "mod n { pub fn h() {} } fn main() { n::h(); }",
    ),
    (
        "an enum's variant through its path",
        "enum E { A } fn main() { let _ = E::A; }",
        "enum F { B } fn main() { let _ = F::B; }",
    ),
    (
        "an associated function through Self",
        "struct S; impl S { fn make() -> S { S } fn g() -> S { Self::make() } } fn main() { S::g(); }",
        "struct S; impl S { fn build() -> S { S } fn g() -> S { Self::build() } } fn main() { S::g(); }",
    ),
    (
        "a top-level function through crate",
        "fn helper() {} fn main() { crate::helper(); }",
        "fn aid() {} fn main() { crate::aid(); }",
    ),
    (
        "an item in a program trait's impl",
        "trait Tr { fn go(&self); } struct S; impl Tr for S { fn go(&self) {} } fn main() { S.go(); }",
        "trait Tr { fn run(&self); } struct S; impl Tr for S { fn run(&self) {} } fn main() { S.run(); }",
    ),
]


@pytest.mark.parametrize("source", ["struct S}", "struct S x }", "enum E }"])
def test_a_malformed_item_header_normalizes_without_crashing(source: str) -> None:
    """Review round 3 found these raising KeyError: an unmatched `}` taken for a body."""
    assert normalized_tokens(source)


@pytest.mark.parametrize("label, a, b", SCOPED_THE_SAME, ids=[case[0] for case in SCOPED_THE_SAME])
def test_a_name_used_within_its_scope_is_still_renamed(label: str, a: str, b: str) -> None:
    assert normalized_tokens(a) == normalized_tokens(b), label


# And the optional commas that are dropped, because rustfmt moves them and the
# program means the same either way.
THE_SAME = [
    ("an array", "fn main() { let a = [1, 2,]; }", "fn main() { let a = [1, 2]; }"),
    ("a one-element array", "fn main() { let a = [1,]; }", "fn main() { let a = [1]; }"),
    ("a call", "fn main() { f(1, 2,); }", "fn main() { f(1, 2); }"),
    ("a formatting macro", 'fn main() { println!("{}", 1,); }', 'fn main() { println!("{}", 1); }'),
    ("a struct literal", "fn main() { let s = S { a: 1, }; }", "fn main() { let s = S { a: 1 }; }"),
    ("a struct definition", "struct S { a: u32, }", "struct S { a: u32 }"),
    ("an enum definition", "enum E { A, B, }", "enum E { A, B }"),
    ("a match", "fn main() { match 1 { _ => 2, } }", "fn main() { match 1 { _ => 2 } }"),
    ("a block arm", "fn main() { match 1 { 1 => { f() }, _ => {} } }", "fn main() { match 1 { 1 => { f() } _ => {} } }"),
    ("a documented item", "/// Adds.\nfn add() {}", "fn add() {}"),
]


@pytest.mark.parametrize("label, a, b", THE_SAME, ids=[case[0] for case in THE_SAME])
def test_optional_commas_and_documenting_comments_are_layout(label: str, a: str, b: str) -> None:
    assert normalized_tokens(a) == normalized_tokens(b), label


def test_a_leading_pipe_is_an_or_pattern_not_a_closure() -> None:
    """`| a | b =>` would otherwise read as a closure whose parameter is `a`."""
    s = dedupe._Stream(tokenize("fn main() { match x { | A | B => 1, _ => 2 } }"))
    pipe = next(k for k, t in enumerate(s.tokens) if t.text == "|")
    assert not dedupe._closure_opens(s, pipe)


def test_swapped_operands_are_different_programs(bank: dict[str, Question], history: History) -> None:
    source = edit(bank["q4"].source, "a / b, a % b", "b / a, b % a")
    assert check("swapped", source, history).admitted


def test_a_one_element_tuple_is_not_a_parenthesized_value() -> None:
    tuple_ = normalized_tokens('fn main() { let t = (5,); println!("{:?}", t); }')
    value = normalized_tokens('fn main() { let t = (5); println!("{:?}", t); }')
    assert tuple_ != value


def test_renaming_never_merges_two_names() -> None:
    """Placeholders are a bijection: `a - b` and `a - a` stay apart."""
    a = 'fn main() { let a = 1; let b = 2; println!("{}", a - b); }'
    b = 'fn main() { let a = 1; let b = 2; println!("{}", a - a); }'
    assert normalized_tokens(a) != normalized_tokens(b)


# --------------------------------------------------------------------------- #
# AC-16 - a near-duplicate goes to the review queue, marked
# --------------------------------------------------------------------------- #

# One statement added to q3. Measured with this module, not guessed: 0.94 against
# q3, below 0.6 against every other bank question.
SORTED_Q3_EDIT = ("    v.dedup();", "    v.sort();\n    v.dedup();")


def test_a_near_duplicate_is_marked_with_the_question_it_is_near(
    bank: dict[str, Question], history: History
) -> None:
    verdict = check("near", edit(bank["q3"].source, *SORTED_Q3_EDIT), history)
    assert verdict.kind == "near_duplicate"
    assert verdict.duplicate_of == "q3"
    assert verdict.similarity is not None and verdict.similarity >= NEAR_DUPLICATE_THRESHOLD
    assert verdict.admitted
    assert verdict.statement == UNIQUENESS_STATEMENT


def test_a_near_duplicate_lands_in_the_review_queue_neither_accepted_nor_dropped(
    bank: dict[str, Question], bank_copy: pathlib.Path
) -> None:
    near = candidate(bank["q3"], "t16", edit(bank["q3"].source, *SORTED_Q3_EDIT))
    report = run(bank_copy, [near])
    assert [v.kind for v in report.verdicts] == ["near_duplicate"]

    queued = load_question(bank_copy, "t16")  # written: not dropped
    assert queued.review is not None
    assert queued.review.near_duplicate_of == "q3"
    assert queued.review.status is None  # not accepted, not rejected
    assert not queued.review.affirmed()
    assert queued.source == near.source


def test_a_candidate_that_passes_joins_the_queue_unmarked(
    bank: dict[str, Question], bank_copy: pathlib.Path
) -> None:
    source = 'fn main() {\n    let s = String::from("héllo");\n    println!("{}", s.len());\n}\n'
    run(bank_copy, [candidate(bank["q3"], "t16-clear", source)])
    queued = load_question(bank_copy, "t16-clear")
    assert queued.review is None or queued.review.near_duplicate_of is None


def test_distinct_bank_questions_are_not_near_duplicates_of_each_other(
    bank: dict[str, Question], history: History
) -> None:
    """The negative control, on real programs at bank length rather than a
    contrived one: each bank question checked against the other three. Short
    programs share a lot of scaffolding (`fn main() {`, `let mut`, `println!`), so
    this is where an uncalibrated threshold would flood the queue first."""
    for qid, question in bank.items():
        verdict = check(qid, question.source, history, excluding=qid)
        assert verdict.kind == "cleared", (qid, verdict)
        assert verdict.closest != qid
        assert verdict.similarity is not None and verdict.similarity < NEAR_DUPLICATE_THRESHOLD


def test_the_threshold_is_configuration_with_an_uncalibrated_default(
    bank: dict[str, Question], history: History
) -> None:
    assert NEAR_DUPLICATE_THRESHOLD == 0.6
    source = edit(bank["q3"].source, *SORTED_Q3_EDIT)
    score = check("near", source, history).similarity
    assert score is not None
    assert check("near", source, history, threshold=score).kind == "near_duplicate"
    above = min(1.0, score + 0.01)
    assert check("near", source, history, threshold=above).kind == "cleared"


@pytest.mark.parametrize("threshold", [0, -0.1, 1.01, float("nan"), True, "0.6"])
def test_a_threshold_out_of_range_is_refused(threshold: object, history: History) -> None:
    with pytest.raises(DedupeError):
        check("x", "fn main() {}", history, threshold=threshold)  # type: ignore[arg-type]


def test_two_near_identical_candidates_in_one_batch_are_caught_against_each_other(
    bank: dict[str, Question], bank_copy: pathlib.Path
) -> None:
    first = 'fn main() {\n    let mut s = String::new();\n    s.push_str("ab");\n    println!("{}", s.len());\n}\n'
    second = re.sub(r"\bs\b", "text", first)
    assert second != first
    report = run(
        bank_copy,
        [candidate(bank["q3"], "t16-a", first), candidate(bank["q3"], "t16-b", second)],
    )
    kinds = [(v.candidate_id, v.kind, v.duplicate_of) for v in report.verdicts]
    assert kinds[0][1] == "cleared"
    assert kinds[1] == ("t16-b", "normalized_duplicate", "t16-a")


# --------------------------------------------------------------------------- #
# AC-17 - the history persists and grows, and its size is visible
# --------------------------------------------------------------------------- #


def test_the_history_persists_and_grows_across_two_runs_in_separate_working_directories(
    bank: dict[str, Question], tmp_path: pathlib.Path, cli
) -> None:
    """Two runs of the command line from two separate working directories, the
    second on a copy of the first's bank, as a clone on another machine would carry
    it. Relative paths, so each run finds its bank through its own cwd."""
    first_dir = tmp_path / "first"
    first_dir.mkdir()
    shutil.copytree(BANK / "questions", first_dir / "bank" / "questions")
    shutil.copy(BANK / "history.json", first_dir / "bank" / "history.json")
    write_candidate(
        first_dir / "near.json",
        candidate(bank["q3"], "t17-a", edit(bank["q3"].source, *SORTED_Q3_EDIT)),
    )

    first = cli("near.json", "--bank", "bank", cwd=first_dir)
    assert first.returncode == 0, first.stderr
    assert (
        "history: 0 → 5 programs recorded (+5: 4 backfilled from the bank, 1 admitted)"
        in first.stdout
    )

    second_dir = tmp_path / "second"
    shutil.copytree(first_dir / "bank", second_dir / "bank")
    write_candidate(
        second_dir / "other.json",
        candidate(
            bank["q8"],
            "t17-b",
            'fn main() {\n    let s = "a,b,,c";\n    println!("{}", s.split(\',\').count());\n}\n',
        ),
    )
    second = cli("other.json", "--bank", "bank", "--json", cwd=second_dir)
    assert second.returncode == 0, second.stderr
    history = json.loads(second.stdout)["history"]
    assert (history["before"], history["after"], history["growth"]) == (5, 6, 1)
    assert history["backfilled"] == []

    carried = load_history(second_dir / "bank")
    assert history_size(carried) == 6
    assert {"q3", "q4", "q7", "q8", "t17-a", "t17-b"} == set(carried.exact_hashes)
    assert history_size(load_history(first_dir / "bank")) == 5  # the first is untouched


def test_the_default_bank_is_found_from_any_working_directory(
    tmp_path: pathlib.Path, cli
) -> None:
    """With no `--bank`, the CLI reads this repository's bank wherever it is run
    from. No candidates means a status report, which writes nothing - so this is
    safe to run against the committed bank."""
    committed = (BANK / "history.json").read_bytes()
    result = cli(cwd=tmp_path)
    assert result.returncode == 0, result.stderr
    assert "programs recorded" in result.stdout
    assert "dry run, nothing written" in result.stdout
    assert (BANK / "history.json").read_bytes() == committed


def test_the_history_size_counts_programs_not_store_entries(history: History) -> None:
    assert history_size(history) == 4
    assert history.total() == 12  # T-14's sum of the three stores: not the same number
    assert history.sizes() == {"exact_hashes": 4, "ast_fingerprints": 4, "token_bigrams": 4}


def test_the_status_view_reports_the_size_and_writes_nothing(bank_copy: pathlib.Path) -> None:
    on_disk = (bank_copy / "history.json").read_bytes()
    report = status(bank_copy)
    assert (report.history_before, report.history_after) == (0, 4)
    assert report.backfilled == ("q3", "q4", "q7", "q8")
    assert report.dry_run
    assert (bank_copy / "history.json").read_bytes() == on_disk


def test_a_dry_run_writes_nothing(bank: dict[str, Question], bank_copy: pathlib.Path) -> None:
    snapshot = {p: p.read_bytes() for p in bank_copy.rglob("*") if p.is_file()}
    near = candidate(bank["q3"], "dry", edit(bank["q3"].source, *SORTED_Q3_EDIT))
    report = run(bank_copy, [near], write=False)
    assert report.verdicts[0].kind == "near_duplicate"
    assert report.history_after == 5
    assert {p: p.read_bytes() for p in bank_copy.rglob("*") if p.is_file()} == snapshot


def test_a_history_reset_to_the_empty_shape_is_rebuilt_from_the_bank(
    bank: dict[str, Question], bank_copy: pathlib.Path
) -> None:
    """Re-running the MVP migration writes the empty history (T-14). The bank still
    holds every question, and the next run's sync rebuilds the history from it."""
    near = candidate(bank["q3"], "t17-reset", edit(bank["q3"].source, *SORTED_Q3_EDIT))
    run(bank_copy, [near])
    save_history(bank_copy, History())
    rebuilt = status(bank_copy)
    assert rebuilt.history_after == 5
    assert set(rebuilt.backfilled) == {"q3", "q4", "q7", "q8", "t17-reset"}


def test_a_changed_bank_source_refreshes_its_entry(
    bank: dict[str, Question], bank_copy: pathlib.Path
) -> None:
    run(bank_copy, [candidate(bank["q3"], "t17-edit", 'fn main() { println!("{}", 1 + 1); }')])
    edited = dataclasses.replace(load_question(bank_copy, "t17-edit"), source='fn main() { println!("{}", 2 * 2); }')
    save_question(bank_copy, edited)
    report = status(bank_copy)
    assert report.refreshed == ("t17-edit",)
    assert any("refreshed: t17-edit" in line for line in report.lines())


def test_the_committed_history_still_loads_in_the_bank_formats_shape() -> None:
    loaded = load_history(BANK)
    assert set(loaded.sizes()) == {"exact_hashes", "ast_fingerprints", "token_bigrams"}


def test_record_refuses_a_different_program_under_an_existing_id() -> None:
    history = record(History(), "q", RICH)
    assert record(history, "q", RICH) == history  # the same program: a no-op
    with pytest.raises(DedupeError, match="never reused"):
        record(history, "q", RICH_RENAMED)
    replaced = record(history, "q", RICH_RENAMED, replace=True)
    assert replaced.exact_hashes["q"] == entry_for(RICH_RENAMED).exact_hash
    assert history_size(replaced) == 1


def test_excluding_leaves_out_only_that_ids_entry() -> None:
    history = record(record(History(), "a", RICH), "b", RICH)
    assert check("edit", RICH, history, excluding="a").duplicate_of == "b"
    only_a = record(History(), "a", RICH)
    assert check("edit", RICH, only_a, excluding="a").kind == "cleared"


def test_a_run_refuses_candidates_it_cannot_take_and_writes_nothing(
    bank: dict[str, Question], bank_copy: pathlib.Path
) -> None:
    snapshot = {p: p.read_bytes() for p in bank_copy.rglob("*") if p.is_file()}
    fresh = candidate(bank["q3"], "ok", 'fn main() { println!("{}", 3); }')
    for bad in (
        [candidate(bank["q3"], "q4", 'fn main() {}')],  # an id the bank holds
        [fresh, dataclasses.replace(fresh, source='fn main() { let x = 9; }')],  # repeated id
        [dataclasses.replace(fresh, review=Review(status="accepted"))],  # already judged
        [dataclasses.replace(fresh, review=Review(near_duplicate_of="q3"))],  # already marked
        [fresh, candidate(bank["q3"], "broken", 'fn main() { let s = "open; }')],  # cannot lex
    ):
        with pytest.raises(DedupeError):
            run(bank_copy, bad)
    assert {p: p.read_bytes() for p in bank_copy.rglob("*") if p.is_file()} == snapshot


def test_a_path_that_is_not_a_bank_is_refused(
    bank: dict[str, Question], tmp_path: pathlib.Path
) -> None:
    """A mistyped `--bank` must not become an empty bank that admits everything."""
    with pytest.raises(DedupeError, match="is not a bank"):
        run(tmp_path / "bnak", [candidate(bank["q3"], "typo", bank["q3"].source)])
    assert not (tmp_path / "bnak").exists()


def test_the_command_line_exit_codes(
    bank: dict[str, Question], bank_copy: pathlib.Path, cli
) -> None:
    here = bank_copy.parent
    passing = write_candidate(here / "pass.json", candidate(bank["q3"], "c1", 'fn main() { println!("{}", 5 % 3); }'))
    repeat = write_candidate(here / "repeat.json", candidate(bank["q4"], "c2", bank["q4"].source))
    assert cli(passing, "--bank", bank_copy, cwd=here).returncode == 0
    assert cli(repeat, "--bank", bank_copy, cwd=here).returncode == 1
    assert cli(passing, "--bank", bank_copy, "--threshold", "2", cwd=here).returncode == 2
    assert cli(here / "absent.json", "--bank", bank_copy, cwd=here).returncode == 2


# --------------------------------------------------------------------------- #
# AC-18 - the wording
# --------------------------------------------------------------------------- #

# The contract forbids "original". "unique" and "novel" are a deliberate widening:
# they make the same claim in other words.
FORBIDDEN = re.compile(r"original|unique|novel", re.IGNORECASE)


def test_the_module_never_says_original() -> None:
    assert "original" not in MODULE.read_text(encoding="utf-8").lower()


def test_every_uniqueness_statement_in_the_corpus_is_the_exact_sentence(
    bank: dict[str, Question], bank_copy: pathlib.Path, cli
) -> None:
    """A run covering all four verdicts, read through every surface this module
    has: the report's lines, its JSON, a dry run's lines, the command line's text
    and JSON output, and its help. Every uniqueness statement is the exact
    sentence - counted, so no second phrasing can hide beside it - and nothing says
    more."""
    q3 = bank["q3"]
    candidates = [
        candidate(q3, "exact", q3.source),
        candidate(q3, "normalized", re.sub(r"\bv\b", "numbers", q3.source)),
        candidate(q3, "near", edit(q3.source, *SORTED_Q3_EDIT)),
        candidate(q3, "clear", 'fn main() {\n    let x = 2u8.pow(3);\n    println!("{}", x);\n}\n'),
    ]
    dry = run(bank_copy, candidates, write=False)
    real = run(bank_copy, candidates)
    assert [v.kind for v in real.verdicts] == [
        "exact_duplicate",
        "normalized_duplicate",
        "near_duplicate",
        "cleared",
    ]

    here = bank_copy.parent
    paths = [write_candidate(here / f"{c.id}.json", dataclasses.replace(c, id=f"{c.id}-cli")) for c in candidates]
    text = cli(*paths, "--bank", bank_copy, "--dry-run", cwd=here)
    as_json = cli(*paths, "--bank", bank_copy, "--dry-run", "--json", cwd=here)
    help_text = cli("--help", cwd=here)
    assert text.returncode == as_json.returncode == 1  # two of four rejected

    corpus = "\n".join(
        [
            *real.lines(),
            *dry.lines(),
            json.dumps(real.to_dict(), ensure_ascii=False),
            text.stdout,
            as_json.stdout,
            help_text.stdout,
        ]
    )
    # The JSON names the bank directory, and pytest builds that path from this
    # test's own name. A path is not wording, so it is masked before the scan.
    corpus = corpus.replace(str(bank_copy), "<bank>")
    assert not FORBIDDEN.search(corpus)
    assert corpus.count("found") == corpus.count(UNIQUENESS_STATEMENT) > 0

    for verdict in real.verdicts:
        line = verdict.describe()
        if verdict.admitted:
            assert verdict.statement == UNIQUENESS_STATEMENT
            assert line.split(": ", 1)[1].startswith(UNIQUENESS_STATEMENT + ";")
            assert verdict.to_dict()["statement"] == UNIQUENESS_STATEMENT
        else:
            assert verdict.statement is None
            assert UNIQUENESS_STATEMENT not in line
            assert "statement" not in verdict.to_dict()


def test_similarity_is_jaccard() -> None:
    assert similarity({"a", "b"}, {"b", "c"}) == pytest.approx(1 / 3)
    assert similarity(set(), set()) == 0.0
    assert similarity({"a"}, {"a"}) == 1.0
