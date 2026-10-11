# The room's image (T-09; T-10: hosting by Discord role).
#
# Built from the repository root, because the binary embeds files from beside
# the crate at compile time: the pages and fonts under `web/` (routes.rs).
# `.dockerignore` admits those and the crate. Deploy with `fly deploy
# --ha=false` (room/README.md, Deploying).
#
# The binary refuses to start without the four DISCORD_* Fly secrets (SPEC §8,
# room/README.md). Nothing is scheduled at start: questions arrive over the
# pipeline channel (SPEC §8.3).
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
RUN cd room && cargo build --release --locked --bin room

# The builder's own Debian release, so the binary's glibc is the one it was
# linked against. No TLS libraries and no certificates: Fly's proxy terminates
# TLS and speaks plain HTTP to this process. Its one outbound connection is to
# Discord at room creation, over rustls with Mozilla's roots compiled into the
# binary (webpki-roots), so the image still needs no CA bundle.
FROM debian:bookworm-slim
COPY --from=build /src/room/target/release/room /usr/local/bin/room
# Nobody: the room needs no privilege, and 8080 is not a privileged port.
USER 65534:65534
ENV PORT=8080
EXPOSE 8080
CMD ["/usr/local/bin/room"]
