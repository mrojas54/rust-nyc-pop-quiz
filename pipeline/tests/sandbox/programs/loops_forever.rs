// AC-12: the time limit is enforced, and it is what bounds a runaway.
//
// This is also the fixture that stands in for the CPU limit, and the README says
// so rather than letting AC-12's "enforced CPU/memory/time limits" imply more.
// `--cpus` throttles a container's share of a core; it never terminates anything,
// so no program can be written that shows it "firing". What stops a program that
// will not stop on its own is this timeout.
//
// No exit code is chosen here, because there is no path on which this program
// exits. Being killed is the entire observation.

fn main() {
    loop {
        std::hint::spin_loop();
    }
}
