/* ===========================================================================
   The ✓ — AC-40.

   "Correct and incorrect are conveyed by shape or glyph as well as colour,
   never by colour alone." Four surfaces render a correct answer — the wall's
   option, the buzzer's "✓ It was X.", take-it-home, and the review screen — and
   each is a different ticket. If each writes its own ✓, one eventually ships
   the green without the glyph.

   HOW THE SEPARATION IS PREVENTED, and it is by construction, not by a test.

   An earlier cut of this file handed callers `{className, glyphHtml}` as two
   fields and let `correctHtml` concatenate them. A code review defeated it in
   one line: keep `m.className`, drop `m.glyphHtml` behind an option, and you
   get `<span class="rn-correct"> A</span>` — colour without glyph — with no new
   reference to the class for a structural test to count.

   So no value in this module ever carries the class WITHOUT the glyph:

     - `openTag()` returns the complete opening tag with the glyph already
       inside it, as one opaque string. `correctHtml` receives that and has no
       access to the class on its own, so it cannot drop the glyph and keep the
       colour — dropping the string drops both, which is not an AC-40 defect.
     - `applyCorrect()` is the DOM equivalent, and does both in one call.
     - `CORRECT_CLASS` is NOT exported. Anything that could obtain the bare
       class name could apply it without the glyph, so nothing can.

   What is still possible, stated plainly rather than papered over: someone can
   edit `openTag` or `applyCorrect` themselves. No source-level test can prevent
   that, and check.test.js does not claim to. Those two functions are small,
   adjacent, and commented as load-bearing, and `EVALUATION.md` puts AC-40's
   real proof on the four consuming surfaces — each of which carries its own
   assertion. This module's job is to make the correct thing the only thing
   reachable from outside.

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
  var CORRECT_CLASS = "rn-correct"; /* never exported — see the header */

  /* The glyph alone. Safe to export: a glyph without the colour is not the
     failure AC-40 describes; only the reverse is. */
  function checkMark() { return CHECK; }

  /* The glyph markup, aria-hidden with the word beside it.

     A screen reader announcing "check mark" before every option is noise; the
     word is the signal. Three channels stay alive — glyph, colour, and
     announced text. */
  function glyphHtml(opts) {
    var esc = PQ.escapeHtml || function (s) { return String(s); };
    var srLabel = opts.srLabel === undefined ? PQ.t("proposed_check_correct") : opts.srLabel;
    return '<span class="rn-check" aria-hidden="true">' + CHECK + "</span>" +
           (srLabel ? '<span class="sr-only">' + esc(srLabel) + "</span>" : "");
  }

  /* ONE OF THE TWO PLACES THE CLASS IS EVER USED, and it emits the glyph in the
     same breath. Returns a complete opening tag with the glyph already inside,
     so nothing downstream ever holds the class separately from it. */
  function openTag(opts) {
    var tag = opts.tag || "span";
    var extra = opts.className ? " " + opts.className : "";
    return "<" + tag + ' class="' + CORRECT_CLASS + extra + '">' + glyphHtml(opts);
  }

  /* Correct, marked. `text` is escaped — option texts are program text and may
     contain < or &. */
  function correctHtml(text, opts) {
    opts = opts || {};
    var esc = PQ.escapeHtml || function (s) { return String(s); };
    var tag = opts.tag || "span";
    return openTag(opts) +
           (text === undefined || text === null ? "" : " " + esc(text)) +
           "</" + tag + ">";
  }

  /* THE OTHER PLACE THE CLASS IS USED — for an element the caller already owns,
     such as a wall option chip or a split bar gaining its ✓ at reveal. Atomic:
     the class and the glyph go on together or not at all, which is why this
     exists instead of exporting the class name. */
  function applyCorrect(el, opts) {
    if (!el || !el.classList) return false;
    opts = opts || {};
    el.classList.add(CORRECT_CLASS);
    el.innerHTML = glyphHtml(opts) + (el.innerHTML || "");
    return true;
  }

  PQ.CHECK = CHECK;
  PQ.checkMark = checkMark;
  PQ.correctHtml = correctHtml;
  PQ.applyCorrect = applyCorrect;
})(typeof window !== "undefined" ? window : globalThis);
