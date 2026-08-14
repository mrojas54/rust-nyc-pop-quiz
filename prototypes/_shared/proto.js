/* ===========================================================================
   Shared prototype plumbing: the source well and its trace highlight-and-dim
   (NOT syntax highlighting — see below), the PROTOTYPE badge, and the polite
   live region that AC-83 asks every state change to go through.
   =========================================================================== */

function escapeHtml(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/* ---------------------------------------------------------------------------
   SourceCode — matched to the design system's components/quiz/SourceCode.jsx:
   white well, 13px mono, muted line numbers, focusable scroll region, and
   NO SYNTAX HIGHLIGHTING.

   The first cut of this stage syntax-highlighted the Rust, which looked like an
   obvious improvement and was wrong. The reveal's trace signals with
   highlight-and-dim (PROJECTOR_SPEC §4.1); a second colour channel running
   underneath it competes with the only thing the room is meant to be reading.
   The design system had already resolved this.

   opts.hl    — 1-based line numbers to highlight (the current trace step)
   opts.focus — [start, end], 1-based inclusive; everything outside is dimmed
   opts.size  — override the 13px body (the wall sets this much larger)
   --------------------------------------------------------------------------- */
function sourceCodeHtml(code, label, opts) {
  opts = opts || {};
  const hl = opts.hl || [];
  const focus = opts.focus || null;
  const lines = String(code).replace(/\n$/, "").split("\n");
  const width = String(lines.length).length + "ch";

  const body = lines.map(function (line, i) {
    const n = i + 1;
    const cls = ["rn-src-line"];
    if (hl.indexOf(n) >= 0) cls.push("hl");
    else if (focus && (n < focus[0] || n > focus[1])) cls.push("dim");
    return '<span class="' + cls.join(" ") + '">' +
           '<span class="rn-src-ln" aria-hidden="true" style="width:' + width + '">' + n + '</span>' +
           escapeHtml(line || " ") + '</span>';
  }).join("");

  return '<div class="rn-src"' + (opts.style ? ' style="' + opts.style + '"' : '') + '>' +
    '<div class="rn-src-head"><span>' + escapeHtml(label || "Source code") + '</span>' +
      (opts.meta ? '<span>' + opts.meta + '</span>' : '') + '</div>' +
    '<div class="rn-src-scroll" role="region" aria-label="' + escapeHtml(label || "Source code") + '" tabindex="0">' +
      '<pre' + (opts.size ? ' style="font-size:' + opts.size + '"' : '') + '><code>' + body + '</code></pre>' +
    '</div></div>';
}

/* The badge is part of the artifact. A high-fidelity mockup must never be
   mistakable for the shipped product, and a caption supplied verbally does not
   travel with a screenshot. */
function mountBadge(label, note) {
  const el = document.createElement("div");
  el.className = "proto-badge";
  el.setAttribute("role", "note");
  el.innerHTML =
    "<b>PROTOTYPE</b>" +
    "<span>" + label + "</span>" +
    "<span>not wired to real data" + (note ? " · " + note : "") + "</span>";
  document.body.appendChild(el);
}

/* AC-83: every state change is announced through a polite live region. */
let _live = null;
function announce(message) {
  if (!_live) {
    _live = document.createElement("div");
    _live.className = "sr-only";
    _live.setAttribute("role", "status");
    _live.setAttribute("aria-live", "polite");
    document.body.appendChild(_live);
  }
  _live.textContent = "";
  window.setTimeout(function () { _live.textContent = message; }, 40);
}

function pct(n, total) {
  return total === 0 ? 0 : Math.round((n / total) * 100);
}

/* ---------------------------------------------------------------------------
   The trace: step the program, one comparison at a time.

   Prose can assert that dedup compares neighbours. Only the tape can show it —
   and the frame that does the work is the one where the second 2 survives
   because it is sitting next to a 3. That is the whole reason 41% of the room
   answered [1, 2, 3], and it is a picture, not a sentence.

   `cellsOnly` renders the tape without the narration, for surfaces where the
   host is speaking the line rather than the screen showing it.
   --------------------------------------------------------------------------- */
function traceStepOf(q, i) {
  return q.trace.steps[Math.max(0, Math.min(i, q.trace.steps.length - 1))];
}

/* The source well for a given trace step: highlighted lines + dimmed surround. */
function traceSourceHtml(q, i, opts) {
  const s = traceStepOf(q, i);
  opts = opts || {};
  return sourceCodeHtml(q.source, opts.label || "What does this program print?", {
    hl: s.lines, focus: s.focus, size: opts.size, style: opts.style, meta: opts.meta
  });
}

/* The callout, the value deltas, and the step progress.

   PROJECTOR_SPEC §3.5: the numeric "Step N of M" label is the accessible
   signal and the dots are supplementary, never colour-only. §4.3: stepping is
   an instant state change, and the active step always carries its number and
   its callout text so a reduced-motion or reduced-vision viewer reads the same
   story as everyone else. */
function traceNoteHtml(q, i, opts) {
  opts = opts || {};
  const t = q.trace;
  const s = traceStepOf(q, i);
  const total = t.steps.length;
  const scale = opts.big ? ' style="font-size:19px"' : "";

  const values = (s.values || []).length
    ? '<div class="trace-values">' + s.values.map(function (v) {
        return '<div class="tval"><span class="tname">' + escapeHtml(v.name) + '</span>' +
          '<span class="twas">' + escapeHtml(v.was) + '</span>' +
          '<span class="tarrow">→</span>' +
          '<span class="tnow">' + escapeHtml(v.now) + '</span></div>';
      }).join("") + '</div>'
    : '';

  const dots = t.steps.map(function (_, n) {
    return '<i class="' + (n === i ? "on" : n < i ? "past" : "") + '"></i>';
  }).join("");

  return '<div class="trace">' +
    '<div class="trace-note"' + scale + '>' +
      '<span class="step-n">Step ' + (i + 1) + ' of ' + total +
        (s.pivot ? ' · the one worth stopping on' : '') + '</span>' +
      escapeHtml(s.note) +
    '</div>' +
    values +
    /* AC-79 / §3.5: the wall takes no interaction beyond host controls, so it
       renders progress only and the host's phone drives the walk. */
    (opts.noNav
      ? '<div class="trace-nav"><span class="trace-dots" aria-hidden="true">' + dots + '</span>' +
        '<span class="pos">Step ' + (i + 1) + ' of ' + total + '</span></div>'
      : '<div class="trace-nav">' +
        '<button class="btn" style="min-height:34px;padding:4px 12px" onclick="traceStep(-1)"' +
          (i === 0 ? " disabled" : "") + ' aria-label="previous step">←</button>' +
        '<button class="btn' + (i < total - 1 ? " btn-primary" : "") +
          '" style="min-height:34px;padding:4px 12px" onclick="traceStep(1)"' +
          (i >= total - 1 ? " disabled" : "") + ' aria-label="next step">→</button>' +
        '<span class="trace-dots" aria-hidden="true">' + dots + '</span>' +
        '<span class="pos">Step ' + (i + 1) + ' of ' + total + '</span>' +
      '</div>') +
  '</div>';
}

/* Announcement text, so the trace is never a visual-only channel. */
function traceSay(q, i) {
  const s = traceStepOf(q, i);
  const v = (s.values || []).map(function (x) { return x.name + " is now " + x.now; }).join(". ");
  return "Step " + (i + 1) + " of " + q.trace.steps.length + ". " + s.note + (v ? " " + v + "." : "");
}

/* ---------------------------------------------------------------------------
   The explanation, in three beats.

   Beat order is the whole point and it is not cosmetic:

     1. what happened        — plain enough that a beginner can repeat it (AC-44)
     2. why the popular
        WRONG answer is
        tempting             — names the majority's reading and makes it
                               reasonable. This is the beat that makes being
                               wrong ordinary instead of embarrassing, and it
                               only works because it comes with a count.
     3. the bit worth
        arguing about        — the thing the segment exists to start

   Beat 2 is the one the old single-paragraph design had no room for.
   --------------------------------------------------------------------------- */
function explainHtml(q, room, opts) {
  opts = opts || {};
  const e = q.explains;
  if (!e) return "";
  const L = e.whyWrong.option;
  const n = room.votes[L];
  const share = pct(n, room.answered);
  const scale = opts.big ? ' style="font-size:18px"' : "";

  return '' +
    '<div class="beat">' +
      '<h3>What happens</h3>' +
      '<p' + scale + '>' + escapeHtml(e.what) + '</p>' +
    '</div>' +
    '<div class="beat beat-company">' +
      '<h3>Why ' + n + ' of us said ' + L + '</h3>' +
      '<p' + scale + '>' + escapeHtml(e.whyWrong.text) + '</p>' +
      '<p class="meta" style="margin:6px 0 0"><b>' + share + '% of the room read it that way.</b> ' +
      'That is not a room getting it wrong — that is a room finding the one place Rust disagrees with everything else they have used.</p>' +
    '</div>' +
    '<div class="beat">' +
      '<h3>The bit worth arguing about</h3>' +
      '<p' + scale + '>' + escapeHtml(e.argue) + '</p>' +
    '</div>';
}
