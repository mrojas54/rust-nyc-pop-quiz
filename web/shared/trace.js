/* ===========================================================================
   The trace — SPEC §5.3, ported from prototypes/_shared/proto.js.

   A trace is an ordered list of steps. Each step may highlight the lines
   executing now, dim everything outside a focus region, carry the words the
   host reads, and show debugger-style value deltas.

     lines   1-based source lines drawn as executing now
     focus   [first, last] 1-based inclusive, kept at full contrast
     note    the words the host reads for this step
     values  [{name, was, now}] — verbatim strings, "—" for absent
     pivot   optional; the one step to pause on (the wall adds "Pause here.")

   THE BOUND. SPEC §3.4 states it outright: "trace_step — in `work` bounded to
   0..M-2; `reveal` enters at M-1 and may step the whole trace." Those are
   0-BASED INDICES into steps[]. The last step (M-1) is the resolving one — its
   `values` names `stdout` — and D-10 withholds it until reveal, so `work` tops
   out one before it at M-2, displayed as "Step M-1 of M".

   This file does not know the phase. The BOUND IS THE CALLER'S, because it is
   room state (T-04a owns it) and the static fallback has no room at all. What
   this file gives you is workMaxIndex()/revealEntryIndex() so no caller has to
   get the index base right by hand, and clampStep() so none can step past it.

   Steppable BOTH ways, everywhere. The wall is host-stepped (AC-79: the wall
   has no interactive control) and take-it-home steps at the reader's own pace
   (touchpoint T-15), but the stepping logic is the same and lives here.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* How far `work` may step: the last index before the resolving step. Clamped
     at 0 so a two-step trace does not produce -1. SPEC §7.4 refuses to affirm a
     question whose trace has fewer than two steps, so M >= 2 in the bank; the
     clamp is for fixtures and the static fallback. */
  function workMaxIndex(total) { return Math.max(0, total - 2); }

  /* Where `reveal` enters: the resolving step. */
  function revealEntryIndex(total) { return Math.max(0, total - 1); }

  /* Keep an index inside [0, maxIndex], where maxIndex defaults to the last
     step. This is the one place a step index is bounded, so "no step beyond
     M-2 in work" (AC-97, AC-102) is one call rather than a convention. */
  function clampStep(i, total, maxIndex) {
    var hi = maxIndex === undefined || maxIndex === null
      ? total - 1
      : Math.min(maxIndex, total - 1);
    if (hi < 0) hi = 0;
    var n = Math.floor(Number(i) || 0);
    return Math.max(0, Math.min(n, hi));
  }

  /* The step at i, bounded. Ported from proto.js's traceStepOf, which clamped
     to the last step; this one also honours a caller's bound. */
  function traceStepOf(q, i, maxIndex) {
    var steps = q.trace.steps;
    return steps[clampStep(i, steps.length, maxIndex)];
  }

  /* The source well for a given trace step: highlighted lines, dimmed surround.

     The phase is passed through to well.js, which decides colour. In `work` and
     `reveal` that means none — the trace is the only signal channel, which is
     the reason the well stops colouring at all (AC-99). */
  function traceSourceHtml(q, i, opts) {
    opts = opts || {};
    var s = traceStepOf(q, i, opts.maxIndex);
    return PQ.sourceWellHtml(q.source, {
      phase: opts.phase,
      label: opts.label || PQ.t("wall_live_well_header"),
      hl: s.lines,
      focus: s.focus,
      size: opts.size,
      style: opts.style,
      meta: opts.meta,
      fit: opts.fit
    });
  }

  /* The callout, the value deltas, and the step progress.

     The numeric "Step N of M" is the accessible signal and the dots are
     supplementary, never colour-only (AC-40, PROJECTOR_SPEC §3.5). The active
     step always carries both its number and its words, so a reduced-motion or
     reduced-vision viewer reads the same story as the room.

     opts.noNav    progress only, no buttons — the wall (AC-79)
     opts.maxIndex the bound; also disables "next" at the bound
     opts.onStep   name of the global step handler for the buttons
     opts.big      the wall's larger note size
  */
  function traceNoteHtml(q, i, opts) {
    opts = opts || {};
    var esc = PQ.escapeHtml;
    var steps = q.trace.steps;
    var total = steps.length;
    var at = clampStep(i, total, opts.maxIndex);
    var s = steps[at];
    var hi = opts.maxIndex === undefined || opts.maxIndex === null
      ? total - 1
      : Math.min(opts.maxIndex, total - 1);
    var scale = opts.big ? ' style="font-size:19px"' : "";
    var handler = opts.onStep || "traceStep";

    var values = (s.values || []).length
      ? '<div class="trace-values">' + s.values.map(function (v) {
          return '<div class="tval"><span class="tname">' + esc(v.name) + "</span>" +
            '<span class="twas">' + esc(v.was === undefined || v.was === null ? "—" : v.was) + "</span>" +
            '<span class="tarrow" aria-hidden="true">→</span>' +
            '<span class="tnow">' + esc(v.now === undefined || v.now === null ? "—" : v.now) + "</span></div>";
        }).join("") + "</div>"
      : "";

    /* Dots span the whole trace, not the bounded part: the room should see how
       far there is to go. Only the STEPPING is bounded. */
    var dots = steps.map(function (_, n) {
      return '<i class="' + (n === at ? "on" : n < at ? "past" : "") + '"></i>';
    }).join("");

    var progress = PQ.t("wall_trace_step", { N: at + 1, M: total });

    return '<div class="trace">' +
      '<div class="trace-note"' + scale + ">" +
        '<span class="step-n">' + esc(progress) + (s.pivot ? " · " + esc(PQ.t("wall_trace_pivot")) : "") + "</span>" +
        esc(s.note) +
      "</div>" +
      values +
      (opts.noNav
        ? '<div class="trace-nav"><span class="trace-dots" aria-hidden="true">' + dots + "</span>" +
          '<span class="pos">' + progress + "</span></div>"
        : '<div class="trace-nav">' +
          '<button class="btn" style="min-height:34px;padding:4px 12px" onclick="' + handler + '(-1)"' +
            (at === 0 ? " disabled" : "") + ' aria-label="' + PQ.escapeAttr(PQ.t("proposed_trace_previous_step")) + '">←</button>' +
          '<button class="btn' + (at < hi ? " btn-primary" : "") +
            '" style="min-height:34px;padding:4px 12px" onclick="' + handler + '(1)"' +
            (at >= hi ? " disabled" : "") + ' aria-label="' + PQ.escapeAttr(PQ.t("proposed_trace_next_step")) + '">→</button>' +
          '<span class="trace-dots" aria-hidden="true">' + dots + "</span>" +
          '<span class="pos">' + progress + "</span>" +
        "</div>") +
    "</div>";
  }

  /* Announcement text, so the trace is never a visual-only channel (AC-83).
     Spoken, not rendered — no markup, no escaping. */
  function traceSay(q, i, maxIndex) {
    var steps = q.trace.steps;
    var at = clampStep(i, steps.length, maxIndex);
    var s = steps[at];
    var v = (s.values || []).map(function (x) {
      return PQ.t("proposed_trace_value_now", { name: x.name, now: x.now });
    }).join(". ");
    return PQ.t("wall_trace_step", { N: at + 1, M: steps.length }) + ". " + s.note + (v ? " " + v + "." : "");
  }

  PQ.workMaxIndex = workMaxIndex;
  PQ.revealEntryIndex = revealEntryIndex;
  PQ.clampStep = clampStep;
  PQ.traceStepOf = traceStepOf;
  PQ.traceSourceHtml = traceSourceHtml;
  PQ.traceNoteHtml = traceNoteHtml;
  PQ.traceSay = traceSay;
})(typeof window !== "undefined" ? window : globalThis);
