//! The sealed module (SPEC.md G-3, AC-61).
//!
//! Everything that joins an option to the verified output lives here and
//! nowhere else: the verified record and its `stdout`, the derived correct
//! option, the explanation's beats, the receipt lines, and the trace's
//! resolving step. [`load`] reads a bank record and splits it on the spot into
//! a [`PublicQuestion`] — which holds none of that — and a vault.
//!
//! **How it is sealed.** The secrets are fields of `vault::Vault`, and `vault`
//! is a private module nested inside this one. Rust field privacy is per
//! module, so not even the rest of *this* file can read them; the vault's only
//! read is [`Scheduled::open`], which demands a [`RevealWitness`]. A witness is
//! minted by [`crate::phase::Machine::revealed`] and nowhere else, and only in
//! `reveal`. So no code — not the public state query, not a route, not a later
//! ticket's helper — can learn the answer from a room that is not in `reveal`.
//! The most-chosen incorrect option is computed at `closed` (§4.5) *inside* the
//! vault and comes back as an opaque [`Verdict`] that opens the same way.
//!
//! **How that is proven.** The `compile_fail` doctests below, on
//! `crate::phase` and on `crate::rooms`, each paired with a twin that compiles
//! and differs only in the one forbidden line — so each can fail for one reason
//! only. Doctests see only what another crate would, so `tests/boundary.rs`
//! adds the in-crate half: it reads the source and asserts that `vault` holds
//! only the allowed functions, that every read in it takes the witness, and
//! that nothing sealed derives `Debug`, `Clone`, `Copy` or `Serialize`.
//!
//! ```compile_fail
//! // AC-61: the vault is a private module; its fields cannot be named.
//! fn peek(s: &room::answers::Scheduled) -> room::question::Letter { s.vault.correct }
//! ```
//!
//! Twin — the same types, reached the only way there is:
//!
//! ```
//! fn read(s: &room::answers::Scheduled, w: &room::phase::RevealWitness<'_>) -> room::question::Letter {
//!     s.open(w).correct
//! }
//! ```
//!
//! **What the rule is.** The correct option is derived exactly as
//! `pipeline/src/popquiz/bank.py`'s `correct_index` derives it, which is the
//! reference (G-2: if one changes, both change): the first rule that applies
//! decides — a does-not-compile record by the `does_not_compile` kind, UB under
//! both borrow models by the `ub` kind, a non-zero exit by the `panic` kind,
//! and otherwise the option whose text equals the verified `stdout` less the
//! one trailing newline `println!` added. The receipt is the twin of
//! `receipt.py`'s `receipt_lines`, tested against the same
//! `bank/fixtures/receipts/*.json`.

use serde::Deserialize;

use crate::copy;
use crate::phase::{RevealWitness, TraceLen};
use crate::question::{Letter, PublicQuestion, TraceStep, ValueRow};
use crate::rooms::Totals;

pub use vault::Verdict;

/// Why a record cannot be scheduled into a room. Plain words, for the
/// organizer's pipeline; never shown to participants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadError(pub String);

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for LoadError {}

fn refuse<T>(id: &str, why: impl std::fmt::Display) -> Result<T, LoadError> {
    Err(LoadError(format!("{id}: {why}")))
}

// --------------------------------------------------------------------------
// The bank record, as `bank/schema/question.schema.json` has it. Unknown
// fields are refused everywhere, like `bank.question_from_dict`.
// --------------------------------------------------------------------------

/// One bank record, parsed and not yet split. Its fields are private; the only
/// things to do with one are to derive its correct option (the twin test does)
/// or to [`load`] it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    id: String,
    source: String,
    #[allow(dead_code)]
    topic: String,
    #[allow(dead_code)]
    difficulty_requested: i64,
    options: Vec<RecordOption>,
    hint: String,
    explains: RecordExplains,
    #[serde(default)]
    trace: RecordTrace,
    verified: Option<Verified>,
    #[allow(dead_code)]
    review: Option<serde_json::Value>,
    #[allow(dead_code)]
    used: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordOption {
    text: String,
    kind: OptionKind,
    why_tempting: Option<String>,
}

// Every kind but `Output` names an answer by kind (the correct-option rule);
// `Output` exists because the schema has it and an unknown kind must be refused.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OptionKind {
    Output,
    DoesNotCompile,
    Ub,
    Panic,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordExplains {
    what: String,
    takeaway: String,
    /// The MVP's single string, kept for AC-73's check. Never rendered (§11).
    #[allow(dead_code)]
    legacy: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordTrace {
    #[serde(default)]
    steps: Vec<RecordStep>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordStep {
    lines: Vec<u32>,
    focus: [u32; 2],
    note: String,
    #[serde(default)]
    values: Vec<RecordValue>,
    #[serde(default)]
    pivot: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordValue {
    name: String,
    was: String,
    now: String,
}

/// The verified record (SPEC §3.2), written by the verifier and never edited.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verified {
    rustc: String,
    edition: String,
    legacy: Option<bool>,
    target_triple: Option<String>,
    flags: Option<Flags>,
    runs: Option<Runs>,
    stdout: Option<String>,
    exit_code: Option<i64>,
    miri: Option<Miri>,
    compile_error_code: Option<Vec<String>>,
    #[allow(dead_code)]
    verified_at: Option<String>,
    #[allow(dead_code)]
    verifier_version: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Flags {
    opt_level: String,
    overflow_checks: bool,
    debug_assertions: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Runs {
    count: i64,
    byte_identical: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Miri {
    clean: bool,
    output_matched: bool,
    version: Option<String>,
    configs: Option<Vec<String>>,
    seeds: Option<Vec<i64>>,
}

impl Record {
    pub fn from_json(json: &str) -> Result<Record, LoadError> {
        serde_json::from_str(json).map_err(|e| LoadError(format!("not a bank record: {e}")))
    }

    /// The twin of `bank.correct_index`: which option is correct, derived from
    /// `verified` (SPEC §3.1, G-2). The rules, in order, and the first that
    /// applies decides — it never falls through to a later one:
    ///
    /// 1. a does-not-compile record: the `does_not_compile` option;
    /// 2. a record that ran, with UB under both borrow models: the `ub` option;
    /// 3. a record that ran and exited non-zero (a panic): the `panic` option;
    /// 4. otherwise, a record that ran: the option whose text equals the output.
    ///
    /// `Ok(None)` when the record has no verified record or nothing matches;
    /// an error when two options match, because then no single option is the
    /// correct one.
    pub fn correct_index(&self) -> Result<Option<usize>, LoadError> {
        let Some(verified) = &self.verified else {
            return Ok(None);
        };
        let of_kind = |kind: OptionKind| -> Vec<usize> {
            self.options
                .iter()
                .enumerate()
                .filter(|(_, o)| o.kind == kind)
                .map(|(i, _)| i)
                .collect()
        };
        let matches: Vec<usize> = match receipt_class(verified) {
            Some(ReceiptClass::DoesNotCompile) => of_kind(OptionKind::DoesNotCompile),
            Some(ReceiptClass::Ran) => {
                if ub_confirmed(verified) {
                    of_kind(OptionKind::Ub)
                } else if verified.exit_code.is_some_and(|code| code != 0) {
                    of_kind(OptionKind::Panic)
                } else {
                    let Some(stdout) = &verified.stdout else {
                        return Ok(None);
                    };
                    let wanted = normalized_output(stdout);
                    self.options
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| o.text == wanted)
                        .map(|(i, _)| i)
                        .collect()
                }
            }
            None => return Ok(None),
        };
        if matches.len() > 1 {
            return refuse(
                &self.id,
                format_args!(
                    "{} options match the verified answer (indexes {:?}), so no single option is the correct one",
                    matches.len(),
                    matches
                ),
            );
        }
        Ok(matches.first().copied())
    }
}

/// `stdout` as an option's text spells it: exactly one trailing newline off,
/// nothing else (the twin of `bank.normalized_output`).
fn normalized_output(stdout: &str) -> &str {
    stdout.strip_suffix('\n').unwrap_or(stdout)
}

/// The two Miri borrow models (AC-9), as `bank.BORROW_MODELS` names them.
const BORROW_MODELS: [&str; 2] = ["stacked_borrows", "tree_borrows"];

/// The twin of `bank.ub_confirmed`: Miri reported UB and its `configs` name
/// both borrow models. A legacy record has no `configs`, so nothing is
/// inferred from its unclean pass (D-16).
fn ub_confirmed(verified: &Verified) -> bool {
    verified.miri.as_ref().is_some_and(|miri| {
        !miri.clean
            && miri
                .configs
                .as_ref()
                .is_some_and(|configs| BORROW_MODELS.iter().all(|m| configs.iter().any(|c| c == m)))
    })
}

enum ReceiptClass {
    Ran,
    DoesNotCompile,
}

/// The twin of `bank.receipt_class`: does-not-compile first, then ran, else
/// neither.
fn receipt_class(verified: &Verified) -> Option<ReceiptClass> {
    if verified
        .compile_error_code
        .as_ref()
        .is_some_and(|codes| !codes.is_empty())
    {
        return Some(ReceiptClass::DoesNotCompile);
    }
    if verified.runs.is_some() {
        return Some(ReceiptClass::Ran);
    }
    None
}

/// The receipt's step lines (SPEC §7.5, G-7) — the Rust twin of
/// `receipt.receipt_lines`, tested against `bank/fixtures/receipts/*.json`.
///
/// A line renders only if the record holds the step it names. `None` for a
/// record that renders no receipt (not an empty list: that would be a receipt
/// with no steps, which is a different and wrong answer).
pub fn receipt_lines(verified: &Verified) -> Option<Vec<String>> {
    match receipt_class(verified)? {
        ReceiptClass::DoesNotCompile => {
            let codes = verified
                .compile_error_code
                .as_deref()
                .unwrap_or_default()
                .join(", ");
            Some(vec![
                copy::RECEIPT_COMPILER_REFUSED.to_string(),
                copy::fill(copy::RECEIPT_ERROR_CODES, &[("codes", &codes)]),
                copy::RECEIPT_NOTHING_RAN.to_string(),
            ])
        }
        ReceiptClass::Ran => {
            let mut lines = Vec::new();
            if let Some(runs) = &verified.runs {
                lines.push(copy::RECEIPT_COMPILED.to_string());
                lines.push(copy::fill(
                    copy::RECEIPT_RAN_N_TIMES,
                    &[("N", &runs.count.to_string())],
                ));
                if runs.byte_identical {
                    lines.push(copy::RECEIPT_OUTPUT_NEVER_VARIED.to_string());
                }
            }
            if let Some(miri) = verified.miri.as_ref().filter(|m| m.output_matched) {
                lines.push(if miri.clean {
                    copy::RECEIPT_MIRI_CLEAN.to_string()
                } else {
                    copy::RECEIPT_MIRI_UB.to_string()
                });
            }
            Some(lines)
        }
    }
}

// --------------------------------------------------------------------------
// What comes out of the vault, and only with a witness.
// --------------------------------------------------------------------------

/// The explanation's two authored beats around the middle one (SPEC §3.1).
#[derive(Debug)]
pub struct Explains {
    pub what: String,
    pub takeaway: String,
}

/// The middle beat, decided once at `closed` (SPEC §4.5, AC-95).
#[derive(Debug, PartialEq, Eq)]
pub enum Middle {
    /// The room's most-chosen incorrect option (ties: the lower letter), how
    /// many chose it, and its authored `why_tempting` text.
    MostChosen {
        letter: Letter,
        count: u32,
        why_tempting: String,
    },
    /// No incorrect option received a vote: everyone was right, or nobody
    /// answered. Wall and host show §11's *nobody read it another way* strings.
    Nobody,
}

/// Everything the vault holds, lent for as long as the witness's room lives.
pub struct Revealed<'v> {
    pub correct: Letter,
    pub explains: &'v Explains,
    pub receipt: &'v [String],
    /// The whole trace, `0..=M-1`, resolving step included.
    pub trace: &'v [TraceStep],
    /// T-12: every option's `why_tempting` (`None` for the correct one), for
    /// take-it-home's middle beat, which §13 repeats for **every** incorrect
    /// option. The wall and the host name only the room's most-chosen one,
    /// through [`Verdict`]; they never read this.
    pub why_tempting: &'v [Option<String>; 5],
    /// T-12: the verified record's detail rows for take-it-home's *How we
    /// know* (§13, AC-87).
    pub how_we_know: &'v HowWeKnow,
}

/// What the verified record says about the machine that ran the question —
/// the rows the wall leaves out and take-it-home shows under *How we know*
/// (SPEC §13, AC-87). Copied from `verified` verbatim; a field the record does
/// not hold is `None` and is never back-filled (G-2, D-16).
#[derive(Debug)]
pub struct HowWeKnow {
    /// A record migrated from the MVP (D-16): `rustc` is the `--version`
    /// string, and there is no target, no flags and no Miri configuration.
    pub legacy: bool,
    /// `rustc -Vv` in full, or `rustc --version` on a legacy record.
    pub rustc: String,
    pub edition: String,
    pub target_triple: Option<String>,
    pub flags: Option<FlagsDetail>,
    /// Present when Miri ran. On a legacy record it holds no configuration:
    /// the pass was run outside the verifier.
    pub miri: Option<MiriDetail>,
}

#[derive(Debug)]
pub struct FlagsDetail {
    pub opt_level: String,
    pub overflow_checks: bool,
    pub debug_assertions: bool,
}

#[derive(Debug)]
pub struct MiriDetail {
    pub version: Option<String>,
    pub configs: Option<Vec<String>>,
    pub seeds: Option<Vec<i64>>,
}

impl HowWeKnow {
    fn of(verified: &Verified) -> HowWeKnow {
        HowWeKnow {
            legacy: verified.legacy.unwrap_or(false),
            rustc: verified.rustc.clone(),
            edition: verified.edition.clone(),
            target_triple: verified.target_triple.clone(),
            flags: verified.flags.as_ref().map(|f| FlagsDetail {
                opt_level: f.opt_level.clone(),
                overflow_checks: f.overflow_checks,
                debug_assertions: f.debug_assertions,
            }),
            miri: verified.miri.as_ref().map(|m| MiriDetail {
                version: m.version.clone(),
                configs: m.configs.clone(),
                seeds: m.seeds.clone(),
            }),
        }
    }
}

mod vault {
    //! The vault. Its fields are private to this module; `tests/boundary.rs`
    //! holds the list of functions allowed here and checks that every read
    //! takes a `RevealWitness`.

    use super::{Explains, HowWeKnow, Middle, Revealed};
    use crate::phase::RevealWitness;
    use crate::question::{Letter, TraceStep};
    use crate::rooms::Totals;

    pub struct Vault {
        correct: Letter,
        why_tempting: [Option<String>; 5],
        explains: Explains,
        receipt: Vec<String>,
        trace: Vec<TraceStep>,
        how_we_know: HowWeKnow,
    }

    impl Vault {
        pub(super) fn seal(
            correct: Letter,
            why_tempting: [Option<String>; 5],
            explains: Explains,
            receipt: Vec<String>,
            trace: Vec<TraceStep>,
            how_we_know: HowWeKnow,
        ) -> Vault {
            Vault {
                correct,
                why_tempting,
                explains,
                receipt,
                trace,
                how_we_know,
            }
        }

        /// §4.5, computed once at `closed` from the frozen totals. The result
        /// is sealed: it names an incorrect option, which is half the join.
        pub fn judge(&self, totals: &Totals) -> Verdict {
            let mut best: Option<(Letter, u32)> = None;
            for letter in Letter::ALL {
                if letter == self.correct {
                    continue;
                }
                let n = totals[letter.index()];
                // Strictly greater, walking A→E: a tie keeps the lower letter.
                if n > 0 && best.is_none_or(|(_, top)| n > top) {
                    best = Some((letter, n));
                }
            }
            Verdict {
                middle: match best {
                    Some((letter, count)) => Middle::MostChosen {
                        letter,
                        count,
                        why_tempting: self.why_tempting[letter.index()]
                            .clone()
                            .unwrap_or_default(),
                    },
                    None => Middle::Nobody,
                },
            }
        }

        /// The vault's one read, and it takes the witness.
        pub fn open<'v>(&'v self, _proof: &RevealWitness<'_>) -> Revealed<'v> {
            Revealed {
                correct: self.correct,
                explains: &self.explains,
                receipt: &self.receipt,
                trace: &self.trace,
                why_tempting: &self.why_tempting,
                how_we_know: &self.how_we_know,
            }
        }
    }

    /// The most-chosen incorrect option, sealed until `reveal`.
    pub struct Verdict {
        middle: Middle,
    }

    impl Verdict {
        pub fn open(&self, _proof: &RevealWitness<'_>) -> &Middle {
            &self.middle
        }
    }
}

/// A question ready for a room: its public half, and its vault.
///
/// Neither field is public, and it derives nothing: no `Debug` to print the
/// vault, no `Clone` to copy it somewhere less careful. Rooms share one through
/// an `Arc`.
pub struct Scheduled {
    public: PublicQuestion,
    vault: vault::Vault,
}

impl Scheduled {
    pub fn public(&self) -> &PublicQuestion {
        &self.public
    }

    /// Decide §4.5's middle beat from the frozen totals. Sealed.
    pub fn judge(&self, totals: &Totals) -> Verdict {
        self.vault.judge(totals)
    }

    /// Open the vault. Needs a witness, which exists only in `reveal`.
    pub fn open<'s>(&'s self, proof: &RevealWitness<'_>) -> Revealed<'s> {
        self.vault.open(proof)
    }
}

/// Read a bank record and split it into its public half and its vault.
///
/// Refuses, in plain words, a record the room cannot run: not five options,
/// not exactly one *does not compile* (AC-24), no verified record, no receipt
/// (§7.5), no single derivable correct option (G-2), a trace shorter than two
/// steps (§7.4, D-10), a record that ran whose final step names no `stdout`
/// (D-10), or an incorrect option with no `why_tempting` (§4.5, AC-95).
pub fn load(json: &str) -> Result<Scheduled, LoadError> {
    let record = Record::from_json(json)?;
    let id = record.id.clone();

    if record.options.len() != 5 {
        return refuse(&id, format_args!("has {} options; a room needs five", record.options.len()));
    }
    let dnc = record
        .options
        .iter()
        .filter(|o| o.kind == OptionKind::DoesNotCompile)
        .count();
    if dnc != 1 {
        return refuse(&id, format_args!("has {dnc} does-not-compile options; it needs exactly one"));
    }

    let Some(verified) = &record.verified else {
        return refuse(&id, "has no verified record, so it has no answer");
    };
    let Some(receipt) = receipt_lines(verified) else {
        return refuse(&id, "renders no receipt, so it cannot be scheduled");
    };
    let Some(correct) = record.correct_index()? else {
        return refuse(&id, "no option matches the verified answer");
    };
    let correct = Letter::from_index(correct).expect("five options");

    let steps = &record.trace.steps;
    let Some(trace_len) = TraceLen::new(steps.len()) else {
        return refuse(&id, "the trace needs at least two steps to walk");
    };
    if matches!(receipt_class(verified), Some(ReceiptClass::Ran))
        && !steps
            .last()
            .is_some_and(|s| s.values.iter().any(|v| v.name == "stdout"))
    {
        return refuse(&id, "the trace's final step names no stdout");
    }

    let mut why_tempting: [Option<String>; 5] = Default::default();
    for (i, option) in record.options.iter().enumerate() {
        let text = option.why_tempting.clone().filter(|t| !t.trim().is_empty());
        if i != correct.index() && text.is_none() {
            return refuse(
                &id,
                format_args!("option {} has no why_tempting text", Letter::ALL[i].as_str()),
            );
        }
        why_tempting[i] = text;
    }

    let trace: Vec<TraceStep> = steps
        .iter()
        .map(|s| TraceStep {
            lines: s.lines.clone(),
            focus: s.focus,
            note: s.note.clone(),
            values: s
                .values
                .iter()
                .map(|v| ValueRow {
                    name: v.name.clone(),
                    was: v.was.clone(),
                    now: v.now.clone(),
                })
                .collect(),
            pivot: s.pivot,
        })
        .collect();

    // The walk-through's half: steps 0..=M-2, and no `stdout` row in any of
    // them (G-3 says "any values entry named stdout", not only the last's).
    let walk: Vec<TraceStep> = trace[..trace.len() - 1]
        .iter()
        .map(|s| TraceStep {
            values: s.values.iter().filter(|v| v.name != "stdout").cloned().collect(),
            ..s.clone()
        })
        .collect();

    let options: [String; 5] = std::array::from_fn(|i| record.options[i].text.clone());

    Ok(Scheduled {
        public: PublicQuestion {
            id: id.clone(),
            source: record.source.clone(),
            options,
            hint: record.hint.clone(),
            walk,
            trace_len,
        },
        vault: vault::Vault::seal(
            correct,
            why_tempting,
            Explains {
                what: record.explains.what.clone(),
                takeaway: record.explains.takeaway.clone(),
            },
            receipt,
            trace,
            HowWeKnow::of(verified),
        ),
    })
}

/// Parse just the `verified` object of a record — for the receipt twin test,
/// which reads `bank/fixtures/receipts/*.json`.
pub fn verified_from_json(json: &serde_json::Value) -> Result<Verified, LoadError> {
    Verified::deserialize(json).map_err(|e| LoadError(format!("not a verified record: {e}")))
}
