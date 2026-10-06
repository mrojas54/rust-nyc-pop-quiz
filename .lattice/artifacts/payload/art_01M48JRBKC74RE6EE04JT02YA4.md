End to end at the code of b93de4f (run on the staged tree, identical). The binary hardcodes discord.com, so a desk room cannot create without the client's Discord app; validated instead with the real router and real Discord backend over a real TCP socket against the repo's Discord mock (the test-full rig), admin channel with a token minted at runtime, via a throwaway uncommitted test (deleted). Transcript, tokens redacted:
room on 127.0.0.1:52015 (TCP); Discord backend against the mock; organizer signed in through the mock's OAuth; tokens minted at runtime, shown as <…>
$ PUT /admin/questions/pq54-e2e  (Bearer <admin token>, the q3 record under id pq54-e2e)
  -> HTTP/1.1 201 Created  {"id":"pq54-e2e","scheduled":"new"}
$ POST /rooms {"question_id":"pq54-e2e"}  (Bearer <organizer session>)
  -> HTTP/1.1 201 Created  room A = 4f811e8bcf13036f0bcb40815f6c4df8
$ POST /rooms/A/put-on-screen  (Bearer <A's host session>)
  -> HTTP/1.1 200 OK  A is live
$ POST /rooms {"question_id":"pq54-e2e"}  (Bearer <organizer session>)  — second room, A live
  -> HTTP/1.1 409 Conflict  reason: "That question is open in another room. Pick another."
$ GET /last
  -> HTTP/1.1 200 OK  take-home slot: null
$ POST /rooms/A/close-answers  (Bearer <A's host session>)
  -> HTTP/1.1 200 OK
$ POST /rooms/A/show-split  (Bearer <A's host session>)
  -> HTTP/1.1 200 OK
$ POST /rooms/A/walk-it  (Bearer <A's host session>)
  -> HTTP/1.1 200 OK
$ POST /rooms/A/reveal  (Bearer <A's host session>)
  -> HTTP/1.1 200 OK
$ POST /rooms/A/release  (Bearer <A's host session>)
  -> HTTP/1.1 200 OK
$ POST /rooms {"question_id":"pq54-e2e"}  (Bearer <organizer session>)  — third room, A released
  -> HTTP/1.1 409 Conflict  reason: "That question has already been run. Pick another."
$ GET /last
  -> HTTP/1.1 200 OK  take-home slot: question_id "pq54-e2e"
$ POST /rooms {"question_id":"q3"}  (Bearer <organizer session>)  — another question
  -> HTTP/1.1 201 Created  created

Suites at b93de4f: just test-room 250 passed / 0 failed / 4 ignored (baseline 236); just test warm 18.1s (room 250, pipeline 710, web 286); just test-web 286; just canary 12 passed; just canary --full 13 passed; test-full lifecycle AC-69 passed; harness_full smoke_on_loopback passed. harness_full burst_on_loopback_at_200 fails on this machine with loopback connection resets (159/200 connected) and fails identically on base e239346 (172/200): environmental, pre-existing.