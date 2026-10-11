//! The public half of a question: what any viewer may be shown, in some phase.
//!
//! Built by [`crate::answers::load`], which splits a bank record in two at the
//! moment it is read. This half holds nothing that joins an option to the
//! verified output: option **texts** (public in every phase, AC-62) in the
//! order the record arrived, the source, the hint, and the trace steps the
//! walk-through may show — `0..=M-2`, with any `values` entry named `stdout`
//! taken out (G-3, D-10). Everything else is in the sealed half.

use serde::Serialize;

use crate::phase::TraceLen;

/// An option's letter: its position in the order the question arrived.
///
/// The room never arranges options. The pushed record's order is the wall's
/// order; the date-drawn arrangement is the pipeline's (AC-23, T-20).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub enum Letter {
    A,
    B,
    C,
    D,
    E,
}

impl Letter {
    pub const ALL: [Letter; 5] = [Letter::A, Letter::B, Letter::C, Letter::D, Letter::E];

    pub fn index(self) -> usize {
        match self {
            Letter::A => 0,
            Letter::B => 1,
            Letter::C => 2,
            Letter::D => 3,
            Letter::E => 4,
        }
    }

    pub fn from_index(i: usize) -> Option<Letter> {
        Letter::ALL.get(i).copied()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Letter::A => "A",
            Letter::B => "B",
            Letter::C => "C",
            Letter::D => "D",
            Letter::E => "E",
        }
    }
}

/// One `values` row of a trace step: `was`/`now` are verbatim strings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ValueRow {
    pub name: String,
    pub was: String,
    pub now: String,
}

/// One trace step (SPEC §5.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TraceStep {
    pub lines: Vec<u32>,
    pub focus: [u32; 2],
    pub note: String,
    pub values: Vec<ValueRow>,
    pub pivot: bool,
}

/// What any viewer may be shown, in some phase.
#[derive(Debug)]
pub struct PublicQuestion {
    pub(crate) id: String,
    pub(crate) source: String,
    pub(crate) options: [String; 5],
    pub(crate) hint: String,
    /// Steps `0..=M-2`, with `stdout` rows removed. Never the final step.
    pub(crate) walk: Vec<TraceStep>,
    pub(crate) trace_len: TraceLen,
}

impl PublicQuestion {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    /// Option texts, in arrival order: index `i` is [`Letter::from_index`]`(i)`.
    pub fn options(&self) -> &[String; 5] {
        &self.options
    }

    pub fn hint(&self) -> &str {
        &self.hint
    }

    /// The walk-through's steps, `0..=M-2`.
    pub fn walk(&self) -> &[TraceStep] {
        &self.walk
    }

    /// `M`, the length of the whole trace, final step included.
    pub fn trace_len(&self) -> TraceLen {
        self.trace_len
    }
}
