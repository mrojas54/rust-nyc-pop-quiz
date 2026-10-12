//! The one way to the answers.
//!
//! [`Room::open`] asks the machine for a witness that exists only in `reveal`
//! and returns the vault's contents and §4.5's verdict, borrowed for as long
//! as the room does not move. [`Room::public`] returns a [`PublicView`], which
//! borrows the public fields one by one and has no path to the vault. This is
//! the only file that opens the vault (`tests/boundary.rs`).

use super::*;

/// What a room shows once it is in `reveal`: the vault's contents and §4.5's
/// verdict, borrowed from the room so they cannot outlive the phase.
pub struct Opened<'r> {
    pub revealed: Revealed<'r>,
    pub middle: &'r Middle,
}

/// Everything about a room that is public in some phase — and nothing that is
/// not. It borrows the room's public fields one by one, never the room, so
/// there is no path from here to the vault.
pub struct PublicView<'r> {
    pub phase: Phase,
    pub trace_step: Option<u16>,
    pub code: &'r str,
    pub join_url: &'r str,
    pub question: &'r PublicQuestion,
    pub present: u32,
    pub answered_live: u32,
    /// `(answered, totals)` from `closed` on.
    pub frozen: Option<(u32, Totals)>,
    pub fit: Option<Fit>,
}

impl Room {
    /// The public state: what any projection may read before `reveal`.
    pub fn public(&self) -> PublicView<'_> {
        PublicView {
            phase: self.machine.phase(),
            trace_step: self.machine.trace_step(),
            code: &self.code,
            join_url: &self.join_url,
            question: self.question.public(),
            present: self.present,
            answered_live: self.answered_live,
            frozen: self.frozen.as_ref().map(|f| (f.answered, f.totals)),
            fit: self.fit,
        }
    }

    /// The answers — only in `reveal`, and only for as long as this borrow.
    pub fn open(&self) -> Option<Opened<'_>> {
        let proof = self.machine.revealed()?;
        let frozen = self
            .frozen
            .as_ref()
            .expect("reveal is reached only through closed, which froze the totals");
        Some(Opened {
            revealed: self.question.open(&proof),
            middle: frozen.verdict.open(&proof),
        })
    }
}
