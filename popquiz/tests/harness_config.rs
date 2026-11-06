//! The harness's own wiring (PQ-41): the justfile and the two workflows say
//! what the review of PR #39 asked them to, and keep saying it. Line-shaped
//! checks on the files themselves — no YAML parser is in the lock — each
//! aimed at the line that does the work, not at a comment that names it.

use std::ops::Range;
use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"))
}

/// Lines that are not comments.
fn code(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect()
}

/// The lines of one top-level job in a workflow (two-space indented key).
fn job<'a>(workflow: &'a str, name: &str) -> Vec<&'a str> {
    let head = format!("  {name}:");
    let mut lines = code(workflow).into_iter().skip_while(|l| *l != head);
    let first = lines.next().unwrap_or_else(|| panic!("no job {name}"));
    std::iter::once(first)
        .chain(lines.take_while(|l| l.is_empty() || l.starts_with("    ")))
        .collect()
}

#[test]
fn the_justfile_has_no_dead_recipe_and_runs_a11y_once() {
    let j = read("justfile");
    assert!(
        !code(&j).iter().any(|l| l.starts_with("_pending")),
        "the _pending recipe is back; nothing calls it"
    );
    let full = code(&j)
        .into_iter()
        .find(|l| l.starts_with("test-full:"))
        .expect("test-full");
    let deps: Vec<&str> = full["test-full:".len()..].split_whitespace().collect();
    assert!(deps.contains(&"test"), "{full}");
    assert!(
        !deps.contains(&"a11y"),
        "test-full runs a11y twice: `test` already runs it through test-web: {full}"
    );
    let harness = j.split("\nharness-full:").nth(1).expect("harness-full");
    assert!(
        harness.contains("--features burst --test harness_rules"),
        "harness-full no longer runs harness_rules"
    );
}

#[test]
fn the_deployed_workflow_installs_nothing_it_does_not_use() {
    let w = read(".github/workflows/deployed-burst.yml");
    let code = code(&w).join("\n");
    assert!(
        !code.contains("casey/just") && !code.contains("JUST_VERSION"),
        "deployed-burst installs just and never calls it"
    );
    let calls_just = code.lines().any(|l| {
        l.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .any(|w| w == "just")
    });
    assert!(!calls_just, "deployed-burst calls just: then install it");
}

#[test]
fn the_deployed_workflow_is_confirmed_and_bound_to_an_environment() {
    let w = read(".github/workflows/deployed-burst.yml");
    let want = "no meetup is running and the restart wipes every room";
    // The confirm job: no environment, no secret, and it compares the input
    // with exactly the sentence.
    let confirm = job(&w, "confirm");
    assert!(
        !confirm
            .iter()
            .any(|l| l.contains("secrets.") || l.contains("environment:")),
        "{confirm:#?}"
    );
    assert!(
        confirm
            .iter()
            .any(|l| l.contains("CONFIRM: ${{ inputs.confirm }}")),
        "{confirm:#?}"
    );
    assert!(
        confirm.iter().any(|l| l.trim() == format!("want='{want}'")),
        "{confirm:#?}"
    );
    assert!(
        confirm
            .iter()
            .any(|l| l.contains(r#"[ "$CONFIRM" = "$want" ] ||"#) && l.contains("exit 2")),
        "{confirm:#?}"
    );
    // The job that reads the secrets waits for it and runs in the environment.
    let run = job(&w, "deployed-burst");
    assert!(run.iter().any(|l| l.trim() == "needs: confirm"), "{run:#?}");
    assert!(
        run.iter()
            .any(|l| l.trim() == "environment: deployed-burst"),
        "{run:#?}"
    );
    assert!(run
        .iter()
        .any(|l| l.contains("secrets.POPQUIZ_ORGANIZER_SESSION")));
    // Every secret read is in that job.
    let elsewhere: Vec<&str> = code(&w)
        .into_iter()
        .filter(|l| l.contains("secrets.") && !run.contains(l))
        .collect();
    assert!(elsewhere.is_empty(), "{elsewhere:#?}");
    // The input exists and is required.
    let inputs = w.split("      confirm:").nth(1).expect("a confirm input");
    assert!(
        inputs.lines().take(4).any(|l| l.trim() == "required: true"),
        "the confirm input is not required"
    );
}

#[test]
fn the_deployed_burst_runs_against_the_cap_fly_toml_states() {
    let w = read(".github/workflows/deployed-burst.yml");
    let run = job(&w, "deployed-burst").join("\n");
    assert!(
        run.contains(
            r#"tomllib.load(open("fly.toml", "rb"))["http_service"]["concurrency"]["hard_limit"]"#
        ),
        "the cap is not read from fly.toml"
    );
    assert!(
        run.contains(r#"--connection-cap "$CAP""#),
        "burst is not given the cap"
    );
    assert!(run.contains("CAP: ${{ steps.cap.outputs.cap }}"));
    assert!(
        !run.contains("> fly.toml") && !run.contains(">> fly.toml"),
        "the workflow writes fly.toml"
    );
    // Over the cap it stops before the room is touched.
    let cap_step = run.split("id: cap").nth(1).expect("the cap step");
    let cap_step = cap_step.split("\n      - name:").next().unwrap();
    assert!(
        cap_step.contains(r#"[ "$peak" -le "$cap" ] ||"#) && cap_step.contains("exit 2"),
        "the cap step does not stop an over-cap run: {cap_step}"
    );
    let (before, after) = run.split_once("id: cap").unwrap();
    assert!(
        !before.contains("$ROOM_URL/") && after.contains("$ROOM_URL/admin/questions"),
        "something reaches the room before the cap check"
    );
}

// The deployed burst's two credentials (GAP-19, SPEC §8.4). They are secrets of
// the `deployed-burst` environment, and this keeps every read of them where
// the workflow's header says: one job, bound to that environment, run by hand,
// reading them only through `env:`.

const BURST_WORKFLOW: &str = "deployed-burst.yml";
const BURST_SECRETS: [&str; 2] = ["POPQUIZ_ORGANIZER_SESSION", "POPQUIZ_ADMIN_TOKEN"];

fn names_a_burst_secret(line: &str) -> bool {
    BURST_SECRETS.iter().any(|s| line.contains(s))
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Every file in `.github/workflows/`, by file name.
fn workflows() -> Vec<(String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows");
    let mut out: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{dir:?}: {e}"))
        .map(|entry| {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                text,
            )
        })
        .collect();
    out.sort();
    out
}

/// Each top-level job of a workflow: its name and the indexes of its lines
/// in `lines` (the workflow's code lines).
fn job_spans(lines: &[&str]) -> Vec<(String, Range<usize>)> {
    let start = lines.iter().position(|l| *l == "jobs:").expect("no jobs:") + 1;
    let end = lines[start..]
        .iter()
        .position(|l| !l.is_empty() && indent(l) == 0)
        .map_or(lines.len(), |p| start + p);
    let heads: Vec<usize> = (start..end)
        .filter(|&i| indent(lines[i]) == 2 && lines[i].trim_end().ends_with(':'))
        .collect();
    heads
        .iter()
        .enumerate()
        .map(|(k, &head)| {
            let stop = heads.get(k + 1).copied().unwrap_or(end);
            let name = lines[head].trim().trim_end_matches(':').to_string();
            (name, head..stop)
        })
        .collect()
}

#[test]
fn the_burst_secrets_are_named_by_no_other_workflow() {
    let all = workflows();
    assert!(
        all.iter().any(|(name, _)| name == BURST_WORKFLOW) && all.len() >= 3,
        "the scan found {:?}",
        all.iter().map(|(name, _)| name).collect::<Vec<_>>()
    );
    for (name, text) in all.iter().filter(|(name, _)| name != BURST_WORKFLOW) {
        let hits: Vec<&str> = code(text)
            .into_iter()
            .filter(|l| names_a_burst_secret(l) || l.contains("deployed-burst"))
            .collect();
        assert!(
            hits.is_empty(),
            "{name} names a deployed-burst secret or its environment; only {BURST_WORKFLOW} reads them: {hits:#?}"
        );
    }
}

#[test]
fn the_burst_secrets_are_read_only_in_the_job_bound_to_the_environment() {
    let w = read(".github/workflows/deployed-burst.yml");
    let lines = code(&w);
    let bound: Vec<Range<usize>> = job_spans(&lines)
        .into_iter()
        .filter(|(_, span)| {
            lines[span.clone()]
                .iter()
                .any(|l| l.trim() == "environment: deployed-burst")
        })
        .map(|(_, span)| span)
        .collect();
    assert!(
        !bound.is_empty(),
        "no job declares environment: deployed-burst"
    );
    let stray: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(i, l)| names_a_burst_secret(l) && !bound.iter().any(|span| span.contains(i)))
        .map(|(_, l)| *l)
        .collect();
    assert!(
        stray.is_empty(),
        "read outside a job bound to the deployed-burst environment: {stray:#?}"
    );
    let wholesale: Vec<&str> = lines
        .iter()
        .filter(|l| {
            let l = l.to_ascii_lowercase();
            l.contains("tojson(secrets") || l.contains("secrets: inherit")
        })
        .copied()
        .collect();
    assert!(
        wholesale.is_empty(),
        "every secret, wholesale: {wholesale:#?}"
    );
}

#[test]
fn the_deployed_workflow_runs_only_by_hand() {
    let w = read(".github/workflows/deployed-burst.yml");
    let lines = code(&w);
    // YAML also reads `"on"`, `'on'` and `true` as the trigger key.
    let on_keys: Vec<&str> = lines
        .iter()
        .filter(|l| !l.is_empty() && indent(l) == 0)
        .filter(|l| {
            let key = l.split(':').next().unwrap().trim_matches(['"', '\'']);
            key == "on" || key == "true"
        })
        .copied()
        .collect();
    assert_eq!(on_keys, ["on:"], "the trigger is not one block `on:`");
    let at = lines.iter().position(|l| *l == "on:").unwrap();
    let triggers: Vec<&str> = lines[at + 1..]
        .iter()
        .take_while(|l| l.is_empty() || indent(l) > 0)
        .filter(|l| indent(l) == 2)
        .map(|l| l.trim())
        .collect();
    assert_eq!(
        triggers,
        ["workflow_dispatch:"],
        "deployed-burst runs on something other than workflow_dispatch"
    );
}

#[test]
fn the_burst_secrets_reach_a_step_only_through_env() {
    let w = read(".github/workflows/deployed-burst.yml");
    let lines = code(&w);
    let mut bad: Vec<&str> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !names_a_burst_secret(line) {
            continue;
        }
        if line.contains("${{") {
            // `NAME: ${{ secrets.NAME }}`, or a presence check, under `env:`.
            let parent = lines[..i]
                .iter()
                .rev()
                .find(|p| !p.trim().is_empty() && indent(p) < indent(line));
            let in_env = parent.is_some_and(|p| matches!(p.trim(), "env:" | "- env:"));
            let (key, value) = line.trim().split_once(": ").unwrap_or_default();
            let mapped = BURST_SECRETS.iter().any(|s| {
                (key == *s && value == format!("${{{{ secrets.{s} }}}}"))
                    || value == format!("${{{{ secrets.{s} != '' }}}}")
            });
            if !(in_env && mapped) {
                bad.push(*line);
            }
            continue;
        }
        // In a script: the session never (the binaries read it from their
        // environment); the token only as printf's argument, piped to
        // `curl --config -` so it is in no process's arguments.
        let shell = |s: &str| line.contains(&format!("${s}")) || line.contains(&format!("${{{s}"));
        if shell("POPQUIZ_ORGANIZER_SESSION") {
            bad.push(*line);
        }
        if shell("POPQUIZ_ADMIN_TOKEN") {
            let t = line.trim();
            let piped = t
                .split_once("printf ")
                .is_some_and(|(_, rest)| rest.contains("$POPQUIZ_ADMIN_TOKEN"))
                && t.ends_with('|')
                && !t.contains('>')
                && !t.contains("echo")
                && lines.get(i + 1).is_some_and(|next| {
                    next.trim().starts_with("curl ") && next.contains("--config -")
                });
            if !piped {
                bad.push(*line);
            }
        }
    }
    assert!(
        bad.is_empty(),
        "a deployed-burst secret reaches a step other than through an env: mapping: {bad:#?}"
    );
    // Nothing traces a script or dumps its environment into the log.
    let traced: Vec<&str> = lines
        .iter()
        .filter(|l| {
            let t = l.trim();
            let set_x = t.starts_with("set ")
                && t.split_whitespace().skip(1).any(|w| {
                    (w.starts_with('-') && !w.starts_with("--") && w.contains('x')) || w == "xtrace"
                });
            set_x
                || t.contains("printenv")
                || t == "env"
                || t.starts_with("env |")
                || t.starts_with("env >")
        })
        .copied()
        .collect();
    assert!(traced.is_empty(), "{traced:#?}");
}

#[test]
fn ci_bounds_test_full_and_runs_the_harness_rules() {
    let ci = read(".github/workflows/ci.yml");
    let full = job(&ci, "test-full");
    assert!(
        full.iter()
            .any(|l| l.trim().starts_with("timeout-minutes:")),
        "test-full has no job timeout: {full:#?}"
    );
    let fast = job(&ci, "test").join("\n");
    assert!(
        fast.contains("--test harness_rules"),
        "the fast job does not run harness_rules"
    );
}
