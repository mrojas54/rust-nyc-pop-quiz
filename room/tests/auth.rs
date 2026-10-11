//! Who may create a room: Discord (SPEC §8, AC-64…AC-70, G-9, T-10).
//!
//! Everything here runs against `common::discord_mock` — an in-process Discord
//! with the token, user and member routes, refresh-token rotation and replay
//! detection, and switchable `429`/`5xx`/down — and a room on the Discord
//! backend (`Rig`). Replaces `tests/standin.rs`: SPEC §8.2's stand-in is
//! deleted, and the first block below proves it stays deleted.
//!
//! | Criterion | Tests |
//! |---|---|
//! | AC-64 | `ac64_*`: the role hosts, no role is denied, no stand-in name survives, no other bearer creates |
//! | AC-65 | `ac65_*`: Administrator and owner without the ID denied; the ID compared whole; `roles` is all that is read |
//! | AC-66 | `ac66_*`: rotation persisted and never replayed, one refresh for two creates, a spent token signs out and nothing else |
//! | AC-67 | `ac67_*`: no participant route reaches the auth module; participants need and carry no credential |
//! | AC-68 | `ac68_*`: organizer B cannot read, control or run again A's room; an organizer session is not a host session |
//! | AC-69 | `ac69_*`: Discord down is a server error, never a denial; host actions never call Discord (`test-full`: `tests/lifecycle.rs`) |
//! | AC-70 | `ac70_*`: *wrong server* is the same for a non-member and another guild; *wrong role*; nothing else said |
//! | §8 retries | `retry_*` |
//! | sign-in | `sign_in_*` |
//! | secrecy | `secrecy_*` |
//! | config | `config_*` |

mod common;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use axum::http::{Method, StatusCode};
use common::discord_mock::{canary, query, snowflake, Membership, Rig};
use futures_util::{SinkExt, StreamExt};
use room::config::{Config, ConfigError, DISCORD_VARS};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

fn repo() -> PathBuf {
    common::repo()
}

fn fresh_hex() -> String {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

const FULL_WALK: [&str; 6] = ["put-on-screen", "close-answers", "show-split", "walk-it", "reveal", "release"];

async fn walk(rig: &Rig, room: &str, host: &str, slugs: &[&str]) {
    for slug in slugs {
        let (status, body) = rig.call(Method::POST, &format!("/rooms/{room}/{slug}"), Some(host), None).await;
        assert_eq!(status, StatusCode::OK, "{slug}: {body}");
    }
}

fn created(body: &Value) -> (String, String) {
    (body["id"].as_str().unwrap().to_string(), body["host_session"].as_str().unwrap().to_string())
}

// --------------------------------------------------------------------------
// AC-64: the role hosts; the stand-in is gone.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac64_a_member_with_the_role_creates_a_room() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    for key in ["id", "code", "join_url", "host_session", "host_resume_url"] {
        assert!(body[key].is_string(), "{key} in {body}");
    }
    assert_eq!(rig.mock.hits("member"), 1, "one member lookup per create");
}

#[tokio::test]
async fn ac64_a_member_without_the_role_is_denied() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(Membership::Member { roles: vec![snowflake()], admin: false }).await;
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["refusal"], "wrong_role");
    assert_eq!(rig.state.room_count(), 0);
}

/// Every file the scan reads: the room's source, its manifest, the image and CI.
fn shipped_files() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![repo().join("room/src"), repo().join(".github/workflows")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out.push(repo().join("room/Cargo.toml"));
    out.push(repo().join("Dockerfile"));
    out.into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            (p.strip_prefix(repo()).unwrap().to_string_lossy().into_owned(), text)
        })
        .collect()
}

#[test]
fn ac64_no_stand_in_survives_in_src_cargo_dockerfile_or_ci() {
    // SPEC §8.2: T-10 deletes the feature and its code. The names are
    // assembled here so this file does not spell them either.
    let names = [
        ["HOST", "DEV", "TOKEN"].join("_"),
        ["dev", "host", "token"].join("-"),
        ["hc0", "question"].join("_"),
        ["Dev", "Host", "Token"].concat(),
    ];
    let files = shipped_files();
    assert!(files.len() >= 20, "the scan found only {} files", files.len());
    assert!(files.iter().any(|(p, _)| p == "Dockerfile") && files.iter().any(|(p, _)| p.ends_with("ci.yml")));
    let mut found = Vec::new();
    for (path, text) in &files {
        for (n, line) in text.lines().enumerate() {
            for name in &names {
                if line.contains(name.as_str()) {
                    found.push(format!("{path}:{}: {}", n + 1, line.trim()));
                }
            }
        }
    }
    assert!(found.is_empty(), "the stand-in survives:\n{}", found.join("\n"));
    assert!(!repo().join("room/src/standin.rs").exists());
}

#[tokio::test]
async fn ac64_any_bearer_that_is_not_an_organizer_session_is_401() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    let (_, body) = rig.create(Some(&session), "q3").await;
    let (_, host_session) = created(&body);
    let issued = rig.mock.issued();
    let hex = fresh_hex();
    let prefix = &session[..session.len() - 1];
    let longer = format!("{session}0");
    let upper = session.to_uppercase();
    let mut bearers: Vec<Option<&str>> = vec![None, Some(""), Some("wrong"), Some(&hex), Some(prefix), Some(&longer), Some(&upper), Some(&host_session)];
    bearers.extend(issued.iter().map(|t| Some(t.as_str())));
    let before = rig.mock.total_hits();
    for bearer in bearers {
        let (status, body) = rig.create(bearer, "q3-again").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "a bearer that names no organizer");
        assert_eq!(body, Value::Null, "a refusal carries no body");
    }
    assert_eq!(rig.mock.total_hits(), before, "nothing reaches Discord without an organizer");
    // And the default router, which has no backend that accepts anything.
    let app = room::router();
    let (status, body) = common::http(&app, Method::POST, "/rooms", Some(&session), Some(json!({ "question_id": "q3" }))).await;
    assert_eq!((status, body), (StatusCode::UNAUTHORIZED, Value::Null));
}

// --------------------------------------------------------------------------
// AC-65: the role ID, in `roles`, and nothing else.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac65_an_administrator_and_owner_without_the_role_id_is_denied() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(Membership::Member { roles: vec![snowflake(), snowflake()], admin: true }).await;
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["refusal"], "wrong_role", "Administrator and ownership grant nothing");
}

#[tokio::test]
async fn ac65_the_role_id_is_compared_whole() {
    let rig = Rig::start().await;
    let role = rig.role();
    let near = [format!("{role}0"), format!("0{role}"), role[..role.len() - 1].to_string(), format!(" {role}")];
    for r in near {
        let (session, _) = rig.sign_in(Membership::Member { roles: vec![r.clone()], admin: false }).await;
        let (status, body) = rig.create(Some(&session), "q3").await;
        assert_eq!((status, body["refusal"].as_str()), (StatusCode::FORBIDDEN, Some("wrong_role")), "{r:?}");
    }
}

#[test]
fn ac65_the_member_record_is_read_for_roles_alone() {
    let src = std::fs::read_to_string(repo().join("room/src/discord.rs")).unwrap();
    let code: Vec<&str> = src.lines().filter(|l| !l.trim_start().starts_with("//")).collect();
    let start = code.iter().position(|l| l.contains("struct Member {")).expect("the member struct");
    let end = start + code[start..].iter().position(|l| l.trim() == "}").unwrap();
    let fields: Vec<&str> = code[start + 1..end].iter().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    assert_eq!(fields, ["roles: Vec<String>,"], "the member record keeps roles and nothing else");
    for word in ["permissions", "is_owner", "owner_id", "\"name\""] {
        assert!(!code.iter().any(|l| l.contains(word)), "discord.rs code reads {word}");
    }
}

// --------------------------------------------------------------------------
// AC-66: rotation persisted; a stale token never silently locks anyone out.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac66_each_refresh_persists_the_rotated_token_and_presents_it_next() {
    let rig = Rig::start().await;
    rig.mock.expires_in(3600);
    let (session, user) = rig.sign_in(rig.host()).await;
    let first = rig.mock.live_refresh(&user).unwrap();
    rig.clock.advance(Duration::from_secs(2 * 3600));
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let second = rig.mock.live_refresh(&user).unwrap();
    assert_ne!(first, second, "Discord rotated it");
    rig.clock.advance(Duration::from_secs(2 * 3600));
    let (status, body) = rig.create(Some(&session), "q3-again").await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    // The second refresh presented the rotated token, so it was persisted;
    // the mock kills an old access token at rotation, so the member lookups
    // succeeding means the rotated access token was used too.
    assert_eq!(rig.mock.presented(), vec![first, second]);
    assert_eq!(rig.mock.replays(), 0);
}

#[tokio::test]
async fn ac66_two_creates_at_once_refresh_once() {
    let rig = Rig::start().await;
    rig.mock.expires_in(3600);
    let (session, _) = rig.sign_in(rig.host()).await;
    rig.clock.advance(Duration::from_secs(2 * 3600));
    let (a, b) = tokio::join!(rig.create(Some(&session), "q3"), rig.create(Some(&session), "q3-again"));
    assert_eq!((a.0, b.0), (StatusCode::CREATED, StatusCode::CREATED), "{} {}", a.1, b.1);
    assert_eq!(rig.mock.presented().len(), 1, "one refresh, not two");
    assert_eq!(rig.mock.replays(), 0, "no spent token was ever presented");
}

#[tokio::test]
async fn ac66_a_refused_refresh_signs_the_organizer_out_and_leaves_their_open_room() {
    let rig = Rig::start().await;
    rig.mock.expires_in(120);
    let (session, user) = rig.sign_in(rig.host()).await;
    let (_, body) = rig.create(Some(&session), "q3").await;
    let (room, host) = created(&body);
    // The refresh token is spent elsewhere; Discord will refuse it. Five
    // minutes on, the access token is due and the room is still open.
    rig.mock.revoke_refresh(&user);
    rig.clock.advance(Duration::from_secs(5 * 60));
    let (status, body) = rig.create(Some(&session), "q3-again").await;
    assert_eq!((status, body), (StatusCode::UNAUTHORIZED, Value::Null), "detected: sign in again");
    assert_eq!(rig.mock.replays(), 1, "the spent token reached Discord once, and was refused");
    assert_eq!(rig.discord.organizer_count(), 0, "signed out, not stuck");
    // The open room never asks Discord: it runs to release.
    walk(&rig, &room, &host, &FULL_WALK).await;
    // And signing in again works.
    let (again, _) = rig.sign_in(rig.host()).await;
    assert_eq!(rig.create(Some(&again), "q3-again").await.0, StatusCode::CREATED);
}

#[tokio::test]
async fn ac66_an_access_token_refused_early_is_refreshed_once() {
    let rig = Rig::start().await;
    let (session, user) = rig.sign_in(rig.host()).await;
    rig.mock.revoke_access(&user);
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(rig.mock.presented().len(), 1);
    assert_eq!(rig.mock.hits("member"), 2, "refused, refreshed, asked again");
}

// --------------------------------------------------------------------------
// AC-67: participants never authenticate.
// --------------------------------------------------------------------------

fn block<'a>(src: &'a str, open: &str, close: &str) -> &'a str {
    let start = src.find(open).unwrap_or_else(|| panic!("{open} in routes.rs"));
    let end = start + src[start..].find(close).unwrap_or_else(|| panic!("{close} in routes.rs"));
    &src[start..end]
}

#[test]
fn ac67_no_participant_route_touches_the_auth_module() {
    let routes = std::fs::read_to_string(repo().join("room/src/routes.rs")).unwrap();
    let mut participant = vec![
        ("routes.rs T-04b", block(&routes, "// T-04b routes", "// end T-04b routes").to_string()),
        ("routes.rs T-06", block(&routes, "// T-06 pages", "// end T-06 pages").to_string()),
    ];
    for f in ["room/src/sessions.rs", "web/buzzer/buzzer.js", "web/buzzer/index.html"] {
        participant.push((f, std::fs::read_to_string(repo().join(f)).unwrap()));
    }
    let touches = ["auth::", "discord", "HostAuth", "authorize_create", "authorize_host", "with_hosted_room", "/auth/", "organizer"];
    for (name, text) in &participant {
        assert!(text.len() > 200, "{name} is too short to be the real thing");
        for t in touches {
            assert!(!text.to_lowercase().contains(&t.to_lowercase()), "{name} mentions {t}");
        }
    }
    // The positive control: the host half does reach it.
    assert!(routes.contains("authorize") || routes.contains("create_room_checked"));
}

#[tokio::test]
async fn ac67_participants_need_and_carry_no_credential() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    let (_, body) = rig.create(Some(&session), "q3").await;
    let (room, host) = created(&body);
    let code = body["code"].as_str().unwrap();
    // A join carries nothing but the code.
    let joined = rig.raw(Method::POST, "/join", &[], Some(json!({ "code": code }))).await;
    assert_eq!(joined.status, StatusCode::CREATED);
    let token = joined.json()["token"].as_str().unwrap().to_string();
    walk(&rig, &room, &host, &["put-on-screen"]).await;
    // The answer carries the participant's own session and nothing else.
    let (status, _) = rig.call(Method::PUT, &format!("/rooms/{room}/answer"), Some(&token), Some(json!({ "letter": "B" }))).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = rig.call(Method::GET, &format!("/rooms/{room}/buzzer"), None, None).await;
    assert_eq!(status, StatusCode::OK);
    // Nothing a participant is handed is an organizer's credential.
    for text in rig.seen.lock().unwrap().iter() {
        if text.contains(&token) {
            assert!(!text.contains(&session) && !text.contains(&host), "a participant response carried an organizer credential");
        }
    }
    // Neither credential is the participant's: the session token opens no host route.
    let (status, _) = rig.call(Method::GET, &format!("/rooms/{room}/host"), Some(&token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = rig.create(Some(&token), "q3-again").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// --------------------------------------------------------------------------
// AC-68: only the creating organizer.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac68_organizer_b_cannot_read_or_control_as_room() {
    let rig = Rig::start().await;
    let (a, _) = rig.sign_in(rig.host()).await;
    let (b, _) = rig.sign_in(rig.host()).await;
    let (_, body) = rig.create(Some(&a), "q3").await;
    let (room, a_host) = created(&body);
    let (_, body_b) = rig.create(Some(&b), "q3-again").await;
    let (_, b_host) = created(&body_b);
    for bearer in [&b, &b_host] {
        let (status, body) = rig.call(Method::GET, &format!("/rooms/{room}/host"), Some(bearer), None).await;
        assert_eq!((status, body), (StatusCode::UNAUTHORIZED, Value::Null));
        for slug in FULL_WALK {
            let (status, _) = rig.call(Method::POST, &format!("/rooms/{room}/{slug}"), Some(bearer), None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{slug}");
        }
    }
    assert!(rig.state.with_hosted_room(&room, Some(&b), |_| ()).is_err());
    // A's room is untouched, and A still runs it.
    walk(&rig, &room, &a_host, &FULL_WALK).await;
}

#[tokio::test]
async fn ac68_an_organizer_session_is_not_a_host_session() {
    let rig = Rig::start().await;
    let (a, _) = rig.sign_in(rig.host()).await;
    let (_, body) = rig.create(Some(&a), "q3").await;
    let (room, _) = created(&body);
    let (status, _) = rig.call(Method::GET, &format!("/rooms/{room}/host"), Some(&a), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "the organizer session creates rooms; it does not host one");
    let (status, _) = rig.call(Method::POST, &format!("/rooms/{room}/put-on-screen"), Some(&a), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ac68_organizer_b_cannot_run_it_again_on_as_room() {
    let rig = Rig::start().await;
    let (a, _) = rig.sign_in(rig.host()).await;
    let (b, _) = rig.sign_in(rig.host()).await;
    let (_, body) = rig.create(Some(&a), "q3").await;
    let (room, host) = created(&body);
    walk(&rig, &room, &host, &FULL_WALK).await;
    let again = format!("/rooms/{room}/run-it-again");
    let (status, body) = rig.call(Method::POST, &again, Some(&b), Some(json!({ "question_id": "q3-again" }))).await;
    assert_eq!((status, body), (StatusCode::UNAUTHORIZED, Value::Null));
    let (status, body) = rig.call(Method::POST, &again, Some(&a), Some(json!({ "question_id": "q3-again" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

// --------------------------------------------------------------------------
// AC-69: checked at the door, never after.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac69_create_with_discord_down_is_a_plain_server_error() {
    let mut rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    rig.mock.down().await;
    let started = Instant::now();
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!((status, body), (StatusCode::SERVICE_UNAVAILABLE, Value::Null), "an outage is not a denial");
    assert!(started.elapsed() < Duration::from_secs(4), "bounded by the deadline: {:?}", started.elapsed());
    assert_eq!(rig.discord.organizer_count(), 1, "an outage signs nobody out");
}

#[tokio::test]
async fn ac69_host_actions_never_call_discord() {
    let mut rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    let (_, body) = rig.create(Some(&session), "q3").await;
    let (room, host) = created(&body);
    let before = rig.mock.total_hits();
    walk(&rig, &room, &host, &FULL_WALK[..5]).await;
    assert_eq!(rig.mock.total_hits(), before, "no host action asked Discord");
    rig.mock.down().await;
    walk(&rig, &room, &host, &FULL_WALK[5..]).await;
    let (status, _) = rig.call(Method::GET, &format!("/rooms/{room}/host"), Some(&host), None).await;
    assert_eq!(status, StatusCode::OK);
}

// --------------------------------------------------------------------------
// AC-70: which condition failed, and nothing about membership.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac70_a_non_member_and_another_guilds_member_get_the_same_wrong_server() {
    let rig = Rig::start().await;
    let (outsider, _) = rig.sign_in(Membership::Absent).await;
    let a = rig.raw(Method::POST, "/rooms", &[("authorization", &format!("Bearer {outsider}"))], Some(json!({ "question_id": "q3" }))).await;
    // A room configured for a guild this member is not in.
    let elsewhere = Rig::start_elsewhere().await;
    let (member, _) = elsewhere.sign_in(elsewhere.host()).await;
    let b = elsewhere.raw(Method::POST, "/rooms", &[("authorization", &format!("Bearer {member}"))], Some(json!({ "question_id": "q3" }))).await;
    assert_eq!(a.status, StatusCode::FORBIDDEN);
    assert_eq!((a.status, &a.body), (b.status, &b.body), "byte for byte the same answer");
    let body = a.json();
    assert_eq!(body["refusal"], "wrong_server");
    assert_eq!(body["reason"], room::copy::HOST_DENIED_WRONG_SERVER);
    let mut keys: Vec<&String> = body.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, ["reason", "refusal"], "nothing else is said");
}

#[tokio::test]
async fn ac70_a_member_without_the_role_hears_wrong_role() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(Membership::Member { roles: vec![], admin: false }).await;
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body, json!({ "refusal": "wrong_role", "reason": room::copy::HOST_DENIED_WRONG_ROLE }));
}

// --------------------------------------------------------------------------
// SPEC §8: retries bounded and backed off.
// --------------------------------------------------------------------------

#[tokio::test]
async fn retry_after_is_honoured_then_the_call_succeeds() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    rig.mock.fault(429, 2, Some(0.15));
    let started = Instant::now();
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(rig.mock.hits("member"), 3);
    assert!(started.elapsed() >= Duration::from_millis(300), "waited Retry-After twice: {:?}", started.elapsed());
}

#[tokio::test]
async fn retry_attempts_are_bounded_and_end_in_a_server_error() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    rig.mock.fault(429, 10, Some(0.01));
    let (status, body) = rig.create(Some(&session), "q3").await;
    assert_eq!((status, body), (StatusCode::SERVICE_UNAVAILABLE, Value::Null));
    assert_eq!(rig.mock.hits("member"), 3, "three tries, no more");
}

#[tokio::test]
async fn retry_server_errors_back_off_then_fail_plain() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    rig.mock.fault(502, 10, None);
    let started = Instant::now();
    let (status, _) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(rig.mock.hits("member"), 3);
    // 20 ms then 40 ms of backoff (fast_retry()).
    assert!(started.elapsed() >= Duration::from_millis(60), "{:?}", started.elapsed());
}

#[tokio::test]
async fn retry_a_retry_after_past_the_deadline_is_not_waited_out() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    rig.mock.fault(429, 10, Some(60.0));
    let started = Instant::now();
    let (status, _) = rig.create(Some(&session), "q3").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(rig.mock.hits("member"), 1);
    assert!(started.elapsed() < Duration::from_secs(1), "{:?}", started.elapsed());
}

#[tokio::test]
async fn retry_a_denial_is_never_retried() {
    let rig = Rig::start().await;
    let (outsider, _) = rig.sign_in(Membership::Absent).await;
    let (status, _) = rig.create(Some(&outsider), "q3").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(rig.mock.hits("member"), 1, "a 404 is an answer");
    assert!(
        rig.log.0.lock().unwrap().iter().any(|l| l == "discord: member 404 (try 1)"),
        "{:?}",
        rig.log.0.lock().unwrap()
    );
}

// --------------------------------------------------------------------------
// Sign-in: the redirect, the callback, the state.
// --------------------------------------------------------------------------

#[tokio::test]
async fn sign_in_redirects_with_both_scopes_and_the_public_redirect_uri() {
    let rig = Rig::start().await;
    let (r, state, cookie) = rig.begin("q3").await;
    let location = r.header("location").unwrap();
    assert!(location.starts_with(&format!("{}/oauth2/authorize?", rig.mock.base)), "{location}");
    assert_eq!(query(location, "response_type").as_deref(), Some("code"));
    assert_eq!(query(location, "scope").as_deref(), Some("identify%20guilds.members.read"));
    assert_eq!(query(location, "client_id").as_deref(), Some(rig.mock.state.client_id.as_str()));
    assert_eq!(
        query(location, "redirect_uri").as_deref(),
        Some("http%3A%2F%2F127.0.0.1%3A3000%2Fauth%2Fdiscord%2Fcallback")
    );
    assert_eq!(state.len(), 64);
    assert_eq!(cookie, format!("pq_oauth={state}"));
    // routes.rs registers the callback by literal (the canary reads literals);
    // it must be the path the redirect URI names.
    let routes = std::fs::read_to_string(repo().join("room/src/routes.rs")).unwrap();
    assert!(routes.contains(&format!(".route(\"{}\"", room::discord::CALLBACK_PATH)));
    let set_cookie = r.header("set-cookie").unwrap();
    for attr in ["HttpOnly", "SameSite=Lax", "Path=/auth/discord", "Max-Age=600"] {
        assert!(set_cookie.contains(attr), "{set_cookie}");
    }
    assert_eq!(r.header("cache-control"), Some("no-store"));
    for bad in ["", "q3%23x", "..%2Fq", "q3%26state%3D1"] {
        let r = rig.raw(Method::GET, &format!("/auth/discord?question={bad}"), &[], None).await;
        assert_eq!(r.status, StatusCode::BAD_REQUEST, "{bad}");
        assert!(r.body.is_empty());
    }
}

#[tokio::test]
async fn sign_in_lands_on_the_host_page_with_a_session_and_nothing_else() {
    let rig = Rig::start().await;
    let (code, _) = rig.mock.account(rig.host());
    let (_, state, cookie) = rig.begin("q3-again").await;
    let r = rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await;
    assert_eq!(r.status, StatusCode::SEE_OTHER);
    let location = r.header("location").unwrap();
    let (path, session) = location.split_once('#').unwrap();
    assert_eq!(path, "/host?question=q3-again");
    assert_eq!(session.len(), 64);
    assert!(session.bytes().all(|b| b.is_ascii_hexdigit()));
    for issued in rig.mock.issued() {
        assert!(!location.contains(&issued), "a Discord token in the landing URL");
    }
    assert!(r.header("set-cookie").unwrap().contains("Max-Age=0"), "the state cookie is cleared");
    assert_eq!(r.header("cache-control"), Some("no-store"));
    assert_eq!(r.header("referrer-policy"), Some("no-referrer"));
    assert!(r.body.is_empty());
}

#[tokio::test]
async fn sign_in_a_wrong_replayed_or_cookieless_state_is_refused_with_nothing_said() {
    let rig = Rig::start().await;
    let refused = |r: &common::discord_mock::Raw| r.status == StatusCode::BAD_REQUEST && r.body.is_empty() && r.header("location").is_none();
    let (code, _) = rig.mock.account(rig.host());
    // A state nobody issued.
    let r = rig.callback(&format!("code={code}&state={}", fresh_hex()), Some(&format!("pq_oauth={}", fresh_hex()))).await;
    assert!(refused(&r));
    // No state at all.
    assert!(refused(&rig.callback(&format!("code={code}"), None).await));
    // A real state without its cookie — and then it is spent.
    let (_, state, cookie) = rig.begin("q3").await;
    assert!(refused(&rig.callback(&format!("code={code}&state={state}"), None).await));
    assert!(refused(&rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await), "spent");
    // Another sign-in's cookie.
    let (_, state_a, _) = rig.begin("q3").await;
    let (_, _, cookie_b) = rig.begin("q3").await;
    assert!(refused(&rig.callback(&format!("code={code}&state={state_a}"), Some(&cookie_b)).await));
    // An expired one.
    let (_, state, cookie) = rig.begin("q3").await;
    rig.clock.advance(Duration::from_secs(11 * 60));
    assert!(refused(&rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await));
    assert_eq!(rig.mock.hits("token"), 0, "no refused callback reached Discord");
    // A replay of a completed sign-in.
    let (_, state, cookie) = rig.begin("q3").await;
    let ok = rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await;
    assert_eq!(ok.status, StatusCode::SEE_OTHER);
    assert!(refused(&rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await));
}

#[tokio::test]
async fn sign_in_declined_at_discord_goes_back_to_the_sign_in_screen() {
    let rig = Rig::start().await;
    let (_, state, cookie) = rig.begin("q3").await;
    let r = rig.callback(&format!("error=access_denied&state={state}"), Some(&cookie)).await;
    assert_eq!(r.status, StatusCode::SEE_OTHER);
    assert_eq!(r.header("location"), Some("/host?question=q3"));
    assert_eq!(rig.discord.organizer_count(), 0);
}

#[tokio::test]
async fn sign_in_with_discord_failing_is_a_server_error() {
    let rig = Rig::start().await;
    let (code, _) = rig.mock.account(rig.host());
    let (_, state, cookie) = rig.begin("q3").await;
    rig.mock.fault(500, 10, None);
    let r = rig.callback(&format!("code={code}&state={state}"), Some(&cookie)).await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(r.body.is_empty());
}

#[tokio::test]
async fn sign_in_is_not_served_without_the_discord_backend() {
    let app = room::router();
    for path in ["/auth/discord?question=q3", "/auth/discord/callback?code=x&state=y"] {
        let (status, _) = common::http(&app, Method::GET, path, None, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
}

// --------------------------------------------------------------------------
// Secrecy: no Discord value or token anywhere a person or a log can read.
// --------------------------------------------------------------------------

/// Every plant found in `texts`.
fn findings(plants: &[(String, String)], texts: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (name, value) in plants {
        if texts.iter().any(|t| t.contains(value.as_str())) {
            out.push(name.clone());
        }
    }
    out
}

async fn frames(url: &str, attach: Option<&str>, out: &mut Vec<String>, want_phase: &str) {
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    if let Some(token) = attach {
        ws.send(Message::Text(json!({ "t": "attach", "token": token }).to_string().into())).await.unwrap();
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let next = tokio::time::timeout_at(deadline, ws.next()).await.expect("a frame");
        let Some(Ok(Message::Text(t))) = next else { panic!("socket closed: {next:?}") };
        let done = serde_json::from_str::<Value>(&t).is_ok_and(|v| v["phase"] == want_phase);
        out.push(t.to_string());
        if done {
            return;
        }
    }
}

#[tokio::test]
async fn secrecy_no_discord_value_or_token_in_any_payload_frame_page_or_log() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    let (outsider, _) = rig.sign_in(Membership::Absent).await;
    let (roleless, _) = rig.sign_in(Membership::Member { roles: vec![], admin: true }).await;
    let _ = rig.create(Some(&outsider), "q3").await;
    let _ = rig.create(Some(&roleless), "q3").await;
    let (_, body) = rig.create(Some(&session), "q3").await;
    let (room, host) = created(&body);
    let code = body["code"].as_str().unwrap().to_string();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(room::ws::serve(listener, rig.app.clone()));
    let joined = rig.raw(Method::POST, "/join", &[], Some(json!({ "code": code }))).await;
    let token = joined.json()["token"].as_str().unwrap().to_string();

    let mut texts = Vec::new();
    let phases = ["live", "closed", "split", "work", "reveal", "released"];
    for page in ["/host", "/host/host.js", "/join", "/join/buzzer.js", "/shared/copy.js"] {
        rig.raw(Method::GET, page, &[], None).await;
    }
    for (slug, phase) in FULL_WALK.iter().zip(phases) {
        walk(&rig, &room, &host, &[slug]).await;
        for viewer in ["wall", "buzzer"] {
            rig.call(Method::GET, &format!("/rooms/{room}/{viewer}"), None, None).await;
        }
        rig.call(Method::GET, &format!("/rooms/{room}/host"), Some(&host), None).await;
        if phase == "live" {
            rig.call(Method::PUT, &format!("/rooms/{room}/answer"), Some(&token), Some(json!({ "letter": "A" }))).await;
        }
        for (viewer, attach) in [("wall", None), ("host", Some(host.as_str())), ("buzzer", Some(token.as_str()))] {
            if viewer == "buzzer" && phase == "released" {
                continue; // release drops the sessions (§3.4)
            }
            frames(&format!("ws://{addr}/rooms/{room}/ws/{viewer}"), attach, &mut texts, phase).await;
        }
    }
    rig.raw(Method::GET, &format!("/wall/{room}"), &[], None).await;
    // Run it again, through the same check.
    rig.call(Method::POST, &format!("/rooms/{room}/run-it-again"), Some(&session), Some(json!({ "question_id": "q3-again" }))).await;

    let m = &rig.mock.state;
    let mut plants = vec![
        ("DISCORD_CLIENT_SECRET".to_string(), m.client_secret.clone()),
        ("DISCORD_GUILD_ID".to_string(), m.guild_id.clone()),
        ("DISCORD_ROLE_ID".to_string(), m.role_id.clone()),
    ];
    for (i, t) in rig.mock.issued().into_iter().enumerate() {
        plants.push((format!("issued token {i}"), t));
    }
    assert!(plants.len() >= 3 + 2 * 3, "tokens were issued");
    // The sign-in start's redirect to Discord is the one place the client id
    // is meant to go (OAuth2's consent URL carries it); everywhere else it is
    // a plant like the rest.
    let seen: Vec<String> = rig.seen.lock().unwrap().clone();
    let (to_discord, rest): (Vec<String>, Vec<String>) = seen.into_iter().partition(|t| t.contains("/oauth2/authorize?"));
    texts.extend(rest);
    let log = rig.log.0.lock().unwrap().clone();
    assert!(log.len() >= 8, "the backend logged its calls: {log:?}");
    texts.extend(log);
    plants.push(("DISCORD_CLIENT_ID".to_string(), m.client_id.clone()));
    let found = findings(&plants, &texts);
    assert!(found.is_empty(), "leaked: {found:?}");
    // The consent URL carries the client id and nothing secret.
    let secret_plants: Vec<(String, String)> = plants.iter().filter(|(n, _)| n != "DISCORD_CLIENT_ID").cloned().collect();
    assert!(findings(&secret_plants, &to_discord).is_empty());
    assert!(texts.len() > 60, "the scan saw {} texts", texts.len());
}

#[test]
fn secrecy_positive_control_the_scan_bites() {
    let plant = canary("CANARY-ACCESS");
    let plants = vec![("token".to_string(), plant.clone())];
    assert_eq!(findings(&plants, &[format!("{{\"deep\":[\"…{plant}…\"]}}")]), ["token"]);
    assert!(findings(&plants, &["nothing here".to_string()]).is_empty());
}

// --------------------------------------------------------------------------
// Config: the four variables.
// --------------------------------------------------------------------------

fn discord_env(secret: &str) -> Vec<(String, String)> {
    vec![
        ("DISCORD_CLIENT_ID".into(), snowflake()),
        ("DISCORD_CLIENT_SECRET".into(), secret.into()),
        ("DISCORD_GUILD_ID".into(), snowflake()),
        ("DISCORD_ROLE_ID".into(), snowflake()),
    ]
}

fn config(pairs: &[(String, String)]) -> Result<Config, ConfigError> {
    let pairs = pairs.to_vec();
    Config::from_vars(move |name| {
        pairs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            // PQ-17's admin token, whichever branch lands first.
            .or_else(|| name.starts_with("POPQUIZ_ADMIN").then(|| "a-test-value".to_string()))
    })
}

#[test]
fn config_requires_the_four_discord_variables() {
    let secret = canary("CANARY-CLIENT-SECRET");
    let full = discord_env(&secret);
    assert!(config(&full).is_ok());
    for var in DISCORD_VARS {
        let missing: Vec<_> = full.iter().filter(|(k, _)| k != var).cloned().collect();
        assert_eq!(config(&missing).err(), Some(ConfigError::MissingDiscord(var)), "{var} missing");
        for blank in ["", "  "] {
            let blanked: Vec<_> = full.iter().map(|(k, v)| (k.clone(), if k == var { blank.to_string() } else { v.clone() })).collect();
            assert_eq!(config(&blanked).err(), Some(ConfigError::MissingDiscord(var)), "{var} blank");
        }
    }
    for var in ["DISCORD_CLIENT_ID", "DISCORD_GUILD_ID", "DISCORD_ROLE_ID"] {
        let named: Vec<_> = full.iter().map(|(k, v)| (k.clone(), if k == var { "organizers".to_string() } else { v.clone() })).collect();
        assert_eq!(config(&named).err(), Some(ConfigError::DiscordId(var)), "a role name is not an id");
    }
}

#[test]
fn config_errors_never_echo_a_value_and_the_redirect_uri_follows_the_public_url() {
    let secret = canary("CANARY-CLIENT-SECRET");
    let mut env = discord_env(&secret);
    let bad_id = canary("CANARY-ID");
    env[2].1 = bad_id.clone();
    let err = config(&env).err().unwrap();
    for text in [err.to_string(), format!("{err:?}")] {
        assert!(text.contains("DISCORD_GUILD_ID"), "{text}");
        assert!(!text.contains(&bad_id) && !text.contains(&secret), "{text}");
    }
    let mut env = discord_env(&secret);
    env.push(("POPQUIZ_PUBLIC_URL".into(), "https://popquiz.rustnyc.org".into()));
    assert_eq!(config(&env).unwrap().redirect_uri(), "https://popquiz.rustnyc.org/auth/discord/callback");
    env.pop();
    env.push(("PORT".into(), "8080".into()));
    assert_eq!(config(&env).unwrap().redirect_uri(), "http://127.0.0.1:8080/auth/discord/callback");
}
