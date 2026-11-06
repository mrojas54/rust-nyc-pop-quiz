//! Credential policy for GitHub workflows (PQ-61, SPEC §8.4).
use std::path::PathBuf;

const BURST: &str = "deployed-burst.yml";
const PIN: &str = include_str!("fixtures/deployed-burst.pinned.yml");

fn validate(_name: &str, _text: &str) -> Result<(), String> {
    Ok(())
}

fn rejects(name: &str, text: &str) {
    assert!(validate(name, text).is_err(), "accepted forbidden workflow: {text}");
}

#[test]
fn checked_in_workflows_obey_the_credential_policy() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows");
    let mut found_burst = false;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if !matches!(path.extension().and_then(|x| x.to_str()), Some("yml" | "yaml")) {
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
        "\n    environment: >-\n      deployed-burst",
        r#"{environment: "deployed-\u0062urst"}"#,
        r#"{environment: "${{ inputs.environment }}"}"#,
    ] {
        rejects("other.yaml", &format!("jobs:\n# comment: |\n  probe: {job}\n"));
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
    rejects("other.yml", "jobs: {probe: {if: secrets.FLY_API_TOKEN != ''}}");
}

#[test]
fn the_fly_exception_requires_the_exact_structural_location() {
    let deploy = include_str!("../../.github/workflows/deploy.yml");
    assert!(validate("deploy.yml", deploy).is_ok());
    rejects("other.yml", deploy);
    rejects("deploy.yml", &format!("{deploy}\n      - run: |\n          cat > /tmp/token <<'END'\n          FLY_API_TOKEN: ${{{{ secrets.FLY_API_TOKEN }}}}\n          END\n"));
    rejects("deploy.yml", &deploy.replace("        env:", "        with:"));
    rejects("deploy.yml", &deploy.replace("jobs:\n", "env: {FLY_API_TOKEN: '${{ secrets.FLY_API_TOKEN }}'}\njobs:\n"));
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
        rejects("other.yml", text);
    }
}

#[test]
fn aliases_are_checked_at_every_use() {
    rejects("other.yml", "env: &credentials {TOKEN: '${{ secrets.POPQUIZ_ADMIN_TOKEN }}'}\njobs: {probe: {env: *credentials}}");
    rejects("other.yml", "name: &protected deployed-burst\njobs: {probe: {environment: *protected}}");
}

#[test]
fn every_change_to_the_secret_holding_workflow_requires_review() {
    assert!(validate(BURST, PIN).is_ok());
    for (old, new) in [
        ("  workflow_dispatch:", "  push:\n  workflow_dispatch:"),
        ("    environment: deployed-burst", "    environment: other"),
        ("curl -sS", "curl --trace-ascii /tmp/trace -sS"),
        ("    steps:", "    steps:\n      - run: echo trace-ascii=/tmp/trace > ~/.curlrc"),
        ("          HAS_ADMIN:", "          BASH_FUNC_printf%%: bad\n          HAS_ADMIN:"),
    ] {
        assert!(PIN.contains(old), "bad test mutation: {old}");
        rejects(BURST, &PIN.replacen(old, new, 1));
    }
}
