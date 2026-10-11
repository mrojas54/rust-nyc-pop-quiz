//! The phase machine (SPEC.md §4, G-6).
//!
//! `idle → live → closed → split → work → reveal → released`. Every transition
//! is a host action; nothing here reads a clock, so nothing can advance on its
//! own, and nothing can be skipped because each phase has exactly one action
//! that leaves it (AC-45, AC-93, AC-97).
//!
//! [`apply`] is the only function that produces a [`Machine`] in any phase but
//! `idle`, and `Machine`'s fields are private to this module, so there is no
//! other way to hold one in `work` or `reveal`. That is what makes
//! [`RevealWitness`] mean something: it is minted by [`Machine::revealed`] and
//! nowhere else, and it is what the sealed `answers` module demands before it
//! will say anything (G-3, AC-61).
//!
//! `←`/`→` ([`Step`]) move `trace_step` inside `work` and `reveal`. They are
//! commands, not transitions: the phase never changes on a step (D-10).
//!
//! ```compile_fail
//! // G-6: a machine cannot be forged into a phase; only `apply` makes one.
//! use room::phase::{Machine, Phase, TraceLen};
//! let len = TraceLen::new(5).unwrap();
//! let m = Machine { phase: Phase::Reveal, trace_step: Some(4), trace_len: len };
//! ```
//!
//! The twin of the snippet above — same paths, and the one line that differs is
//! the struct literal — so the failure above can only be that line:
//!
//! ```
//! use room::phase::{apply, Applied, Command, HostAction, Machine, Phase, State, TraceLen};
//! let len = TraceLen::new(5).unwrap();
//! let Ok(Applied::Room(m)) = apply(State::Unmade(len), Command::Host(HostAction::CreateRoom))
//! else { panic!("Create a room makes an idle room") };
//! let _: &Machine = &m;
//! assert_eq!(m.phase(), Phase::Idle);
//! ```

use std::marker::PhantomData;

use serde::Serialize;

use crate::copy;

/// The seven phases, in G-6 order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Idle,
    Live,
    Closed,
    Split,
    Work,
    Reveal,
    Released,
}

impl Phase {
    /// Every phase, in order.
    pub const ALL: [Phase; 7] = [
        Phase::Idle,
        Phase::Live,
        Phase::Closed,
        Phase::Split,
        Phase::Work,
        Phase::Reveal,
        Phase::Released,
    ];

    /// Position in G-6 order. An exhaustive match, so a new phase breaks the
    /// build here before it can slip past [`Phase::ALL`].
    pub fn index(self) -> usize {
        match self {
            Phase::Idle => 0,
            Phase::Live => 1,
            Phase::Closed => 2,
            Phase::Split => 3,
            Phase::Work => 4,
            Phase::Reveal => 5,
            Phase::Released => 6,
        }
    }

    /// The host screen's phase label (SPEC §11, AC-49).
    pub fn host_label(self) -> &'static str {
        match self {
            Phase::Idle => copy::HOST_PHASE_IDLE,
            Phase::Live => copy::HOST_PHASE_LIVE,
            Phase::Closed => copy::HOST_PHASE_CLOSED,
            Phase::Split => copy::HOST_PHASE_SPLIT,
            Phase::Work => copy::HOST_PHASE_WORK,
            Phase::Reveal => copy::HOST_PHASE_REVEAL,
            Phase::Released => copy::HOST_PHASE_RELEASED,
        }
    }

    /// The one host action that leaves this phase — the host screen's single
    /// primary action (AC-49). This table *is* the machine: [`apply`] accepts
    /// exactly this action in exactly this phase.
    pub fn next_action(self) -> HostAction {
        match self {
            Phase::Idle => HostAction::PutOnScreen,
            Phase::Live => HostAction::CloseAnswers,
            Phase::Closed => HostAction::ShowSplit,
            Phase::Split => HostAction::WalkIt,
            Phase::Work => HostAction::Reveal,
            Phase::Reveal => HostAction::ReleaseRoom,
            Phase::Released => HostAction::RunItAgain,
        }
    }

    /// The phase [`Phase::next_action`] leads to. `None` for `released`: *Run it
    /// again* makes a new room and never re-enters this one (G-10).
    fn successor(self) -> Option<Phase> {
        match self {
            Phase::Idle => Some(Phase::Live),
            Phase::Live => Some(Phase::Closed),
            Phase::Closed => Some(Phase::Split),
            Phase::Split => Some(Phase::Work),
            Phase::Work => Some(Phase::Reveal),
            Phase::Reveal => Some(Phase::Released),
            Phase::Released => None,
        }
    }

    /// Whether `←`/`→` mean anything here (D-10).
    fn steps_the_trace(self) -> bool {
        match self {
            Phase::Work | Phase::Reveal => true,
            Phase::Idle | Phase::Live | Phase::Closed | Phase::Split | Phase::Released => false,
        }
    }
}

/// The eight host actions of AC-45, and no others.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostAction {
    CreateRoom,
    PutOnScreen,
    CloseAnswers,
    ShowSplit,
    WalkIt,
    Reveal,
    ReleaseRoom,
    RunItAgain,
}

impl HostAction {
    pub const ALL: [HostAction; 8] = [
        HostAction::CreateRoom,
        HostAction::PutOnScreen,
        HostAction::CloseAnswers,
        HostAction::ShowSplit,
        HostAction::WalkIt,
        HostAction::Reveal,
        HostAction::ReleaseRoom,
        HostAction::RunItAgain,
    ];

    /// The button's words (SPEC §11, *Host, actions*).
    pub fn label(self) -> &'static str {
        match self {
            HostAction::CreateRoom => copy::HOST_ACTION_CREATE,
            HostAction::PutOnScreen => copy::HOST_ACTION_PUT_ON_SCREEN,
            HostAction::CloseAnswers => copy::HOST_ACTION_CLOSE,
            HostAction::ShowSplit => copy::HOST_ACTION_SHOW_SPLIT,
            HostAction::WalkIt => copy::HOST_ACTION_WALK,
            HostAction::Reveal => copy::HOST_ACTION_REVEAL,
            HostAction::ReleaseRoom => copy::HOST_ACTION_RELEASE,
            HostAction::RunItAgain => copy::HOST_ACTION_RUN_AGAIN,
        }
    }

    /// The route segment for this action (`POST /rooms/{id}/<slug>`).
    pub fn slug(self) -> &'static str {
        match self {
            HostAction::CreateRoom => "create",
            HostAction::PutOnScreen => "put-on-screen",
            HostAction::CloseAnswers => "close-answers",
            HostAction::ShowSplit => "show-split",
            HostAction::WalkIt => "walk-it",
            HostAction::Reveal => "reveal",
            HostAction::ReleaseRoom => "release",
            HostAction::RunItAgain => "run-it-again",
        }
    }
}

/// `←` and `→`: step the trace. Never a phase transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    Back,
    Forward,
}

impl Step {
    pub const ALL: [Step; 2] = [Step::Back, Step::Forward];

    pub fn slug(self) -> &'static str {
        match self {
            Step::Back => "step-back",
            Step::Forward => "step-forward",
        }
    }
}

/// Everything the host can send: one of the eight actions, or a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    Host(HostAction),
    Step(Step),
}

impl Command {
    /// All ten commands: the eight actions, then `←`, `→`.
    pub fn all() -> [Command; 10] {
        let h = HostAction::ALL;
        [
            Command::Host(h[0]),
            Command::Host(h[1]),
            Command::Host(h[2]),
            Command::Host(h[3]),
            Command::Host(h[4]),
            Command::Host(h[5]),
            Command::Host(h[6]),
            Command::Host(h[7]),
            Command::Step(Step::Back),
            Command::Step(Step::Forward),
        ]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Command::Host(a) => a.slug(),
            Command::Step(s) => s.slug(),
        }
    }
}

/// `M`, the number of trace steps. At least two: `work` shows `0..=M-2`, so a
/// one-step trace would leave the walk-through empty (SPEC §7.4, D-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceLen(u16);

impl TraceLen {
    pub fn new(m: usize) -> Option<TraceLen> {
        if m < 2 {
            return None;
        }
        u16::try_from(m).ok().map(TraceLen)
    }

    pub fn get(self) -> u16 {
        self.0
    }

    /// The last step `work` may show: `M-2`. The final step is the resolving
    /// one and is withheld until `reveal` (G-3, D-10).
    fn last_in_work(self) -> u16 {
        self.0 - 2
    }

    /// The final step, `M-1`, where `reveal` enters.
    fn last(self) -> u16 {
        self.0 - 1
    }
}

/// One room's position in the machine. Fields are private: the only way to
/// hold a `Machine` is to be handed one by [`apply`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine {
    phase: Phase,
    trace_step: Option<u16>,
    trace_len: TraceLen,
}

impl Machine {
    /// The one phase value every projection reads (AC-81).
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// `Some` only in `work` and `reveal` (SPEC §3.4).
    pub fn trace_step(&self) -> Option<u16> {
        self.trace_step
    }

    pub fn trace_len(&self) -> TraceLen {
        self.trace_len
    }

    /// Whether `←` / `→` would be accepted right now. The host screen disables
    /// the other one; [`apply`] refuses it either way.
    pub fn can_step(&self, step: Step) -> bool {
        apply(State::Room(self.clone()), Command::Step(step)).is_ok()
    }

    /// The proof that this machine is in `reveal`, and the only way to get one.
    ///
    /// The witness borrows the machine, so it cannot outlive the state that
    /// produced it: a witness taken in `reveal` is gone before the room can
    /// move on.
    pub fn revealed(&self) -> Option<RevealWitness<'_>> {
        match self.phase {
            Phase::Reveal => Some(RevealWitness {
                _machine: PhantomData,
            }),
            Phase::Idle
            | Phase::Live
            | Phase::Closed
            | Phase::Split
            | Phase::Work
            | Phase::Released => None,
        }
    }
}

/// Evidence that a room is in `reveal`. The sealed `answers` module opens only
/// for one of these (G-3, AC-61).
///
/// Its one field is private, so nothing outside this module can build it; the
/// only constructor is [`Machine::revealed`], which returns `None` in every
/// other phase. The crate forbids `unsafe`, so it cannot be conjured either.
///
/// ```compile_fail
/// // AC-61: a witness cannot be forged.
/// use room::phase::RevealWitness;
/// let w: RevealWitness<'static> = RevealWitness { _machine: std::marker::PhantomData };
/// ```
///
/// Twin — same paths; the only difference is where the witness comes from:
///
/// ```
/// use room::phase::{Machine, RevealWitness};
/// fn from_the_machine(m: &Machine) -> Option<RevealWitness<'_>> { m.revealed() }
/// ```
pub struct RevealWitness<'m> {
    _machine: PhantomData<&'m Machine>,
}

/// What [`apply`] acts on: a room that does not exist yet (only *Create a room*
/// means anything), or one that does.
#[derive(Clone, Debug)]
pub enum State {
    Unmade(TraceLen),
    Room(Machine),
}

/// A legal command's result.
#[derive(Debug, PartialEq, Eq)]
pub enum Applied {
    /// The room's new position (a transition, or a step).
    Room(Machine),
    /// *Run it again*: make a **new** room. This one stays `released`.
    NewRoom,
}

/// A refused command, with the reason in plain words.
///
/// The reasons are API diagnostics for the host page, which only ever offers
/// the one legal action; they are not SPEC §11 copy and are never shown to
/// participants. They avoid §11's *Forbidden* patterns all the same.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub from: Option<Phase>,
    pub command: Command,
    pub reason: String,
}

/// The phase machine: the only way a phase changes (G-6, AC-45).
///
/// Pure — no clock, no randomness, no I/O — so there is no timer and nothing
/// advances on its own. Exactly seven transitions are legal (*Create a room*
/// into `idle`, then one per phase up to `released`); *Run it again* from
/// `released` asks for a new room; `←`/`→` step the trace inside `work`
/// (`0..=M-2`) and `reveal` (`0..=M-1`). Everything else is refused.
pub fn apply(state: State, command: Command) -> Result<Applied, Refused> {
    let machine = match state {
        State::Unmade(trace_len) => {
            return match command {
                Command::Host(HostAction::CreateRoom) => Ok(Applied::Room(Machine {
                    phase: Phase::Idle,
                    trace_step: None,
                    trace_len,
                })),
                Command::Host(_) | Command::Step(_) => Err(Refused {
                    from: None,
                    command,
                    reason: "There is no room yet. Create a room comes first.".into(),
                }),
            };
        }
        State::Room(machine) => machine,
    };
    let phase = machine.phase;
    let refuse = |reason: String| Refused {
        from: Some(phase),
        command,
        reason,
    };

    match command {
        Command::Host(HostAction::CreateRoom) => Err(refuse(
            "This room already exists. Create a room makes a separate one.".into(),
        )),
        Command::Host(action) if action == phase.next_action() => match phase.successor() {
            None => Ok(Applied::NewRoom),
            Some(next) => {
                let len = machine.trace_len;
                let trace_step = match next {
                    // D-10: the walk-through starts at the first step…
                    Phase::Work => Some(0),
                    // …and the reveal enters at the one that prints.
                    Phase::Reveal => Some(len.last()),
                    Phase::Idle | Phase::Live | Phase::Closed | Phase::Split | Phase::Released => None,
                };
                Ok(Applied::Room(Machine {
                    phase: next,
                    trace_step,
                    trace_len: len,
                }))
            }
        },
        Command::Host(action) => Err(refuse(match phase {
            Phase::Released => "The room is released. Run it again starts a new room.".into(),
            _ => format!(
                "{} isn't next. {} comes next.",
                action.label(),
                phase.next_action().label()
            ),
        })),
        Command::Step(step) => {
            if !phase.steps_the_trace() {
                return Err(refuse(
                    "The trace only steps while walking it through or at the answer.".into(),
                ));
            }
            let at = machine
                .trace_step
                .expect("work and reveal always hold a trace step");
            let len = machine.trace_len;
            let ceiling = match phase {
                Phase::Work => len.last_in_work(),
                _ => len.last(),
            };
            let to = match step {
                Step::Back if at == 0 => {
                    return Err(refuse("That's the first step.".into()));
                }
                Step::Back => at - 1,
                Step::Forward if at >= ceiling => {
                    return Err(refuse(match phase {
                        Phase::Work => "That's the last step before the reveal.".into(),
                        _ => "That's the last step.".into(),
                    }));
                }
                Step::Forward => at + 1,
            };
            Ok(Applied::Room(Machine {
                phase,
                trace_step: Some(to),
                trace_len: len,
            }))
        }
    }
}

// Pins `apply`'s signature: a state and a command in, a result out. If a clock,
// a context or a callback were ever added, this line stops compiling (G-6).
const _: fn(State, Command) -> Result<Applied, Refused> = apply;

#[cfg(test)]
mod tests {
    use super::*;

    fn created(m: usize) -> Machine {
        match apply(
            State::Unmade(TraceLen::new(m).unwrap()),
            Command::Host(HostAction::CreateRoom),
        ) {
            Ok(Applied::Room(m)) => m,
            other => panic!("create: {other:?}"),
        }
    }

    #[test]
    fn a_trace_needs_two_steps() {
        assert!(TraceLen::new(0).is_none());
        assert!(TraceLen::new(1).is_none());
        assert_eq!(TraceLen::new(2).unwrap().get(), 2);
    }

    #[test]
    fn the_witness_exists_only_in_reveal() {
        let mut m = created(5);
        for phase in Phase::ALL {
            assert_eq!(m.phase(), phase);
            assert_eq!(m.revealed().is_some(), phase == Phase::Reveal, "{phase:?}");
            match apply(State::Room(m.clone()), Command::Host(phase.next_action())) {
                Ok(Applied::Room(next)) => m = next,
                Ok(Applied::NewRoom) => assert_eq!(phase, Phase::Released),
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}
