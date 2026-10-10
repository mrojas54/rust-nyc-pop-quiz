//! `burst` (T-21): the room under load, on its own protocol.
//!
//! EVALUATION.md's harness row: *200 synthetic participants; the deadline write
//! burst in isolation (AC-54); then a full segment (AC-52, AC-53, AC-41).*
//! `just burst <url> [--participants N] [--question ID] [--connection-cap M] [--out PATH] …`
//! (`--help` lists the rest). The question defaults to `burst-q3`, a harness
//! id, so a run that omits `--question` cannot retire a bank question; off
//! loopback a bank id is refused unless `--spend-bank-question` is given.
//!
//! | Criterion | Measure | Pass |
//! |---|---|---|
//! | AC-54 | each isolated deadline burst: every participant writes once inside a 2 s window, nothing else in flight; the worst cycle's p95 | p95 < 500 ms |
//! | AC-53 | every answer write while `live`: the isolated bursts, the churn and the final deadline burst | p95 < 500 ms |
//! | AC-41 | the reveal: from the host's `POST reveal` to each buzzer's first `reveal` frame | p95 <= 2 s |
//! | AC-52 | every write saved as sent; the host's live count one per session; after close, each session's refused write restates *its own* final answer; the split's totals are the finals exactly | exact |
//!
//! It is the T-03 spike's measurement (`spike-burst.rs`, now frozen with the
//! spike) carried onto the room's real routes and sockets through the client
//! `smoke` uses (`room_client.rs`): joins by code, a buzzer socket per
//! participant, answers as `PUT /rooms/{id}/answer` on a kept-alive
//! connection, phases driven by the host routes. The numbers, the verdict and
//! the report are `burst_report.rs`, which `just test` checks on recorded
//! samples. Every write is timed from the moment it is sent on an already open
//! connection; the client's own scheduling delay is measured beside it
//! (`send_lag`), never added to it.
//!
//! **Credentials.** Exactly smoke's: the organizer session a signed-in host
//! page holds, from `POPQUIZ_ORGANIZER_SESSION` and never from the command line.
//! `burst` holds no secret that creates rooms and never touches the admin
//! token; the question is scheduled before it runs (room/README.md, *Burst*).
//!
//! **Where it runs.** Against `https://…` it is the deployed substrate, and
//! that is the only run that can meet AC-53's *run against the deployed
//! substrate* clause. Against a loopback room (`tests/harness_full.rs`, in
//! `test-full`) it proves the harness; the report says `"substrate":
//! "loopback"` and never claims that clause.
//!
//! **The connection cap.** The run holds `2n + 3` connections at its peak: two
//! per participant (its kept-alive HTTP connection and its buzzer socket), the
//! wall's and the host's sockets, and the host's HTTP connection. Fly's proxy
//! routes at most `hard_limit` (`fly.toml`) to the one machine and refuses the
//! rest, and cannot tell a harness from a phone. Off loopback
//! `--connection-cap` (that `hard_limit`) is required, and a run whose peak is
//! over it is invalid, never a pass.
//!
//! Exit: `0` all four criteria pass as measured, at the criteria's conditions;
//! `1` a criterion missed; `2` the run is invalid, was below the criteria's
//! conditions without missing, or could not run — do not quote it as a pass.

#[path = "room_client.rs"]
mod client;
#[path = "burst_report.rs"]
pub(crate) mod numbers;

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use client::{
    all_reach, create_refused, explain, millis, open_ws, question_allowed, wait_for_room, watch, Http, Target, Watcher, LETTERS, MAX_PARTICIPANTS,
};
use numbers::{Cycle, Exit, Measured};

/// How long any one phase may take to reach every socket.
const WAIT: Duration = Duration::from_secs(20);
/// The `spike` shape: every write inside the window's last 50 ms.
const SPIKE_TAIL_MS: u64 = 50;

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Offsets uniform over the whole window — SPEC §9's literal wording.
    Uniform,
    /// All inside the window's last 50 ms: a room reacting to one countdown.
    Spike,
}

impl Shape {
    fn as_str(self) -> &'static str {
        match self {
            Shape::Uniform => "uniform",
            Shape::Spike => "spike",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Args {
    pub url: String,
    pub participants: usize,
    /// The id the question is scheduled under on the room.
    pub question: String,
    pub seed: u64,
    pub out: Option<String>,
    pub shapes: Vec<Shape>,
    /// Isolated bursts per shape.
    pub cycles: usize,
    pub churn_secs: u64,
    pub window_ms: u64,
    /// Quiet time between stages, so no burst overlaps anything else.
    pub gap_ms: u64,
    /// The proxy's connection cap the room runs behind (`hard_limit`).
    pub connection_cap: Option<u64>,
    /// `--spend-bank-question`: a bank id off loopback, on purpose.
    pub spend_bank: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            url: String::new(),
            participants: MAX_PARTICIPANTS,
            question: "burst-q3".into(),
            seed: 1,
            out: None,
            // Both shapes run and the worst drives AC-54: letting only the
            // gentler shape reach the exit code would mean the harder one could
            // never fail the run.
            shapes: vec![Shape::Uniform, Shape::Spike],
            cycles: 2,
            churn_secs: 20,
            window_ms: 2000,
            gap_ms: 1500,
            connection_cap: None,
            spend_bank: false,
        }
    }
}

const USAGE: &str = "\
burst - the room under load, on its own protocol (T-21)

  --url <base>          http(s):// base of the room (required)
  --participants <n>    1-200, default 200
  --question <id>       the scheduled question to create the room on (default burst-q3,
                        a harness id; off loopback a bank id is refused)
  --spend-bank-question allow a bank id off loopback: its release retires that question
  --connection-cap <m>  the proxy's hard_limit (fly.toml); required off loopback.
                        A run whose peak, 2n+3 connections, is over it is invalid
  --seed <n>            makes the letters and offsets reproducible (default 1)
  --out <path>          also write the JSON report here
  --burst-shape <s>     uniform | spike | both   (default both; the worst drives AC-54)
  --cycles <n>          isolated bursts per shape (default 2)
  --churn-secs <n>      the segment's changed-minds window (default 20)
  --window-ms <n>       the burst window (default 2000)
  --gap-ms <n>          quiet time between stages (default 1500)

POPQUIZ_ORGANIZER_SESSION must be in the environment (never on the command line).
Exit: 0 all criteria pass - 1 a criterion missed - 2 run invalid or could not run, do not quote.";

pub fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = argv.iter();
    let num = |flag: &str, v: Option<&String>| -> Result<u64, String> {
        v.ok_or(format!("{flag} needs a value"))?.parse().map_err(|_| format!("{flag} wants a number"))
    };
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--url" => a.url = it.next().ok_or("--url needs a value")?.clone(),
            "--participants" => {
                a.participants = num(flag, it.next())? as usize;
                if !(1..=MAX_PARTICIPANTS).contains(&a.participants) {
                    return Err(format!("--participants must be 1 to {MAX_PARTICIPANTS}"));
                }
            }
            "--question" => a.question = it.next().filter(|q| !q.is_empty()).ok_or("--question needs an id")?.clone(),
            "--seed" => a.seed = num(flag, it.next())?,
            "--out" => a.out = Some(it.next().ok_or("--out needs a path")?.clone()),
            "--cycles" => a.cycles = num(flag, it.next())? as usize,
            "--churn-secs" => a.churn_secs = num(flag, it.next())?,
            "--window-ms" => a.window_ms = num(flag, it.next())?.max(SPIKE_TAIL_MS),
            "--gap-ms" => a.gap_ms = num(flag, it.next())?,
            "--connection-cap" => a.connection_cap = Some(num(flag, it.next())?),
            "--spend-bank-question" => a.spend_bank = true,
            "--burst-shape" => {
                a.shapes = match it.next().map(String::as_str) {
                    Some("uniform") => vec![Shape::Uniform],
                    Some("spike") => vec![Shape::Spike],
                    Some("both") => vec![Shape::Uniform, Shape::Spike],
                    other => return Err(format!("--burst-shape wants uniform, spike or both, not {other:?}")),
                }
            }
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("unknown argument {other:?}\n\n{USAGE}")),
        }
    }
    if a.url.is_empty() {
        return Err(USAGE.into());
    }
    if a.cycles == 0 {
        return Err("--cycles must be at least 1: AC-54 is the isolated burst".into());
    }
    guard(&a)?;
    Ok(a)
}

/// What a run must not do whatever built its arguments: talk plain http
/// across a network, spend a bank question off loopback by default, or run
/// against a deployed room without knowing its connection cap.
pub fn guard(a: &Args) -> Result<Target, String> {
    let target = Target::parse(&a.url)?;
    question_allowed(&target, &a.question, a.spend_bank)?;
    if !target.is_loopback() && a.connection_cap.is_none() {
        return Err(format!(
            "--connection-cap is required off loopback: the hard_limit in fly.toml. The run holds {} connections at its peak (2n+3).",
            numbers::expected_peak_connections(a.participants)
        ));
    }
    Ok(target)
}

// ---------------------------------------------------------------------------
// Schedules — derived from the seed, so a run reproduces exactly.
// ---------------------------------------------------------------------------

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn draw(seed: u64, stage: u64, who: usize, k: u64) -> u64 {
    splitmix64(seed ^ (stage << 48) ^ ((who as u64) << 16) ^ k)
}

/// A participant's offset inside the window for one burst.
pub fn burst_offset_ms(seed: u64, stage: u64, who: usize, shape: Shape, window_ms: u64) -> u64 {
    let r = draw(seed, stage, who, 0);
    match shape {
        Shape::Uniform => r % window_ms.max(1),
        Shape::Spike => window_ms.saturating_sub(SPIKE_TAIL_MS) + r % SPIKE_TAIL_MS,
    }
}

/// A letter for one write. Uniform over A–E: this is a load test, and the
/// letters only have to be each session's own so a swap can be caught.
pub fn letter(seed: u64, stage: u64, who: usize) -> usize {
    (draw(seed, stage, who, 1) % 5) as usize
}

// ---------------------------------------------------------------------------
// Writes
// ---------------------------------------------------------------------------

struct Person {
    http: Http,
    token: String,
    watcher: Watcher,
    /// The last letter the room saved for this session.
    saved: Option<usize>,
}

/// One timed write's outcome.
#[derive(Debug, Default, Clone)]
struct Wrote {
    /// Where in the stage it was scheduled.
    offset: Duration,
    ms: f64,
    lag_ms: f64,
    /// `200 {saved: <the letter>}`.
    acked: bool,
}

/// Each participant's writes this stage: `(offset from start, letter)`,
/// ascending — a client sends one at a time, so an unsorted schedule would
/// book its own sleep as send lag.
type Plan = Vec<(Duration, usize)>;

/// What became of the writes the room never answered.
#[derive(Default)]
struct Unanswered {
    /// Where in the stage each was scheduled.
    offsets: Vec<Duration>,
    /// The first few, as said.
    errors: Vec<String>,
}

/// A failed write is the harness's own failure only when it could not reach
/// the room at all (a fresh connect); a write that was sent and never
/// answered — no response in time, the connection closed under it — is the
/// room's, and an AC-52 miss.
fn harness_side(e: &str) -> bool {
    e.contains(": connect ") || e.contains(": TLS to ")
}

/// Run every participant's plan concurrently from `start`. Returns the people
/// (their connections kept open), every answered write, the harness errors
/// (a write that could not reach the room), and the writes the room never
/// answered.
async fn write_stage(people: Vec<Person>, plans: Vec<Plan>, start: Instant, room: &str) -> (Vec<Person>, Vec<Wrote>, Vec<String>, Unanswered) {
    let tasks = people.into_iter().zip(plans).map(|(mut p, plan)| {
        let path = format!("/rooms/{room}/answer");
        tokio::spawn(async move {
            let mut wrote = Vec::new();
            let mut errors = Vec::new();
            let mut unanswered = Unanswered::default();
            for (offset, l) in plan {
                let at = start + offset;
                tokio::time::sleep_until(at.into()).await;
                let sent = Instant::now();
                let r = p.http.call("PUT", &path, Some(&p.token), Some(&json!({ "letter": LETTERS[l] }))).await;
                let took = sent.elapsed();
                match r {
                    Ok(r) => {
                        let acked = r.status == 200 && r.json()["saved"] == LETTERS[l];
                        if acked {
                            p.saved = Some(l);
                        }
                        wrote.push(Wrote { offset, ms: millis(took), lag_ms: millis(sent.saturating_duration_since(at)), acked });
                    }
                    Err(e) if harness_side(&e) => errors.push(e),
                    Err(e) => {
                        unanswered.offsets.push(offset);
                        unanswered.errors.push(e);
                    }
                }
            }
            (p, wrote, errors, unanswered)
        })
    });
    let mut people = Vec::new();
    let mut all = Vec::new();
    let mut errors = Vec::new();
    let mut unanswered = Unanswered::default();
    for t in futures_util::future::join_all(tasks).await {
        match t {
            Ok((p, w, e, u)) => {
                people.push(p);
                all.extend(w);
                errors.extend(e);
                unanswered.offsets.extend(u.offsets);
                unanswered.errors.extend(u.errors);
            }
            Err(e) => errors.push(format!("a client task died: {e}")),
        }
    }
    (people, all, errors, unanswered)
}

// ---------------------------------------------------------------------------
// The run
// ---------------------------------------------------------------------------

fn ulimit_nofile() -> u64 {
    std::process::Command::new("sh")
        .arg("-c")
        .arg("ulimit -n")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            let s = s.trim();
            if s == "unlimited" {
                Some(u64::MAX)
            } else {
                s.parse().ok()
            }
        })
        .unwrap_or(0)
}

fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// A host action; `Err` unless the room answered `200`.
async fn act(host: &mut Http, room: &str, session: &str, slug: &str) -> Result<Value, String> {
    let r = host.call("POST", &format!("/rooms/{room}/{slug}"), Some(session), None).await?;
    let body = r.json();
    if r.status != 200 {
        return Err(format!("{slug}: {} {body}", r.status));
    }
    Ok(body)
}

fn everyone<'a>(wall: &'a Watcher, host: &'a Watcher, people: &'a [Person]) -> Vec<&'a Watcher> {
    let mut all = vec![wall, host];
    all.extend(people.iter().map(|p| &p.watcher));
    all
}

/// The whole run against `args.url` as the organizer whose session is
/// `token`: the report and its exit code. `Err` is "could not run" (exit 2):
/// the room unreachable, the session refused, the question not scheduled, or
/// the room gone mid-run.
pub async fn run(args: &Args, token: &str, invocation: String) -> Result<(Value, Exit), String> {
    let target = Arc::new(guard(args)?);
    let tls = target.tls_connector();
    let n = args.participants;
    let seed = args.seed;
    let mut m = Measured {
        invocation,
        started_at_unix_ms: now_unix_ms(),
        url: target.base.clone(),
        loopback: target.is_loopback(),
        question: args.question.clone(),
        seed,
        participants: n,
        window_ms: args.window_ms,
        churn_secs: args.churn_secs,
        shapes: args.shapes.len(),
        connection_cap: args.connection_cap,
        ulimit_nofile: ulimit_nofile(),
        ..Measured::default()
    };
    let gap = Duration::from_millis(args.gap_ms);

    let peak = numbers::expected_peak_connections(n);
    match args.connection_cap {
        Some(cap) => eprintln!(
            "· expected peak connections {peak} vs cap {cap}{}",
            if peak as u64 > cap { " — OVER: the proxy would refuse some participants; this run cannot be a result for AC-53" } else { "" }
        ),
        None => eprintln!("· expected peak connections {peak} (loopback: no proxy cap)"),
    }

    eprintln!("· the room answers");
    wait_for_room(&target, &tls).await?;

    eprintln!("· Create a room on {}", args.question);
    let mut host_http = Http::new(&target, &tls);
    let r = host_http.call("POST", "/rooms", Some(token), Some(&json!({ "question_id": args.question }))).await?;
    let created = r.json();
    match r.status {
        201 => {}
        401 => return Err("create with POPQUIZ_ORGANIZER_SESSION: 401 — the session is unknown or expired; sign in again at /host".into()),
        403 => return Err(format!("create: 403 {} — Discord says this organizer may not host", created["reason"])),
        409 => return Err(create_refused(created["reason"].as_str().unwrap_or_default(), &args.question)),
        s => return Err(format!("create: {s} {created}")),
    }
    let field = |k: &str| created[k].as_str().map(str::to_string).ok_or(format!("create: no {k} in the response"));
    let (room, code, session) = (field("id")?, field("code")?, field("host_session")?);
    m.room_id = room.clone();

    let wall = watch(open_ws(&target, &format!("/rooms/{room}/ws/wall"), None).await?, None);
    let host = watch(open_ws(&target, &format!("/rooms/{room}/ws/host"), Some(&session)).await?, None);

    eprintln!("· {n} participants join and attach");
    let joins = (0..n).map(|_| {
        let mut http = Http::new(&target, &tls);
        let (target, code, room) = (target.clone(), code.clone(), room.clone());
        tokio::spawn(async move {
            let t = Instant::now();
            http.open().await?;
            let connect = millis(t.elapsed());
            let t = Instant::now();
            let r = http.call("POST", "/join", None, Some(&json!({ "code": code }))).await?;
            let join = millis(t.elapsed());
            let body = r.json();
            if r.status != 201 || body["room_id"] != room.as_str() {
                return Err(format!("join: {} {body}", r.status));
            }
            let token = body["token"].as_str().ok_or("join: no token")?.to_string();
            let ws = open_ws(&target, &format!("/rooms/{room}/ws/buzzer"), Some(&token)).await?;
            Ok::<_, String>((Person { http, token, watcher: watch(ws, None), saved: None }, connect, join))
        })
    });
    let mut people = Vec::with_capacity(n);
    for j in futures_util::future::join_all(joins).await {
        match j.map_err(|e| e.to_string()).and_then(|r| r) {
            Ok((p, connect, join)) => {
                m.connect_ms.push(connect);
                m.join_ms.push(join);
                people.push(p);
            }
            Err(e) => m.errors.push(e),
        }
    }
    m.connected = people.len();
    if people.is_empty() {
        return Err(format!("no participant could join: {}", m.errors.first().cloned().unwrap_or_default()));
    }
    all_reach(&everyone(&wall, &host, &people), "idle", WAIT).await?;

    eprintln!("· live");
    act(&mut host_http, &room, &session, "put-on-screen").await?;
    all_reach(&everyone(&wall, &host, &people), "live", WAIT).await?;
    tokio::time::sleep(gap).await;

    // AC-54: each burst alone. Every participant writes once inside the window
    // and nothing else is in flight; a gap separates the bursts.
    let mut stage = 0u64;
    for &shape in &args.shapes {
        for index in 0..args.cycles {
            stage += 1;
            eprintln!("· isolated burst: {} #{index}", shape.as_str());
            let plans: Vec<Plan> = (0..people.len())
                .map(|who| vec![(Duration::from_millis(burst_offset_ms(seed, stage, who, shape, args.window_ms)), letter(seed, stage, who))])
                .collect();
            let start = Instant::now() + Duration::from_millis(200);
            let (back, wrote, errors, unanswered) = write_stage(people, plans, start, &room).await;
            people = back;
            m.scheduled_writes += people.len();
            m.send_lag_ms.extend(wrote.iter().map(|w| w.lag_ms));
            m.errors.extend(errors);
            m.unanswered_writes += unanswered.offsets.len();
            m.unanswered_first.extend(unanswered.errors);
            m.cycles.push(Cycle {
                shape: shape.as_str(),
                index,
                acked: wrote.len() == people.len() && wrote.iter().all(|w| w.acked),
                writes_ms: wrote.iter().map(|w| w.ms).collect(),
            });
            tokio::time::sleep(gap).await;
        }
    }
    // Each session counted once: the host's live count settles at N.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        m.host_answered_after_bursts = host.last("live").and_then(|h| h["answered"].as_u64());
        if m.host_answered_after_bursts == Some(n as u64) || Instant::now() > deadline {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // AC-53: the segment. A third change their minds during the churn, then
    // every final answer lands inside one window: the deadline burst.
    eprintln!("· churn for {}s, then the deadline burst", args.churn_secs);
    stage += 1;
    let churn = Duration::from_secs(args.churn_secs);
    let deadline_stage = stage + 1;
    let plans: Vec<Plan> = (0..people.len())
        .map(|who| {
            let mut plan = Vec::new();
            if who % 3 == 0 && !churn.is_zero() {
                let at = draw(seed, stage, who, 2) % churn.as_millis().max(1) as u64;
                plan.push((Duration::from_millis(at), letter(seed, stage, who)));
            }
            let off = burst_offset_ms(seed, deadline_stage, who, Shape::Uniform, args.window_ms);
            plan.push((churn + gap + Duration::from_millis(off), letter(seed, deadline_stage, who)));
            plan
        })
        .collect();
    let churn_count: usize = plans.iter().map(|p| p.len() - 1).sum();
    let start = Instant::now() + Duration::from_millis(200);
    let (back, wrote, errors, unanswered) = write_stage(people, plans, start, &room).await;
    people = back;
    m.errors.extend(errors);
    m.scheduled_writes += churn_count + people.len();
    m.send_lag_ms.extend(wrote.iter().map(|w| w.lag_ms));
    m.unacked_writes = wrote.iter().filter(|w| !w.acked).count();
    // Each plan's deadline write is scheduled past `churn + gap`; every
    // churn write before it.
    let boundary = churn + gap;
    m.unanswered_writes += unanswered.offsets.len();
    m.unanswered_deadline = unanswered.offsets.iter().filter(|o| **o >= boundary).count();
    m.unanswered_first.extend(unanswered.errors);
    let (deadline, churned): (Vec<&Wrote>, Vec<&Wrote>) = wrote.iter().partition(|w| w.offset >= boundary);
    let churn_ms: Vec<f64> = churned.iter().map(|w| w.ms).collect();
    let deadline_ms: Vec<f64> = deadline.iter().map(|w| w.ms).collect();
    m.churn_ms = churn_ms;
    m.deadline_ms = deadline_ms;

    let mut expected = [0u32; 5];
    for p in &people {
        if let Some(l) = p.saved {
            expected[l] += 1;
        }
    }
    m.expected_totals = expected;

    eprintln!("· closed: every session's write refused, its own answer restated");
    act(&mut host_http, &room, &session, "close-answers").await?;
    all_reach(&everyone(&wall, &host, &people), "closed", WAIT).await?;
    let probes = people.into_iter().map(|mut p| {
        let path = format!("/rooms/{room}/answer");
        tokio::spawn(async move {
            let other = LETTERS[(p.saved.unwrap_or(0) + 1) % 5];
            let r = p.http.call("PUT", &path, Some(&p.token), Some(&json!({ "letter": other }))).await;
            (p, r.map(|r| (r.status, r.json())))
        })
    });
    let mut people = Vec::new();
    for t in futures_util::future::join_all(probes).await {
        let Ok((p, r)) = t else {
            m.errors.push("a client task died after close".into());
            continue;
        };
        match r {
            Ok((status, body)) => {
                if status == 409 {
                    m.post_close_refused += 1;
                }
                if status == 409 && p.saved.is_some_and(|l| body["saved"] == LETTERS[l]) {
                    m.post_close_restated += 1;
                }
            }
            Err(e) => m.errors.push(e),
        }
        people.push(p);
    }

    eprintln!("· split: the totals are the final answers");
    act(&mut host_http, &room, &session, "show-split").await?;
    all_reach(&everyone(&wall, &host, &people), "split", WAIT).await?;
    let first = people.first().and_then(|p| p.watcher.last("split")).unwrap_or(Value::Null);
    m.split_totals = serde_json::from_value(first["counts"]["totals"].clone()).ok();
    m.split_answered = first["counts"]["answered"].as_u64();
    m.split_disagreeing = people
        .iter()
        .filter(|p| p.watcher.last("split").map(|f| f["counts"]["totals"] != json!(expected)).unwrap_or(true))
        .count();

    eprintln!("· work, to M-2");
    let body = act(&mut host_http, &room, &session, "walk-it").await?;
    all_reach(&everyone(&wall, &host, &people), "work", WAIT).await?;
    let steps = body["step"]["m"].as_u64().ok_or(format!("work: no step.m in {body}"))?;
    let mut at = body["step"]["at"].as_u64().unwrap_or(0);
    while at + 2 < steps {
        at = act(&mut host_http, &room, &session, "step-forward").await?["step"]["at"].as_u64().ok_or("step-forward: no step.at")?;
    }

    // AC-41: one broadcast, timed on this process's clock from the host's
    // POST to each buzzer's first `reveal` frame. A missing receipt is never a
    // fast one: it is counted, and it invalidates the run.
    eprintln!("· reveal");
    let t0 = Instant::now();
    act(&mut host_http, &room, &session, "reveal").await?;
    let buzzers: Vec<&Watcher> = people.iter().map(|p| &p.watcher).collect();
    if let Err(e) = all_reach(&buzzers, "reveal", WAIT).await {
        m.errors.push(e);
    }
    m.reveal_ms = buzzers.iter().filter_map(|w| w.arrived("reveal")).map(|a| millis(a.saturating_duration_since(t0))).collect();

    eprintln!("· released");
    act(&mut host_http, &room, &session, "release").await?;
    if let Err(e) = all_reach(&everyone(&wall, &host, &people), "released", WAIT).await {
        m.errors.push(e);
    }

    Ok(numbers::report(&m))
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::process::ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return std::process::ExitCode::from(2);
        }
    };
    let Some(token) = std::env::var("POPQUIZ_ORGANIZER_SESSION").ok().filter(|t| !t.is_empty()) else {
        eprintln!("burst: POPQUIZ_ORGANIZER_SESSION must be set in the environment (never on the command line): sign in at <url>/host and copy what follows the # in the address");
        return std::process::ExitCode::from(2);
    };
    let invocation = format!("burst {}", argv.join(" "));
    match run(&args, &token, invocation).await {
        Ok((report, code)) => {
            let text = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into());
            println!("{text}");
            // Written before the exit code is acted on, always, so a failing run
            // still leaves a full artifact behind.
            if let Some(path) = &args.out {
                if let Some(dir) = std::path::Path::new(path).parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                if let Err(e) = std::fs::write(path, &text) {
                    eprintln!("burst: could not write {path}: {e}");
                }
            }
            for note in report["verdict"]["notes"].as_array().into_iter().flatten() {
                eprintln!("burst: {}", note.as_str().unwrap_or_default());
            }
            eprintln!(
                "burst: {} — n={} of {} — exit {}",
                report["substrate"].as_str().unwrap_or("?"),
                report["client"]["participants_connected"],
                report["client"]["participants_requested"],
                code as i32
            );
            std::process::ExitCode::from(code as u8)
        }
        Err(e) => {
            let e = explain(&e);
            eprintln!("burst could not run: {e}");
            if let Some(path) = &args.out {
                let _ = std::fs::write(path, json!({ "schema": numbers::SCHEMA, "could_not_run": e, "quotable": false }).to_string());
            }
            std::process::ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Result<Args, String> {
        parse_args(&s.split_whitespace().map(String::from).collect::<Vec<_>>())
    }

    #[test]
    fn arguments() {
        let a = args("--url https://x.fly.dev --connection-cap 400").unwrap();
        assert_eq!((a.participants, a.question.as_str(), a.cycles, a.churn_secs, a.window_ms), (200, "burst-q3", 2, 20, 2000));
        assert_eq!((a.connection_cap, a.spend_bank), (Some(400), false));
        assert_eq!(a.shapes, vec![Shape::Uniform, Shape::Spike]);
        let a = args("--url http://127.0.0.1:9 --participants 12 --question burst-q3 --cycles 1 --burst-shape spike --churn-secs 3 --out r.json").unwrap();
        assert_eq!((a.participants, a.question.as_str(), a.cycles, a.out.as_deref()), (12, "burst-q3", 1, Some("r.json")));
        assert_eq!(a.shapes, vec![Shape::Spike]);
        for bad in ["", "--participants 5", "--url u --participants 0", "--url u --participants 201", "--url u --cycles 0", "--url u --burst-shape wide", "--url u --token t", "--url u --question"] {
            assert!(args(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn offsets_stay_inside_the_window_and_the_spike_inside_its_tail() {
        for who in 0..200 {
            assert!(burst_offset_ms(9, 1, who, Shape::Uniform, 2000) < 2000);
            let s = burst_offset_ms(9, 1, who, Shape::Spike, 2000);
            assert!((1950..2000).contains(&s), "{s}");
            assert!(letter(9, 1, who) < 5);
        }
        // Reproducible from the seed, and not the same draw for every stage.
        assert_eq!(burst_offset_ms(9, 3, 7, Shape::Uniform, 2000), burst_offset_ms(9, 3, 7, Shape::Uniform, 2000));
        let a: Vec<u64> = (0..50).map(|w| burst_offset_ms(9, 1, w, Shape::Uniform, 2000)).collect();
        let b: Vec<u64> = (0..50).map(|w| burst_offset_ms(9, 2, w, Shape::Uniform, 2000)).collect();
        assert_ne!(a, b);
    }
}
