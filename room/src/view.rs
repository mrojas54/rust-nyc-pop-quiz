//! The public state query: every wall, buzzer and host payload (SPEC.md §4).
//!
//! Each projection branches on [`Room::open`] — on the witness, not on the
//! phase. Before `reveal` (and after it, in `released`) the builders are handed
//! a [`PublicView`] and nothing else, and a `PublicView` has no path to the
//! sealed `answers` module, so no pre-reveal payload *can* carry the join
//! between an option and the verified output (G-3, AC-61). Only the `reveal`
//! builders see an [`Opened`].
//!
//! Every payload carries exactly one `phase`, read from the room's one phase
//! value, so the wall, the buzzers and the host cannot disagree about it
//! within one broadcast (AC-81).
//!
//! These are data for T-05 (wall), T-06 (buzzer) and T-07 (host) to render.
//! The strings in them are SPEC §11's, from [`crate::copy`]; where a line needs
//! a value only the phone knows — the participant's own answer — the phone
//! fills it (AC-58), and the payload carries the counts it needs.

use serde::Serialize;

use crate::answers::Middle;
use crate::copy;
use crate::phase::{Phase, Step};
use crate::question::{Letter, TraceStep};
use crate::rooms::{Opened, PublicView, Room, Totals, Urls};

/// Who a payload is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Viewer {
    Wall,
    Buzzer,
    Host,
}

impl Viewer {
    pub const ALL: [Viewer; 3] = [Viewer::Wall, Viewer::Buzzer, Viewer::Host];
}

/// An option as any pre-reveal payload has it: exactly `{letter, text}`, in the
/// order the question arrived. There is no third field to put a mark in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OptionView {
    pub letter: Letter,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Bar {
    pub letter: Letter,
    pub count: u32,
    /// Whole percent of `answered`; `0` when nobody answered (§4.4).
    pub percent: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SplitView {
    pub bars: Vec<Bar>,
    pub answered: u32,
    pub present: u32,
    /// `‹answered› of ‹present› in the room answered`.
    pub line: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TraceView {
    /// 0-based; the wall shows `Step ‹at+1› of ‹m›`.
    pub at: u16,
    pub m: u16,
    pub label: String,
    pub step: TraceStep,
    /// *Pause here.* on a `pivot` step.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReceiptView {
    pub heading: &'static str,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WallMiddle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter: Option<Letter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    pub line: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WallReveal {
    pub correct: Letter,
    pub mark: &'static str,
    pub middle: WallMiddle,
    pub receipt: ReceiptView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReleasedView {
    pub title: &'static str,
    pub link: String,
    pub line: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WallPayload {
    pub phase: Phase,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub well_header: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Syntax colour on the source: `live`, `closed`, `split` only (§5.3, AC-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<OptionView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<SplitView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beats: Option<[&'static str; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<TraceView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reveal: Option<WallReveal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<ReleasedView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HintView {
    pub text: String,
    pub action: &'static str,
    pub shown: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Counts {
    pub totals: Totals,
    pub answered: u32,
    pub present: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuzzerPayload {
    pub phase: Phase,
    pub code: String,
    pub lines: Vec<String>,
    /// The five letters; never the option texts, the source or the trace (G-8).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letters: Option<[Letter; 5]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    /// In `live` only (§4.2, D-8): every buzzer gets it; showing it is local.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<HintView>,
    /// From `split` on: the phone computes *‹n› people said ‹X›* from these
    /// and its own answer, which it never sends (AC-58).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts: Option<Counts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct: Option<Letter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<&'static str>,
    pub foot: &'static str,
    // T-04b: the calling session's own saved answer, on the per-session paths
    // only (the join response, T-04c's re-attach). Never another session's;
    // the public buzzer query never sets it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yours: Option<Letter>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ActionView {
    /// The route segment: `POST /rooms/{id}/<action>`.
    pub action: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FirstScreen {
    pub resume: String,
    /// SPEC §8.1's two sentences, verbatim (AC-62, AC-63).
    pub not_a_guarantee: [&'static str; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HostStep {
    pub at: u16,
    pub m: u16,
    pub label: String,
    /// The step's words: what the host reads.
    pub note: String,
    pub back: ActionView,
    pub forward: ActionView,
    pub can_back: bool,
    pub can_forward: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Beat {
    pub heading: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReadAloud {
    pub heading: &'static str,
    pub what: Beat,
    /// §4.5: the most-chosen incorrect option's *Why ‹n› of us said ‹X›* and its
    /// `why_tempting`, or *Why nobody said anything else* with no text.
    pub middle: Beat,
    pub takeaway: Beat,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HostPayload {
    pub phase: Phase,
    pub code: String,
    /// The phase label (AC-49).
    pub label: &'static str,
    /// The one primary action (AC-49).
    pub primary: ActionView,
    pub present: u32,
    /// `answered_live` while `live`; the frozen `answered` from `closed` on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered: Option<u32>,
    /// From `live` on, once the wall has measured (AC-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fit: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_screen: Option<FirstScreen>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<HostStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_aloud: Option<ReadAloud>,
}

// --------------------------------------------------------------------------
// The three projections. Each branches on the witness, not on the phase.
// --------------------------------------------------------------------------

pub fn wall(room: &Room, urls: &Urls) -> WallPayload {
    match room.open() {
        Some(opened) => revealed::wall(room.public(), &opened),
        None => sealed::wall(room.public(), urls),
    }
}

pub fn buzzer(room: &Room) -> BuzzerPayload {
    match room.open() {
        Some(opened) => revealed::buzzer(room.public(), &opened),
        None => sealed::buzzer(room.public()),
    }
}

// T-04b: the buzzer for one session, carrying its own saved answer.
pub fn buzzer_for(room: &Room, yours: Option<Letter>) -> BuzzerPayload {
    BuzzerPayload { yours, ..buzzer(room) }
}

pub fn host(room: &Room) -> HostPayload {
    let view = room.public();
    let mut payload = match room.open() {
        Some(opened) => revealed::host(view, &opened),
        None => sealed::host(view),
    };
    if room.phase() == Phase::Idle {
        payload.first_screen = Some(FirstScreen {
            resume: copy::fill(copy::HOST_FIRST_RESUME, &[("resume link", room.host_resume_url())]),
            not_a_guarantee: [
                copy::NOT_A_GUARANTEE_OPTIONS_PUBLIC,
                copy::NOT_A_GUARANTEE_HOST_HONEST,
            ],
        });
    }
    if room.machine().trace_step().is_some() {
        if let Some(step) = payload.step.as_mut() {
            step.can_back = room.machine().can_step(Step::Back);
            step.can_forward = room.machine().can_step(Step::Forward);
        }
    }
    payload
}

/// One projection by name, as JSON.
pub fn project(room: &Room, viewer: Viewer, urls: &Urls) -> serde_json::Value {
    let json = match viewer {
        Viewer::Wall => serde_json::to_value(wall(room, urls)),
        Viewer::Buzzer => serde_json::to_value(buzzer(room)),
        Viewer::Host => serde_json::to_value(host(room)),
    };
    json.expect("payloads are plain data")
}

// --------------------------------------------------------------------------
// Shared pieces. None of these takes anything from the vault.
// --------------------------------------------------------------------------

fn options(view: &PublicView<'_>) -> Vec<OptionView> {
    Letter::ALL
        .iter()
        .zip(view.question.options())
        .map(|(&letter, text)| OptionView {
            letter,
            text: text.clone(),
        })
        .collect()
}

fn split(view: &PublicView<'_>) -> Option<SplitView> {
    let (answered, totals) = view.frozen?;
    let bars = Letter::ALL
        .iter()
        .map(|&letter| {
            let count = totals[letter.index()];
            let percent = if answered == 0 {
                0
            } else {
                ((u64::from(count) * 100 + u64::from(answered) / 2) / u64::from(answered)) as u32
            };
            Bar {
                letter,
                count,
                percent,
            }
        })
        .collect();
    Some(SplitView {
        bars,
        answered,
        present: view.present,
        line: copy::fill(
            copy::WALL_SPLIT_ANSWERED,
            &[
                ("answered", &answered.to_string()),
                ("present", &view.present.to_string()),
            ],
        ),
    })
}

fn counts(view: &PublicView<'_>) -> Option<Counts> {
    view.frozen.map(|(answered, totals)| Counts {
        totals,
        answered,
        present: view.present,
    })
}

fn step_label(at: u16, m: u16) -> String {
    copy::fill(
        copy::WALL_TRACE_STEP,
        &[("N", &(at + 1).to_string()), ("M", &m.to_string())],
    )
}

fn trace_view(at: u16, m: u16, step: &TraceStep) -> TraceView {
    TraceView {
        at,
        m,
        label: step_label(at, m),
        step: step.clone(),
        pause: step.pivot.then_some(copy::WALL_TRACE_PIVOT),
    }
}

fn host_step(at: u16, m: u16, note: &str) -> HostStep {
    HostStep {
        at,
        m,
        label: step_label(at, m),
        note: note.to_string(),
        back: ActionView {
            action: Step::Back.slug(),
            label: "←",
        },
        forward: ActionView {
            action: Step::Forward.slug(),
            label: "→",
        },
        can_back: false,
        can_forward: false,
    }
}

fn empty_wall(view: &PublicView<'_>) -> WallPayload {
    WallPayload {
        phase: view.phase,
        code: view.code.to_string(),
        title: None,
        join: None,
        well_header: None,
        source: None,
        colour: None,
        options: None,
        strip: None,
        split: None,
        beats: None,
        trace: None,
        reveal: None,
        released: None,
    }
}

fn host_base(view: &PublicView<'_>) -> HostPayload {
    let action = view.phase.next_action();
    HostPayload {
        phase: view.phase,
        code: view.code.to_string(),
        label: view.phase.host_label(),
        primary: ActionView {
            action: action.slug(),
            label: action.label(),
        },
        present: view.present,
        answered: match view.phase {
            Phase::Idle => None,
            Phase::Live => Some(view.answered_live),
            Phase::Closed | Phase::Split | Phase::Work | Phase::Reveal | Phase::Released => {
                view.frozen.map(|(answered, _)| answered)
            }
        },
        fit: match view.phase {
            Phase::Idle => None,
            _ => view.fit.map(|f| f.host_line()),
        },
        first_screen: None,
        step: None,
        read_aloud: None,
    }
}

/// Before `reveal`, and in `released`: builders that are handed a
/// [`PublicView`] and nothing else.
mod sealed {
    use super::*;

    pub(super) fn wall(view: PublicView<'_>, urls: &Urls) -> WallPayload {
        let mut p = empty_wall(&view);
        let q = view.question;
        match view.phase {
            Phase::Idle => {
                p.title = Some(copy::WALL_IDLE_TITLE);
                p.join = Some(copy::fill(copy::WALL_IDLE_JOIN, &[("link", view.join_url)]));
            }
            Phase::Live | Phase::Closed | Phase::Split => {
                p.source = Some(q.source().to_string());
                p.colour = Some(true);
                p.options = Some(options(&view));
                match view.phase {
                    Phase::Live => {
                        p.well_header = Some(copy::WALL_LIVE_WELL_HEADER);
                        p.join = Some(copy::fill(copy::WALL_LIVE_JOIN, &[("link", view.join_url)]));
                    }
                    Phase::Closed => p.strip = Some(copy::WALL_CLOSED),
                    _ => p.split = split(&view),
                }
            }
            Phase::Work => {
                p.source = Some(q.source().to_string());
                p.colour = Some(false);
                p.options = Some(options(&view));
                p.beats = Some([
                    copy::WALL_WORK_LEAD,
                    copy::WALL_WORK_NO_ANSWER,
                    copy::WALL_WORK_NOBODY,
                ]);
                // The public walk holds steps 0..=M-2 only; there is no final
                // step here to show, whatever `trace_step` says (D-10).
                let at = view.trace_step.unwrap_or(0);
                if let Some(step) = q.walk().get(usize::from(at)) {
                    p.trace = Some(trace_view(at, q.trace_len().get(), step));
                }
            }
            Phase::Released => {
                p.released = Some(ReleasedView {
                    title: copy::WALL_RELEASED_TITLE,
                    link: urls.home.clone(),
                    line: copy::WALL_RELEASED_LINE,
                });
            }
            // `reveal` always holds a witness, so it never comes here; if it
            // did, it would get the title card and nothing sealed.
            Phase::Reveal => p.title = Some(copy::WALL_IDLE_TITLE),
        }
        p
    }

    pub(super) fn buzzer(view: PublicView<'_>) -> BuzzerPayload {
        let mut p = BuzzerPayload {
            phase: view.phase,
            code: view.code.to_string(),
            lines: Vec::new(),
            letters: None,
            locked: None,
            hint: None,
            counts: None,
            correct: None,
            mark: None,
            foot: copy::BUZZER_FOOT,
            yours: None,
        };
        match view.phase {
            Phase::Idle => p.lines = vec![copy::BUZZER_IDLE.into()],
            Phase::Live => {
                p.letters = Some(Letter::ALL);
                p.locked = Some(false);
                p.hint = Some(HintView {
                    text: view.question.hint().to_string(),
                    action: copy::BUZZER_HINT_ACTION,
                    shown: copy::BUZZER_HINT_SHOWN,
                });
            }
            Phase::Closed => {
                p.letters = Some(Letter::ALL);
                p.locked = Some(true);
                p.lines = vec![copy::BUZZER_CLOSED.into()];
            }
            Phase::Split => {
                p.lines = vec![copy::BUZZER_SPLIT_LOOKUP.into(), copy::BUZZER_SPLIT_WHERE.into()];
                p.counts = counts(&view);
                p.foot = copy::BUZZER_FOOT_COMPUTED;
            }
            Phase::Work => {
                p.lines = vec![
                    copy::BUZZER_WORK_LOOKUP.into(),
                    copy::BUZZER_WORK_WALKING.into(),
                    copy::BUZZER_WORK_NOTHING.into(),
                ];
                p.counts = counts(&view);
                p.foot = copy::BUZZER_FOOT_COMPUTED;
            }
            Phase::Released => {
                p.mark = Some("✓");
                p.lines = vec![copy::BUZZER_RELEASED.into()];
            }
            Phase::Reveal => {}
        }
        p
    }

    pub(super) fn host(view: PublicView<'_>) -> HostPayload {
        let mut p = host_base(&view);
        if let (Phase::Work, Some(at)) = (view.phase, view.trace_step) {
            let q = view.question;
            if let Some(step) = q.walk().get(usize::from(at)) {
                p.step = Some(host_step(at, q.trace_len().get(), &step.note));
            }
        }
        p
    }
}

/// `reveal`: builders that are handed what the vault lent the room.
mod revealed {
    use super::*;

    fn middle_line(middle: &Middle) -> WallMiddle {
        match middle {
            Middle::MostChosen { letter, count, .. } => WallMiddle {
                letter: Some(*letter),
                count: Some(*count),
                line: copy::fill(
                    copy::WALL_REVEAL_MOST_CHOSEN,
                    &[("n", &count.to_string()), ("X", letter.as_str())],
                ),
            },
            Middle::Nobody => WallMiddle {
                letter: None,
                count: None,
                line: copy::WALL_REVEAL_NOBODY_ELSE.into(),
            },
        }
    }

    fn step_at<'o>(view: &PublicView<'_>, opened: &'o Opened<'_>) -> (u16, &'o TraceStep) {
        let trace = opened.revealed.trace;
        let at = view.trace_step.unwrap_or(0).min((trace.len() - 1) as u16);
        (at, &trace[usize::from(at)])
    }

    pub(super) fn wall(view: PublicView<'_>, opened: &Opened<'_>) -> WallPayload {
        let mut p = empty_wall(&view);
        p.source = Some(view.question.source().to_string());
        p.colour = Some(false);
        p.options = Some(options(&view));
        p.split = split(&view);
        let (at, step) = step_at(&view, opened);
        p.trace = Some(trace_view(at, view.question.trace_len().get(), step));
        p.reveal = Some(WallReveal {
            correct: opened.revealed.correct,
            mark: "✓",
            middle: middle_line(opened.middle),
            receipt: ReceiptView {
                heading: copy::RECEIPT_HEADING,
                lines: opened.revealed.receipt.to_vec(),
            },
        });
        p
    }

    pub(super) fn buzzer(view: PublicView<'_>, opened: &Opened<'_>) -> BuzzerPayload {
        let correct = opened.revealed.correct;
        BuzzerPayload {
            phase: view.phase,
            code: view.code.to_string(),
            lines: vec![
                copy::BUZZER_REVEAL_LOOKUP.into(),
                copy::BUZZER_REVEAL_ON_SCREEN.into(),
                copy::fill(copy::BUZZER_REVEAL_IT_WAS, &[("Y", correct.as_str())]),
                copy::BUZZER_REVEAL_HOST_READING.into(),
            ],
            letters: None,
            locked: None,
            hint: None,
            counts: counts(&view),
            correct: Some(correct),
            mark: None,
            foot: copy::BUZZER_FOOT_COMPUTED,
            yours: None,
        }
    }

    pub(super) fn host(view: PublicView<'_>, opened: &Opened<'_>) -> HostPayload {
        let mut p = host_base(&view);
        let (at, step) = step_at(&view, opened);
        p.step = Some(host_step(at, view.question.trace_len().get(), &step.note));
        let explains = opened.revealed.explains;
        p.read_aloud = Some(ReadAloud {
            heading: copy::HOST_REVEAL_READ_ALOUD,
            what: Beat {
                heading: copy::HOST_REVEAL_BEAT_WHAT.into(),
                text: Some(explains.what.clone()),
            },
            middle: match opened.middle {
                Middle::MostChosen {
                    letter,
                    count,
                    why_tempting,
                } => Beat {
                    heading: copy::fill(
                        copy::HOST_REVEAL_BEAT_WHY,
                        &[("n", &count.to_string()), ("X", letter.as_str())],
                    ),
                    text: Some(why_tempting.clone()),
                },
                Middle::Nobody => Beat {
                    heading: copy::HOST_REVEAL_NOBODY_ELSE.into(),
                    text: None,
                },
            },
            takeaway: Beat {
                heading: copy::HOST_REVEAL_BEAT_REMEMBER.into(),
                text: Some(explains.takeaway.clone()),
            },
        });
        p
    }
}
