//! The phase machine, exhaustively (AC-45, AC-93, AC-97, G-6, D-10).

mod common;

use room::phase::{apply, Applied, Command, HostAction, Machine, Phase, Refused, State, Step, TraceLen};

fn create(m: usize) -> Machine {
    match apply(
        State::Unmade(TraceLen::new(m).unwrap()),
        Command::Host(HostAction::CreateRoom),
    ) {
        Ok(Applied::Room(machine)) => machine,
        other => panic!("create: {other:?}"),
    }
}

fn act(m: &Machine, c: Command) -> Result<Applied, Refused> {
    apply(State::Room(m.clone()), c)
}

fn moved(m: &Machine, c: Command) -> Machine {
    match act(m, c) {
        Ok(Applied::Room(next)) => next,
        other => panic!("{c:?} from {:?}: {other:?}", m.phase()),
    }
}

/// A machine at `phase` on entry, reached only by legal host actions.
fn at(phase: Phase, m: usize) -> Machine {
    let mut machine = create(m);
    while machine.phase() != phase {
        machine = moved(&machine, Command::Host(machine.phase().next_action()));
    }
    machine
}

/// The whole table, written out by hand: rows are the room's state (no room
/// yet, then the seven phases on entry), columns the ten commands in
/// `Command::all()` order — Create, Put on screen, Close, Split, Walk, Reveal,
/// Release, Run again, ←, →. `.` is refused; a phase name is where it goes;
/// `step` stays in the phase and moves the trace; `new` is a new room.
const TABLE: [(&str, [&str; 10]); 8] = [
    ("unmade",   ["idle", ".",    ".",      ".",     ".",    ".",      ".",        ".",   ".",    "."   ]),
    ("idle",     [".",    "live", ".",      ".",     ".",    ".",      ".",        ".",   ".",    "."   ]),
    ("live",     [".",    ".",    "closed", ".",     ".",    ".",      ".",        ".",   ".",    "."   ]),
    ("closed",   [".",    ".",    ".",      "split", ".",    ".",      ".",        ".",   ".",    "."   ]),
    ("split",    [".",    ".",    ".",      ".",     "work", ".",      ".",        ".",   ".",    "."   ]),
    // work enters at step 0: ← is refused, → steps.
    ("work",     [".",    ".",    ".",      ".",     ".",    "reveal", ".",        ".",   ".",    "step"]),
    // reveal enters at M-1: ← steps, → is refused.
    ("reveal",   [".",    ".",    ".",      ".",     ".",    ".",      "released", ".",   "step", "."   ]),
    ("released", [".",    ".",    ".",      ".",     ".",    ".",      ".",        "new", ".",    "."   ]),
];

fn name(p: Phase) -> &'static str {
    match p {
        Phase::Idle => "idle",
        Phase::Live => "live",
        Phase::Closed => "closed",
        Phase::Split => "split",
        Phase::Work => "work",
        Phase::Reveal => "reveal",
        Phase::Released => "released",
    }
}

#[test]
fn every_state_and_command_does_exactly_what_the_table_says() {
    const M: usize = 5;
    assert_eq!(Command::all().len(), 10);
    let mut legal = 0;
    for (row, (state_name, expected)) in TABLE.iter().enumerate() {
        let state = if row == 0 {
            State::Unmade(TraceLen::new(M).unwrap())
        } else {
            let phase = Phase::ALL[row - 1];
            assert_eq!(name(phase), *state_name);
            State::Room(at(phase, M))
        };
        let from = match &state {
            State::Room(m) => Some(m.phase()),
            State::Unmade(_) => None,
        };
        for (col, command) in Command::all().into_iter().enumerate() {
            let got = match apply(state.clone(), command) {
                Err(refused) => {
                    assert_eq!(refused.command, command);
                    assert_eq!(refused.from, from);
                    assert!(!refused.reason.is_empty(), "a refusal says why");
                    ".".to_string()
                }
                Ok(Applied::NewRoom) => "new".to_string(),
                Ok(Applied::Room(m)) if Some(m.phase()) == from => "step".to_string(),
                Ok(Applied::Room(m)) => name(m.phase()).to_string(),
            };
            if got != "." {
                legal += 1;
            }
            assert_eq!(got, expected[col], "{state_name} × {command:?}");
        }
    }
    // Seven transitions, *Run it again*, and the two steps available on entry.
    assert_eq!(legal, 7 + 1 + 2);
}

#[test]
fn the_host_actions_are_exactly_the_eight_of_ac_45() {
    let labels: Vec<&str> = HostAction::ALL.iter().map(|a| a.label()).collect();
    assert_eq!(
        labels,
        [
            "Create a room",
            "Start",
            "Close answers",
            "Show the room its split",
            "Trace",
            "Reveal",
            "End Pop Quiz",
            "Run it again",
        ]
    );
    // Each phase's one primary action is a distinct host action, never
    // *Create a room*, and together they are the other seven (AC-49).
    let mut next: Vec<HostAction> = Phase::ALL.iter().map(|p| p.next_action()).collect();
    next.sort_by_key(|a| HostAction::ALL.iter().position(|b| b == a));
    assert_eq!(next, HostAction::ALL[1..]);
}

#[test]
fn no_transition_skips_a_phase_and_reveal_needs_the_walk_through() {
    // AC-93, AC-97, G-6: every legal transition moves exactly one phase on;
    // reveal is reachable only from work, and work only from split.
    for phase in Phase::ALL {
        for step_to in [0, 1, 3] {
            let mut m = at(phase, 5);
            // Also try from every reachable trace position, not only entry.
            for _ in 0..step_to {
                if let Ok(Applied::Room(n)) = act(&m, Command::Step(Step::Forward)) {
                    m = n;
                }
            }
            for command in Command::all() {
                if let Ok(Applied::Room(next)) = act(&m, command) {
                    let (a, b) = (phase.index(), next.phase().index());
                    assert!(b == a || b == a + 1, "{phase:?} → {:?}", next.phase());
                    if next.phase() == Phase::Reveal && phase != Phase::Reveal {
                        assert_eq!(phase, Phase::Work);
                    }
                    if next.phase() == Phase::Work && phase != Phase::Work {
                        assert_eq!(phase, Phase::Split);
                    }
                }
            }
        }
    }
    assert_eq!(Phase::ALL.map(|p| p.index()), [0, 1, 2, 3, 4, 5, 6]);
}

#[test]
fn released_never_re_enters_the_room() {
    let released = at(Phase::Released, 5);
    assert_eq!(act(&released, Command::Host(HostAction::RunItAgain)), Ok(Applied::NewRoom));
    for command in Command::all() {
        if let Ok(Applied::Room(m)) = act(&released, command) {
            panic!("released moved to {:?} on {command:?}", m.phase());
        }
    }
}

fn walk_bounds(m: usize) {
    let len = m as u16;
    // work enters at 0 and never reaches the resolving step M-1 (D-10).
    let mut work = moved(&at(Phase::Split, m), Command::Host(HostAction::WalkIt));
    assert_eq!(work.trace_step(), Some(0));
    assert!(act(&work, Command::Step(Step::Back)).is_err());
    for expected in 1..=len - 2 {
        work = moved(&work, Command::Step(Step::Forward));
        assert_eq!(work.trace_step(), Some(expected));
        assert_eq!(work.phase(), Phase::Work);
    }
    let refused = act(&work, Command::Step(Step::Forward)).unwrap_err();
    assert_eq!(refused.reason, "That's the last step before the reveal.");
    for expected in (0..len - 2).rev() {
        work = moved(&work, Command::Step(Step::Back));
        assert_eq!(work.trace_step(), Some(expected));
    }

    // reveal enters at M-1, from wherever the walk-through stood, and may step
    // the whole trace.
    for from in [0, len - 2] {
        let mut w = moved(&at(Phase::Split, m), Command::Host(HostAction::WalkIt));
        for _ in 0..from {
            w = moved(&w, Command::Step(Step::Forward));
        }
        let mut reveal = moved(&w, Command::Host(HostAction::Reveal));
        assert_eq!(reveal.trace_step(), Some(len - 1));
        assert!(act(&reveal, Command::Step(Step::Forward)).is_err());
        for expected in (0..len - 1).rev() {
            reveal = moved(&reveal, Command::Step(Step::Back));
            assert_eq!(reveal.trace_step(), Some(expected));
            assert_eq!(reveal.phase(), Phase::Reveal);
        }
        assert!(act(&reveal, Command::Step(Step::Back)).is_err());
        for expected in 1..len {
            reveal = moved(&reveal, Command::Step(Step::Forward));
            assert_eq!(reveal.trace_step(), Some(expected));
        }
        // Leaving reveal drops the trace position.
        let released = moved(&reveal, Command::Host(HostAction::ReleaseRoom));
        assert_eq!(released.trace_step(), None);
    }
}

#[test]
fn trace_step_bounds_on_a_five_step_trace() {
    walk_bounds(5);
}

#[test]
fn trace_step_bounds_on_q3s_real_trace() {
    let q3 = common::q3();
    let m = q3.public().trace_len().get() as usize;
    assert_eq!(m, 6);
    walk_bounds(m);
}

#[test]
fn trace_step_bounds_on_the_shortest_trace() {
    // M = 2: work shows step 0 only; neither arrow moves it.
    walk_bounds(2);
    let work = moved(&at(Phase::Split, 2), Command::Host(HostAction::WalkIt));
    assert!(act(&work, Command::Step(Step::Forward)).is_err());
    assert!(act(&work, Command::Step(Step::Back)).is_err());
}

#[test]
fn steps_outside_work_and_reveal_are_refused_and_never_change_the_phase() {
    for phase in [Phase::Idle, Phase::Live, Phase::Closed, Phase::Split, Phase::Released] {
        let m = at(phase, 5);
        assert_eq!(m.trace_step(), None);
        for s in Step::ALL {
            let refused = act(&m, Command::Step(s)).unwrap_err();
            assert_eq!(
                refused.reason,
                "The trace only steps while walking it through or at the answer."
            );
        }
    }
}

#[test]
fn refusal_reasons_carry_no_forbidden_copy() {
    // SPEC §11's Forbidden row, applied to the machine's diagnostics too.
    let forbidden = [
        "turn to", "volunteer", "find someone", "wrong", "incorrect", "✗", "argu",
    ];
    for row in 0..8 {
        let state = if row == 0 {
            State::Unmade(TraceLen::new(5).unwrap())
        } else {
            State::Room(at(Phase::ALL[row - 1], 5))
        };
        for c in Command::all() {
            if let Err(r) = apply(state.clone(), c) {
                let lower = r.reason.to_lowercase();
                for f in forbidden {
                    assert!(!lower.contains(f), "{:?}: {}", c, r.reason);
                }
            }
        }
    }
}
