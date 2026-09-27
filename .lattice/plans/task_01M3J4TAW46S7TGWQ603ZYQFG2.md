# PQ-35: README in the client's voice, with a demo

Client request 2026-09-27 15:2x: make the root README.md sound like the client, and add a demo. Voice: the client's saved writing-style profile (the my-writing-style skill under ~/.claude/skills or the anthropic-skills plugin) is the reference; rewrite the prose of README.md in that voice without changing any fact, command or path — facts and commands stay exactly as the code has them (run each before quoting it; never write down what a program prints). Demo: a Demo section near the top with (1) the live room's public entry points only (join at /join, the short link /{code}, the wall at /wall/{room_id}; never the host URL or the token), (2) screenshots of the wall, the buzzer and the host phone at idle and reveal, captured from a real local run of the stand-in (cargo run --features dev-host-token) after the copy trim (PQ-34) so they show the shipped copy, stored under docs/demo/ with sizes kept small, (3) a five-line try-it walkthrough. Out of scope: room/README.md, pipeline/README.md, web/README.md (they are technical and stay), any code change, the mvp/ deck. Depends on PQ-34 (screenshots must show the trimmed copy). Fast-track; one Sonnet review for voice and accuracy. The client reads the PR.

# Plan (delegator, 2026-09-27)

## Files
- `README.md` — rewritten (cleared).
- `docs/demo/*.png` — new, six screenshots (cleared): `wall-idle.png`, `wall-reveal.png`, `buzzer-idle.png`, `buzzer-reveal.png`, `host-idle.png`, `host-reveal.png`.
- Nothing else. The capture script lives in the session scratchpad, not the repo (no new dependency: it drives the Playwright Chromium already cached on this machine through an existing playwright-core 1.62.1 install, revision 1234 matching).

## README shape (voice: my-writing-style, Voice markers + Mechanics/Avoid)
1. Title + a thesis-first opener in her words, built on PHILOSOPHY §1 (the understanding is the product; one fresh question, last five minutes).
2. **Demo** (heading in her voice): the public entry points (`/join`, `/{code}`, `/wall/{room_id}`) with `https://rustnyc-popquiz.fly.dev` named once; never `/host`, `?question=`, or the token name as a URL. Six screenshots (idle + reveal × wall/buzzer/host) with alt text. Try-it in ≤5 lines. One line: the deployed room's copy updates at the client's next `fly deploy` of `main`.
3. Run it locally: `just setup` / `just test` / `just test-full`, requirements (just, cargo, uv, node), the stand-in run command from room/README (`HOST_DEV_TOKEN=<any value>` placeholder, never a value).
4. What's here: links to room/, web/, pipeline/, mvp/, bank/ READMEs and PHILOSOPHY.md, SPEC.md, DESIGN.md, BUILDPLAN.md, EVALUATION.md; PRD.md named as prior art the synthesis superseded.
5. Prior art: dtolnay/rust-quiz (kept, facts unchanged).

## Checks (the exit check)
- Every command quoted is run on this branch, exit code recorded; every path `test -e`; every link target exists.
- `rg -n "host\?question|HOST_DEV_TOKEN=[0-9a-f]" README.md docs/` → nothing.
- PNGs < 500 KB each (resize with `sips` if needed; no lossy compression).
- `just test` green (sub-recipes if test-pipeline cannot start here), time noted.
- Sonnet review: voice PASS + facts PASS.

## Criteria touched
No behaviour criterion changes (docs only). House rules held: never write down what a program prints (the README quotes no program output; screenshots are machine captures from a real run); AC-40/AC-33 are the UI's, unaffected.

## Choices / contract tension (deviate-with-flag)
1. **Stale facts.** The current README describes Val Town + Modal, Discord sign-in as current, an empty scaffold ("no product behaviour yet"), PRD.md as "the place to start", and "most recipes have no suite". CLAUDE.md says the synthesis wins over PRD and the room is Rust on Fly. "Every fact stays true" cannot hold for facts already false. Side taken: drop the superseded Val Town/Modal data-flow diagrams and reference-implementation section (PRD.md still carries them), state the current shape from CLAUDE.md / room/README.md / justfile, and keep every fact that is still true.
2. **q3's reveal in the README.** The stand-in seeds only q3, and mvp/answer-history.json records q3 as still fresh. The repo is private (gh: PRIVATE) and bank/questions/q3.json already holds the verified output, so the screenshots expose nothing a reader of this repo cannot already see. Taking the reveal shots as briefed; flagging to the Orchestrator that a public repo would spend q3 via the front page.
3. Em-dashes: the profile uses them ("Em-dashes and colons carry the turns"); the PQ-34 client ruling banned them for on-screen host copy only. README uses few.
