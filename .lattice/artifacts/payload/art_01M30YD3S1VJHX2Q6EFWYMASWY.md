END-TO-END VALIDATION — a real browser, real layout, real fonts over HTTP. Not unit tests.

Method: mirrored web/shared to a scratch tree, served it with python3 -m http.server on 127.0.0.1:8731, and drove it in a c11 browser surface (claude-in-chrome is blocked by standing policy). A harness page renders all seven phases on the wall's real 1120x630 canvas, runs the full three-stage type model against the layout the browser actually produced, and asserts 34 checks.

RESULT: 34/34, 0 failures, 1 honestly skipped.

The type model, measured against real layout (not stubs):
  live / closed / split   derived 18.4px, floor 14.2px, rendered 18.4px, 0 refit passes, verdict fits
  work / reveal           derived 19.8px, floor 14.2px, rendered 19.8px, 0 refit passes, verdict fits
The trace layout's extra 13px of text box (190 vs 177) produces the larger size, as SPEC 5.2 says it should. Every phase came back 'fits' with zero overflow, so the refit loop was correctly not needed — and the loop's own behaviour is covered by unit tests that inject measurements.

AC-99, measured by counting rendered token spans:
  colour in live    true
  colour in closed  true
  colour in split   true
  colour in work    FALSE
  colour in reveal  FALSE
  no source in idle       (empty string)
  no source in released   (empty string)

D-14: document.fonts.check passes for both 'Cascadia Mono' and 'Instrument Serif' with every file served from web/shared/fonts/. No font CDN is contacted; tokens.css imports the local fonts.css.

AC-40: correctHtml emits the glyph and the rn-correct class together.
AC-83: announce() creates the polite live region and it carries 'The question is on the screen.'
AC-33: the well keeps overflow internal (computed overflow-x: auto) and measured overflow is zero in every phase.

THE ONE SKIP, and why it is a skip rather than a pass or a fail. The 'no horizontal page scroll' check compares scrollWidth against documentElement.clientWidth, and the c11 browser pane reported clientWidth 0 — it was collapsed, so every element laid out at right=24 and any page would 'overflow'. That is a measurement artifact, not a result, so the harness now skips it rather than recording a green it did not earn. AC-33's page-scroll clause binds the review surface and take-it-home (T-12), neither of which is this ticket; its clause that does bind here — the well scrolls in its own container — passed.

One harness bug found and fixed along the way, worth recording because it is the kind of thing that would bite T-05: transform: scale() does not reduce an element's layout width, so a 1120px canvas still claims 1120px of the page even at scale(0.62). The wall will need its scaled canvas clipped by a wrapper, or the projector page will scroll.