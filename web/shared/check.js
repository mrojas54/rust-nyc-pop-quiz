/* ===========================================================================
   The ✓ — AC-40.

   "Correct and incorrect are conveyed by shape or glyph as well as colour,
   never by colour alone." Four surfaces render a correct answer — the wall's
   option, the buzzer's "✓ It was X.", take-it-home, and the review screen — and
   each of them is a different ticket. If each writes its own ✓, one of them
   eventually ships the green without the glyph.

   So the glyph and the colour class are produced by the SAME expression here.
   There is no exported path that yields `rn-correct` without also yielding the
   ✓, which is what makes AC-40 a property of this file rather than a habit
   four tickets have to keep.

   The inverse is deliberately absent. SPEC §11's Forbidden row bans the
   BALLOT X (U+2717) and DESIGN.md says it appears nowhere; AC-94 says no
   participant surface may mark a participant's own answer incorrect. There is
   no `incorrect()` to call, and the glyph itself is not written in this file —
   check.test.js greps for it, so the absence is checked rather than intended.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  var CHECK = "✓"; /* ✓ U+2713 CHECK MARK */
  var CORRECT_CLASS = "rn-correct";

  /* The glyph alone, for a caller assembling its own markup. Still returns the
     glyph and nothing but — a caller cannot get the class from here. */
  function checkMark() { return CHECK; }

  /* Correct, marked. Returns markup carrying the glyph AND the colour class,
     inseparably.

     `text` is escaped by the caller's escapeHtml — option texts are program
     text and may contain < or &.

     The ✓ is aria-hidden and `srLabel` carries the word, because a screen
     reader announcing "check mark" before every option is noise; the word
     "Correct" is the signal. That keeps three channels alive: glyph, colour,
     and announced text. */
  function correctHtml(text, opts) {
    opts = opts || {};
    var esc = PQ.escapeHtml || function (s) { return String(s); };
    var srLabel = opts.srLabel === undefined ? "Correct" : opts.srLabel;
    var tag = opts.tag || "span";
    var extra = opts.className ? " " + opts.className : "";

    return "<" + tag + ' class="' + CORRECT_CLASS + extra + '">' +
             '<span class="rn-check" aria-hidden="true">' + CHECK + "</span>" +
             (srLabel ? '<span class="sr-only">' + esc(srLabel) + "</span>" : "") +
             (text === undefined || text === null ? "" : " " + esc(text)) +
           "</" + tag + ">";
  }

  /* The class, for a caller that must put it on an element it already owns —
     a wall option chip, a split bar. Returns the class AND the glyph markup
     together so the caller cannot take one without seeing the other. */
  function correctParts(opts) {
    opts = opts || {};
    var esc = PQ.escapeHtml || function (s) { return String(s); };
    var srLabel = opts.srLabel === undefined ? "Correct" : opts.srLabel;
    return {
      className: CORRECT_CLASS,
      glyph: CHECK,
      glyphHtml: '<span class="rn-check" aria-hidden="true">' + CHECK + "</span>" +
                 (srLabel ? '<span class="sr-only">' + esc(srLabel) + "</span>" : "")
    };
  }

  PQ.CHECK = CHECK;
  PQ.CORRECT_CLASS = CORRECT_CLASS;
  PQ.checkMark = checkMark;
  PQ.correctHtml = correctHtml;
  PQ.correctParts = correctParts;
})(typeof window !== "undefined" ? window : globalThis);
