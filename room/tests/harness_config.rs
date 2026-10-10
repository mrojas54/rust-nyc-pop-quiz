//! The harness's own wiring (PQ-41): the justfile and the two workflows say
//! what the review of PR #39 asked them to, and keep saying it. Line-shaped
//! checks on the files themselves — no YAML parser is in the lock — each
//! aimed at the line that does the work, not at a comment that names it.

use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"))
}

/// Lines that are not comments.
fn code(text: &str) -> Vec<&str> {
    text.lines().filter(|l| !l.trim_start().starts_with('#')).collect()
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
    assert!(!code(&j).iter().any(|l| l.starts_with("_pending")), "the _pending recipe is back; nothing calls it");
    let full = code(&j).into_iter().find(|l| l.starts_with("test-full:")).expect("test-full");
    let deps: Vec<&str> = full["test-full:".len()..].split_whitespace().collect();
    assert!(deps.contains(&"test"), "{full}");
    assert!(!deps.contains(&"a11y"), "test-full runs a11y twice: `test` already runs it through test-web: {full}");
    let harness = j.split("\nharness-full:").nth(1).expect("harness-full");
    assert!(harness.contains("--features burst --test harness_rules"), "harness-full no longer runs harness_rules");
}

#[test]
fn the_deployed_workflow_installs_nothing_it_does_not_use() {
    let w = read(".github/workflows/deployed-burst.yml");
    let code = code(&w).join("\n");
    assert!(!code.contains("casey/just") && !code.contains("JUST_VERSION"), "deployed-burst installs just and never calls it");
    assert!(!code.contains("just "), "deployed-burst calls just: then install it");
}

#[test]
fn the_deployed_workflow_is_confirmed_and_bound_to_an_environment() {
    let w = read(".github/workflows/deployed-burst.yml");
    let want = "no meetup is running and the restart wipes every room";
    // The confirm job: no environment, no secret, and it compares the input
    // with exactly the sentence.
    let confirm = job(&w, "confirm");
    assert!(!confirm.iter().any(|l| l.contains("secrets.") || l.contains("environment:")), "{confirm:#?}");
    assert!(confirm.iter().any(|l| l.contains("CONFIRM: ${{ inputs.confirm }}")), "{confirm:#?}");
    assert!(confirm.iter().any(|l| l.trim() == format!("want='{want}'")), "{confirm:#?}");
    assert!(confirm.iter().any(|l| l.contains(r#"[ "$CONFIRM" = "$want" ] ||"#) && l.contains("exit 2")), "{confirm:#?}");
    // The job that reads the secrets waits for it and runs in the environment.
    let run = job(&w, "deployed-burst");
    assert!(run.iter().any(|l| l.trim() == "needs: confirm"), "{run:#?}");
    assert!(run.iter().any(|l| l.trim() == "environment: deployed-burst"), "{run:#?}");
    assert!(run.iter().any(|l| l.contains("secrets.POPQUIZ_ORGANIZER_SESSION")));
    // Every secret read is in that job.
    let elsewhere: Vec<&str> = code(&w).into_iter().filter(|l| l.contains("secrets.") && !run.contains(l)).collect();
    assert!(elsewhere.is_empty(), "{elsewhere:#?}");
    // The input exists and is required.
    let inputs = w.split("      confirm:").nth(1).expect("a confirm input");
    assert!(inputs.lines().take(4).any(|l| l.trim() == "required: true"), "the confirm input is not required");
}

#[test]
fn the_deployed_burst_runs_against_the_cap_fly_toml_states() {
    let w = read(".github/workflows/deployed-burst.yml");
    let run = job(&w, "deployed-burst").join("\n");
    assert!(run.contains(r#"tomllib.load(open("fly.toml", "rb"))["http_service"]["concurrency"]["hard_limit"]"#), "the cap is not read from fly.toml");
    assert!(run.contains(r#"--connection-cap "$CAP""#), "burst is not given the cap");
    assert!(run.contains("CAP: ${{ steps.cap.outputs.cap }}"));
    assert!(!run.contains("> fly.toml") && !run.contains(">> fly.toml"), "the workflow writes fly.toml");
}

#[test]
fn ci_bounds_test_full_and_runs_the_harness_rules() {
    let ci = read(".github/workflows/ci.yml");
    let full = job(&ci, "test-full");
    assert!(full.iter().any(|l| l.trim().starts_with("timeout-minutes:")), "test-full has no job timeout: {full:#?}");
    let fast = job(&ci, "test").join("\n");
    assert!(fast.contains("--test harness_rules"), "the fast job does not run harness_rules");
}
