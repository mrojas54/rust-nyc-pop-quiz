Validated at HEAD dd36b44. End to end over real TCP: a scratch binary (outside the repo) serves room::router_with over a test HostAuth, q3 scheduled, capacity 3, on 127.0.0.1:38917, driven by curl through a whole segment.
- join ABC -> 409 malformed + its §11 sentence; unknown code -> 404 unknown; lower-case code joins; 3 joins then the 4th -> 409 full (no token).
- answer in idle -> 409 {phase: idle, saved: null}.
- live: A, B, C -> 200 saved A/B/C; letter F -> 400; wrong token -> 401; no token -> 401; a second session answers C.
- host payload while live: present 3, answered 2; the public GET buzzer carries no 'yours'.
- after close-answers: E -> 409 {phase: closed, reason: 'answers are closed', saved: C}.
- split wall: bars [0,0,2,0,0], '2 of 3 in the room answered'. Each final answer counted once, the late change refused.
- after walk-it/reveal/release: the old token -> 401, join -> 409 already_ended.
In-process: just test-room green, 9 test binaries, 14 new tests in tests/sessions.rs, ~5 s warm. Mutations (capacity >= to >, last-write-wins to first-write-wins) each turned the matching tests red. just test-pipeline not run: the client's pyenv lacks libintl (known environment issue); CI runs the whole just test.