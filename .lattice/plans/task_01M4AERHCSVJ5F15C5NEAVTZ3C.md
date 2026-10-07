# PQ-65: Smoke and burst give the wrong advice when a room already holds the question

Follow-up of PQ-54 (PR #50, merged at 2877149): one room per question. Since PQ-54, POST /rooms and run-it-again answer 409 'That question is open in another room. Pick another.' while another unreleased, un-ended room holds the question.

PROBLEM. smoke and burst answer every 409 on create with the same hint, written when the only 409s were 'not scheduled' and 'already run': 'schedule ... over the pipeline channel first; if it has already been run on this machine, restart it (fly apps restart) and schedule it again' (room/src/bin/smoke.rs:369-373, room/src/bin/burst.rs:357). For the new refusal that is wrong and harmful: restarting the machine wipes every in-memory room, the night's room included. The right advice is to wait for the other room to go quiet (30 min with no host action before start, 20 once started, 4 h at most; docs/RUNBOOK.md failure table) or to use another question id.

SECOND EFFECT. A smoke or burst run that creates a room on the night's question and does not release it now holds that question against the real create until it goes quiet. smoke walks to release, but an aborted run leaves its room open.

SCOPE. (1) Print the room's reason verbatim for every 409 on create, and make the hint depend on it: not scheduled -> schedule first; already run -> pick another id; open in another room -> wait for it to go quiet or pick another id; never suggest a restart for the held case. Both binaries. (2) Tests: each 409 reason maps to its hint, and the held case never mentions 'restart' (in-process router, no network). (3) README 'Burst' and the smoke passage name the held case.

NOT IN SCOPE. The --question q3 default and refusing a bank id off loopback are PQ-41 item (B); land them there. If PQ-41 is dispatched first, fold this ticket into it.

Acceptance: just test green and under 60 s; each new check shown failing under its mutation; no change to room/src/rooms.rs.
