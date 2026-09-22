END-TO-END VALIDATION, reviewed HEAD 9745025. Loaded the four committed records off disk and rendered what each surface would show, rather than only running unit tests.

RECEIPTS (SPEC 7.5). q3, q4, q7 each render the four-line list from a legacy record: 'Compiled / Ran 5 times / Output never varied / Miri ran clean'. q8 renders the three-line does-not-compile list: 'Compiler refused it / Error E0502 / Nothing ran' — no run count, no Miri line. Heading 'How we know' on all four. AC-13, AC-43, AC-87.

TAKE-IT-HOME DETAIL (AC-87, SPEC 13). All four report legacy=True and target='not recorded'; q3/q4/q7 report miri_ran_outside_verifier=True (the page's 'run separately' row), q8 False because nothing ran. Nothing back-filled.

DERIVED ANSWER (AC-7, G-2). Every correct option is index 4 and every text is byte-equal to its verified.stdout normalized: q3 '[1, 2, 3, 2, 1]', q4 '-3 -1\n-4 1', q7 the 48-char sorted vector, q8 the does_not_compile option (stdout is None, as it must be).

ABSENT IS ABSENT (G-2, D-16). Read from the raw JSON: q3/q4/q7 hold exactly edition, legacy, miri, runs, rustc, stdout. q8 holds exactly compile_error_code, edition, legacy, rustc. target_triple, flags, exit_code, verified_at and verifier_version are absent keys on all four — not keys holding null.

WALL RULE (D-15, SPEC 5.2). q3: no option fails. q4: options 0,1,2,4 fail — the correct one among them. q7: 0,1,2,4 fail — correct among them. q8: only option 3 fails — correct NOT among them. Matches the generated review notes.

AFFIRM STATE (AC-72, AC-95, G-12). No record is affirmed. q3 has 4/4 incorrect options carrying a why_tempting; q4, q7, q8 have 0/4 and 0 trace steps, so all three are blocked at affirm, which is the intended state for a question awaiting re-verification.

HISTORY (AC-17). version 1, three stores, all empty, total 0. T-17 fills and reads it.

REPRODUCIBILITY. Re-ran migrate() into a clean temporary directory: all four question files and history.json are byte-identical to the committed ones.

HARNESS. just test green warm at 0.87s against the committed tree (budget 60s): 140 pipeline tests, the room canary, the web layout suite. No rustc, no Miri, no Docker, no network.