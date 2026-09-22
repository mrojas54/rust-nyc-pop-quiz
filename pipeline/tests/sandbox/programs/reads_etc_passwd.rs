// AC-12: no host filesystem access.
//
// This is the fixture that could easily ship a false claim, so it is worth being
// exact about what it observes. `/etc/passwd` exists inside *any* container and is
// world-readable, so "the sandbox denied the read" would simply be untrue, and a
// suite asserting it would be asserting something the sandbox does not do.
//
// What AC-12 actually asks for is no *host* filesystem. Two things establish it,
// and this program observes the second:
//
//   1. No host path is mounted — a fact about the argv, checked in `just test` by
//      `configuration_verdicts` (HOST_FS_ISOLATED). Nothing a program does can
//      show this, which is why it is proven structurally.
//   2. The filesystem the program *can* see is the image's own, and it is
//      read-only. That is observable from in here.
//
// Exit codes, chosen by this program so the suite never reads its output:
//
//   0 — /etc/passwd read, and every write attempt refused. The expected result.
//   1 — a write succeeded, so the root filesystem is not read-only.
//   2 — /etc/passwd could not be read. The sandbox does not claim this, so it
//       means something other than containment changed, and the suite should say
//       so rather than treat it as a pass.

use std::fs;
use std::io::Write;

fn main() {
    if fs::read_to_string("/etc/passwd").is_err() {
        std::process::exit(2);
    }

    // The root filesystem is mounted read-only, so each of these must fail.
    // /work is deliberately absent from this list: it is the tmpfs, and it is
    // supposed to be writable — that is where a candidate gets compiled.
    for path in ["/etc/passwd", "/etc/hosts", "/usr/local/bin/escape", "/root/escape"] {
        if let Ok(mut file) = fs::OpenOptions::new().write(true).create(true).open(path) {
            if file.write_all(b"x").is_ok() {
                eprintln!("wrote to {path}");
                std::process::exit(1);
            }
        }
    }

    std::process::exit(0);
}
