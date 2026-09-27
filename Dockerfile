# The room's image (T-09): the M1 skeleton, built with SPEC §8.2's stand-in.
#
# Built from the repository root, because the binary embeds files from beside
# the crate at compile time: the pages and fonts under `web/` (routes.rs) and
# the HC-0 question, `bank/questions/q3.json` (src/standin.rs). `.dockerignore`
# admits exactly those and the crate. Deploy with `fly deploy --ha=false`
# (room/README.md, Deploying).
#
# `--features dev-host-token` is what makes this the stand-in build: *Create a
# room* takes the `HOST_DEV_TOKEN` Fly secret as its bearer and the room starts
# with q3 scheduled. Without that secret the binary refuses to start. T-10
# removes the feature, and this line with it.
#
# `rust:1.96-slim-bookworm` matches the cargo that resolved Cargo.lock (1.96.1),
# so `--locked` cannot fail here for a reason that never shows up locally. It is
# **not** the SPEC §7.2 / T-15a verification pin and must never be read as one:
# that pin ties a question's verified answer to an exact compiler, and nothing
# about compiling the server needs it (room/README.md, The toolchain).

FROM rust:1.96-slim-bookworm AS build
WORKDIR /src
COPY room/Cargo.toml room/Cargo.lock room/
COPY room/src room/src
COPY web web
COPY bank/questions/q3.json bank/questions/q3.json
RUN cd room && cargo build --release --locked --features dev-host-token --bin room

# The builder's own Debian release, so the binary's glibc is the one it was
# linked against. No TLS libraries and no certificates: Fly's proxy terminates
# TLS and speaks plain HTTP to this process, which makes no outbound connection.
FROM debian:bookworm-slim
COPY --from=build /src/room/target/release/room /usr/local/bin/room
# Nobody: the room needs no privilege, and 8080 is not a privileged port.
USER 65534:65534
ENV PORT=8080
EXPOSE 8080
CMD ["/usr/local/bin/room"]
