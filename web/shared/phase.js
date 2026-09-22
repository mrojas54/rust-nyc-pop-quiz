/* ===========================================================================
   The phase, and what it permits — AC-99 as a pure function.

   SPEC §2 G-6 fixes the order: idle -> live -> closed -> split -> work ->
   reveal -> released. Each transition is a host action and none is skippable.
   The room's own phase machine is T-04a's (room/src/phase.rs); this file is the
   web side's read-only view of the same vocabulary, so the wall, the buzzer and
   the host phone all answer "may I colour this?" the same way.

   AC-99 IS THREE-VALUED, NOT TWO, and that is the whole reason this file exists
   rather than a boolean somewhere in well.js:

     live, closed, split   source rendered, syntax colour ON
     work, reveal          source rendered, syntax colour OFF
     idle, released        no source at all

   The middle pair is the criterion. A trace runs in `work` and `reveal` and it
   signals by highlight-and-dim; a second colour channel underneath competes
   with the one thing the room is being asked to follow (PHILOSOPHY §5, as
   amended twice). The first pair is the client's call from the T-20 drive:
   "keep the colour, love it" — in the reading phases nothing is competing with
   anything.

   Both predicates THROW on an unknown phase. A typo must not fall through to
   "colour on": that is an AC-99 failure that looks like a render bug, and it
   would reach the room before anyone noticed.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* SPEC §2, G-6. Order matters: the static fallback steps it with Space and
     Esc (AC-102), and the host phone's labels are indexed by it. */
  var PHASES = ["idle", "live", "closed", "split", "work", "reveal", "released"];

  var RENDERS_SOURCE = ["live", "closed", "split", "work", "reveal"];
  var COLOUR_ALLOWED = ["live", "closed", "split"];

  function assertPhase(phase) {
    if (PHASES.indexOf(phase) < 0) {
      throw new Error(
        "unknown phase " + JSON.stringify(phase) + " — expected one of " +
        PHASES.join(", ") + " (SPEC §2 G-6)"
      );
    }
    return phase;
  }

  /* Does this phase put the program on the screen at all? */
  function rendersSource(phase) {
    return RENDERS_SOURCE.indexOf(assertPhase(phase)) >= 0;
  }

  /* May the source carry syntax colour in this phase? (AC-99) */
  function colourAllowed(phase) {
    return COLOUR_ALLOWED.indexOf(assertPhase(phase)) >= 0;
  }

  /* Is a trace running? The inverse of colourAllowed among the phases that
     render source — named separately because the two happen to coincide today
     and the REASONS are different. If a phase is ever added that renders source
     with neither colour nor a trace, this is the one that must not move. */
  function tracing(phase) {
    return assertPhase(phase) === "work" || phase === "reveal";
  }

  /* Frozen: three tickets read this, and a caller that reordered or extended
     it would be redefining the phase machine from the outside. */
  PQ.PHASES = Object.freeze(PHASES.slice());
  PQ.assertPhase = assertPhase;
  PQ.rendersSource = rendersSource;
  PQ.colourAllowed = colourAllowed;
  PQ.tracing = tracing;
})(typeof window !== "undefined" ? window : globalThis);
