//! The room's data models: what a question, a phase, an answer and a payload are.
//!
//! - [`phase`] — the machine: `idle → … → released`, host actions only, no
//!   skipping, `trace_step` (G-6).
//! - [`answers`] — **sealed**: everything that joins an option to the verified
//!   output, readable only with a witness that exists only in `reveal` (G-3,
//!   AC-61). The proof is a set of `compile_fail` doctests with compiling twins,
//!   plus `tests/boundary.rs`.
//! - [`affirmation`] — whether a question is affirmed for its text: the twin
//!   of the pipeline's rule, which the room checks before it runs one (G-12).
//! - [`question`] — the public half of a question.
//! - [`club`] — which club a room belongs to, and its time zone.
//! - [`view`] — the public state query: wall, buzzer and host payloads.
//! - [`used`] — the used-question ledger and the take-it-home snapshot.

pub mod affirmation;
pub mod answers;
pub mod club;
pub mod phase;
pub mod question;
pub mod used;
pub mod view;
