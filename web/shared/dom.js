/* ===========================================================================
   The two primitives every other module here needs.

   Classic script, no module graph. web/README.md: the prototype loads
   _shared/*.js through plain <script src> and communicates through globals, so
   the port stays classic and the browser needs no bundler. Everything attaches
   to one namespace object so four <script src> tags compose in any order.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* Ported verbatim from prototypes/_shared/proto.js.

     Every renderer in web/shared/ runs source text, host-authored notes and
     value strings through this before they reach innerHTML. The source is
     program text and will contain < and & the moment anyone writes a generic
     or a bit-and. */
  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  /* escapeHtml is safe for TEXT content. An attribute value also has to survive
     its own quotes, and `aria-label="..."` in well.js is exactly that position.
     The prototype had no attribute escaper because its labels were literals;
     T-05/T-06/T-07 may pass a bank-derived label, and a question title with a
     double quote in it would otherwise break out of the attribute. */
  function escapeAttr(s) {
    return escapeHtml(s).replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  /* AC-83: every state change is announced through a polite live region.

     Ported from proto.js, including the 40ms clear-then-set. That delay is not
     decoration: assistive technology coalesces a mutation into the previous
     announcement if the node's text changes in the same tick, so a phase change
     that lands within a frame of the last one would be silently dropped. The
     room only ever hears each transition once, so dropping one loses it.

     SPEC §11's "Live region (AC-83), verbatim" row holds the ten strings this
     is called with. They live in copy.js, not here. */
  var _live = null;
  function announce(message) {
    var doc = root.document;
    if (!doc || !doc.body) return false;
    if (!_live) {
      _live = doc.createElement("div");
      _live.className = "sr-only";
      _live.setAttribute("role", "status");
      _live.setAttribute("aria-live", "polite");
      doc.body.appendChild(_live);
    }
    _live.textContent = "";
    var later = function () { _live.textContent = message; };
    if (typeof root.setTimeout === "function") root.setTimeout(later, 40);
    else later();
    return true;
  }

  /* Test seam only: forget the region so a fresh document starts clean. */
  function _resetLiveRegion() { _live = null; }

  PQ.escapeHtml = escapeHtml;
  PQ.escapeAttr = escapeAttr;
  PQ.announce = announce;
  PQ._resetLiveRegion = _resetLiveRegion;
})(typeof window !== "undefined" ? window : globalThis);
