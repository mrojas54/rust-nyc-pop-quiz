/* ===========================================================================
   The wall's type model — SPEC §5.2, AC-100, AC-78.

   "The wall sizes itself to the question; legibility is the floor. Nothing is
   chosen by hand." (DESIGN.md)

   Question length is not bounded — nothing rejects a long question up front.
   The type grows to fill the wall when the program is short and shrinks toward
   the smallest legible size when it is long. A question is too long only when
   it will not fit EVEN AT the floor, and that is an emergent limit belonging to
   the room, not a rule imposed on the generator.

   THREE STAGES, and the third is the one that makes AC-100 honest:

     1. derive   arithmetic from the room's measurements and the source's shape
     2. refit    render, MEASURE, shrink, up to six times — "fits, legibly" is
                 a measured claim, not a computed guess
     3. verdict  fits | clipped_x | clipped_y | clipped_xy, and a red edge on
                 the side that lost content

   T-19 MIRRORS THIS IN PYTHON. `bank-audit` fits every bank question against
   the reading layout offline, with no browser to measure in, so it reimplements
   stage 1 only. floorPx(), sourceMetrics() and desiredFontPx() are the three it
   must match, the constants below are the ones it must share, and SPEC §5.2's
   worked examples are the fixture both test against. Keep the arithmetic in
   those three functions and nowhere else.

   NO ROUNDING IN HERE. The prototype rounded to one decimal inside the refit
   loop; SPEC §5.2's pseudocode rounds nowhere and states its worked examples
   "to one decimal" as presentation. Two implementations that round at different
   moments disagree in the third decimal and then, after six refit passes, in
   the first. Round at the edge — where a number is displayed or asserted.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* --- The room. Organizer-set; SPEC §5.2 gives these defaults and labels them
         a hypothesis until HC-1 measures the venue. Every number below moves
         with them. ------------------------------------------------------- */
  var ROOM_DEFAULTS = {
    screen_width_ft: 15,    /* the client's estimate, "pretty big, maybe 15ft" */
    screen_height_ft: 8.44, /* default = width x 9/16 */
    back_row_ft: 20
  };

  var TYPE_CONSTANTS = {
    CANVAS_W: 1120,        /* the wall lays out here and scales as a unit (§5.1) */
    CANVAS_H: 630,
    CAP_RATIO: 0.7,        /* cap height as a fraction of font-size, monospace */
    READ_RATIO: 150,       /* cap height >= viewing distance / 150. A rule of
                              thumb, labelled one: 150 for carefully-read text,
                              200 for basic legibility. Code is read carefully.
                              AC-78 is `felt` and October settles it. */
    MAX_FONT: 46,          /* a five-line program should fill the wall, not
                              become a poster. The FLOOR WINS over this cap. */
    LINE_HEIGHT: 1.6,      /* MEASURED: components.css `.rn-src pre` */
    GUTTER_CH: 3.5,        /* the 2ch line-number gutter + 1.5ch margin. Leaving
                              this out is what made the first cut overflow by
                              40-80px. */
    CHAR_ADVANCE: 0.6,     /* monospace advance width, in em */
    OPTIONS_BLOCK_PX: 190, /* three rows of 58px + two 8px gaps. A CONSTANT
                              because options are constrained to one line of 29
                              characters (D-15), so the wall never improvises a
                              wrapped option. */
    BEAT_BLOCK_PX: 203,    /* the step note, clamped to three lines, + values */
    OPTION_MAX_CHARS: 29,  /* 510px cell - 40 chip - 14 gap - 2x14 pad = 428px
                              = 29 chars of 14.4px, at the wall's 24px option
                              size (D-15, confirmed at touchpoint T-22) */
    REFIT_MAX_PASSES: 6,
    REFIT_SHRINK: 0.97,
    OVERFLOW_TOLERANCE_PX: 2
  };

  /* --- The code areas. TEXT boxes, measured 2026-09-18 from the prototype at
         its 1120x630 design size: the header, borders and padding are already
         subtracted, so nothing further is deducted below.

         "split" HERE IS A PHASE, NOT A LAYOUT. SPEC uses `split` for the bars
         after close and gives it the READING layout. The prototype also had a
         `split` LAYOUT MODE — source beside options, 564x426 — which is the
         alternative the client rejected at touchpoint T-20 item 15 in favour of
         full-width. Those constants are deliberately not ported. The wall is
         full-width in every phase; nothing is ever beside the source. --- */
  var CODE_AREA = {
    reading: { w: 994, h: 177 }, /* live, closed, split — under a 190px options block */
    trace:   { w: 994, h: 190 }  /* work, reveal — under a 203px beat block */
  };

  function codeAreaFor(phase) {
    PQ.assertPhase(phase);
    if (!PQ.rendersSource(phase)) return null;
    return PQ.tracing(phase) ? CODE_AREA.trace : CODE_AREA.reading;
  }

  /* --- Stage 1: derive ------------------------------------------------- */

  /* The smallest legible size for this room, in canvas px.

       screen_h_in = screen_height_ft * 12
       cap_in      = back_row_ft * 12 / 150
       font_in     = cap_in / 0.7
       floor_px    = font_in / screen_h_in * 630

     At the 15 / 8.44 / 20 default this is 14.2px. */
  function floorPx(room) {
    var r = room || ROOM_DEFAULTS;
    var K = TYPE_CONSTANTS;
    var screenHIn = r.screen_height_ft * 12;
    var capIn = (r.back_row_ft * 12) / K.READ_RATIO;
    var fontIn = capIn / K.CAP_RATIO;
    return (fontIn / screenHIn) * K.CANVAS_H;
  }

  /* A source's shape: how many lines, and how wide the widest is. Uses well.js's
     line splitter so the two can never disagree about what a line is. */
  function sourceMetrics(code) {
    var lines = PQ.sourceLines(code);
    var widest = 0;
    for (var i = 0; i < lines.length; i++) {
      if (lines[i].length > widest) widest = lines[i].length;
    }
    return { lines: lines.length, widestChars: widest };
  }

  /* What this source WANTS, before the floor is applied — min of the height
     constraint, the width constraint, and the 46 cap.

     Exposed separately because "below the floor" is the interesting fact: SPEC
     §5.2's worked examples report q5 at 12.3px and q2 at 11.1px, which are what
     the source wants, not what the wall would render. `bank-audit` flags a
     question exactly when this drops under floorPx(). */
  function desiredFontPx(metrics, area) {
    var K = TYPE_CONSTANTS;
    var byHeight = area.h / (metrics.lines * K.LINE_HEIGHT);
    var byWidth = area.w / ((metrics.widestChars + K.GUTTER_CH) * K.CHAR_ADVANCE);
    return Math.min(byHeight, byWidth, K.MAX_FONT);
  }

  /* The size the wall starts at: what the source wants, floored at legibility.

     THE FLOOR WINS OVER THE 46 CAP. If a room is small enough that floorPx()
     exceeds 46, the floor is what renders and the wall reports it (§5.2) —
     legibility is not negotiable against a cap that exists only to stop a
     five-line program becoming a poster. */
  function derivedFontPx(code, phase, room) {
    var area = codeAreaFor(phase);
    if (!area) return null;
    var floor = floorPx(room);
    var wants = desiredFontPx(sourceMetrics(code), area);
    return {
      fontPx: Math.max(floor, wants),
      wants: wants,
      floorPx: floor,
      belowFloor: wants < floor,
      cappedAtMax: wants >= TYPE_CONSTANTS.MAX_FONT,
      floorAboveCap: floor > TYPE_CONSTANTS.MAX_FONT,
      area: area
    };
  }

  /* --- Stage 2: refit -------------------------------------------------- */

  /* Measure the well: how far its content overflows its scroll region.
     Returns null when there is nothing to measure, so a caller can tell
     "no overflow" from "never rendered". */
  function measureWell(el) {
    if (!el || typeof el.scrollWidth !== "number") return null;
    return {
      overX: el.scrollWidth - el.clientWidth,
      overY: el.scrollHeight - el.clientHeight,
      cw: el.clientWidth, ch: el.clientHeight,
      sw: el.scrollWidth, sh: el.scrollHeight
    };
  }

  function overflows(m) {
    var t = TYPE_CONSTANTS.OVERFLOW_TOLERANCE_PX;
    return !!m && (m.overX > t || m.overY > t);
  }

  /* The measured refit. The constants above are a first guess: the well has a
     header and padding they never knew about, and the options block under a
     full-width source is ~190px tall, not the ~50px the first cut assumed.
     Both put code through the join strip on 2026-09-03 while the note beneath
     said "fits, legibly".

     While anything overflows and fontPx > floor, at most six times:
       k       = min(overX > 2 ? content_w / scroll_w : 1,
                     overY > 2 ? content_h / scroll_h : 1)
       fontPx  = max(floor, fontPx * k * 0.97)

     `render` is injected rather than done here: this file must not know what a
     wall looks like, and injecting it is also what makes the loop testable
     without a browser. The caller renders at the size it is handed and returns
     a measurement.

     The wall refits on entering EVERY phase that renders the source — live,
     closed, split, work, reveal — against that phase's code area. The trace
     phases never use a hard-coded size. */
  function refit(startPx, floor, renderAndMeasure) {
    var K = TYPE_CONSTANTS;
    var fontPx = startPx;
    var m = renderAndMeasure(fontPx);
    var passes = 0;

    while (passes < K.REFIT_MAX_PASSES && overflows(m) && fontPx > floor) {
      var kx = m.overX > K.OVERFLOW_TOLERANCE_PX ? m.cw / m.sw : 1;
      var ky = m.overY > K.OVERFLOW_TOLERANCE_PX ? m.ch / m.sh : 1;
      var k = Math.min(kx, ky);
      var next = Math.max(floor, fontPx * k * K.REFIT_SHRINK);
      if (next >= fontPx) break; /* at the floor, or k >= 1: shrinking no further */
      fontPx = next;
      m = renderAndMeasure(fontPx);
      passes++;
    }

    return { fontPx: fontPx, passes: passes, measurement: m, atFloor: fontPx <= floor };
  }

  /* --- Stage 3: the verdict -------------------------------------------- */

  /* The measured verdict. SPEC §3.4 makes this room state — written by the
     wall after each refit, and copied into the `used` record at release as
     AC-100's evidence. SPEC §11 binds each value to a host-phone line:

       fits        -> "fits the room"
       clipped_y   -> "too long for this room — clipped at the bottom"
       clipped_x   -> "too wide for this room — clipped at the right"
       clipped_xy  -> "too long and too wide for this room"

     PER-AXIS TOLERANCE. Each axis is tested against OVERFLOW_TOLERANCE_PX on
     its own, so 1px of horizontal slop beside 40px of vertical overflow is
     `clipped_y`, not `clipped_xy`. The verdict is words on the host's phone
     telling them which edge lost content; naming both when only one clipped
     sends them looking at the wrong side of the screen mid-segment. The 2px
     tolerance is there because scrollWidth/clientWidth are integers rounded
     from fractional layout, so a well that fits exactly can report 1px.

     AN UNMEASURED WELL HAS NO VERDICT, and this returns null for one.

     That is the whole point of AC-100: "*fits* is a measured claim after
     layout, not a computed guess", and DESIGN.md says nothing is silently
     clipped. Returning "fits" for a well nobody measured would assert the
     measured claim without the measurement — the exact failure the criterion
     exists to prevent. Returning a clipped value instead would put a false
     "too long for this room" on the host's phone during a question that is
     fine. So neither: null means "not measured yet", which is faithful to
     §3.4, where `fit` is written by the wall after a refit and simply does not
     exist before one.

     CALLERS MUST NOT TREAT null AS "fits". There is no §11 fit line for null
     and copy.js has no entry for it; the host phone shows the fit line from
     `live` on, by which point the wall has refitted at least once. */
  function fitVerdict(m) {
    if (!m) return null;
    var t = TYPE_CONSTANTS.OVERFLOW_TOLERANCE_PX;
    var x = m.overX > t;
    var y = m.overY > t;
    if (x && y) return "clipped_xy";
    if (x) return "clipped_x";
    if (y) return "clipped_y";
    return "fits";
  }

  /* The red edge on the side that lost content, from the verdict. Colour is
     never the only signal (AC-40): the host phone carries the matching words
     from SPEC §11, and the edge is a 4px border — a width difference readable
     without hue. */
  function fitEdgeClasses(verdict) {
    if (!verdict || verdict === "fits") return "";
    var cls = "";
    if (verdict === "clipped_x" || verdict === "clipped_xy") cls += " clipped-x";
    if (verdict === "clipped_y" || verdict === "clipped_xy") cls += " clipped-y";
    return cls;
  }

  function applyClippedEdge(el, verdict) {
    if (!el || !el.classList) return false;
    el.classList.remove("clipped-x");
    el.classList.remove("clipped-y");
    var cls = fitEdgeClasses(verdict).trim();
    if (!cls) return false;
    cls.split(/\s+/).forEach(function (c) { el.classList.add(c); });
    return true;
  }

  /* --- The bank's side of AC-100 --------------------------------------- */

  /* One line of at most 29 characters (D-15). An option over that is a
     `bank-audit` failure, never a layout the wall improvises — the wall has no
     design for a wrapped option. T-19 enforces this offline; it is here so the
     rule has one definition. */
  function optionFits(text) {
    var s = String(text);
    return s.indexOf("\n") < 0 && s.length <= TYPE_CONSTANTS.OPTION_MAX_CHARS;
  }

  PQ.ROOM_DEFAULTS = ROOM_DEFAULTS;
  PQ.TYPE_CONSTANTS = TYPE_CONSTANTS;
  PQ.CODE_AREA = CODE_AREA;
  PQ.codeAreaFor = codeAreaFor;
  PQ.floorPx = floorPx;
  PQ.sourceMetrics = sourceMetrics;
  PQ.desiredFontPx = desiredFontPx;
  PQ.derivedFontPx = derivedFontPx;
  PQ.measureWell = measureWell;
  PQ.overflows = overflows;
  PQ.refit = refit;
  PQ.fitVerdict = fitVerdict;
  PQ.fitEdgeClasses = fitEdgeClasses;
  PQ.applyClippedEdge = applyClippedEdge;
  PQ.optionFits = optionFits;
})(typeof window !== "undefined" ? window : globalThis);
