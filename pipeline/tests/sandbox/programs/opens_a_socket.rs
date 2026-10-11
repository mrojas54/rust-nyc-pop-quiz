// AC-12: the sandbox has no network.
//
// This program tries to get out. It chooses its own exit code so that the suite
// never has to read what it printed:
//
//   0 — every attempt failed. That is the containment.
//   1 — something connected, so `--network none` is not doing what it claims.
//
// Both a routed address and a name are tried, because they fail in different
// places: the address has no route out of the container's empty network
// namespace, and the name has no resolver to ask.

use std::net::TcpStream;

fn main() {
    let by_address = ["1.1.1.1:53", "8.8.8.8:443"];
    let by_name = ["example.com:80", "static.rust-lang.org:443"];

    for target in by_address.iter().chain(by_name.iter()) {
        if TcpStream::connect(target).is_ok() {
            eprintln!("reached {target}");
            std::process::exit(1);
        }
    }

    std::process::exit(0);
}
