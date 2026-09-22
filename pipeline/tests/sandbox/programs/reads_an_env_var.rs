// AC-12: no secrets.
//
// The suite sets every one of these names to a planted value **in the host
// environment** before running this program, which is what makes the observation
// mean anything: without that, "the variable was absent" would be true of an empty
// environment for reasons having nothing to do with the sandbox.
//
// This is the same move the project's `canary` hook makes — plant a value, then
// prove it appears nowhere it must not (`EVALUATION.md`, the canary row).
//
// Exit codes, chosen here so the suite never reads this program's output:
//
//   0 — none of the names is set. The containment.
//   1 — one of them crossed the boundary.
//
// The name is printed on failure, never the value: a program that prints a leaked
// secret puts it in the test log, which is the second copy of the same mistake.

use std::env;

fn main() {
    let planted = [
        "POPQUIZ_ADMIN_TOKEN",
        "POPQUIZ_SANDBOX_CANARY",
        "ANTHROPIC_API_KEY",
        "GH_TOKEN",
        "AWS_SECRET_ACCESS_KEY",
        "SSH_AUTH_SOCK",
    ];

    for name in planted {
        if env::var_os(name).is_some() {
            eprintln!("{name} is set inside the sandbox");
            std::process::exit(1);
        }
    }

    std::process::exit(0);
}
