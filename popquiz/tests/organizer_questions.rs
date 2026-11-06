//! Choosing the question on the host phone (SPEC §8.5, PQ-80).
//!
//! `GET /organizer/questions?club=‹club›` lists, as `id` and `topic` only, the
//! questions scheduled for that club that the club can run today. It takes the
//! organizer session and the same check as *Create a room*. The pick goes
//! through `POST /rooms` as before, which checks everything again.
//!
//! | Case | Tests |
//! |---|---|
//! | the auth matrix (R2): 401, 403 `wrong_server` / `wrong_role`, an unknown club, 503 | `auth_*` (Discord mock) |
//! | the listing's shape: `id` and `topic` only, `no-store`, `no-referrer` | `list_*` |
//! | who can run what today (R3): affirmed, today's date, not used, held by no room | `ready_*` |
//! | the record carries its meetup date (R1-a) | `date_*` |
//! | the state changed between the list and the press | `race_*` |
//! | one way in: the list is exactly what the admin channel scheduled | `picker_lists_exactly_what_the_admin_channel_scheduled` |
//! | the bare host page signs in | `sign_in_*` |
//!
//! Every record here is `bank/questions/q3.json`, sometimes under another id;
//! nothing here writes down what a program prints.

mod common;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::body::Body;
use axum::http::{header, HeaderMap, Method, Request, StatusCode};
use axum::Router;
use common::discord_mock::{snowflake, Membership, Rig};
use common::*;
use room::admin::AdminToken;
use room::answers::{self, Scheduled};
use room::lifecycle::ManualClock;
use room::rooms::{AppState, Urls};
use serde_json::{json, Value};
use tower::ServiceExt;

/// The room's reason for a record arranged for another date (SPEC §11, R1-a).
const OTHER_DATE: &str = "That question was scheduled for another date.";

/// A week after [`MEETUP_DATE`], and a week before it.
const NEXT_WEEK: &str = "2026-10-21";
const LAST_WEEK: &str = "2026-10-07";

const FULL_WALK: [&str; 6] = [
    "put-on-screen",
    "close-answers",
    "show-split",
    "walk-it",
    "reveal",
    "release",
];

// --------------------------------------------------------------------------
// The fixture: a room server reached over the admin channel, as `popquiz
// schedule` reaches it, on a meetup evening, with two clubs.
// --------------------------------------------------------------------------

/// A token for these tests, drawn per process and never written down.
fn token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).unwrap();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Pushed {
    app: Router,
    state: Arc<AppState>,
    token: String,
    clock: Arc<ManualClock>,
}

fn pushed() -> Pushed {
    pushed_with(Vec::new())
}

/// The fixture with `boot` already in the room, as `AppState::new`'s list
/// puts records in without the admin channel.
fn pushed_with(boot: Vec<Scheduled>) -> Pushed {
    let token = token();
    let clock = ManualClock::new(meetup_evening());
    let clubs = room::club::Clubs::parse(
        snowflake(),
        Some(&format!("la={}:America/Los_Angeles", snowflake())),
    )
    .expect("two clubs");
    let state = Arc::new(
        AppState::new(Arc::new(TestAuth), boot, Urls::default())
            .with_clock(clock.clone())
            .with_clubs(clubs),
    );
    let app = room::router_with_admin(
        state.clone(),
        AdminToken::from_var(Some(token.clone())).unwrap(),
    );
    Pushed {
        app,
        state,
        token,
        clock,
    }
}

/// q3 under another id, affirmed for its new text.
fn record(id: &str) -> Value {
    let mut v = q3_json();
    v["id"] = id.into();
    affirmed(&v)
}

/// The admin `PUT` body: the record, and the meetup date it was arranged for.
fn body(record: &Value, date: Option<&str>) -> Value {
    let mut v = record.clone();
    if let Some(d) = date {
        v["meetup_date"] = d.into();
    }
    v
}

/// `PUT /admin/clubs/{club}/questions/{id}`, as `popquiz schedule` sends it.
async fn push_dated(p: &Pushed, club: &str, record: &Value, date: Option<&str>) -> (StatusCode, Value) {
    let id = record["id"].as_str().unwrap();
    http(
        &p.app,
        Method::PUT,
        &format!("/admin/clubs/{club}/questions/{id}"),
        Some(&p.token),
        Some(body(record, date)),
    )
    .await
}

/// A push for today's meetup, which must be accepted.
async fn push(p: &Pushed, club: &str, record: &Value) {
    let (status, body) = push_dated(p, club, record, Some(MEETUP_DATE)).await;
    assert!(status.is_success(), "push {}: {status} {body}", record["id"]);
}

fn list_uri(club: Option<&str>) -> String {
    match club {
        Some(c) => format!("/organizer/questions?club={c}"),
        None => "/organizer/questions".to_string(),
    }
}

/// The listing, raw: status, headers, JSON body (or `Null`).
async fn list_raw(app: &Router, bearer: Option<&str>, club: Option<&str>) -> (StatusCode, HeaderMap, Value) {
    let mut req = Request::builder().method(Method::GET).uri(list_uri(club));
    if let Some(b) = bearer {
        req = req.header(header::AUTHORIZATION, format!("Bearer {b}"));
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    (status, headers, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// The ids [`ORGANIZER`] is offered for `club`; the listing must answer 200.
async fn offered(p: &Pushed, club: &str) -> Vec<String> {
    let (status, _, body) = list_raw(&p.app, Some(ORGANIZER), Some(club)).await;
    assert_eq!(status, StatusCode::OK, "list {club}: {body}");
    ids(&body)
}

fn ids(body: &Value) -> Vec<String> {
    body["questions"]
        .as_array()
        .unwrap_or_else(|| panic!("a list of questions: {body}"))
        .iter()
        .map(|q| q["id"].as_str().expect("an id").to_string())
        .collect()
}

async fn create(app: &Router, bearer: &str, club: &str, id: &str) -> (StatusCode, Value) {
    http(
        app,
        Method::POST,
        "/rooms",
        Some(bearer),
        Some(json!({ "question_id": id, "club": club })),
    )
    .await
}

/// *Create a room*, which must succeed: `(room id, host session)`.
async fn open(app: &Router, bearer: &str, club: &str, id: &str) -> (String, String) {
    let (status, body) = create(app, bearer, club, id).await;
    assert_eq!(status, StatusCode::CREATED, "create {club}/{id}: {body}");
    (
        body["id"].as_str().unwrap().to_string(),
        body["host_session"].as_str().unwrap().to_string(),
    )
}

async fn release(app: &Router, room: &str, host: &str) {
    for slug in FULL_WALK {
        let (status, body) = http(
            app,
            Method::POST,
            &format!("/rooms/{room}/{slug}"),
            Some(host),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{slug}: {body}");
    }
}

// --------------------------------------------------------------------------
// R2: the organizer session and that club's role, as *Create a room* checks
// them. Driven through the Discord mock, as `tests/auth.rs` is.
// --------------------------------------------------------------------------

async fn rig_list(rig: &Rig, bearer: Option<&str>, club: Option<&str>) -> (StatusCode, Value) {
    rig.call(Method::GET, &list_uri(club), bearer, None).await
}

#[tokio::test]
async fn auth_no_session_or_an_unknown_one_is_401_saying_nothing() {
    let rig = Rig::start().await;
    for bearer in [None, Some("not-a-session")] {
        assert_eq!(
            rig_list(&rig, bearer, None).await,
            (StatusCode::UNAUTHORIZED, Value::Null),
            "{bearer:?}"
        );
    }
}

#[tokio::test]
async fn auth_a_signed_in_host_is_answered() {
    let rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    for club in [None, Some("rustnyc"), Some("nyc")] {
        let (status, body) = rig_list(&rig, Some(&session), club).await;
        assert_eq!(status, StatusCode::OK, "{club:?}: {body}");
        assert!(body["questions"].is_array(), "{club:?}: {body}");
    }
}

#[tokio::test]
async fn auth_another_clubs_role_is_403_wrong_role() {
    let rig = Rig::start().await;
    let (la, _) = rig.sign_in(rig.la_host()).await;
    let (nyc, _) = rig.sign_in(rig.host()).await;
    for (session, club) in [(&la, None), (&la, Some("rustnyc")), (&nyc, Some("la"))] {
        let (status, body) = rig_list(&rig, Some(session), club).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{club:?}: {body}");
        assert_eq!(body["refusal"], "wrong_role", "{club:?}: {body}");
        assert_eq!(body["reason"], room::copy::HOST_DENIED_WRONG_ROLE);
    }
    let (status, body) = rig_list(&rig, Some(&la), Some("la")).await;
    assert_eq!(status, StatusCode::OK, "la's own list: {body}");
}

#[tokio::test]
async fn auth_not_in_the_server_is_403_wrong_server() {
    let rig = Rig::start().await;
    let (outsider, _) = rig.sign_in(Membership::Absent).await;
    let (status, body) = rig_list(&rig, Some(&outsider), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["refusal"], "wrong_server");
    assert_eq!(body["reason"], room::copy::HOST_DENIED_WRONG_SERVER);
}

#[tokio::test]
async fn auth_an_unknown_or_malformed_club_is_refused_as_a_missing_role() {
    let rig = Rig::start().await;
    let both = Membership::Member {
        roles: vec![snowflake(), rig.role(), rig.la_role()],
        admin: false,
    };
    let (session, _) = rig.sign_in(both).await;
    let long = "x".repeat(40);
    // Never an empty list or a 404: the answer does not say which clubs exist.
    for club in ["mars", "NYC", "la%2F..%2Fnyc", "", "a%20b", long.as_str()] {
        let (status, body) = rig_list(&rig, Some(&session), Some(club)).await;
        assert_eq!(
            (status, &body["refusal"]),
            (StatusCode::FORBIDDEN, &Value::from("wrong_role")),
            "{club:?}: {body}"
        );
    }
}

#[tokio::test]
async fn auth_an_expired_session_is_401() {
    let rig = Rig::start().await;
    rig.mock.expires_in(120);
    let (session, user) = rig.sign_in(rig.host()).await;
    // Discord will refuse the refresh; five minutes on, the access token is due.
    rig.mock.revoke_refresh(&user);
    rig.clock.advance(Duration::from_secs(5 * 60));
    assert_eq!(
        rig_list(&rig, Some(&session), None).await,
        (StatusCode::UNAUTHORIZED, Value::Null),
        "signed out: sign in again"
    );
}

#[tokio::test]
async fn auth_discord_down_is_503_not_a_denial() {
    let mut rig = Rig::start().await;
    let (session, _) = rig.sign_in(rig.host()).await;
    rig.mock.down().await;
    assert_eq!(
        rig_list(&rig, Some(&session), None).await,
        (StatusCode::SERVICE_UNAVAILABLE, Value::Null),
        "an outage is not a denial"
    );
    assert_eq!(rig.discord.organizer_count(), 1, "an outage signs nobody out");
}

// --------------------------------------------------------------------------
// The listing's shape.
// --------------------------------------------------------------------------

#[tokio::test]
async fn list_is_id_and_topic_only_and_never_cached() {
    let p = pushed();
    push(&p, "rustnyc", &q3_json()).await;
    let (status, headers, body) = list_raw(&p.app, Some(ORGANIZER), Some("rustnyc")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "questions": [{ "id": "q3", "topic": q3_json()["topic"] }] }),
        "id and topic, nothing else"
    );
    let get = |h: header::HeaderName| headers.get(h).and_then(|v| v.to_str().ok());
    assert_eq!(get(header::CACHE_CONTROL), Some("no-store"));
    assert_eq!(get(header::REFERRER_POLICY), Some("no-referrer"));
    // Belt and braces for the keys check: nothing of the record but the two.
    let text = body.to_string();
    for absent in ["options", "correct", "stdout", "hint", "why_tempting", "meetup_date", MEETUP_DATE] {
        assert!(!text.contains(absent), "{absent} in {text}");
    }
}

#[tokio::test]
async fn list_is_sorted_by_id_and_scoped_to_the_named_club() {
    let p = pushed();
    push(&p, "rustnyc", &record("q3-zeta")).await;
    push(&p, "rustnyc", &q3_json()).await;
    push(&p, "la", &record("q3-la")).await;
    assert_eq!(offered(&p, "rustnyc").await, ["q3", "q3-zeta"]);
    assert_eq!(offered(&p, "nyc").await, ["q3", "q3-zeta"], "nyc is rustnyc");
    assert_eq!(offered(&p, "la").await, ["q3-la"]);
    let (status, _, body) = list_raw(&p.app, Some(ORGANIZER), None).await;
    assert_eq!((status, ids(&body)), (StatusCode::OK, vec!["q3".into(), "q3-zeta".into()]), "no club is the default club");
}

#[tokio::test]
async fn list_does_not_need_or_take_the_admin_token() {
    let p = pushed();
    push(&p, "rustnyc", &q3_json()).await;
    let (status, _, body) = list_raw(&p.app, Some(&p.token), None).await;
    assert_eq!((status, body), (StatusCode::UNAUTHORIZED, Value::Null));
}

// --------------------------------------------------------------------------
// R3: offered only what the club can run today. Every one of these is also a
// refusal at create, which stays the authority.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ready_a_question_a_live_room_holds_drops_out_until_the_room_ends() {
    let p = pushed();
    push(&p, "rustnyc", &q3_json()).await;
    let _room = open(&p.app, ORGANIZER, "rustnyc", "q3").await;
    assert_eq!(offered(&p, "rustnyc").await, Vec::<String>::new(), "held only: empty");
    // Another club's list is its own.
    push(&p, "la", &q3_json()).await;
    assert_eq!(offered(&p, "la").await, ["q3"]);
    // The room goes quiet and ends unreleased: q3 is not used, so it is back.
    p.clock.advance(room::lifecycle::IDLE_QUIET + Duration::from_secs(60));
    assert_eq!(offered(&p, "rustnyc").await, ["q3"], "an ended room holds nothing");
}

#[tokio::test]
async fn ready_a_released_question_drops_out_for_that_club_only() {
    let p = pushed();
    push(&p, "rustnyc", &q3_json()).await;
    push(&p, "la", &q3_json()).await;
    let (room, host) = open(&p.app, ORGANIZER, "rustnyc", "q3").await;
    release(&p.app, &room, &host).await;
    assert_eq!(offered(&p, "rustnyc").await, Vec::<String>::new(), "used only: empty");
    assert_eq!(offered(&p, "la").await, ["q3"], "never twice is per club");
}

#[tokio::test]
async fn ready_an_unaffirmed_or_stale_record_is_not_offered() {
    let mut v = q3_json();
    v["id"] = "q3-stale".into();
    // The hint is affirmed text; an edit after the review makes it stale.
    v["hint"] = "Edited on the afternoon of the meetup.".into();
    let stale = answers::load(&v.to_string()).expect("a stale record still loads");
    assert!(stale.refusal("q3-stale").is_some(), "the fixture is stale");
    let p = pushed_with(vec![stale]);
    assert!(!offered(&p, "rustnyc").await.contains(&"q3-stale".to_string()));
    let (status, _) = create(&p.app, ORGANIZER, "rustnyc", "q3-stale").await;
    assert_eq!(status, StatusCode::CONFLICT, "and create still refuses it");
}

#[tokio::test]
async fn ready_nothing_pushed_is_an_empty_list_not_an_error() {
    let p = pushed();
    assert_eq!(offered(&p, "rustnyc").await, Vec::<String>::new());
}

// --------------------------------------------------------------------------
// R1-a: the record carries the meetup date it was arranged for, and the room
// offers it, or creates on it, on that date only.
// --------------------------------------------------------------------------

#[tokio::test]
async fn date_a_record_for_another_date_is_neither_offered_nor_created() {
    let p = pushed();
    let (status, body) = push_dated(&p, "rustnyc", &q3_json(), Some(NEXT_WEEK)).await;
    assert!(status.is_success(), "next week's push is accepted: {body}");
    let (status, body) = push_dated(&p, "rustnyc", &record("q3-old"), Some(LAST_WEEK)).await;
    assert!(status.is_success(), "an old push is accepted: {body}");
    assert_eq!(offered(&p, "rustnyc").await, Vec::<String>::new(), "other dates only: empty");
    for id in ["q3", "q3-old"] {
        assert_eq!(
            create(&p.app, ORGANIZER, "rustnyc", id).await,
            (StatusCode::CONFLICT, json!({ "reason": OTHER_DATE })),
            "{id}: the host link is gated too"
        );
    }
    // Re-pushed for today, it is offered and runs.
    push(&p, "rustnyc", &q3_json()).await;
    assert_eq!(offered(&p, "rustnyc").await, ["q3"]);
    open(&p.app, ORGANIZER, "rustnyc", "q3").await;
}

#[tokio::test]
async fn date_a_push_without_a_well_formed_meetup_date_is_refused() {
    let p = pushed();
    for date in [None, Some(""), Some("2026-1-14"), Some("2026-10-14T00:00:00"), Some("14 October")] {
        let (status, body) = push_dated(&p, "rustnyc", &q3_json(), date).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{date:?}: {body}");
    }
    assert_eq!(
        create(&p.app, ORGANIZER, "rustnyc", "q3").await,
        (StatusCode::CONFLICT, json!({ "reason": room::copy::ROOM_NOTHING_SCHEDULED })),
        "nothing was scheduled"
    );
}

#[tokio::test]
async fn date_the_club_s_own_zone_decides_today() {
    // 2026-10-15T04:30Z: past midnight in New York, 21:30 the evening before in
    // Los Angeles.
    let p = pushed();
    p.clock
        .set(SystemTime::UNIX_EPOCH + Duration::from_secs(1_792_024_200 + 4 * 3600));
    push(&p, "la", &q3_json()).await;
    let (status, body) = push_dated(&p, "rustnyc", &q3_json(), Some(MEETUP_DATE)).await;
    assert!(status.is_success(), "{body}");
    assert_eq!(offered(&p, "la").await, ["q3"], "still the 14th in LA");
    assert_eq!(offered(&p, "rustnyc").await, Vec::<String>::new(), "the 15th in New York");
}

// --------------------------------------------------------------------------
// The state changed between the list and the press: create refuses with the
// existing reason, and the list no longer offers it.
// --------------------------------------------------------------------------

#[tokio::test]
async fn race_another_organizer_opened_it_first() {
    let p = pushed();
    push(&p, "rustnyc", &q3_json()).await;
    assert_eq!(offered(&p, "rustnyc").await, ["q3"]);
    open(&p.app, OTHER_ORGANIZER, "rustnyc", "q3").await;
    assert_eq!(
        create(&p.app, ORGANIZER, "rustnyc", "q3").await,
        (StatusCode::CONFLICT, json!({ "reason": room::copy::ROOM_QUESTION_OPEN_ELSEWHERE }))
    );
    assert_eq!(offered(&p, "rustnyc").await, Vec::<String>::new());
}

#[tokio::test]
async fn race_another_organizer_ran_it_first() {
    let p = pushed();
    push(&p, "rustnyc", &q3_json()).await;
    assert_eq!(offered(&p, "rustnyc").await, ["q3"]);
    let (room, host) = open(&p.app, OTHER_ORGANIZER, "rustnyc", "q3").await;
    release(&p.app, &room, &host).await;
    assert_eq!(
        create(&p.app, ORGANIZER, "rustnyc", "q3").await,
        (StatusCode::CONFLICT, json!({ "reason": room::copy::ROOM_QUESTION_ALREADY_RUN }))
    );
    assert_eq!(offered(&p, "rustnyc").await, Vec::<String>::new());
}

// --------------------------------------------------------------------------
// One way in (decision 5): the admin `PUT` is the only scheduling path, and the
// picker offers exactly what it scheduled.
// --------------------------------------------------------------------------

/// The host projection with this room's random values taken out.
async fn host_view(app: &Router, room: &str, host: &str) -> String {
    let (status, view) = http(app, Method::GET, &format!("/rooms/{room}/host"), Some(host), None).await;
    assert_eq!(status, StatusCode::OK, "{view}");
    let code = view["code"].as_str().map(str::to_string);
    let mut text = view.to_string().replace(room, "ROOM").replace(host, "HOST");
    if let Some(code) = code {
        text = text.replace(&code, "CODE");
    }
    text
}

#[tokio::test]
async fn picker_lists_exactly_what_the_admin_channel_scheduled() {
    // The phone's way: the laptop pushed a shortlist, the organizer picks.
    let phone = pushed();
    push(&phone, "rustnyc", &q3_json()).await;
    push(&phone, "la", &record("q3-la")).await;
    assert_eq!(phone.state.room_count(), 0, "listing made nothing");
    let picked = offered(&phone, "rustnyc").await;
    assert_eq!(picked, ["q3"], "exactly what was pushed for the club");
    let (a, a_host) = open(&phone.app, ORGANIZER, "rustnyc", &picked[0]).await;

    // The host link's way: the same push, the id named in `/host?question=q3`.
    let link = pushed();
    push(&link, "rustnyc", &q3_json()).await;
    let (b, b_host) = open(&link.app, ORGANIZER, "rustnyc", "q3").await;

    assert_eq!(
        host_view(&phone.app, &a, &a_host).await,
        host_view(&link.app, &b, &b_host).await,
        "the same room either way"
    );
    // Releasing uses it, on either path, and only then.
    release(&phone.app, &a, &a_host).await;
    assert_eq!(offered(&phone, "rustnyc").await, Vec::<String>::new());
    assert_eq!(offered(&phone, "la").await, ["q3-la"]);
    let used = http(&phone.app, Method::GET, "/admin/used", Some(&phone.token), None).await.1;
    assert_eq!(used.as_array().map(Vec::len), Some(1), "{used}");
}

// --------------------------------------------------------------------------
// The bare host page (`/host`, no question in the link) signs in.
// --------------------------------------------------------------------------

#[tokio::test]
async fn sign_in_without_a_question_lands_on_the_bare_host_page() {
    let rig = Rig::start().await;
    for (start, landing) in [
        ("/auth/discord", "/host"),
        ("/auth/discord?question=", "/host"),
        ("/auth/discord?club=la", "/host?club=la"),
    ] {
        let (code, _) = rig.mock.account(rig.host());
        let r = rig.raw(Method::GET, start, &[], None).await;
        assert_eq!(r.status, StatusCode::SEE_OTHER, "{start}");
        let location = r.header("location").unwrap().to_string();
        let state = common::discord_mock::query(&location, "state").expect("state");
        let cookie = r
            .header("set-cookie")
            .and_then(|c| c.split(';').next())
            .unwrap()
            .to_string();
        let back = rig
            .callback(&format!("code={code}&state={state}"), Some(&cookie))
            .await;
        assert_eq!(back.status, StatusCode::SEE_OTHER, "{start}");
        let (path, session) = back
            .header("location")
            .unwrap()
            .split_once('#')
            .expect("a session in the fragment");
        assert_eq!(path, landing, "{start}");
        assert!(!session.is_empty());
    }
    // A malformed question is still refused.
    let r = rig.raw(Method::GET, "/auth/discord?question=..%2Fq3", &[], None).await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
}
