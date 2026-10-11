// AC-12: the memory limit is enforced.
//
// Unlike the isolation fixtures, this program chooses no exit code. Being stopped
// is the observation, and the verdict comes from how it was stopped — the cgroup's
// OOM killer (SIGKILL, 137) or the allocator refusing outright and aborting
// (SIGABRT, 134). If this program ever reaches its own last line, the limit did not
// fire, and exit 1 says so.
//
// Every page is touched after allocation. A `Vec` of zeroes can be backed by pages
// the kernel accounts only when they are written, so an untouched allocation can
// sail past a cgroup limit that a touched one trips immediately. Allocating without
// touching is the way this fixture would quietly stop testing anything.

fn main() {
    let block = 64 * 1024 * 1024; // 64 MiB at a time
    let mut held: Vec<Vec<u8>> = Vec::new();

    // Well past any limit the pin would sensibly configure. Bounded rather than
    // infinite so that a sandbox which does not stop it fails the suite instead of
    // hanging it — a hang and a containment failure should not look the same.
    for _ in 0..256 {
        let mut chunk = vec![0u8; block];
        let mut i = 0;
        while i < chunk.len() {
            chunk[i] = 1;
            i += 4096;
        }
        held.push(chunk);
    }

    eprintln!("allocated {} MiB without being stopped", held.len() * block / (1024 * 1024));
    std::process::exit(1);
}
