# The T-03 burst spike's image. THROWAWAY.
#
# Not the room's deploy: that is T-09, which owns `room/Dockerfile` and
# `room/fly.toml`. This one builds a single feature-gated binary and nothing
# else, and the app it runs is meant to be destroyed once the spike's numbers
# are recorded.
#
# The `rust:1.96-slim` tag below is **not** the SPEC.md §7.2 / T-15a
# verification pin, and must never be mistaken for it. That pin — the full
# `rustc -Vv` release and commit-hash plus the nightly's date for Miri — is
# defined once, in T-15a's sandbox image, and it exists so a question's verified
# answer can be tied to an exact compiler. Nothing about compiling this spike
# server needs it. README.md's "The toolchain" section makes the same
# distinction for the crate itself; this comment is here so the image does not
# quietly reintroduce the confusion.

# Matches the cargo that resolved Cargo.lock (1.96.1), so `--locked` cannot fail
# on the builder for a reason that never shows up locally.
FROM rust:1.96-slim AS build
WORKDIR /src

# The whole crate, then one bin. Copying just the manifests first to cache
# dependencies would be worth it in a loop; this image is built a handful of
# times in total, so the simpler Dockerfile wins.
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked --features spike --bin spike-server

FROM debian:bookworm-slim
# No TLS libraries, no certificates: the Fly proxy terminates TLS and speaks
# plain HTTP to this process, so the binary itself never makes an outbound
# connection.
COPY --from=build /src/target/release/spike-server /usr/local/bin/spike-server

ENV PORT=8080
EXPOSE 8080
CMD ["/usr/local/bin/spike-server"]
