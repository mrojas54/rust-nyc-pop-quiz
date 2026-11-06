//! Credential policy for GitHub workflows (PQ-61, SPEC §8.4).
use std::path::PathBuf;

const BURST: &str = "deployed-burst.yml";
const PIN: &str = include_str!("fixtures/deployed-burst.pinned.yml");

// YAML decoding is the parser's job. This policy walks decoded values, including
// aliases at their use sites. It never infers structure from source lines.
fn validate(name: &str, text: &str) -> Result<(), String> {
    if name == BURST && text != PIN {
        return Err(
            "burst workflow changed: review every step against SPEC §8.4 before updating its pin"
                .into(),
        );
    }
    let root: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
    let jobs = root
        .get("jobs")
        .and_then(|v| v.as_mapping())
        .ok_or("jobs must be a mapping")?;
    for (id, job) in jobs {
        let id = id.as_str().ok_or("job IDs must be strings")?;
        let job = job.as_mapping().ok_or("each job must be a mapping")?;
        for key in job.keys().filter_map(|key| key.as_str()) {
            if ["environment", "secrets"]
                .iter()
                .any(|reserved| key.eq_ignore_ascii_case(reserved) && key != *reserved)
            {
                return Err(format!(
                    "jobs.{id}.{key}: use the canonical lowercase property name"
                ));
            }
        }
        if let Some(environment) = job.get("environment") {
            let environment = environment
                .as_str()
                .or_else(|| environment.get("name").and_then(|v| v.as_str()))
                .ok_or("environment must have a literal string name")?;
            if environment.contains("${{") {
                return Err(format!(
                    "jobs.{id}.environment: computed environments require review"
                ));
            }
            if environment.trim().eq_ignore_ascii_case("deployed-burst")
                && !(name == BURST && id == "deployed-burst")
            {
                return Err(format!(
                    "jobs.{id}.environment: only the burst job may bind deployed-burst"
                ));
            }
        }
        if job.contains_key("secrets") {
            return Err(format!(
                "jobs.{id}.secrets: reusable-workflow secret forwarding requires review"
            ));
        }
    }
    if name == BURST {
        let triggers = root
            .get("on")
            .and_then(|v| v.as_mapping())
            .ok_or("missing triggers")?;
        if triggers.len() != 1 || !triggers.contains_key("workflow_dispatch") {
            return Err("burst must run only by hand".into());
        }
        if root["jobs"]["deployed-burst"]["environment"].as_str() != Some("deployed-burst") {
            return Err("burst job must bind its environment".into());
        }
    }
    walk(name, &root, &mut Vec::new())
}

fn approved_read(name: &str, path: &[String], value: &str) -> bool {
    // Actual tree path, not a line that happens to look like an env entry.
    if path.len() != 6 || path[0] != "jobs" || path[2] != "steps" || path[4] != "env" {
        return false;
    }
    if name == "deploy.yml" && path[1] == "deploy" && path[3] == "2" && path[5] == "FLY_API_TOKEN" {
        return value == "${{ secrets.FLY_API_TOKEN }}";
    }
    if name != BURST || path[1] != "deployed-burst" {
        return false;
    }
    matches!(
        (path[5].as_str(), value),
        (
            "HAS_SESSION",
            "${{ secrets.POPQUIZ_ORGANIZER_SESSION != '' }}"
        ) | ("HAS_ADMIN", "${{ secrets.POPQUIZ_ADMIN_TOKEN != '' }}")
            | ("POPQUIZ_ADMIN_TOKEN", "${{ secrets.POPQUIZ_ADMIN_TOKEN }}")
            | (
                "POPQUIZ_ORGANIZER_SESSION",
                "${{ secrets.POPQUIZ_ORGANIZER_SESSION }}"
            )
    )
}

fn walk(name: &str, node: &serde_yaml_ng::Value, path: &mut Vec<String>) -> Result<(), String> {
    use serde_yaml_ng::Value;
    match node {
        Value::Mapping(map) => {
            for (key, value) in map {
                let key = key.as_str().ok_or("mapping keys must be strings")?;
                // GitHub does not support YAML merge keys; don't silently give
                // them different semantics from the workflow runner.
                if key == "<<" || key.contains("${{") {
                    return Err(format!(
                        "{}: merge or expression key requires review",
                        path.join(".")
                    ));
                }
                path.push(key.into());
                walk(name, value, path)?;
                path.pop();
            }
        }
        Value::Sequence(items) => {
            for (i, value) in items.iter().enumerate() {
                path.push(i.to_string());
                walk(name, value, path)?;
                path.pop();
            }
        }
        Value::String(value) => {
            // Deliberately conservative: do not implement a second expression
            // evaluator. Any reference to secrets in an expression-bearing
            // scalar must be a complete, approved value at its approved path.
            // This includes multiline expressions and braces inside strings.
            let expression =
                value.contains("${{") || path.last().is_some_and(|k| k.eq_ignore_ascii_case("if"));
            if expression
                && value.to_ascii_lowercase().contains("secrets")
                && !approved_read(name, path, value)
            {
                return Err(format!("{}: unapproved secret expression", path.join(".")));
            }
        }
        Value::Tagged(_) => {
            return Err(format!(
                "{}: custom YAML tags require review",
                path.join(".")
            ))
        }
        _ => {}
    }
    Ok(())
}

fn rejects(name: &str, text: &str) {
    serde_yaml_ng::from_str::<serde_yaml_ng::Value>(text)
        .unwrap_or_else(|e| panic!("invalid negative fixture: {e}"));
    assert!(
        validate(name, text).is_err(),
        "accepted forbidden workflow: {text}"
    );
}

#[test]
fn checked_in_workflows_obey_the_credential_policy() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows");
    let mut found_burst = false;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if !matches!(
            path.extension().and_then(|x| x.to_str()),
            Some("yml" | "yaml")
        ) {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        found_burst |= name == BURST;
        let text = std::fs::read_to_string(&path).unwrap();
        validate(name, &text).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    assert!(found_burst);
}

#[test]
fn decoded_yaml_cannot_hide_the_burst_environment() {
    for job in [
        r#"{runs-on: ubuntu-latest, environment: deployed-burst, steps: [{run: 'true'}]}"#,
        r#"{environment: {name: DEPLOYED-BURST}}"#,
        r#"{Environment: deployed-burst}"#,
        r#"{environment: ' deployed-burst '}"#,
        "\n    environment: >-\n      deployed-burst",
        r#"{environment: "deployed-\u0062urst"}"#,
        r#"{environment: "${{ inputs.environment }}"}"#,
    ] {
        rejects(
            "other.yaml",
            &format!("jobs:\n# comment: |\n  probe: {job}\n"),
        );
    }
}

#[test]
fn decoded_yaml_cannot_hide_secret_reads() {
    for value in [
        r#""${{ s\u0065crets.POPQUIZ_ADMIN_\u0054OKEN }}""#,
        "${{ secrets.popquiz_organizer_session }}",
        "${{ toJSON(true && secrets) }}",
        "${{ secrets['POPQUIZ_ADMIN_TOKEN'] }}",
        "${{ secrets[inputs.key] }}",
        "${{ format('}}{0}', toJSON(secrets)) }}",
        ">-\n            ${{ toJSON(\n            secrets) }}",
    ] {
        rejects("other.yml", &format!("jobs:\n# comment: |\n  probe:\n    steps:\n      - env:\n          TOKEN: {value}\n"));
    }
    rejects("other.yml", "jobs: {probe: {secrets: inherit}}");
    rejects(
        "other.yml",
        "jobs: {probe: {if: secrets.FLY_API_TOKEN != ''}}",
    );
}

#[test]
fn the_fly_exception_requires_the_exact_structural_location() {
    let deploy = include_str!("../../.github/workflows/deploy.yml");
    assert!(validate("deploy.yml", deploy).is_ok());
    rejects("other.yml", deploy);
    rejects("deploy.yml", &format!("{deploy}\n      - run: |\n          cat > /tmp/token <<'END'\n          FLY_API_TOKEN: ${{{{ secrets.FLY_API_TOKEN }}}}\n          END\n"));
    rejects(
        "deploy.yml",
        &deploy.replace("        env:", "        with:"),
    );
    rejects(
        "deploy.yml",
        &deploy.replace(
            "jobs:\n",
            "env: {FLY_API_TOKEN: '${{ secrets.FLY_API_TOKEN }}'}\njobs:\n",
        ),
    );
}

#[test]
fn inserting_a_step_does_not_move_a_secret_out_of_its_env_mapping() {
    let deploy = include_str!("../../.github/workflows/deploy.yml");
    let with_step = deploy.replace("    steps:\n", "    steps:\n      - run: echo ready\n");
    validate("deploy.yml", &with_step).expect("step positions are not credential policy");
}

#[test]
fn burst_yaml_formatting_preserves_the_reviewed_execution() {
    // Serialization removes YAML comments and changes scalar presentation;
    // decoded scripts, expressions and job configuration must stay the same.
    let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(PIN).unwrap();
    let reformatted = serde_yaml_ng::to_string(&parsed).unwrap();
    assert_ne!(PIN, reformatted);
    validate(BURST, &reformatted).expect("equivalent YAML needs no new baseline");
}

#[test]
fn harmless_yaml_forms_are_accepted() {
    for text in [
        "jobs: {test: {environment: staging, steps: [{run: 'true'}]}}",
        "jobs:\n  test:\n    steps:\n      - run: | # shell comment\n          printf '%s\\n' hello\n        with:\n          restore-keys: |\n            cargo-\n",
        "jobs:\n  test:\n    steps:\n      - run: printf '%s\\n' hello\n",
        "# comment: |\njobs: {test: {steps: [{run: 'true'}]}}",
        "jobs: {test: {steps: [{run: '${{\n inputs.command\n }}'}]}}",
        "jobs: {test: {steps: [{run: &hello 'echo hello'}, {run: *hello}]}}",
    ] {
        validate("ci.yml", text).unwrap_or_else(|e| panic!("{text}: {e}"));
    }
}

#[test]
fn ambiguous_yaml_fails_closed() {
    for text in [
        "jobs: {}\njobs: {probe: {environment: deployed-burst}}",
        "jobs: {}\n---\njobs: {}",
        "jobs: {probe: !custom {environment: staging}}",
        "jobs: {probe: {<<: {environment: staging}}}",
        "jobs: [",
        "jobs: {probe: {environment: [staging]}}",
    ] {
        assert!(
            validate("other.yml", text).is_err(),
            "accepted ambiguous YAML: {text}"
        );
    }
}

#[test]
fn aliases_are_checked_at_every_use() {
    rejects("other.yml", "env: &credentials {TOKEN: '${{ secrets.POPQUIZ_ADMIN_TOKEN }}'}\njobs: {probe: {env: *credentials}}");
    rejects(
        "other.yml",
        "name: &protected deployed-burst\njobs: {probe: {environment: *protected}}",
    );
    // The anchor itself is permitted. Only the alias destination can reject it.
    let deploy = include_str!("../../.github/workflows/deploy.yml");
    let aliased = format!(
        "{}\n      - run: *fly_token\n",
        deploy.replace("FLY_API_TOKEN: ${{", "FLY_API_TOKEN: &fly_token ${{")
    );
    serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&aliased).unwrap();
    let error = validate("deploy.yml", &aliased).unwrap_err();
    assert!(error.contains("jobs.deploy.steps.3.run"), "{error}");
}

#[test]
fn secret_mentions_in_expression_bearing_scalars_require_review() {
    let text = "jobs:\n  test:\n    steps:\n      - run: |\n          echo 'No secrets are printed'\n          echo '${{ github.sha }}'\n";
    rejects("ci.yml", text);
    validate("ci.yml", &text.replace("${{ github.sha }}", "literal")).unwrap();
    validate("ci.yml", &text.replace("secrets", "credentials")).unwrap();
}

#[test]
fn every_change_to_the_secret_holding_workflow_requires_review() {
    assert!(validate(BURST, PIN).is_ok());
    for (old, new) in [
        ("  workflow_dispatch:", "  push:\n  workflow_dispatch:"),
        ("    environment: deployed-burst", "    environment: other"),
        ("curl -sS", "curl --trace-ascii /tmp/trace -sS"),
        (
            "    steps:",
            "    steps:\n      - run: echo trace-ascii=/tmp/trace > ~/.curlrc",
        ),
        (
            "          HAS_ADMIN:",
            "          BASH_FUNC_printf%%: bad\n          HAS_ADMIN:",
        ),
    ] {
        assert!(PIN.contains(old), "bad test mutation: {old}");
        rejects(BURST, &PIN.replacen(old, new, 1));
    }
}
