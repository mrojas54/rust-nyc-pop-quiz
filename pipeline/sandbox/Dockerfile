# check=skip=InvalidDefaultArgInFrom
#
# That first line is a BuildKit parser directive and only counts as one on line 1,
# which is why it sits above this comment rather than below it. The ARGs further
# down deliberately have no defaults, which the linter flags as
# InvalidDefaultArgInFrom because `rust:${RUST_VERSION}-slim-bookworm` is not a
# valid image name until the argument is supplied. That is the intended behaviour —
# a build that does not pass the pin must fail — so the check is skipped rather
# than left to print a warning on every build. A warning nobody can act on is how a
# build gets trained to ignore warnings.

# The verification sandbox (T-15a, AC-12).
#
# This file holds no version literal. Every version arrives as a build argument
# read out of pin.toml by `just sandbox-build`, because the pin is defined in one
# place (SPEC.md §7.2, D-17) and a Dockerfile with `FROM rust:1.x` in it would be
# a second place that has to agree.
#
# The ARGs have no defaults on purpose. A bare `docker build` fails on the
# missing value instead of quietly picking one, which turns "remember to pass the
# pin" into a build error. `pipeline/tests/test_sandbox.py` asserts that no
# version-shaped literal has crept back in.

ARG RUST_VERSION
FROM rust:${RUST_VERSION}-slim-bookworm

ARG NIGHTLY

# Candidate programs are untrusted input, so nothing runs as root. The uid is
# fixed rather than left to useradd's next-free so that the image is the same
# whatever base it is built on.
RUN useradd --create-home --uid 10001 --shell /bin/sh sandbox

# /work is where a candidate is compiled and run. At run time it is replaced by a
# tmpfs; creating it here with the right owner means the container has a valid
# working directory even if someone runs the image without the mount.
RUN mkdir -p /work && chown sandbox:sandbox /work
WORKDIR /work

# Everything below runs as the user that will run candidates, which is what puts
# Miri's sysroot in that user's cache where it can be found again at run time.
USER sandbox

# The pinned nightly, with only what Miri needs. `miri setup` builds the standard
# library for Miri here, at BUILD time, which is the whole reason the container
# can run with --network none: there is nothing left to download.
#
# --profile minimal keeps the image small; rust-src is what miri setup compiles
# against, and without it the setup step is the thing that would reach for the
# network later.
RUN rustup toolchain install "nightly-${NIGHTLY}" \
      --profile minimal \
      --component miri \
      --component rust-src \
 && cargo "+nightly-${NIGHTLY}" miri setup

# Where `miri setup` just put the sysroot (it says so at the end of the step above).
# Without this, Miri re-validates the sysroot on every run by staging a temp dir
# inside it, which the read-only root refuses. Set in the image rather than passed
# with `-e`, because the argv carries no environment at all — that absence is what
# the ENV_NOT_PASSED verdict reads. This is the image describing itself, not the
# host leaking in.
ENV MIRI_SYSROOT=/home/sandbox/.cache/miri

# No ENTRYPOINT and no CMD. Every invocation passes its own command — compile,
# run, or Miri — and the source arrives on stdin rather than as an argument or a
# mount (see pipeline/src/popquiz/sandbox.py). A default command here would be a
# fourth way to run the image that nothing tests.
