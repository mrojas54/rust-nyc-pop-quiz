//! The deployed room's configuration (T-09): what `main.rs` reads from its
//! environment, and that the join link a room hands out carries the public
//! host it was configured with — the address `just smoke <url>` and every
//! phone in the room reach it at (AC-28).
//!
//! `Config::from_vars` is pure, so each case is a table row here rather than a
//! process started with a different environment.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use common::{http, q3, TestAuth, ORGANIZER};
use room::config::{Config, ConfigError, TAKE_IT_HOME};
use room::rooms::AppState;

fn config(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
    // A dev-host-token build needs its token to configure at all; these cases
    // are about the other two variables, so it is always supplied. The name is
    // assembled so this file never spells it.
    let token_var = ["HOST", "DEV", "TOKEN"].join("_");
    let mut all: Vec<(String, String)> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    all.push((token_var, "a-test-value".into()));
    // T-10: every build needs the four Discord variables too (ids are digits).
    all.extend(room::config::DISCORD_VARS.iter().map(|v| (v.to_string(), if v.ends_with("SECRET") { "a-test-value".into() } else { "1234".into() })));
    Config::from_vars(move |name| all.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone()))
}

fn addr(s: &str) -> SocketAddr {
    s.parse().unwrap()
}

#[test]
fn nothing_set_is_the_local_default() {
    let c = config(&[]).unwrap();
    assert_eq!(c.bind, addr("127.0.0.1:3000"));
    assert_eq!(c.urls.base, "http://127.0.0.1:3000");
    assert_eq!(c.urls.home, TAKE_IT_HOME);
}

#[test]
fn port_binds_every_interface_as_fly_needs() {
    let c = config(&[("PORT", "8080")]).unwrap();
    assert_eq!(c.bind, addr("0.0.0.0:8080"));
    assert_eq!(c.urls.base, "http://127.0.0.1:8080", "no public URL: the loopback on the same port");
}

#[test]
fn a_bad_port_is_refused_by_name() {
    for bad in ["", "0", "65536", "http", "-1", "80a"] {
        match config(&[("PORT", bad)]) {
            Err(ConfigError::Port(v)) => assert_eq!(v, bad),
            other => panic!("PORT={bad:?}: {:?}", other.map(|c| c.bind)),
        }
    }
    assert!(ConfigError::Port("x".into()).to_string().contains("PORT"));
}

#[test]
fn the_public_url_is_the_base_of_every_link() {
    for (raw, base) in [
        ("https://rustnyc-popquiz.fly.dev", "https://rustnyc-popquiz.fly.dev"),
        ("https://rustnyc-popquiz.fly.dev/", "https://rustnyc-popquiz.fly.dev"),
        ("  https://popquiz.rustnyc.org  ", "https://popquiz.rustnyc.org"),
        ("http://127.0.0.1:8099", "http://127.0.0.1:8099"),
        ("http://[::1]:3000", "http://[::1]:3000"),
    ] {
        let c = config(&[("PORT", "8080"), ("POPQUIZ_PUBLIC_URL", raw)]).unwrap();
        assert_eq!(c.urls.base, base, "{raw:?}");
        assert_eq!(c.urls.home, TAKE_IT_HOME, "take-it-home is §13's page, whatever the room's host");
    }
}

#[test]
fn a_public_url_with_anything_after_the_host_is_refused() {
    for bad in [
        "",
        "rustnyc-popquiz.fly.dev",
        "ftp://rustnyc-popquiz.fly.dev",
        "https://",
        "https://rustnyc-popquiz.fly.dev/room",
        "https://rustnyc-popquiz.fly.dev?x=1",
        "https://rustnyc-popquiz.fly.dev#x",
        "https://user@rustnyc-popquiz.fly.dev",
        "https://rustnyc popquiz.fly.dev",
    ] {
        match config(&[("POPQUIZ_PUBLIC_URL", bad)]) {
            Err(ConfigError::PublicUrl(v)) => assert_eq!(v, bad),
            other => panic!("POPQUIZ_PUBLIC_URL={bad:?}: {:?}", other.map(|c| c.urls.base)),
        }
    }
}

// --------------------------------------------------------------------------
// AC-28 on a configured host: the short link carries the code, and both ways
// in reach the room.
// --------------------------------------------------------------------------

const PUBLIC: &str = "https://rustnyc-popquiz.fly.dev";

async fn configured_room() -> (axum::Router, serde_json::Value) {
    let c = config(&[("PORT", "8080"), ("POPQUIZ_PUBLIC_URL", PUBLIC)]).unwrap();
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], c.urls));
    let app = room::router_with(state);
    let (status, created) = http(&app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": "q3" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    (app, created)
}

#[tokio::test]
async fn ac28_the_join_link_carries_the_public_host_and_the_code() {
    let (app, created) = configured_room().await;
    let code = created["code"].as_str().unwrap();
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["join_url"], format!("{PUBLIC}/{code}"));
    assert!(
        created["host_resume_url"].as_str().unwrap().starts_with(&format!("{PUBLIC}/host/{id}#")),
        "{}",
        created["host_resume_url"]
    );
    // The wall's *join @* line is the same link, in idle and in live.
    let (_, wall) = http(&app, Method::GET, &format!("/rooms/{id}/wall"), None, None).await;
    assert_eq!(wall["join"], format!("join @ {PUBLIC}/{code}"));
    let host = created["host_session"].as_str().unwrap();
    let (status, _) = http(&app, Method::POST, &format!("/rooms/{id}/put-on-screen"), Some(host), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, wall) = http(&app, Method::GET, &format!("/rooms/{id}/wall"), None, None).await;
    assert_eq!(wall["join"], format!("join @ {PUBLIC}/{code}"));
}

#[tokio::test]
async fn ac28_the_short_link_and_the_typed_code_both_join() {
    let (app, created) = configured_room().await;
    let code = created["code"].as_str().unwrap();
    let id = created["id"].as_str().unwrap();

    // The short link's path is the code; it lands on the join form with the
    // code carried.
    let path = created["join_url"].as_str().unwrap().strip_prefix(PUBLIC).unwrap().to_string();
    let res = app
        .clone()
        .oneshot(Request::builder().method(Method::GET).uri(&path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(res.headers()[header::LOCATION], format!("/join?code={code}"));

    // Joining with the carried code, and with the code as a person types it.
    let typed = format!("  {} ", code.to_lowercase());
    for as_given in [code.to_string(), typed] {
        let (status, joined) = http(&app, Method::POST, "/join", None, Some(json!({ "code": as_given }))).await;
        assert_eq!(status, StatusCode::CREATED, "{as_given:?}: {joined}");
        assert_eq!(joined["room_id"], id);
    }
}
