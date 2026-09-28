Validation, PQ-17 (HEAD ea44e46).
(1) The real binary (default build), driven over HTTP by curl, with a random token held in a shell variable and never printed:
  PUT q3 with no token, with a wrong token, and GET /admin/nothing with no token: each 401 with content-length 0 and no WWW-Authenticate.
  PUT q3 with the right token: 201 {id:q3, scheduled:new}. Again: 200 replaced.
  PUT q3's record at /admin/questions/q9: 400 'The path names q9; the record is q3.'
  GET /admin/used: 200 [].
  POST /rooms with the admin token as bearer: 401 (it opens nothing else).
  The binary's whole log (one startup line) does not contain the token.
  Started with POPQUIZ_ADMIN_TOKEN empty: exit 2, the error names the variable and no value.
(2) Repo scan: an untracked file with POPQUIZ_ADMIN_TOKEN=<64 random hex> makes just secret-scan fail (rc 101), naming the file and the value's length, not the value. With the file removed it passes again (rc 0, 1 s).
(3) Warm runs: just test-room green, 176 tests, 11.9 s (main baseline 10.9 s). just test-web: 195 pass, 0.7 s. just canary: green, 3.6 s, including the live admin plant and the scan of the real binary's stdout and stderr. The CI feature step (dev-host-token,smoke: standin, smoke_config, admin) is green.
just test-pipeline cannot start on this machine (pyenv libintl); CI runs the whole just test. The loopback-socket tests need the sandbox bypass here (local binding is off in-tab).