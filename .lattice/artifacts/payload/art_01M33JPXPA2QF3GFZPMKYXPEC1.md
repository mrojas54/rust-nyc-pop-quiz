Validated end to end on HEAD ff99bd27914293e2738de33a193f26d525041ea2, on the real sandbox image (not a stand-in), Docker 29.7.2, Apple Silicon host.

just test-full: EXIT=0, 1:12.4 wall-clock with the image already built (the build-skip path exercised: tag popquiz-sandbox:1.98.1-2026-09-19-25a914853547 matched, no rebuild).
Cold image build, no layer cache, base image local: 120 s.
just test (inner loop): 56 pipeline tests + room + web, warm 0.71 s (budget 60 s).

The image reports, recorded into pin.toml (not typed): release 1.98.1, commit-hash 48a229ceaefd4985c50990b14116b6d856af0985, miri 0.1.0 (420ed2a0c3 2026-09-18), host aarch64-unknown-linux-gnu.

AC-12 containment suite, 9 passed in 67.3 s:
- test_a_program_that_opens_a_socket_cannot_reach_the_network PASSED
- test_a_program_that_reads_etc_passwd_sees_only_the_images_own_filesystem PASSED
- test_no_host_environment_variable_reaches_a_program PASSED (canaries planted on the host first)
- test_a_program_that_allocates_past_the_limit_is_killed PASSED
- test_a_program_that_loops_forever_is_killed_on_expiry PASSED
- test_the_container_does_not_survive_the_timeout PASSED
- test_the_image_reports_the_pinned_toolchain PASSED
- test_miri_is_installed_and_needs_no_network PASSED
- test_a_trivial_program_runs_under_miri_in_the_sandbox PASSED

What this does not prove, stated in the README: --cpus throttles and never kills, so the timeout is what bounds a runaway; no fixture forks, so --pids-limit is set but unexercised; MEMORY_LIMIT is inferred from exit 137/134 when the module did not do the killing. Validated on aarch64 only; x86_64 is proven when CI's test-full runs on the PR.