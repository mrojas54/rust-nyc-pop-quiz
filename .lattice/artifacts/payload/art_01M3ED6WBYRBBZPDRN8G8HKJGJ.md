End to end over a real socket: a throwaway scratchpad server (never committed) serving room::router_with() on 127.0.0.1:38417 with q3 scheduled and a validation-only HostAuth, driven by curl through all seven phases. Also: just test-room green warm 2.6 s (45 tests + 12 doctests), test-web green (103), cargo check --features spike --all-targets green; test-pipeline cannot start on this machine (client env: pyenv python3.12 missing libintl.8.dylib, known, unrelated — no pipeline file changed). Mutation check: opening each boundary in a scratch copy turned exactly its test red (witness field pub -> RevealWitness doctest; Machine fields pub -> phase doctest; Room.question pub -> rooms line 24; PublicView::open -> rooms line 34; witness-free vault fn, Debug on Vault, pre-reveal builder calling room.open(), pub(crate) witness constructor -> boundary.rs). Curl transcript:
CREATE 201 code=ZRJACG
put-on-screen, no bearer -> 401
reveal from idle -> {"reason":"Reveal isn't next. Put it on the screen comes next."} 409
--- idle
  wall: phase=idle keys=['join', 'title'] has_check=False stdout_rows=0 at=None
  buzzer: phase=idle keys=['foot', 'lines'] has_check=False stdout_rows=0 at=None
  host: phase=idle keys=['first_screen', 'label', 'present', 'primary'] has_check=False stdout_rows=0 at=None
>>> put-on-screen 200
--- put-on-screen
  wall: phase=live keys=['colour', 'join', 'options', 'source', 'well_header'] has_check=False stdout_rows=0 at=None
  buzzer: phase=live keys=['foot', 'hint', 'letters', 'lines', 'locked'] has_check=False stdout_rows=0 at=None
  host: phase=live keys=['answered', 'label', 'present', 'primary'] has_check=False stdout_rows=0 at=None
>>> close-answers 200
--- close-answers
  wall: phase=closed keys=['colour', 'options', 'source', 'strip'] has_check=False stdout_rows=0 at=None
  buzzer: phase=closed keys=['foot', 'letters', 'lines', 'locked'] has_check=False stdout_rows=0 at=None
  host: phase=closed keys=['answered', 'label', 'present', 'primary'] has_check=False stdout_rows=0 at=None
>>> show-split 200
--- show-split
  wall: phase=split keys=['colour', 'options', 'source', 'split'] has_check=False stdout_rows=0 at=None
  buzzer: phase=split keys=['counts', 'foot', 'lines'] has_check=False stdout_rows=0 at=None
  host: phase=split keys=['answered', 'label', 'present', 'primary'] has_check=False stdout_rows=0 at=None
>>> walk-it 200
--- walk-it
  wall: phase=work keys=['beats', 'colour', 'options', 'source', 'trace'] has_check=False stdout_rows=0 at=0
  buzzer: phase=work keys=['counts', 'foot', 'lines'] has_check=False stdout_rows=0 at=None
  host: phase=work keys=['answered', 'label', 'present', 'primary', 'step'] has_check=False stdout_rows=0 at=0
>>> step-forward  200
>>> step-forward  200
>>> step-forward  200
>>> step-forward  200
>>> step-forward  409
  wall at the bound: phase=work keys=['beats', 'colour', 'options', 'source', 'trace'] has_check=False stdout_rows=0 at=4
>>> reveal 200
--- reveal
  wall: phase=reveal keys=['colour', 'options', 'reveal', 'source', 'split', 'trace'] has_check=True stdout_rows=1 at=5
  buzzer: phase=reveal keys=['correct', 'counts', 'foot', 'lines'] has_check=True stdout_rows=0 at=None
  host: phase=reveal keys=['answered', 'label', 'present', 'primary', 'read_aloud', 'step'] has_check=False stdout_rows=0 at=5
  wall.reveal: {"correct": "E", "mark": "✓", "middle": {"line": "Nobody read it another way."}, "receipt": {"heading": "How we know", "lines": ["✓ Compiled", "✓ Ran 5 times", "✓ Output never varied", "✓ Miri ran clean"]}}
>>> release 200
--- released
  wall: phase=released keys=['released'] has_check=False stdout_rows=0 at=None
  buzzer: phase=released keys=['foot', 'lines', 'mark'] has_check=True stdout_rows=0 at=None
  host: phase=released keys=['answered', 'label', 'present', 'primary'] has_check=False stdout_rows=0 at=None
>>> run-it-again, same question -> {"reason":"That question has already been run. Pick another."} 409