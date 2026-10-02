//! `smoke` (T-09): the deployed room, driven end to end.
//!
//! EVALUATION.md's harness row: *the deployed room driven end to end through
//! all seven phases with mock participants — the pre-checkpoint sanity run.*
//! `just smoke <url> [--participants N] [--question ID] [--out PATH]`. The
//! organizer's credential comes from
//! `POPQUIZ_ORGANIZER_SESSION` in the environment and never from the command
//! line, so it stays out of shell history and process listings. It is the
//! organizer session a signed-in host page holds after Discord sign-in (T-10):
//! sign in at `<url>/host?question=q3`, and it is everything after the `#` in
//! the address the page lands on. It is one person's, it lasts 12 hours, and
//! every room it creates passes Discord's live role check (SPEC §8) — smoke
//! holds no shared secret that creates rooms. q3 must be scheduled first, over
//! the pipeline channel (SPEC §8.3) — under its own id, or (`--question`) under
//! a harness id such as `smoke-q3` so that the run's release never retires a
//! bank question (room/README.md, *Burst*). Whatever the id, the record must be
//! q3's: the secrecy scan reads its secrets from q3.
//!
//! Against `<url>` (https/wss through the Fly edge, or plain http/ws on
//! loopback), in order:
//!
//! 1. the pages answer: `GET /join`, `GET /host`;
//! 2. *Create a room* with no bearer and with a wrong one is `401` with no body
//!    (AC-64, on the deployed build); with the organizer session it creates a
//!    room on q3;
//! 3. the join link is `<url>/<code>` and `GET /<code>` is `303 → /join?code=`
//!    (AC-28); half the participants join with the code as given, half with
//!    it typed lower-case with spaces;
//! 4. a wall socket, a host socket and one buzzer socket per participant are
//!    attached, and every phase — idle, live, closed, split, work, reveal,
//!    released — is driven by the host routes and awaited on **every** socket;
//! 5. in `live` each participant answers, a third of them change their minds,
//!    and then every final answer lands inside one 2 s window (the deadline
//!    burst); every write is timed on an already-open connection;
//! 6. after close, a write is refused (`409`, saved answer restated), and the
//!    split's totals equal the final answers exactly;
//! 7. `work` stops at step M-2: one more step is refused;
//! 8. the reveal's arrival on every buzzer is timed;
//! 9. after release a join is refused `already_ended`.
//!
//! **The secrecy scan.** Every frame and every JSON response in `idle` to
//! `work` is scanned for q3's secrets: both beats of `explains`, every
//! `why_tempting`, the resolving step's note, a ✓, and a key named `correct`.
//! At `reveal` the positive control checks that they do arrive (the host's
//! read-aloud, the wall's resolving step), so a scan that saw nothing fails.
//! This is the deployed-room scan the canary's `--url` hook cannot run until
//! T-25 can plant a canary question; it scans the real question instead.
//!
//! **What its numbers are not.** The write and reveal timings are smoke's, from
//! wherever it runs. They are not `burst`'s AC-53/AC-54/AC-41 figures.
//!
//! The HTTP and socket client is `room_client.rs`, shared with `burst` (T-21).
//! `--out` also writes the result as JSON, for CI to keep.
//!
//! Exit: `0` every check passed; `1` a check failed; `2` it could not run
//! (arguments, no session, the room unreachable, q3 not scheduled or already
//! run, the organizer refused by Discord).

#[path = "room_client.rs"]
mod client;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use serde_json::{json, Value};

use client::{all_reach, millis, open_ws, wait_for_room, watch, Http, OnFrame, Rng, Target, Watcher, LETTERS, MAX_PARTICIPANTS};

/// The HC-0 question, as the verifier wrote it — where the secrets come from.
const Q3: &str = include_str!("../../../bank/questions/q3.json");
/// How long any one phase may take to reach every socket.
const WAIT: Duration = Duration::from_secs(20);
/// SPEC §9's deadline burst: the final writes all land inside this window.
const BURST_WINDOW: Duration = Duration::from_secs(2);
const DEFAULT_PARTICIPANTS: usize = 20;
const PRE_REVEAL: [&str; 5] = ["idle", "live", "closed", "split", "work"];

// --------------------------------------------------------------------------
// Arguments.
// --------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub url: String,
    pub participants: usize,
    /// The id q3's record is scheduled under on the room.
    pub question: String,
    pub out: Option<String>,
}

const USAGE: &str = "usage: smoke --url <http(s)://host[:port]> [--participants N] [--question ID] [--out PATH]   (POPQUIZ_ORGANIZER_SESSION in the environment)";

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut url = None;
    let mut participants = DEFAULT_PARTICIPANTS;
    let mut question = "q3".to_string();
    let mut out = None;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--url" => url = Some(it.next().ok_or("--url needs a value")?),
            "--participants" => {
                let n = it.next().ok_or("--participants needs a value")?;
                participants = n
                    .parse()
                    .ok()
                    .filter(|n| (1..=MAX_PARTICIPANTS).contains(n))
                    .ok_or(format!("--participants must be 1 to {MAX_PARTICIPANTS}, not {n:?}"))?;
            }
            "--question" => question = it.next().filter(|q| !q.is_empty()).ok_or("--question needs an id")?,
            "--out" => out = Some(it.next().ok_or("--out needs a path")?),
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("unknown argument {other:?}\n{USAGE}")),
        }
    }
    Ok(Args {
        url: url.ok_or(USAGE)?,
        participants,
        question,
        out,
    })
}

// --------------------------------------------------------------------------
// The secrets, and the scan.
// --------------------------------------------------------------------------

struct Secrets {
    texts: Vec<(String, String)>,
}

impl Secrets {
    fn from_record(q: &Value) -> Secrets {
        let mut texts = vec![
            ("explains.what".to_string(), q["explains"]["what"].as_str().unwrap_or_default().to_string()),
            ("explains.takeaway".to_string(), q["explains"]["takeaway"].as_str().unwrap_or_default().to_string()),
            ("check mark".to_string(), "✓".to_string()),
        ];
        for (i, o) in q["options"].as_array().into_iter().flatten().enumerate() {
            if let Some(w) = o["why_tempting"].as_str() {
                texts.push((format!("why_tempting {}", LETTERS[i]), w.to_string()));
            }
        }
        if let Some(last) = q["trace"]["steps"].as_array().and_then(|s| s.last()) {
            texts.push(("the resolving note".to_string(), last["note"].as_str().unwrap_or_default().to_string()));
        }
        texts.retain(|(_, t)| !t.is_empty());
        Secrets { texts }
    }

    /// The names of every secret `v` carries, and `key correct` if any object
    /// has that key.
    fn found_in(&self, v: &Value) -> Vec<String> {
        let mut strings = Vec::new();
        let mut keys = false;
        walk(v, &mut strings, &mut keys);
        let mut hits: Vec<String> = self
            .texts
            .iter()
            .filter(|(_, t)| strings.iter().any(|s| s.contains(t.as_str())))
            .map(|(name, _)| name.clone())
            .collect();
        if keys {
            hits.push("key correct".into());
        }
        hits
    }

    fn get(&self, name: &str) -> &str {
        &self.texts.iter().find(|(n, _)| n == name).expect("a named secret").1
    }
}

fn walk<'a>(v: &'a Value, strings: &mut Vec<&'a str>, correct_key: &mut bool) {
    match v {
        Value::String(s) => strings.push(s),
        Value::Array(items) => items.iter().for_each(|i| walk(i, strings, correct_key)),
        Value::Object(map) => {
            for (k, child) in map {
                *correct_key |= k == "correct";
                strings.push(k);
                walk(child, strings, correct_key);
            }
        }
        _ => {}
    }
}

/// What every check found wrong, and how much was scanned.
#[derive(Default)]
struct Findings {
    failures: Vec<String>,
    scanned: usize,
}

type Shared = Arc<Mutex<Findings>>;

fn fail(f: &Shared, msg: impl Into<String>) {
    let msg = msg.into();
    eprintln!("  FAIL {msg}");
    f.lock().unwrap().failures.push(msg);
}

fn check(f: &Shared, ok: bool, msg: impl Into<String>) {
    if !ok {
        fail(f, msg);
    }
}

/// Scan a payload that belongs to `phase`; only pre-reveal phases are held to
/// the rule.
fn scan(f: &Shared, secrets: &Secrets, phase: &str, surface: &str, v: &Value) {
    if !PRE_REVEAL.contains(&phase) {
        return;
    }
    f.lock().unwrap().scanned += 1;
    let hits = secrets.found_in(v);
    if !hits.is_empty() {
        fail(f, format!("LEAK: {surface} in {phase} carries {}", hits.join(", ")));
    }
}

// --------------------------------------------------------------------------
// Sockets: `room_client`'s watcher, with every frame scanned.
// --------------------------------------------------------------------------

fn watch_scanned(ws: client::Ws, surface: String, secrets: Arc<Secrets>, findings: Shared) -> Watcher {
    let hook: OnFrame = Arc::new(move |frame: Option<&Value>| match frame {
        None => fail(&findings, format!("{surface}: a frame that is not JSON")),
        Some(frame) => scan(&findings, &secrets, frame["phase"].as_str().unwrap_or(""), &surface, frame),
    });
    watch(ws, Some(hook))
}

// --------------------------------------------------------------------------
// Numbers.
// --------------------------------------------------------------------------

/// The nearest-rank percentile, in milliseconds.
fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return f64::NAN;
    }
    let rank = ((p / 100.0) * sorted_ms.len() as f64).ceil().max(1.0) as usize;
    sorted_ms[rank.min(sorted_ms.len()) - 1]
}

fn summary(name: &str, mut ms: Vec<f64>) -> String {
    ms.sort_by(f64::total_cmp);
    format!(
        "{name}: n={} p50={:.1}ms p95={:.1}ms max={:.1}ms",
        ms.len(),
        percentile(&ms, 50.0),
        percentile(&ms, 95.0),
        ms.last().copied().unwrap_or(f64::NAN)
    )
}

// --------------------------------------------------------------------------
// The run.
// --------------------------------------------------------------------------

struct Participant {
    http: Http,
    token: String,
    watcher: Watcher,
    fin: usize,
}

struct Ctx {
    target: Arc<Target>,
    tls: Option<tokio_rustls::TlsConnector>,
    secrets: Arc<Secrets>,
    findings: Shared,
}

impl Ctx {
    fn http(&self) -> Http {
        Http::new(&self.target, &self.tls)
    }
}

/// Every socket: the wall, the host and each buzzer.
fn everyone<'a>(wall: &'a Watcher, host: &'a Watcher, people: &'a [Participant]) -> Vec<&'a Watcher> {
    let mut all = vec![wall, host];
    all.extend(people.iter().map(|p| &p.watcher));
    all
}

/// A host action; its response is the host payload, scanned under the phase
/// it names.
async fn act(ctx: &Ctx, host: &mut Http, room: &str, session: &str, slug: &str) -> Result<Value, String> {
    let r = host.call("POST", &format!("/rooms/{room}/{slug}"), Some(session), None).await?;
    let body = r.json();
    if r.status != 200 {
        return Err(format!("{slug}: {} {body}", r.status));
    }
    let phase = body["phase"].as_str().unwrap_or("").to_string();
    scan(&ctx.findings, &ctx.secrets, &phase, &format!("POST {slug}"), &body);
    Ok(body)
}

/// The pages answer, once the machine does (`wait_for_room` waits out a
/// stopped machine's start).
async fn pages_answer(ctx: &Ctx) -> Result<(), String> {
    wait_for_room(&ctx.target, &ctx.tls).await?;
    let mut http = ctx.http();
    for page in ["/join", "/host"] {
        let r = http.call("GET", page, None, None).await?;
        check(&ctx.findings, r.status == 200, format!("GET {page}: {}", r.status));
        let html = String::from_utf8_lossy(&r.body);
        check(&ctx.findings, html.contains("<html") || html.contains("<!doctype") || html.contains("<!DOCTYPE"), format!("GET {page}: not a page"));
    }
    Ok(())
}

/// What a run that could run found.
pub struct Outcome {
    pub substrate: &'static str,
    pub lines: Vec<String>,
    pub failures: Vec<String>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }
}

/// The whole run against `args.url`, as the organizer whose session is
/// `token`. `Err` is "could not run" (exit 2); an `Outcome` with failures is a
/// failed check (exit 1).
pub async fn run(args: &Args, token: &str) -> Result<Outcome, String> {
    let target = Arc::new(Target::parse(&args.url)?);
    let tls = target.tls_connector();
    let q3: Value = serde_json::from_str(Q3).map_err(|e| format!("q3.json: {e}"))?;
    let ctx = Ctx {
        target: target.clone(),
        tls,
        secrets: Arc::new(Secrets::from_record(&q3)),
        findings: Shared::default(),
    };
    let f = &ctx.findings;
    let n = args.participants;
    let seed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
    let mut rng = Rng(seed);
    let q = args.question.as_str();
    let substrate = if target.is_loopback() { "loopback" } else { "deployed" };
    let mut report = vec![format!("smoke {} ({substrate}) on {q} with {n} participants (seed {seed})", target.base)];
    let started = Instant::now();

    println!("· the pages");
    pages_answer(&ctx).await?;

    println!("· Create a room: refused without a session, made with the organizer's (AC-64)");
    let mut host_http = ctx.http();
    let create = json!({ "question_id": q });
    for (label, bearer) in [("no bearer", None), ("a wrong bearer", Some("not-the-host-token"))] {
        let r = host_http.call("POST", "/rooms", bearer, Some(&create)).await?;
        check(f, r.status == 401, format!("create with {label}: {} (want 401)", r.status));
        check(f, r.body.is_empty(), format!("create with {label}: the refusal has a body"));
    }
    let r = host_http.call("POST", "/rooms", Some(token), Some(&create)).await?;
    let created = r.json();
    match r.status {
        201 => {}
        401 => return Err("create with POPQUIZ_ORGANIZER_SESSION: 401 — the session is unknown or expired; sign in again at /host".into()),
        403 => return Err(format!("create: 403 {} — Discord says this organizer may not host", created["reason"])),
        409 => {
            return Err(format!(
                "create: 409 {} — schedule q3's record as {q} over the pipeline channel first; if it has already been run on this machine, restart it (fly apps restart) and schedule it again",
                created["reason"]
            ))
        }
        s => return Err(format!("create: {s} {created}")),
    }
    scan(f, &ctx.secrets, "idle", "POST /rooms", &created);
    let field = |k: &str| created[k].as_str().map(str::to_string).ok_or(format!("create: no {k} in the response"));
    let (room, code, session) = (field("id")?, field("code")?, field("host_session")?);

    println!("· the join link carries the host and the code (AC-28)");
    let join_url = field("join_url")?;
    check(f, join_url == format!("{}/{code}", target.base), format!("join_url is {join_url}, not {}/{code}", target.base));
    let r = host_http.call("GET", &format!("/{code}"), None, None).await?;
    check(f, r.status == 303, format!("GET /{code}: {} (want 303)", r.status));
    check(
        f,
        r.location.as_deref() == Some(&format!("/join?code={code}")),
        format!("GET /{code} goes to {:?}", r.location),
    );

    println!("· the wall and the host attach");
    let wall = watch_scanned(open_ws(&target, &format!("/rooms/{room}/ws/wall"), None).await?, "wall".into(), ctx.secrets.clone(), f.clone());
    let host = watch_scanned(open_ws(&target, &format!("/rooms/{room}/ws/host"), Some(&session)).await?, "host".into(), ctx.secrets.clone(), f.clone());

    println!("· {n} participants join and attach");
    let t = Instant::now();
    let joins = (0..n).map(|i| {
        let mut http = ctx.http();
        let (target, secrets, findings) = (target.clone(), ctx.secrets.clone(), f.clone());
        let (code, room) = (code.clone(), room.clone());
        tokio::spawn(async move {
            // Half with the code as the short link carries it, half as typed.
            let given = if i % 2 == 0 { code.clone() } else { format!("  {} ", code.to_lowercase()) };
            let r = http.call("POST", "/join", None, Some(&json!({ "code": given }))).await?;
            let body = r.json();
            if r.status != 201 {
                return Err(format!("join {i} with {given:?}: {} {body}", r.status));
            }
            scan(&findings, &secrets, "idle", "POST /join", &body);
            if body["room_id"] != room.as_str() {
                return Err(format!("join {i}: room_id {}", body["room_id"]));
            }
            let token = body["token"].as_str().ok_or(format!("join {i}: no token"))?.to_string();
            let ws = open_ws(&target, &format!("/rooms/{room}/ws/buzzer"), Some(&token)).await?;
            let watcher = watch_scanned(ws, format!("buzzer {i}"), secrets, findings);
            Ok(Participant { http, token, watcher, fin: 0 })
        })
    });
    let mut people = Vec::with_capacity(n);
    for j in futures_util::future::join_all(joins).await {
        people.push(j.map_err(|e| e.to_string())??);
    }
    report.push(format!("joined and attached {n} in {:.0}ms", millis(t.elapsed())));

    all_reach(&everyone(&wall, &host, &people), "idle", WAIT).await?;
    let deadline = Instant::now() + WAIT;
    loop {
        let present = host.last("idle").map(|h| h["present"].clone());
        if present == Some(json!(n)) {
            break;
        }
        if Instant::now() > deadline {
            fail(f, format!("the host's idle count is {present:?}, not {n}"));
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    println!("· live");
    act(&ctx, &mut host_http, &room, &session, "put-on-screen").await?;
    all_reach(&everyone(&wall, &host, &people), "live", WAIT).await?;

    println!("· answers, changed minds, then the deadline burst");
    for p in people.iter_mut() {
        p.fin = rng.below(5) as usize;
    }
    let plans: Vec<(usize, Option<usize>, Duration)> = people
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let first = if i % 3 == 0 { Some((p.fin + 1 + rng.below(4) as usize) % 5) } else { None };
            (p.fin, first, Duration::from_millis(rng.below(BURST_WINDOW.as_millis() as u64)))
        })
        .collect();
    let burst_start = Instant::now() + Duration::from_millis(500);
    let writes = people.into_iter().zip(plans).map(|(mut p, (fin, first, offset))| {
        let room = room.clone();
        tokio::spawn(async move {
            let mut lat = Vec::new();
            let mut errs = Vec::new();
            let path = format!("/rooms/{room}/answer");
            let mut write = async |http: &mut Http, token: &str, letter: usize| -> Result<(), String> {
                let t = Instant::now();
                let r = http.call("PUT", &path, Some(token), Some(&json!({ "letter": LETTERS[letter] }))).await?;
                let took = t.elapsed();
                let body = r.json();
                if r.status != 200 || body["saved"] != LETTERS[letter] {
                    return Err(format!("PUT answer {}: {} {body}", LETTERS[letter], r.status));
                }
                lat.push(millis(took));
                Ok(())
            };
            if let Some(first) = first {
                if let Err(e) = write(&mut p.http, &p.token, first).await {
                    errs.push(e);
                }
            }
            tokio::time::sleep_until((burst_start + offset).into()).await;
            if let Err(e) = write(&mut p.http, &p.token, fin).await {
                errs.push(e);
            }
            (p, lat, errs)
        })
    });
    let mut people = Vec::with_capacity(n);
    let mut latencies = Vec::new();
    let mut expected = [0u32; 5];
    for w in futures_util::future::join_all(writes).await {
        let (p, lat, errs) = w.map_err(|e| e.to_string())?;
        errs.into_iter().for_each(|e| fail(f, e));
        latencies.extend(lat);
        expected[p.fin] += 1;
        people.push(p);
    }
    let burst_took = burst_start.elapsed();
    report.push(summary("answer writes (kept-alive connection, incl. the burst)", latencies));
    report.push(format!("burst: {n} final writes scheduled inside {}ms; done {:.0}ms after it opened", BURST_WINDOW.as_millis(), millis(burst_took)));

    println!("· closed");
    act(&ctx, &mut host_http, &room, &session, "close-answers").await?;
    all_reach(&everyone(&wall, &host, &people), "closed", WAIT).await?;
    let p0 = &mut people[0];
    let letter = LETTERS[(p0.fin + 1) % 5];
    let r = p0.http.call("PUT", &format!("/rooms/{room}/answer"), Some(&p0.token), Some(&json!({ "letter": letter }))).await?;
    let body = r.json();
    scan(f, &ctx.secrets, "closed", "PUT answer after close", &body);
    check(f, r.status == 409, format!("a write after close: {} (want 409)", r.status));
    check(f, body["saved"] == LETTERS[p0.fin], format!("the refused write restates {} (want {})", body["saved"], LETTERS[p0.fin]));

    println!("· split: the totals are the final answers, exactly");
    act(&ctx, &mut host_http, &room, &session, "show-split").await?;
    all_reach(&everyone(&wall, &host, &people), "split", WAIT).await?;
    let mut disagree = 0;
    for p in &people {
        let frame = p.watcher.last("split").unwrap_or(Value::Null);
        if frame["counts"]["totals"] != json!(expected) || frame["counts"]["answered"] != json!(n) {
            disagree += 1;
        }
    }
    check(f, disagree == 0, format!("{disagree} buzzers' split totals differ from the final answers {expected:?}"));
    report.push(format!("split totals {expected:?}, answered {n}: {}", if disagree == 0 { "exact on every buzzer" } else { "MISMATCH" }));

    println!("· work stops at M-2");
    let body = act(&ctx, &mut host_http, &room, &session, "walk-it").await?;
    all_reach(&everyone(&wall, &host, &people), "work", WAIT).await?;
    let m = body["step"]["m"].as_u64().ok_or(format!("work: no step.m in {body}"))?;
    let mut at = body["step"]["at"].as_u64().unwrap_or(u64::MAX);
    check(f, at == 0, format!("work enters at step {at}, not 0"));
    while at + 2 < m {
        at = act(&ctx, &mut host_http, &room, &session, "step-forward").await?["step"]["at"].as_u64().unwrap_or(u64::MAX);
    }
    check(f, at + 2 == m, format!("work reached step {at} of M={m}"));
    let r = host_http.call("POST", &format!("/rooms/{room}/step-forward"), Some(&session), None).await?;
    scan(f, &ctx.secrets, "work", "POST step-forward (refused)", &r.json());
    check(f, r.status == 409, format!("a step past M-2 in work: {} (want 409)", r.status));

    println!("· reveal");
    let t0 = Instant::now();
    act(&ctx, &mut host_http, &room, &session, "reveal").await?;
    let arrivals = all_reach(&people.iter().map(|p| &p.watcher).collect::<Vec<_>>(), "reveal", WAIT).await?;
    all_reach(&[&wall, &host], "reveal", WAIT).await?;
    report.push(summary("reveal reaching every buzzer (from the host's POST)", arrivals.iter().map(|a| millis(a.duration_since(t0))).collect()));
    // The positive control: the secrets do reach the surfaces that may show them.
    let s = &ctx.secrets;
    let host_reveal = host.last("reveal").unwrap_or(Value::Null);
    let wall_reveal = wall.last("reveal").unwrap_or(Value::Null);
    check(f, s.found_in(&host_reveal).contains(&"explains.what".to_string()), "reveal: the host's read-aloud does not carry the explanation — the scan may be blind");
    check(f, s.found_in(&wall_reveal).contains(&"the resolving note".to_string()), "reveal: the wall does not carry the resolving step — the scan may be blind");
    // Across every reveal surface: each kind of secret the scan looks for shows
    // up at least once, so none of its pre-reveal checks can pass by being blind.
    // Only the most-chosen incorrect option's why_tempting is read aloud (§4.5),
    // so one why_tempting is what reveal owes.
    let mut surfaced: Vec<String> = s.found_in(&host_reveal);
    surfaced.extend(s.found_in(&wall_reveal));
    for p in &people {
        surfaced.extend(s.found_in(&p.watcher.last("reveal").unwrap_or(Value::Null)));
    }
    for want in ["explains.what", "explains.takeaway", "the resolving note", "check mark", "key correct"] {
        check(f, surfaced.iter().any(|h| h == want), format!("reveal: no surface carries {want} — the scan may be blind to it"));
    }
    check(f, surfaced.iter().any(|h| h.starts_with("why_tempting")), "reveal: no surface carries any why_tempting — the scan may be blind to it");
    let correct = people[0].watcher.last("reveal").map(|b| b["correct"].clone()).unwrap_or(Value::Null);
    check(f, correct.is_string(), "reveal: the buzzer names no correct letter");
    check(f, !s.get("the resolving note").is_empty(), "q3 has no resolving note to scan for");

    println!("· released");
    act(&ctx, &mut host_http, &room, &session, "release").await?;
    all_reach(&everyone(&wall, &host, &people), "released", WAIT).await?;
    let r = host_http.call("POST", "/join", None, Some(&json!({ "code": code }))).await?;
    check(f, r.status == 409 && r.json()["refusal"] == "already_ended", format!("a join after release: {} {}", r.status, r.json()));

    let findings = f.lock().unwrap();
    report.push(format!(
        "secrecy scan: {} pre-reveal frames and responses, {} leaks",
        findings.scanned,
        findings.failures.iter().filter(|m| m.starts_with("LEAK")).count()
    ));
    report.push(format!("all seven phases on {} sockets in {:.1}s", n + 2, started.elapsed().as_secs_f64()));
    Ok(Outcome {
        substrate,
        lines: report,
        failures: findings.failures.clone(),
    })
}

/// The result as JSON, for `--out` (CI keeps it as an artifact). Never the
/// organizer session: `run` is handed it and nothing here sees it.
pub fn report_json(args: &Args, result: &Result<Outcome, String>) -> Value {
    let (status, exit, substrate, lines, failures) = match result {
        Ok(o) if o.passed() => ("pass", 0, Some(o.substrate), o.lines.clone(), vec![]),
        Ok(o) => ("fail", 1, Some(o.substrate), o.lines.clone(), o.failures.clone()),
        Err(e) => ("could_not_run", 2, None, vec![], vec![client::explain(e)]),
    };
    json!({
        "schema": "rustnyc-popquiz/smoke-report/1",
        "ticket": "T-21",
        "url": args.url,
        "question": args.question,
        "participants": args.participants,
        "substrate": substrate,
        "status": status,
        "exit_code": exit,
        "lines": lines,
        "failures": failures,
    })
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::process::ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return std::process::ExitCode::from(2);
        }
    };
    let Some(token) = std::env::var("POPQUIZ_ORGANIZER_SESSION").ok().filter(|t| !t.is_empty()) else {
        eprintln!("smoke: POPQUIZ_ORGANIZER_SESSION must be set in the environment (never on the command line): sign in at <url>/host?question=q3 and copy what follows the # in the address");
        return std::process::ExitCode::from(2);
    };
    let result = run(&args, &token).await;
    if let Some(path) = &args.out {
        if let Err(e) = std::fs::write(path, serde_json::to_string_pretty(&report_json(&args, &result)).unwrap_or_default()) {
            eprintln!("smoke: could not write {path}: {e}");
        }
    }
    match result {
        Ok(o) if o.passed() => {
            println!("\nSMOKE PASS");
            o.lines.iter().for_each(|l| println!("  {l}"));
            std::process::ExitCode::SUCCESS
        }
        Ok(o) => {
            println!("\nSMOKE FAIL");
            o.lines.iter().for_each(|l| println!("  {l}"));
            println!("{} checks failed:", o.failures.len());
            o.failures.iter().for_each(|m| println!("  - {m}"));
            std::process::ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("\nSMOKE COULD NOT RUN: {}", client::explain(&e));
            std::process::ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Result<Args, String> {
        parse_args(s.split_whitespace().map(String::from))
    }

    #[test]
    fn arguments() {
        assert_eq!(
            args("--url https://x.fly.dev").unwrap(),
            Args { url: "https://x.fly.dev".into(), participants: 20, question: "q3".into(), out: None }
        );
        let a = args("--url http://127.0.0.1:3000 --participants 200 --question smoke-q3 --out r.json").unwrap();
        assert_eq!((a.participants, a.question.as_str(), a.out.as_deref()), (200, "smoke-q3", Some("r.json")));
        for bad in ["", "--participants 5", "--url", "--url u --participants 0", "--url u --participants 201", "--url u --token t", "--url u --question"] {
            assert!(args(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_json_report_names_its_status_and_substrate() {
        let a = args("--url http://127.0.0.1:3000 --question smoke-q3").unwrap();
        let passed = Ok(Outcome { substrate: "loopback", lines: vec!["x".into()], failures: vec![] });
        let r = report_json(&a, &passed);
        assert_eq!((r["status"].as_str(), r["exit_code"].as_i64(), r["substrate"].as_str()), (Some("pass"), Some(0), Some("loopback")));
        assert_eq!(r["question"], "smoke-q3");
        let failed = Ok(Outcome { substrate: "deployed", lines: vec![], failures: vec!["LEAK: …".into()] });
        assert_eq!(report_json(&a, &failed)["exit_code"], 1);
        let r = report_json(&a, &Err("create: 404".into()));
        assert_eq!((r["status"].as_str(), r["exit_code"].as_i64()), (Some("could_not_run"), Some(2)));
        assert!(r["substrate"].is_null());
    }

    #[test]
    fn nearest_rank() {
        let v: Vec<f64> = (1..=200).map(f64::from).collect();
        assert_eq!(percentile(&v, 95.0), 190.0);
        assert_eq!(percentile(&v, 50.0), 100.0);
        assert_eq!(percentile(&[7.0], 95.0), 7.0);
    }

    #[test]
    fn the_scan_finds_each_secret_and_only_those() {
        let q: Value = serde_json::from_str(Q3).unwrap();
        let s = Secrets::from_record(&q);
        // explains ×2, the check mark, four why_tempting, the resolving note.
        assert_eq!(s.texts.len(), 8, "{:?}", s.texts.iter().map(|t| &t.0).collect::<Vec<_>>());
        for (name, text) in &s.texts {
            let planted = json!({ "phase": "live", "deep": [{ "x": format!("…{text}…") }] });
            assert_eq!(s.found_in(&planted), vec![name.clone()]);
        }
        assert_eq!(s.found_in(&json!({ "correct": "E" })), vec!["key correct".to_string()]);
        let clean = json!({ "phase": "live", "source": q["source"], "options": q["options"].as_array().unwrap().iter().map(|o| &o["text"]).collect::<Vec<_>>() });
        assert!(s.found_in(&clean).is_empty());
    }
}
