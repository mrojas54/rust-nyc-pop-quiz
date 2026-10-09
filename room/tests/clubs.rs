//! Several clubs on one room server (D-26, AC-103…AC-105).
//!
//! One Discord server, one question bank, and a host role per club. Driven
//! through the Discord mock (`common::discord_mock`), as `tests/auth.rs` is.
//!
//! | Criterion | Tests |
//! |---|---|
//! | AC-103 | `ac103_*`: a club's role hosts that club only; an unknown or malformed club is refused as a missing role is |
//! | AC-104 | `ac104_*`: two clubs, one question, two rooms at once; each club's `/last` is its own |
//! | AC-105 | `ac105_*`: never twice is per club; the ledger names the club |
//! | config | `config_*`: `POPQUIZ_CLUBS` |

mod common;

use axum::http::{Method, StatusCode};
use common::discord_mock::{canary, query, snowflake, Membership, Rig};
use room::config::{Config, ConfigError};
use serde_json::Value;

const FULL_WALK: [&str; 6] = ["put-on-screen", "close-answers", "show-split", "walk-it", "reveal", "release"];

async fn walk(rig: &Rig, room: &str, host: &str) {
    for slug in FULL_WALK {
        let (status, body) = rig.call(Method::POST, &format!("/rooms/{room}/{slug}"), Some(host), None).await;
        assert_eq!(status, StatusCode::OK, "{slug}: {body}");
    }
}

fn created(body: &Value) -> (String, String) {
    (body["id"].as_str().unwrap().to_string(), body["host_session"].as_str().unwrap().to_string())
}

async fn last(rig: &Rig, club: &str) -> String {
    let r = rig.raw(Method::GET, &format!("/last/{club}"), &[], None).await;
    assert_eq!(r.status, StatusCode::OK);
    String::from_utf8(r.body).unwrap()
}

fn snapshot_is_empty(page: &str) -> bool {
    page.contains("id=\"take-home\">null</script>")
}

/// A member holding both clubs' roles, for the tests that are not about roles.
fn both(rig: &Rig) -> Membership {
    Membership::Member {
        roles: vec![snowflake(), rig.role(), rig.la_role()],
        admin: false,
    }
}

#[tokio::test]
async fn ac103_a_clubs_role_hosts_that_club_only() {
    let rig = Rig::start().await;
    let (nyc, _) = rig.sign_in(rig.host()).await;
    let (la, _) = rig.sign_in(rig.la_host()).await;

    // Each role hosts its own club ...
    assert_eq!(rig.create_for(Some(&nyc), "nyc", "q3").await.0, StatusCode::CREATED);
    assert_eq!(rig.create_for(Some(&la), "la", "q3").await.0, StatusCode::CREATED);
    // ... and not the other's, naming the missing role either way.
    let (status, body) = rig.create_for(Some(&nyc), "la", "q3-again").await;
    assert_eq!((status, &body["refusal"]), (StatusCode::FORBIDDEN, &Value::from("wrong_role")), "{body}");
    let (status, body) = rig.create_for(Some(&la), "nyc", "q3-again").await;
    assert_eq!((status, &body["refusal"]), (StatusCode::FORBIDDEN, &Value::from("wrong_role")), "{body}");
    assert_eq!(rig.state.room_count(), 2, "the refusals made no room");
}

#[tokio::test]
async fn ac103_a_request_that_names_no_club_is_the_default_club() {
    let rig = Rig::start().await;
    let (la, _) = rig.sign_in(rig.la_host()).await;
    let (status, body) = rig.create(Some(&la), "q3").await;
    assert_eq!((status, &body["refusal"]), (StatusCode::FORBIDDEN, &Value::from("wrong_role")), "{body}");
    let (nyc, _) = rig.sign_in(rig.host()).await;
    assert_eq!(rig.create(Some(&nyc), "q3").await.0, StatusCode::CREATED);
}

#[tokio::test]
async fn ac103_an_unknown_or_malformed_club_is_refused_as_a_missing_role() {
    let rig = Rig::start().await;
    let (both_session, _) = rig.sign_in(both(&rig)).await;
    let known = rig.create_for(Some(&both_session), "la", "q3-again").await;
    assert_eq!(known.0, StatusCode::CREATED, "{}", known.1);
    // Even a member of every role gets the same answer for a club that does
    // not exist: the refusal does not say which clubs do.
    for club in ["mars", "NYC", "la/../nyc", "", "a b", &"x".repeat(40)] {
        let (status, body) = rig.create_for(Some(&both_session), club, "q3").await;
        assert_eq!((status, &body["refusal"]), (StatusCode::FORBIDDEN, &Value::from("wrong_role")), "{club:?}: {body}");
    }
}

#[tokio::test]
async fn ac103_the_sign_in_carries_the_club_back_to_the_host_page() {
    let rig = Rig::start().await;
    let (code, _) = rig.mock.account(rig.la_host());
    let r = rig.raw(Method::GET, "/auth/discord?question=q3&club=la", &[], None).await;
    assert_eq!(r.status, StatusCode::SEE_OTHER);
    let state = query(r.header("location").unwrap(), "state").unwrap();
    let cookie = r.header("set-cookie").and_then(|c| c.split(';').next()).unwrap().to_string();
    let cb = rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await;
    let location = cb.header("location").unwrap();
    let (path, session) = location.split_once('#').expect("a session in the fragment");
    assert_eq!(path, "/host?question=q3&club=la");
    assert!(!session.is_empty());
    // The default club's link keeps the shape it always had.
    let (code, _) = rig.mock.account(rig.host());
    let (_, state, cookie) = rig.begin("q3").await;
    let cb = rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await;
    assert!(cb.header("location").unwrap().starts_with("/host?question=q3#"));
    // A club that is not a slug never reaches Discord.
    let r = rig.raw(Method::GET, "/auth/discord?question=q3&club=NYC%21", &[], None).await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ac104_two_clubs_run_the_same_question_in_two_rooms_at_once() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;
    let (a, b) = (
        rig.create_for(Some(&session), "nyc", "q3").await,
        rig.create_for(Some(&session), "la", "q3").await,
    );
    assert_eq!((a.0, b.0), (StatusCode::CREATED, StatusCode::CREATED), "{} {}", a.1, b.1);
    assert_ne!(a.1["id"], b.1["id"]);
    assert_ne!(a.1["code"], b.1["code"]);
    // One live room per club and question (GAP-8) still holds inside a club.
    let (status, body) = rig.create_for(Some(&session), "la", "q3").await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(body["reason"].as_str().unwrap().contains("open in another room"), "{body}");
}

#[tokio::test]
async fn ac104_each_clubs_take_it_home_page_is_its_own() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;
    assert!(snapshot_is_empty(&last(&rig, "nyc").await));
    assert!(snapshot_is_empty(&last(&rig, "la").await));

    let (id, host) = created(&rig.create_for(Some(&session), "nyc", "q3").await.1);
    walk(&rig, &id, &host).await;

    let nyc = last(&rig, "nyc").await;
    assert!(!snapshot_is_empty(&nyc) && nyc.contains("\"question_id\":\"q3\""), "nyc released q3");
    assert!(snapshot_is_empty(&last(&rig, "la").await), "la has released nothing, so its page carries nothing");
    assert!(snapshot_is_empty(&last(&rig, "mars").await), "an unknown club is an empty page, not an error");
    assert!(snapshot_is_empty(&last(&rig, "NYC%21").await));
    assert_eq!(last(&rig, "nyc").await, rig_last(&rig).await, "/last is the default club's page");
}

async fn rig_last(rig: &Rig) -> String {
    let r = rig.raw(Method::GET, "/last", &[], None).await;
    String::from_utf8(r.body).unwrap()
}

#[tokio::test]
async fn ac104_a_released_la_wall_links_to_las_page() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;
    let (id, host) = created(&rig.create_for(Some(&session), "la", "q3").await.1);
    walk(&rig, &id, &host).await;
    let (status, wall) = rig.call(Method::GET, &format!("/rooms/{id}/wall"), None, None).await;
    assert_eq!(status, StatusCode::OK);
    let link = wall["released"]["link"].as_str().expect("a released wall links to take-it-home");
    let home = room::rooms::Urls::default().home;
    assert_eq!(link, format!("{home}/la"), "the default club's link is {home} itself");
}

#[tokio::test]
async fn ac105_never_twice_is_per_club_and_the_ledger_names_the_club() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;

    let (id, host) = created(&rig.create_for(Some(&session), "nyc", "q3").await.1);
    walk(&rig, &id, &host).await;

    // NYC has run q3: it may not again ...
    let (status, body) = rig.create_for(Some(&session), "nyc", "q3").await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(body["reason"].as_str().unwrap().contains("already been run"), "{body}");
    // ... and LA, which has not, may, once.
    let (id, host) = created(&rig.create_for(Some(&session), "la", "q3").await.1);
    walk(&rig, &id, &host).await;
    let (status, body) = rig.create_for(Some(&session), "la", "q3").await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(body["reason"].as_str().unwrap().contains("already been run"), "{body}");

    let ledger: Vec<(String, String)> = rig
        .state
        .used()
        .all()
        .into_iter()
        .map(|e| (e.club.to_string(), e.question_id))
        .collect();
    assert_eq!(ledger, [("nyc".into(), "q3".into()), ("la".into(), "q3".into())]);
}

#[tokio::test]
async fn ac105_run_it_again_stays_in_the_old_rooms_club() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;
    let (id, host) = created(&rig.create_for(Some(&session), "la", "q3").await.1);
    walk(&rig, &id, &host).await;
    let (status, body) = rig
        .call(
            Method::POST,
            &format!("/rooms/{id}/run-it-again"),
            Some(&session),
            Some(serde_json::json!({ "question_id": "q3-again", "club": "nyc" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (id2, host2) = created(&body);
    walk(&rig, &id2, &host2).await;
    let ledger: Vec<(String, String)> = rig
        .state
        .used()
        .all()
        .into_iter()
        .map(|e| (e.club.to_string(), e.question_id))
        .collect();
    assert_eq!(ledger, [("la".into(), "q3".into()), ("la".into(), "q3-again".into())], "the club named on a re-run is ignored");
}

// --------------------------------------------------------------------------
// config
// --------------------------------------------------------------------------

fn config_with(clubs: Option<&str>) -> Result<Config, ConfigError> {
    let (id, secret, guild, role) = (snowflake(), canary("CANARY-CLIENT-SECRET"), snowflake(), snowflake());
    let clubs = clubs.map(str::to_string);
    Config::from_vars(move |name| match name {
        "DISCORD_CLIENT_ID" => Some(id.clone()),
        "DISCORD_CLIENT_SECRET" => Some(secret.clone()),
        "DISCORD_GUILD_ID" => Some(guild.clone()),
        "DISCORD_ROLE_ID" => Some(role.clone()),
        "POPQUIZ_CLUBS" => clubs.clone(),
        n if n.starts_with("POPQUIZ_ADMIN") => Some("a-test-value".into()),
        _ => None,
    })
}

#[test]
fn config_popquiz_clubs_is_optional_and_adds_clubs() {
    assert!(config_with(None).is_ok(), "the server as it was before D-26");
    assert!(config_with(Some("la=222:America/Los_Angeles")).is_ok());
    assert!(config_with(Some("la=222:America/Los_Angeles, chi=333:America/Chicago")).is_ok());
}

#[test]
fn config_a_bad_club_stops_the_room_and_never_echoes_a_role() {
    let err = config_with(Some("la=2x2:America/Los_Angeles")).err().unwrap();
    assert!(matches!(err, ConfigError::Clubs(_)), "{err}");
    assert!(err.to_string().contains("POPQUIZ_CLUBS"), "{err}");
    assert!(!err.to_string().contains("2x2"), "{err}");
    assert!(config_with(Some("nyc=222:America/New_York")).is_err(), "nyc is taken by DISCORD_ROLE_ID");
    assert!(config_with(Some("la=222:Mars/Olympus")).is_err());
}

// --------------------------------------------------------------------------
// Review of PR #52, P1: a scheduled arrangement belongs to one club.
// --------------------------------------------------------------------------

/// `q3` with its four wrong options rotated: the same question, arranged for a
/// different meetup date (the pipeline's `arrange` moves the answer slot and
/// reshuffles the others).
fn q3_arranged(rotate: usize) -> room::answers::Scheduled {
    let mut v = common::q3_json();
    let options = v["options"].as_array_mut().unwrap();
    options.rotate_left(rotate);
    common::load(&v)
}

fn options_of(rig: &Rig, room: &str) -> Vec<String> {
    rig.state.with_room(room, |r| r.public().question.options().to_vec()).unwrap()
}

#[tokio::test]
async fn p1_each_club_keeps_its_own_arrangement_of_one_question() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;
    let nyc = room::club::ClubSlug::parse("nyc").unwrap();
    let la = room::club::ClubSlug::parse("la").unwrap();
    // NYC is scheduled for one date, then LA for another, before either room.
    rig.state.schedule_for(&nyc, q3_arranged(1)).unwrap();
    rig.state.schedule_for(&la, q3_arranged(2)).unwrap();

    let (a, b) = (
        rig.create_for(Some(&session), "nyc", "q3").await,
        rig.create_for(Some(&session), "la", "q3").await,
    );
    assert_eq!((a.0, b.0), (StatusCode::CREATED, StatusCode::CREATED), "{} {}", a.1, b.1);
    let (nyc_room, la_room) = (options_of(&rig, a.1["id"].as_str().unwrap()), options_of(&rig, b.1["id"].as_str().unwrap()));
    assert_eq!(nyc_room, q3_arranged(1).public().options().to_vec(), "NYC's room has NYC's order");
    assert_eq!(la_room, q3_arranged(2).public().options().to_vec(), "LA's room has LA's order");
    assert_ne!(nyc_room, la_room);
}

#[tokio::test]
async fn p1_a_club_with_nothing_scheduled_gets_no_room_even_if_another_has_it() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;
    let nyc = room::club::ClubSlug::parse("nyc").unwrap();
    // The rig schedules its fixtures for both clubs; this id is NYC's alone.
    let mut v = common::q3_json();
    v["id"] = "q3-nyc-only".into();
    rig.state.schedule_for(&nyc, common::load(&v)).unwrap();
    assert_eq!(rig.create_for(Some(&session), "nyc", "q3-nyc-only").await.0, StatusCode::CREATED);
    let (status, body) = rig.create_for(Some(&session), "la", "q3-nyc-only").await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(body["reason"].as_str().unwrap().contains("No question is scheduled"), "{body}");
}

#[tokio::test]
async fn p1_a_running_room_blocks_only_its_own_clubs_repush() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(both(&rig)).await;
    let (nyc, la) = (room::club::ClubSlug::parse("nyc").unwrap(), room::club::ClubSlug::parse("la").unwrap());
    rig.state.schedule_for(&la, q3_arranged(2)).unwrap();
    assert_eq!(rig.create_for(Some(&session), "la", "q3").await.0, StatusCode::CREATED);
    // LA's room holds LA's record, so LA cannot replace it ...
    assert!(rig.state.schedule_for(&la, q3_arranged(3)).is_err());
    // ... and that says nothing about NYC's.
    assert!(rig.state.schedule_for(&nyc, q3_arranged(1)).is_ok());
    assert!(rig.state.schedule_for(&nyc, q3_arranged(4)).is_ok(), "NYC may still re-push its own");
}
