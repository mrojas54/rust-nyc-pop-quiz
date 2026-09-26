//! The room record (SPEC.md §3.4, §4.4, §4.5) driven directly, without HTTP.

mod common;

use std::cell::Cell;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use room::answers::Middle;
use room::auth::OrganizerId;
use room::phase::{Command, HostAction, Phase, Step};
use room::question::Letter;
use room::rooms::{new_code, CloseSnapshot, LiveCounts, Room, Urls, CODE_ALPHABET, ROOM_LIFETIME};
use room::view;

fn t0() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_000_000)
}

fn new_room() -> Room {
    Room::create(
        Arc::new(common::q3()),
        OrganizerId("organizer-1".into()),
        "ABC234".into(),
        &Urls::default(),
        t0(),
    )
    .expect("a room")
}

/// Drive `room` to `phase` with legal host actions; `totals` is the close
/// snapshot, if the drive passes `closed`.
fn drive(room: &mut Room, phase: Phase, totals: [u32; 5]) {
    while room.phase() != phase {
        let action = room.phase().next_action();
        room.act(Command::Host(action), t0(), || CloseSnapshot::from_totals(totals))
            .unwrap_or_else(|r| panic!("{r:?}"));
    }
}

#[test]
fn a_new_room_has_the_shape_of_section_3_4() {
    let room = new_room();
    assert_eq!(room.phase(), Phase::Idle);
    assert_eq!(room.code(), "ABC234");
    assert_eq!(room.question_id(), "q3");
    assert_eq!(room.id().len(), 32);
    assert_eq!(room.expires_at().duration_since(room.created_at()).unwrap(), ROOM_LIFETIME);
    assert_eq!(ROOM_LIFETIME, Duration::from_secs(4 * 3600));
    assert_eq!(room.released_at(), None);
    assert!(room.open().is_none());
    assert!(room.public().frozen.is_none());
    assert_eq!(room.public().join_url, "http://127.0.0.1:3000/ABC234");
    assert!(room.host_resume_url().starts_with(&format!("http://127.0.0.1:3000/host/{}#", room.id())));
}

#[test]
fn codes_are_six_characters_never_o_0_i_or_1() {
    assert_eq!(CODE_ALPHABET.len(), 32);
    for forbidden in [b'O', b'0', b'I', b'1'] {
        assert!(!CODE_ALPHABET.contains(&forbidden));
    }
    let mut seen = std::collections::HashSet::new();
    for _ in 0..10_000 {
        let code = new_code();
        assert_eq!(code.len(), 6);
        assert!(code.bytes().all(|b| CODE_ALPHABET.contains(&b)), "{code}");
        seen.insert(code);
    }
    // 32^6 ≈ 10^9: ten thousand draws should essentially never collide.
    assert!(seen.len() > 9_990);
}

#[test]
fn the_host_session_never_rotates() {
    let mut room = new_room();
    let resume = room.host_resume_url().to_string();
    let session = resume.rsplit('#').next().unwrap().to_string();
    assert_eq!(session.len(), 64);
    assert!(!room.host_matches(""));
    assert!(!room.host_matches(&session[..63]));
    for phase in Phase::ALL {
        drive(&mut room, phase, [0; 5]);
        assert!(room.host_matches(&session), "{phase:?}");
        assert_eq!(room.host_resume_url(), resume);
    }
}

#[test]
fn closed_freezes_answered_and_totals_while_present_keeps_counting() {
    let mut room = new_room();
    drive(&mut room, Phase::Live, [0; 5]);
    assert!(room.accepts_answers());
    room.set_live_counts(LiveCounts { present: 12, answered_live: 9 });
    assert_eq!(view::host(&room).answered, Some(9));

    let calls = Cell::new(0);
    room.act(Command::Host(HostAction::CloseAnswers), t0(), || {
        calls.set(calls.get() + 1);
        CloseSnapshot::from_totals([1, 5, 0, 0, 3])
    })
    .unwrap();
    assert!(!room.accepts_answers());
    assert_eq!(room.public().frozen, Some((9, [1, 5, 0, 0, 3])));

    // People keep arriving and leaving; the frozen numbers do not move.
    room.set_live_counts(LiveCounts { present: 15, answered_live: 2 });
    for phase in [Phase::Closed, Phase::Split, Phase::Work, Phase::Reveal] {
        drive(&mut room, phase, [9, 9, 9, 9, 9]);
        assert_eq!(room.public().frozen, Some((9, [1, 5, 0, 0, 3])), "{phase:?}");
        assert_eq!(room.public().present, 15);
        assert_eq!(view::host(&room).answered, Some(9));
    }
    assert_eq!(calls.get(), 1, "the close snapshot is taken once");

    let split = view::wall(&room, &Urls::default()).split.unwrap();
    assert_eq!(split.line, "9 of 15 in the room answered");
    assert_eq!(split.bars.iter().map(|b| b.count).collect::<Vec<_>>(), [1, 5, 0, 0, 3]);
    assert_eq!(split.bars.iter().map(|b| b.percent).collect::<Vec<_>>(), [11, 56, 0, 0, 33]);
}

#[test]
fn a_split_with_no_answers_shows_empty_bars() {
    let mut room = new_room();
    drive(&mut room, Phase::Split, [0; 5]);
    let split = view::wall(&room, &Urls::default()).split.unwrap();
    assert!(split.bars.iter().all(|b| b.count == 0 && b.percent == 0));
    assert_eq!(split.line, "0 of 0 in the room answered");
}

#[test]
fn released_at_is_written_by_release_and_nothing_else() {
    let mut room = new_room();
    for phase in [Phase::Live, Phase::Closed, Phase::Split, Phase::Work, Phase::Reveal] {
        drive(&mut room, phase, [0; 5]);
        assert_eq!(room.released_at(), None, "{phase:?}");
    }
    let later = t0() + Duration::from_secs(600);
    room.act(Command::Host(HostAction::ReleaseRoom), later, || unreachable!()).unwrap();
    assert_eq!(room.released_at(), Some(later));
    assert!(room.open().is_none(), "the answers close again at release");
}

#[test]
fn only_reveal_opens_the_answers() {
    let mut room = new_room();
    for phase in Phase::ALL {
        drive(&mut room, phase, [0, 2, 0, 0, 1]);
        assert_eq!(room.open().is_some(), phase == Phase::Reveal, "{phase:?}");
    }
}

#[test]
fn every_change_bumps_the_revision() {
    let mut room = new_room();
    let r0 = room.revision();
    room.set_live_counts(LiveCounts { present: 1, answered_live: 0 });
    let r1 = room.revision();
    assert!(r1 > r0);
    room.set_live_counts(LiveCounts { present: 1, answered_live: 0 });
    assert_eq!(room.revision(), r1, "no change, no broadcast");
    drive(&mut room, Phase::Live, [0; 5]);
    assert!(room.revision() > r1);
}

/// q3's correct option is E (the verifier's stdout). §4.5, at `reveal`.
fn verdict(totals: [u32; 5]) -> (view::WallPayload, view::HostPayload, Option<(Letter, u32)>) {
    let mut room = new_room();
    drive(&mut room, Phase::Reveal, totals);
    let opened = room.open().unwrap();
    assert_eq!(opened.revealed.correct, Letter::E);
    let middle = match opened.middle {
        Middle::MostChosen { letter, count, .. } => Some((*letter, *count)),
        Middle::Nobody => None,
    };
    drop(opened);
    (view::wall(&room, &Urls::default()), view::host(&room), middle)
}

fn assert_agree(totals: [u32; 5], expected: Option<(Letter, u32)>) {
    let (wall, host, middle) = verdict(totals);
    assert_eq!(middle, expected, "{totals:?}");
    let wall_middle = wall.reveal.unwrap().middle;
    let host_middle = host.read_aloud.unwrap().middle;
    match expected {
        Some((letter, n)) => {
            assert_eq!(wall_middle.letter, Some(letter));
            assert_eq!(wall_middle.count, Some(n));
            assert_eq!(wall_middle.line, format!("{n} of us said {}", letter.as_str()));
            // The host names the same option, with the same count.
            assert_eq!(host_middle.heading, format!("Why {n} of us said {}", letter.as_str()));
            assert!(host_middle.text.is_some_and(|t| !t.is_empty()));
        }
        None => {
            assert_eq!(wall_middle.letter, None);
            assert_eq!(wall_middle.line, "Nobody read it another way.");
            assert_eq!(host_middle.heading, "Why nobody said anything else");
            assert_eq!(host_middle.text, None);
        }
    }
}

#[test]
fn the_most_chosen_incorrect_option_is_named_the_same_on_wall_and_host() {
    assert_agree([1, 5, 2, 0, 3], Some((Letter::B, 5)));
    // Ties go to the lower letter.
    assert_agree([1, 5, 5, 0, 3], Some((Letter::B, 5)));
    assert_agree([4, 0, 0, 4, 9], Some((Letter::A, 4)));
    // The correct option is never it, even as the room's top pick.
    assert_agree([2, 0, 0, 0, 40], Some((Letter::A, 2)));
    // The does-not-compile option can be it.
    assert_agree([0, 0, 0, 3, 1], Some((Letter::D, 3)));
}

#[test]
fn nobody_read_it_another_way() {
    // Everyone right.
    assert_agree([0, 0, 0, 0, 7], None);
    // Nobody answered.
    assert_agree([0, 0, 0, 0, 0], None);
}

#[test]
fn the_why_tempting_text_is_the_named_options_own() {
    let q = common::q3_json();
    let (_, host, _) = verdict([0, 0, 6, 1, 0]);
    let text = host.read_aloud.unwrap().middle.text.unwrap();
    assert_eq!(text, q["options"][2]["why_tempting"].as_str().unwrap());
}

#[test]
fn work_shows_steps_up_to_m_minus_2_and_reveal_the_rest() {
    let mut room = new_room();
    drive(&mut room, Phase::Work, [0; 5]);
    let m = 6;
    let mut last_note;
    loop {
        let wall = view::wall(&room, &Urls::default());
        let trace = wall.trace.unwrap();
        assert!(trace.at <= m - 2);
        assert_eq!(trace.m, m);
        assert!(trace.step.values.iter().all(|v| v.name != "stdout"));
        assert_eq!(wall.colour, Some(false));
        let host = view::host(&room);
        let step = host.step.unwrap();
        assert_eq!(step.at, trace.at);
        assert_eq!(step.note, trace.step.note);
        last_note = step.note;
        if room.act(Command::Step(Step::Forward), t0(), || unreachable!()).is_err() {
            break;
        }
    }
    let q = common::q3_json();
    let steps = q["trace"]["steps"].as_array().unwrap();
    assert_eq!(last_note, steps[4]["note"].as_str().unwrap());

    room.act(Command::Host(HostAction::Reveal), t0(), || unreachable!()).unwrap();
    let wall = view::wall(&room, &Urls::default());
    let trace = wall.trace.unwrap();
    assert_eq!(trace.at, m - 1);
    assert_eq!(trace.step.note, steps[5]["note"].as_str().unwrap());
    assert!(trace.step.values.iter().any(|v| v.name == "stdout"));
    let receipt = wall.reveal.unwrap().receipt;
    assert_eq!(receipt.heading, "How we know");
    assert_eq!(
        receipt.lines,
        ["✓ Compiled", "✓ Ran 5 times", "✓ Output never varied", "✓ Miri ran clean"]
    );
}
