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
// reading them only through a step's `env:`, in a workflow that is the one
// pinned below. GitHub reads secret and environment names in any case, and
// substitutes `${{ }}` into a script before it runs, comment lines included,
// so these checks read raw lines and match names in lowercase.

const BURST_WORKFLOW: &str = "deployed-burst.yml";
const BURST_SECRETS: [&str; 2] = ["POPQUIZ_ORGANIZER_SESSION", "POPQUIZ_ADMIN_TOKEN"];

/// `deployed-burst.yml` as checked against SPEC §8.4, byte for byte. Any step
/// of the job that holds a secret can reach it without naming it: through its
/// own environment (`BASH_FUNC_printf%%` replaces the shell's printf), a file
/// an earlier step left (`~/.curlrc`), an action it runs, or the shell. So any
/// change to the file is a change to the credential path.
const BURST_PIN: &str = include_str!("fixtures/deployed-burst.pinned.yml");

/// Every line, in every workflow, that reads the secrets context. GitHub
/// substitutes `${{ }}` anywhere in a value, and `if:` is an expression
/// without the braces.
const SECRET_READS: [(&str, &str); 5] = [
    ("deploy.yml", "FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}"),
    (
        "deployed-burst.yml",
        "HAS_SESSION: ${{ secrets.POPQUIZ_ORGANIZER_SESSION != '' }}",
    ),
    (
        "deployed-burst.yml",
        "HAS_ADMIN: ${{ secrets.POPQUIZ_ADMIN_TOKEN != '' }}",
    ),
    (
        "deployed-burst.yml",
        "POPQUIZ_ADMIN_TOKEN: ${{ secrets.POPQUIZ_ADMIN_TOKEN }}",
    ),
    (
        "deployed-burst.yml",
        "POPQUIZ_ORGANIZER_SESSION: ${{ secrets.POPQUIZ_ORGANIZER_SESSION }}",
    ),
];

fn names_a_burst_secret(line: &str) -> bool {
    let line = line.to_ascii_lowercase();
    BURST_SECRETS
        .iter()
        .any(|s| line.contains(&s.to_ascii_lowercase()))
}

/// Lowercase, without whitespace: `toJSON( secrets )` reads as `tojson(secrets)`.
fn squash(line: &str) -> String {
    line.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Reads every secret at once, or one picked by a computed key.
fn reads_secrets_wholesale(line: &str) -> bool {
    let s = squash(line);
    s.contains("(secrets")
        || s.contains(",secrets")
        || s.contains("secrets[")
        || s.contains("secrets.*")
        || s.trim_start_matches('-').starts_with("secrets:")
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// A line's key, if it is `key: …` (quotes dropped, lowercase).
fn key(line: &str) -> Option<String> {
    let t = line.trim().trim_start_matches("- ");
    let (k, _) = t.split_once(':')?;
    let k = k.trim().trim_matches(['"', '\'']);
    (!k.is_empty()
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
    .then(|| k.to_ascii_lowercase())
}

/// Which lines sit inside a block scalar (`run: |` and the like): there a `#`
/// line is script, not a YAML comment, and a trailing `|` is a shell pipe.
fn in_block(lines: &[&str]) -> Vec<bool> {
    let mut out = Vec::with_capacity(lines.len());
    let mut block: Option<usize> = None;
    for line in lines {
        if let Some(at) = block {
            if line.trim().is_empty() || indent(line) > at {
                out.push(true);
                continue;
            }
            block = None;
        }
        out.push(false);
        let t = line.trim_end();
        if ["|", "|-", "|+", ">", ">-", ">+"]
            .iter()
            .any(|ind| t.ends_with(&format!(": {ind}")))
        {
            block = Some(indent(line));
        }
    }
    out
}

/// YAML comments: inert, unlike a `#` line inside a script.
fn yaml_comment(lines: &[&str], block: &[bool], i: usize) -> bool {
    !block[i] && lines[i].trim_start().starts_with('#')
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

/// Each top-level job: its name and the indexes of its raw lines. Every line
/// at job depth must be a plain `name:` heading, or the scan refuses it.
fn job_spans(lines: &[&str]) -> Vec<(String, Range<usize>)> {
    let block = in_block(lines);
    let start = lines.iter().position(|l| *l == "jobs:").expect("no jobs:") + 1;
    let end = (start..lines.len())
        .find(|&i| {
            !lines[i].trim().is_empty() && indent(lines[i]) == 0 && !yaml_comment(lines, &block, i)
        })
        .unwrap_or(lines.len());
    let heads: Vec<usize> = (start..end)
        .filter(|&i| {
            indent(lines[i]) == 2 && !lines[i].trim().is_empty() && !yaml_comment(lines, &block, i)
        })
        .collect();
    heads
        .iter()
        .enumerate()
        .map(|(k, &head)| {
            let (name, rest) = lines[head].trim().split_once(':').unwrap_or_else(|| {
                panic!(
                    "a line at job depth that is not a job heading: {:?}",
                    lines[head]
                )
            });
            let rest = rest.trim();
            assert!(
                key(lines[head]).is_some() && (rest.is_empty() || rest.starts_with('#')),
                "a job heading the scan cannot read: {:?}",
                lines[head]
            );
            let stop = heads.get(k + 1).copied().unwrap_or(end);
            (name.to_string(), head..stop)
        })
        .collect()
}

#[test]
fn the_deployed_workflow_is_the_one_checked_against_the_contract() {
    let now = read(".github/workflows/deployed-burst.yml");
    let now: Vec<&str> = now.lines().collect();
    let pin: Vec<&str> = BURST_PIN.lines().collect();
    if let Some(i) = (0..now.len().max(pin.len())).find(|&i| now.get(i) != pin.get(i)) {
        panic!(
            "{BURST_WORKFLOW} differs from its pin at line {}:\n  now:    {:?}\n  pinned: {:?}\n\
             Every step of the job that holds the two secrets can reach them. Check the change \
             against SPEC §8.4 (nothing echoed, on a command line or in a file), then copy the \
             file to popquiz/tests/fixtures/deployed-burst.pinned.yml",
            i + 1,
            now.get(i),
            pin.get(i)
        );
    }
    assert_eq!(
        read(".github/workflows/deployed-burst.yml"),
        BURST_PIN,
        "{BURST_WORKFLOW} differs from its pin in line endings"
    );
}

#[test]
fn every_workflow_reads_secrets_only_on_the_listed_lines() {
    for (name, text) in workflows() {
        let lines: Vec<&str> = text.lines().collect();
        let block = in_block(&lines);
        let mut bad: Vec<String> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let at = format!("{name}:{}: {line}", i + 1);
            // A read of the secrets context, wherever it sits, comments too.
            let expression =
                line.contains("${{") || line.contains("}}") || key(line).as_deref() == Some("if");
            if expression
                && squash(line).contains("secrets")
                && !SECRET_READS.contains(&(name.as_str(), line.trim()))
            {
                bad.push(format!("an unlisted read of secrets: {at}"));
            }
            // Every expression opens and closes on one line, so the check
            // above sees it whole.
            if line.matches("${{").count() != line.matches("}}").count()
                || line.split("${{").skip(1).any(|rest| !rest.contains("}}"))
            {
                bad.push(format!("an expression split across lines: {at}"));
            }
            if block[i] || yaml_comment(&lines, &block, i) {
                continue;
            }
            // Outside a script, no escape: a double-quoted YAML string decodes
            // `\x24` to `$` and `\` at a line's end joins two lines.
            if line.contains('\\') {
                bad.push(format!("a backslash outside a script: {at}"));
            }
            // No block scalar but a script or a path list: a folded value
            // (`>-`) joins lines into one name or one expression.
            let t = line.trim().trim_start_matches("- ");
            let value = match key(line) {
                Some(_) => t.split_once(':').unwrap().1.trim(),
                None => t,
            };
            if value.starts_with(['|', '>'])
                && !(value == "|" && matches!(key(line).as_deref(), Some("run" | "path")))
            {
                bad.push(format!(
                    "a block scalar other than `run: |` or `path: |`: {at}"
                ));
            }
        }
        assert!(
            bad.is_empty(),
            "{name} (SPEC §8.4; a new secret read goes in SECRET_READS): {bad:#?}"
        );
    }
}

#[test]
fn the_burst_secrets_and_environment_are_named_by_no_other_workflow() {
    let all = workflows();
    assert!(
        all.iter().any(|(name, _)| name == BURST_WORKFLOW) && all.len() >= 3,
        "the scan found {:?}",
        all.iter().map(|(name, _)| name).collect::<Vec<_>>()
    );
    for (name, text) in all.iter().filter(|(name, _)| name != BURST_WORKFLOW) {
        let lines: Vec<&str> = text.lines().collect();
        let mut hits: Vec<&str> = lines
            .iter()
            .filter(|l| names_a_burst_secret(l) || reads_secrets_wholesale(l))
            .copied()
            .collect();
        // An environment named deployed-burst, in any case, or by an expression.
        for (i, line) in lines.iter().enumerate() {
            if key(line).as_deref() != Some("environment") {
                continue;
            }
            let value = line.split_once(':').unwrap().1.trim();
            let value = if value.is_empty() || value.starts_with('#') {
                lines[i + 1..]
                    .iter()
                    .find(|l| key(l).as_deref() == Some("name"))
                    .map_or("", |l| l.split_once(':').unwrap().1.trim())
            } else {
                value
            };
            if value.contains("${{") || squash(value).contains("deployed-burst") {
                hits.push(line);
            }
        }
        assert!(
            hits.is_empty(),
            "{name} reads a deployed-burst secret, every secret, or the deployed-burst environment; only {BURST_WORKFLOW} may (SPEC §8.4): {hits:#?}"
        );
    }
}

#[test]
fn the_burst_secrets_are_read_only_in_the_job_bound_to_the_environment() {
    let w = read(".github/workflows/deployed-burst.yml");
    let lines: Vec<&str> = w.lines().collect();
    let block = in_block(&lines);
    let bound: Vec<(String, Range<usize>)> = job_spans(&lines)
        .into_iter()
        .filter(|(_, span)| {
            span.clone()
                .any(|i| indent(lines[i]) == 4 && lines[i].trim() == "environment: deployed-burst")
        })
        .collect();
    assert_eq!(
        bound
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["deployed-burst"],
        "the job-level `environment: deployed-burst` is not on the deployed-burst job alone"
    );
    let span = &bound[0].1;
    let stray: Vec<&str> = (0..lines.len())
        .filter(|&i| {
            names_a_burst_secret(lines[i]) && !yaml_comment(&lines, &block, i) && !span.contains(&i)
        })
        .map(|i| lines[i])
        .collect();
    assert!(
        stray.is_empty(),
        "read outside the job bound to the deployed-burst environment: {stray:#?}"
    );
    let wholesale: Vec<&str> = lines
        .iter()
        .filter(|l| reads_secrets_wholesale(l))
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
    let lines: Vec<&str> = w.lines().collect();
    let block = in_block(&lines);
    // Every expression closes on its own line, so the scan below sees it whole.
    let open: Vec<&str> = lines
        .iter()
        .filter(|l| l.split("${{").skip(1).any(|rest| !rest.contains("}}")))
        .copied()
        .collect();
    assert!(
        open.is_empty(),
        "an expression split across lines: {open:#?}"
    );
    // An expression naming a secret is `NAME: ${{ secrets.NAME }}`, or a
    // presence check, under a step's `env:` — never in a script, a `with:`,
    // a job's or the workflow's `env:`.
    let mut bad: Vec<&str> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let names_secret = line
            .split("${{")
            .skip(1)
            .map(|rest| rest.split("}}").next().unwrap())
            .any(|expr| squash(expr).contains("secrets") || names_a_burst_secret(expr));
        if !names_secret {
            continue;
        }
        let parent = (0..i).rev().find(|&p| {
            !lines[p].trim().is_empty()
                && !yaml_comment(&lines, &block, p)
                && indent(lines[p]) < indent(line)
        });
        let step_env = !block[i]
            && parent.is_some_and(|p| {
                (indent(lines[p]) == 8 && lines[p].trim() == "env:")
                    || (indent(lines[p]) == 6 && lines[p].trim() == "- env:")
            });
        let (key, value) = line.trim().split_once(": ").unwrap_or_default();
        let mapped = BURST_SECRETS.iter().any(|s| {
            (key == *s && value == format!("${{{{ secrets.{s} }}}}"))
                || value == format!("${{{{ secrets.{s} != '' }}}}")
        });
        if !(step_env && mapped) {
            bad.push(line);
        }
    }
    assert!(
        bad.is_empty(),
        "a secret reaches the workflow other than through a step's env: mapping: {bad:#?}"
    );
    // Nothing changes how a step's shell runs, or what the next step inherits.
    let rewired: Vec<&str> = (0..lines.len())
        .filter(|&i| !yaml_comment(&lines, &block, i))
        .filter(|&i| {
            let l = lines[i];
            (!block[i] && matches!(key(l).as_deref(), Some("shell" | "defaults")))
                || ["GITHUB_ENV", "GITHUB_PATH", "BASH_ENV", "SHELLOPTS"]
                    .iter()
                    .any(|v| l.contains(v))
        })
        .map(|i| lines[i])
        .collect();
    assert!(rewired.is_empty(), "{rewired:#?}");
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
