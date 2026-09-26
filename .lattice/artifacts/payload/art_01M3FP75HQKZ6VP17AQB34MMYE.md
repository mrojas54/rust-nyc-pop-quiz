Validation, 2026-09-26, HEAD 9de2485 (c11 browser surface:26 against the #[ignore]d serve_for_browser harness: real router, TestAuth, real session map, /shared served by the test-file helper; host driven by curl on the real action routes).
1. AC-28 by link: GET /3AV497 -> 303 -> /join?code=3AV497; the page auto-joined and attached; idle shows the room code, You're in..., and the no-account foot.
2. put-on-screen: five letter buttons plus Show me a hint; the submission line reads 'tap a letter'. Tap C -> 'saved — C'; the host's answered count moved to 1.
3. AC-48: the hint appeared from the held frame, with 'Only you can see this...'; the live region said 'Hint shown, only to you.'
4. AC-85: measured button heights 55-59 px. There was no horizontal scroll even at a 153 px pane width (scrollWidth == innerWidth).
5. close-answers: 'answers are closed' and 'you said C', letters disabled. A page reload re-attached the stored token: same session, present stayed at 1, saved C restored from session.saved (AC-37 path).
6. show-split / walk-it: '1 people said C, including you.' with the computed-on-this-phone foot. (There is only one participant, so the §11 plural reads '1 people'; see the review note.)
7. reveal: '✓ It was E.' inside .rn-correct with the glyph, computed colour rgb(56,161,105), plus 'You were the only one...'. No cross mark and no red on C (AC-94, AC-40).
8. release: '✓ Nothing about you was recorded.' Re-joining by that code then gives the already_ended sentence.
9. AC-29, typed: 'hello' -> malformed sentence; 'zzz222' -> unknown sentence; the released room's code -> already-ended sentence. The field is kept each time.
Live-region lines were observed for each phase.
just test-room + just test-web: green in 15 s (loopback binds need the sandbox bypass, as tests/wiring.rs already does). just test-pipeline cannot start on this machine (libintl); CI runs it.