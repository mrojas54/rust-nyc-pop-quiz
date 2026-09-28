//! What outlives a room (T-11, SPEC.md §3.1 `used`, §4.6, §13; G-10, AC-92).
//!
//! Two things, both written by the `released` transition and nothing else:
//!
//! - the **used record** — `{meetup_date, room_id, released_at, fit}`, the
//!   pipeline's `Used` (`pipeline/src/popquiz/bank.py`) field for field — in
//!   the append-only [`UsedLedger`]. A question is *used* when its room is
//!   **released**, not when a deck is built and not at `reveal` (G-10). A room
//!   that never reaches release records nothing.
//! - the **take-it-home snapshot** ([`TakeHome`]) — the last released question
//!   as `/last` shows it (§13), copied from what the reveal witness lends, and
//!   replaced whole at every release. **No room state**: no count, no split, no
//!   most-chosen option (AC-56, AC-95, D-12).
//!
//! Neither carries anything about a person (AC-56, AC-57).
//!
//! **Seams.** T-25's `GET /admin/used` and T-20's `popquiz sync` read
//! [`UsedLedger::all`]; T-12's `/last` reads `AppState::take_home`.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::question::{Letter, TraceStep};
use crate::rooms::{Fit, Opened, PublicView};

/// The zone a meetup's date is read in: Rust NYC meets in New York. A
/// constant here until configuration grows a place for it (T-09 / T-20).
pub const MEETUP_ZONE: &str = "America/New_York";

/// The pipeline's `Used`, exactly: these four fields, these names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UsedRecord {
    /// `YYYY-MM-DD`: the room's creation date in [`MEETUP_ZONE`].
    pub meetup_date: String,
    pub room_id: String,
    /// RFC 3339, UTC: `YYYY-MM-DDTHH:MM:SSZ`.
    pub released_at: String,
    /// The wall's last verdict before release — the one it measured in
    /// `reveal`, the last refit of the night (§3.4). `null` if the wall never
    /// reported one: nothing observed it, so nothing is written.
    pub fit: Option<Fit>,
}

/// One ledger line: which question, and its `used` record. The pipeline keeps
/// `used` on the question, so the question id is the key it syncs by.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UsedEntry {
    pub question_id: String,
    pub used: UsedRecord,
}

// AC-57, at compile time, as `sessions.rs` does for `Session`: these patterns
// name every field and have no `..`, so a new field stops the crate compiling.
const _: fn(UsedRecord) = |UsedRecord { meetup_date: _, room_id: _, released_at: _, fit: _ }| {};
const _: fn(UsedEntry) = |UsedEntry { question_id: _, used: _ }| {};

/// The used-question ledger. In memory, append-only; its only writer is the
/// release transition in `rooms.rs` (G-10), which `tests/used.rs` checks by
/// scanning the source.
#[derive(Default)]
pub struct UsedLedger {
    entries: Mutex<Vec<UsedEntry>>,
}

impl UsedLedger {
    pub(crate) fn append(&self, entry: UsedEntry) {
        self.entries.lock().unwrap_or_else(|p| p.into_inner()).push(entry);
    }

    /// Every record, in the order the rooms were released.
    pub fn all(&self) -> Vec<UsedEntry> {
        self.entries.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Whether a question has been run: *never twice* (G-10).
    pub fn contains(&self, question_id: &str) -> bool {
        self.entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter()
            .any(|e| e.question_id == question_id)
    }
}

// --------------------------------------------------------------------------
// The take-it-home snapshot (§13). T-12 renders it at `/last`.
// --------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TakeHomeOption {
    pub letter: Letter,
    pub text: String,
    /// The one option marked ✓ (glyph as well as colour, AC-40).
    pub correct: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TakeHomeReceipt {
    pub heading: &'static str,
    pub lines: Vec<String>,
}

/// The last released question, as `/last` shows it. Built at release from the
/// witnessed reads, never before; it holds the question and nothing about the
/// room that ran it.
///
/// Not here yet, because the sealed module does not lend them: the
/// `why_tempting` text of *every* incorrect option and the verified record's
/// detail rows for *How we know* (§13). T-12 adds them through a witnessed read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TakeHome {
    pub question_id: String,
    /// `YYYY-MM-DD`, for **‹date›'s question**.
    pub meetup_date: String,
    pub source: String,
    /// Source with syntax colour (§13).
    pub colour: bool,
    pub options: Vec<TakeHomeOption>,
    pub correct: Letter,
    pub mark: &'static str,
    /// The whole trace, `0..=M-1`, for the reader to step at their pace.
    pub trace: Vec<TraceStep>,
    /// **What happens**.
    pub what: String,
    /// **What to remember**.
    pub takeaway: String,
    /// **How we know** — the wall's list (§7.5).
    pub receipt: TakeHomeReceipt,
}

const _: fn(TakeHome) = |TakeHome {
                             question_id: _,
                             meetup_date: _,
                             source: _,
                             colour: _,
                             options: _,
                             correct: _,
                             mark: _,
                             trace: _,
                             what: _,
                             takeaway: _,
                             receipt: _,
                         }| {};

/// Copy the snapshot out of what `reveal` lends. Called by the release
/// transition while the room still holds its reveal witness.
pub(crate) fn take_home(question_id: &str, meetup_date: String, view: &PublicView<'_>, opened: &Opened<'_>) -> TakeHome {
    let correct = opened.revealed.correct;
    TakeHome {
        question_id: question_id.to_string(),
        meetup_date,
        source: view.question.source().to_string(),
        colour: true,
        options: Letter::ALL
            .iter()
            .zip(view.question.options())
            .map(|(&letter, text)| TakeHomeOption {
                letter,
                text: text.clone(),
                correct: letter == correct,
            })
            .collect(),
        correct,
        mark: "✓",
        trace: opened.revealed.trace.to_vec(),
        what: opened.revealed.explains.what.clone(),
        takeaway: opened.revealed.explains.takeaway.clone(),
        receipt: TakeHomeReceipt {
            heading: crate::copy::RECEIPT_HEADING,
            lines: opened.revealed.receipt.to_vec(),
        },
    }
}

// --------------------------------------------------------------------------
// Dates, without a date crate: the civil calendar (Howard Hinnant's
// algorithms) and the US Eastern rule — daylight time from 02:00 on the second
// Sunday of March to 02:00 on the first Sunday of November (since 2007).
// --------------------------------------------------------------------------

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Sunday = 0. 1970-01-01 was a Thursday.
fn weekday(days: i64) -> i64 {
    (days + 4).rem_euclid(7)
}

/// The `n`th Sunday (1-based) of a month, as days since the epoch.
fn nth_sunday(y: i64, m: i64, n: i64) -> i64 {
    let first = days_from_civil(y, m, 1);
    first + (7 - weekday(first)) % 7 + 7 * (n - 1)
}

fn epoch_secs(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64) - i64::from(e.duration().subsec_nanos() > 0),
    }
}

/// New York's offset from UTC at instant `secs`, in seconds.
fn eastern_offset(secs: i64) -> i64 {
    let (y, _, _) = civil_from_days(secs.div_euclid(86_400));
    // 02:00 EST is 07:00 UTC; 02:00 EDT is 06:00 UTC.
    let start = nth_sunday(y, 3, 2) * 86_400 + 7 * 3600;
    let end = nth_sunday(y, 11, 1) * 86_400 + 6 * 3600;
    if (start..end).contains(&secs) {
        -4 * 3600
    } else {
        -5 * 3600
    }
}

/// The meetup's date: `t`'s civil date in [`MEETUP_ZONE`], `YYYY-MM-DD`.
pub fn meetup_date(t: SystemTime) -> String {
    let secs = epoch_secs(t);
    let (y, m, d) = civil_from_days((secs + eastern_offset(secs)).div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// `t` as RFC 3339 in UTC, to the second: `YYYY-MM-DDTHH:MM:SSZ`.
pub fn rfc3339(t: SystemTime) -> String {
    let secs = epoch_secs(t);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    let s = secs.rem_euclid(86_400);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", s / 3600, s / 60 % 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at(y: i64, m: i64, d: i64, hh: i64, mm: i64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs((days_from_civil(y, m, d) * 86_400 + hh * 3600 + mm * 60) as u64)
    }

    #[test]
    fn the_calendar_round_trips() {
        for z in [-800_000, -1, 0, 1, 11_016, 20_724, 60_000] {
            let (y, m, d) = civil_from_days(z);
            assert_eq!(days_from_civil(y, m, d), z);
        }
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(weekday(days_from_civil(2026, 9, 27)), 0, "2026-09-27 is a Sunday");
    }

    #[test]
    fn rfc3339_is_utc_to_the_second() {
        assert_eq!(rfc3339(at(2026, 10, 15, 1, 41) + Duration::from_millis(7_900)), "2026-10-15T01:41:07Z");
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn the_meetup_date_is_new_yorks() {
        // 21:00 EDT on 14 Oct is 01:00 UTC on 15 Oct.
        assert_eq!(meetup_date(at(2026, 10, 15, 1, 0)), "2026-10-14");
        // 21:00 EST on 14 Jan is 02:00 UTC on 15 Jan.
        assert_eq!(meetup_date(at(2026, 1, 15, 2, 0)), "2026-01-14");
        assert_eq!(meetup_date(at(2026, 1, 15, 5, 0)), "2026-01-15");
        // DST 2026: 8 March 07:00 UTC to 1 November 06:00 UTC.
        assert_eq!(eastern_offset(epoch_secs(at(2026, 3, 8, 6, 59))), -5 * 3600);
        assert_eq!(eastern_offset(epoch_secs(at(2026, 3, 8, 7, 0))), -4 * 3600);
        assert_eq!(eastern_offset(epoch_secs(at(2026, 11, 1, 5, 59))), -4 * 3600);
        assert_eq!(eastern_offset(epoch_secs(at(2026, 11, 1, 6, 0))), -5 * 3600);
    }
}
