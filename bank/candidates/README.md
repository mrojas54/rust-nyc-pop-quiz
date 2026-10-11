# candidates

Questions drafted but not yet verified. Nothing here is in the bank: the suites read
`bank/questions/` only, and every file there must derive its answer from a record the
pinned verifier wrote (AC-7), which a candidate does not have yet.

To promote one, on a machine with the sandbox image (`just sandbox-build`):

    just verify bank/candidates/<id>.json --expect <ran|does_not_compile>
    git mv bank/candidates/<id>.json bank/questions/<id>.json

`--expect` is each file's `review.reason`. If the verifier rejects a candidate it leaves the
file untouched, and the program (or the answer I expected from a local run) is what to
look at. The trace is empty and nothing is affirmed, so an organizer still has to write
the walk-through steps and read the prose before any of these can reach a deck.
