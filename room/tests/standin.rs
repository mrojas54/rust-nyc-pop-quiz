//! AC-64's stand-in half (SPEC §8.2, D-19, T-09).
//!
//! `HOST_DEV_TOKEN` authorizes *Create a room* only in a build with the
//! `dev-host-token` feature, and a build without it has no path that reads or
//! compares the token, so it cannot ship enabled. Three parts:
//!
//! - **Every build:** the structural scan. In the server's source — `src/`
//!   less `src/bin/`, whose client tools each sit behind their own feature —
//!   the variable's name appears only in `standin.rs`, that file is gated
//!   whole, and every other line naming the stand-in carries the feature gate
//!   directly above it.
//! - **Without the feature** (`just test`): the state the binary serves,
//!   configured with `HOST_DEV_TOKEN` *set*, refuses *Create a room* for no
//!   bearer, a wrong one and that very token alike — `401`, no body.
//! - **With the feature** (CI's feature step): missing or empty is a startup
//!   error; a wrong or missing bearer is `401` with no body; the right one
//!   creates a room on the seeded q3; the token authorizes nothing else.

mod common;

use std::path::{Path, PathBuf};

use axum::http::{Method, StatusCode};
use serde_json::json;

use common::http;
use room::config::Config;

const GATE: &str = r#"#[cfg(feature = "dev-host-token")]"#;
/// The variable's name, assembled so that this file's own code does not
/// spell it (the scan below reads `src/`, not `tests/`, but keep it honest).
fn var_name() -> String {
    ["HOST", "DEV", "TOKEN"].join("_")
}

/// A token-shaped value for the tests; drawn per run, so no literal credential
/// is in the repository.
fn fresh_token() -> String {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn vars<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name| pairs.iter().find(|(k, _)| *k == name).map(|(_, v)| v.to_string())
}

// --------------------------------------------------------------------------
// Every build: the source says the stand-in is gated.
// --------------------------------------------------------------------------

fn server_sources() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if path != root.join("bin") {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                let name = rel(&root, &path);
                out.push((name, std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    assert!(out.len() >= 10, "found {} source files", out.len());
    out
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/")
}

fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

#[test]
fn ac64_the_stand_in_module_is_gated_whole() {
    let sources = server_sources();
    let (_, standin) = sources
        .iter()
        .find(|(name, _)| name == "standin.rs")
        .expect("src/standin.rs exists");
    let first_code = standin
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !is_comment(l))
        .unwrap();
    assert_eq!(
        first_code,
        r#"#![cfg(feature = "dev-host-token")]"#,
        "standin.rs must open with the feature gate on the whole module"
    );
}

#[test]
fn ac64_the_variable_is_named_only_in_the_stand_in() {
    let name = var_name();
    let mut found = Vec::new();
    for (file, text) in server_sources() {
        for (n, line) in text.lines().enumerate() {
            if !is_comment(line) && line.contains(&name) && file != "standin.rs" {
                found.push(format!("src/{file}:{}: {}", n + 1, line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "the stand-in's variable is read outside standin.rs:\n{}", found.join("\n"));
}

#[test]
fn ac64_every_line_naming_the_stand_in_is_gated() {
    let names = ["standin", "DevHostToken", "host_token"];
    let mut ungated = Vec::new();
    let mut gated = 0;
    for (file, text) in server_sources() {
        if file == "standin.rs" {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if is_comment(line) || !names.iter().any(|n| line.contains(n)) {
                continue;
            }
            let above = lines[..i]
                .iter()
                .rev()
                .map(|l| l.trim())
                .find(|l| !l.is_empty() && !is_comment(l));
            if above == Some(GATE) {
                gated += 1;
            } else {
                ungated.push(format!("src/{file}:{}: {}", i + 1, line.trim()));
            }
        }
    }
    assert!(ungated.is_empty(), "lines naming the stand-in without {GATE} directly above:\n{}", ungated.join("\n"));
    // The scan is looking at something: the module line, the config field, its
    // read and its initializer, and the serving state's one line.
    assert!(gated >= 5, "only {gated} gated lines found; the scan is not seeing the wiring");
}

// --------------------------------------------------------------------------
// Without the feature: nothing accepts the token.
// --------------------------------------------------------------------------

#[cfg(not(feature = "dev-host-token"))]
#[tokio::test]
async fn ac64_a_build_without_the_feature_refuses_create_for_any_bearer() {
    let token = fresh_token();
    let name = var_name();
    let env = [(name.as_str(), token.as_str())];
    let config = Config::from_vars(vars(&env)).expect("a default build ignores the variable");
    let app = room::router_with(std::sync::Arc::new(room::serving_state(config)));
    for bearer in [None, Some("wrong"), Some(token.as_str())] {
        for question in ["q3", "canary"] {
            let (status, body) = http(&app, Method::POST, "/rooms", bearer, Some(json!({ "question_id": question }))).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "bearer {:?}", bearer.map(|_| "…"));
            assert_eq!(body, serde_json::Value::Null, "a refusal carries no body");
        }
    }
    assert!(!cfg!(feature = "dev-host-token"));
}

// --------------------------------------------------------------------------
// With the feature: exactly that bearer, for creation only.
// --------------------------------------------------------------------------

#[cfg(feature = "dev-host-token")]
mod with_the_feature {
    use super::*;
    use room::config::ConfigError;

    fn app_with(token: &str) -> axum::Router {
        let name = var_name();
        let env = [(name.as_str(), token)];
        let config = Config::from_vars(vars(&env)).expect("configured");
        room::router_with(std::sync::Arc::new(room::serving_state(config)))
    }

    #[test]
    fn ac64_missing_or_empty_is_a_startup_error() {
        let name = var_name();
        assert!(matches!(Config::from_vars(vars(&[])), Err(ConfigError::MissingHostToken)));
        for blank in ["", " ", "\t\n"] {
            let env = [(name.as_str(), blank)];
            assert!(matches!(Config::from_vars(vars(&env)), Err(ConfigError::MissingHostToken)), "{blank:?}");
        }
    }

    #[test]
    fn ac64_the_startup_error_names_the_variable_and_nothing_else() {
        let text = ConfigError::MissingHostToken.to_string();
        assert!(text.contains(&var_name()), "{text}");
    }

    #[tokio::test]
    async fn ac64_a_wrong_or_missing_bearer_is_401_with_no_body() {
        let token = fresh_token();
        let app = app_with(&token);
        let prefix = &token[..token.len() - 1];
        let longer = format!("{token}0");
        let upper = token.to_uppercase();
        for bearer in [None, Some(""), Some("wrong"), Some(prefix), Some(longer.as_str()), Some(upper.as_str())] {
            let (status, body) = http(&app, Method::POST, "/rooms", bearer, Some(json!({ "question_id": "q3" }))).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(body, serde_json::Value::Null, "a refusal carries no body");
        }
    }

    #[tokio::test]
    async fn ac64_the_right_bearer_creates_a_room_on_the_seeded_question() {
        let token = fresh_token();
        let app = app_with(&token);
        let (status, created) = http(&app, Method::POST, "/rooms", Some(&token), Some(json!({ "question_id": "q3" }))).await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        for key in ["id", "code", "join_url", "host_session", "host_resume_url"] {
            assert!(created[key].is_string(), "{key} in {created}");
        }
        assert!(!created.to_string().contains(&token), "the create response never echoes the token");
        // Seeded: q3 and nothing else.
        let (status, _) = http(&app, Method::POST, "/rooms", Some(&token), Some(json!({ "question_id": "q4" }))).await;
        assert_eq!(status, StatusCode::CONFLICT, "only q3 is scheduled");
    }

    #[tokio::test]
    async fn ac64_the_token_grants_nothing_else() {
        let token = fresh_token();
        let app = app_with(&token);
        let (_, created) = http(&app, Method::POST, "/rooms", Some(&token), Some(json!({ "question_id": "q3" }))).await;
        let id = created["id"].as_str().unwrap();
        // Host commands and the host projection take the room's own session.
        let (status, _) = http(&app, Method::POST, &format!("/rooms/{id}/put-on-screen"), Some(&token), None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "the token is not a host session");
        let (status, _) = http(&app, Method::GET, &format!("/rooms/{id}/host"), Some(&token), None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let host = created["host_session"].as_str().unwrap();
        let (status, _) = http(&app, Method::POST, &format!("/rooms/{id}/put-on-screen"), Some(host), None).await;
        assert_eq!(status, StatusCode::OK, "the room's own session still works");
    }

    #[tokio::test]
    async fn ac64_run_it_again_goes_through_the_same_check() {
        let token = fresh_token();
        let app = app_with(&token);
        let (_, created) = http(&app, Method::POST, "/rooms", Some(&token), Some(json!({ "question_id": "q3" }))).await;
        let id = created["id"].as_str().unwrap();
        let host = created["host_session"].as_str().unwrap();
        for slug in ["put-on-screen", "close-answers", "show-split", "walk-it", "reveal", "release"] {
            let (status, body) = http(&app, Method::POST, &format!("/rooms/{id}/{slug}"), Some(host), None).await;
            assert_eq!(status, StatusCode::OK, "{slug}: {body}");
        }
        let again = format!("/rooms/{id}/run-it-again");
        for bearer in [None, Some(host), Some("wrong")] {
            let (status, body) = http(&app, Method::POST, &again, bearer, Some(json!({ "question_id": "q3" }))).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(body, serde_json::Value::Null);
        }
        // The right token passes the check, and the room refuses the question
        // it has already run (G-10) — in words, not as a denial.
        let (status, body) = http(&app, Method::POST, &again, Some(&token), Some(json!({ "question_id": "q3" }))).await;
        assert_eq!(status, StatusCode::CONFLICT, "{body}");
        assert!(body["reason"].is_string());
    }
}
