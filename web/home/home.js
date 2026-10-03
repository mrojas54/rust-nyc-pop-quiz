/* ===========================================================================
   Take it home — SPEC §13, T-12. Ported from prototypes/take-it-home.html.

   The last released question, alone, at the reader's own pace: the source
   with colour, the five options with the ✓ on the answer, the three beats with
   the middle one repeated for EVERY incorrect option, the trace steppable both
   ways over the whole of it, and *How we know* — the wall's list, the machine
   it ran on, and AC-71's sentence.

   NO ROOM STATE, OF ANY KIND (AC-56, D-12). The snapshot this page draws from
   (`used::TakeHome`, written by the room's `released` transition) carries no
   count, no split and no most-chosen option, and nothing here computes one.

   TWO LAYERS, as in wall.js:

     html(snapshot, opts)  PURE. The snapshot in, the page's markup out. No DOM,
                           no clock. web/test/home.test.js drives it in node:vm.
     boot()                THE BROWSER. Read the snapshot the room wrote into
                           the page, render it, and step the trace on the
                           buttons and on ← →.

   WHERE THE PAGE DEPARTS FROM THE PROTOTYPE, and why (SPEC outranks the
   prototype on copy and order; the prototype wins on look):

     - Section order follows §11's "Take it home, headings in order": the
       question, What happens, Why… (one per incorrect option), What to
       remember, Walk it yourself, How we know.
     - Prototype prose that §11 does not author is not shipped: the masthead's
       sub-line, the rotation notice, the "go as slowly as you like" callout,
       the "How you got here" handover, "the answer" beside the ✓, and the
       receipt's Miri caveat. The key hint reuses §11's `static_step_trace`.
     - The trace well carries no syntax colour: it signals by highlight-and-dim,
       exactly as the wall's does (AC-99). The program well, above, has colour.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  function esc(s) { return PQ.escapeHtml(s); }

  /* PROPOSED-§11 (F-34). Strings this page needs that SPEC §11 does not author
     yet. They live in web/shared/copy.js's PROPOSED-§11 block, words unchanged,
     so both lints cover them (T-22); this table names their keys, and
     `PQ.Home.PROPOSED` reads them through for anything that wants the words. */
  var PROPOSED = {
    /* The page before the first release. */
    nothing_yet: "proposed_home_nothing_yet",
    /* §13: "the Miri row saying the check was run separately". */
    miri_separately: "proposed_home_miri_separately",
    /* How we know, the machine's rows — the prototype's own labels. */
    row_compiler: "proposed_home_row_compiler",
    row_edition: "proposed_home_row_edition",
    row_target: "proposed_home_row_target",
    row_flags: "proposed_home_row_flags",
    row_miri: "proposed_home_row_miri",
    miri_seeds: "proposed_home_miri_seeds"
  };
  function p(name) { return PQ.t(PROPOSED[name]); }

  var MONTHS = ["January", "February", "March", "April", "May", "June", "July",
    "August", "September", "October", "November", "December"];

  /* `2026-10-14` as a reader says it: `October 14`. The meetup date is already
     New York's (used.rs); nothing here consults a clock or a time zone. */
  function spokenDate(ymd) {
    var m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(ymd));
    if (!m) return String(ymd);
    return MONTHS[Number(m[2]) - 1] + " " + Number(m[3]);
  }

  /* The question-shaped object trace.js reads: the source and every step. */
  function traceQuestion(snap) {
    return { source: snap.source, trace: { steps: snap.trace } };
  }

  /* --- Pieces ------------------------------------------------------------ */

  function mastheadHtml(title) {
    return '<header class="masthead">' +
      '<div class="kicker">' + esc(PQ.t("title_wordmark")) + "</div>" +
      "<h1>" + esc(title) + "</h1>" +
    "</header>";
  }

  /* The program, in colour (§13). `live` is the phase whose rule this well
     follows — source with syntax colour — and the only one well.js offers it
     in besides `closed` and `split`. The page is not in a phase; it borrows the
     reading phases' rule, which is what §13 asks for. */
  function programHtml(snap) {
    return PQ.sourceWellHtml(snap.source, { phase: "live", label: PQ.t("wall_live_well_header") });
  }

  /* The five options, the answer marked with the ✓ through check.js (glyph and
     colour together, AC-40). Nobody's choice is marked (AC-94); there is no
     choice here to mark. */
  function optionsHtml(snap) {
    return '<div class="opts" data-provenance="machine">' + snap.options.map(function (o) {
      return '<div class="opt">' +
        '<span class="l">' + esc(o.letter) + "</span>" +
        '<span class="otext">' + esc(o.text) + "</span>" +
        (o.correct ? PQ.correctHtml(null, { className: "tick" }) : "") +
      "</div>";
    }).join("") + "</div>";
  }

  /* The three beats, without the room. The middle one is repeated for every
     incorrect option (§13): what makes a reading reasonable is a fact about the
     language, not about that night's room, so it needs no count. Human-reviewed
     prose, under the human provenance marker (AC-74). */
  function beatsHtml(snap) {
    var why = snap.why.map(function (w) {
      return '<div class="beat beat-company">' +
        "<h3>" + esc(PQ.t("home_heading_why", { X: w.letter })) + ": <code>" + esc(w.text) + "</code></h3>" +
        "<p>" + esc(w.why_tempting) + "</p>" +
      "</div>";
    }).join("");
    return '<section class="sec beats by-human" data-provenance="human">' +
      '<span class="provenance human" aria-hidden="true">✎</span>' +
      '<div class="beat"><h3>' + esc(PQ.t("home_heading_what")) + "</h3><p>" + esc(snap.what) + "</p></div>" +
      why +
      '<div class="beat"><h3>' + esc(PQ.t("home_heading_remember")) + "</h3><p>" + esc(snap.takeaway) + "</p></div>" +
    "</section>";
  }

  /* §11's `← → step the trace`, with the arrows set as keys. */
  function keyHintHtml() {
    return '<div class="keyhint">' +
      esc(PQ.t("static_step_trace")).replace(/[←→]/g, function (k) { return "<kbd>" + k + "</kbd>"; }) +
    "</div>";
  }

  /* The trace at step `at`, the whole of it steppable both ways: no bound but
     the ends (§13 — the reader's own pace, nothing timed, nothing recorded). */
  function walkInnerHtml(snap, at) {
    var q = traceQuestion(snap);
    return PQ.traceSourceHtml(q, at, { phase: "reveal", label: PQ.t("wall_live_well_header") }) +
      PQ.traceNoteHtml(q, at, { onStep: "PopQuizHomeStep" });
  }

  function walkHtml(snap, at) {
    return '<section class="sec walk">' +
      '<h2 class="sec-h">' + esc(PQ.t("home_heading_walk")) + "</h2>" +
      '<div class="walk-body" id="home-walk">' + walkInnerHtml(snap, at) + "</div>" +
      keyHintHtml() +
    "</section>";
  }

  function row(label, valueHtml) {
    return '<div class="row"><span class="k">' + esc(label) + '</span><span class="v">' + valueHtml + "</span></div>";
  }

  /* `-C opt-level=0 -C overflow-checks=on -C debug-assertions=on`: the flag set
     as rustc spells it, from the record's three fields. */
  function flagsText(f) {
    function onOff(b) { return b ? "on" : "off"; }
    return "-C opt-level=" + f.opt_level +
      " -C overflow-checks=" + onOff(f.overflow_checks) +
      " -C debug-assertions=" + onOff(f.debug_assertions);
  }

  /* The machine's rows (§13, AC-87). Only what the record holds; for a legacy
     record the target reads *not recorded* and the Miri row says the check was
     run separately. Nothing is back-filled (G-2, D-16). */
  function machineRowsHtml(m) {
    var out = [];
    /* The -Vv block scrolls sideways, so it takes the focus (AC-82): a
       scroller the keyboard cannot reach is a scroller it cannot scroll. */
    out.push(row(p("row_compiler"), '<pre class="vv" tabindex="0" role="region" aria-label="' +
      PQ.escapeAttr(p("row_compiler")) + '">' + esc(m.compiler) + "</pre>"));
    out.push(row(p("row_edition"), esc(m.edition)));
    out.push(row(p("row_target"), m.target === null || m.target === undefined
      ? '<span class="not-recorded">' + esc(PQ.t("home_not_recorded")) + "</span>"
      : esc(m.target)));
    if (m.flags) out.push(row(p("row_flags"), esc(flagsText(m.flags))));
    if (m.miri) {
      var mi = m.miri;
      var parts = [];
      if (mi.version) parts.push(esc(mi.version));
      if (mi.configs && mi.configs.length) parts.push(esc(mi.configs.join(", ")));
      if (mi.seeds && mi.seeds.length) parts.push(esc(p("miri_seeds") + " " + mi.seeds.join(", ")));
      /* A legacy record's pass ran outside the verifier and recorded no
         configuration: say so rather than print an empty row. */
      out.push(row(p("row_miri"), m.legacy || !parts.length
        ? '<span class="separately">' + esc(p("miri_separately")) + "</span>"
        : parts.join('<span class="sep"> · </span>')));
    }
    return out.join("");
  }

  /* How we know: the wall's list verbatim (answers::receipt_lines), the machine
     beneath it, then AC-71's sentence. Machine-established, under the machine
     provenance marker (AC-74). */
  function howWeKnowHtml(snap) {
    return '<section class="sec how">' +
      '<h2 class="sec-h">' + esc(PQ.t("home_heading_how_we_know")) + "</h2>" +
      '<div class="receipt by-machine" data-provenance="machine">' +
        '<div class="steps">' + snap.receipt.lines.map(function (l) {
          return '<span class="receipt-line">' + esc(l) + "</span>";
        }).join("") + "</div>" +
        '<div class="machine">' + machineRowsHtml(snap.machine) + "</div>" +
        '<p class="caveat">' + esc(PQ.t("home_machine_only")) + "</p>" +
      "</div>" +
    "</section>";
  }

  /* --- The one pure renderer -------------------------------------------- */

  /* The page's markup for a snapshot (`used::TakeHome`), or for `null` — no
     release yet.

       opts.step   the trace step to draw, 0-based; clamped to the trace */
  function html(snap, opts) {
    opts = opts || {};
    if (!snap) {
      return '<div class="sheet nothing-yet">' +
        mastheadHtml(p("nothing_yet")) +
      "</div>";
    }
    var at = PQ.clampStep(opts.step || 0, snap.trace.length);
    return '<div class="sheet">' +
      mastheadHtml(PQ.t("home_heading_question", { date: spokenDate(snap.meetup_date) })) +
      '<section class="sec program">' + programHtml(snap) + optionsHtml(snap) + "</section>" +
      beatsHtml(snap) +
      walkHtml(snap, at) +
      howWeKnowHtml(snap) +
    "</div>";
  }

  /* --- The browser ------------------------------------------------------- */

  /* Mount a snapshot into `el`. Returns `step(d)`, which moves the trace by
     `d` and redraws only the walk: the rest of the page never moves under the
     reader. Stepping counts nothing and sends nothing. */
  function mount(el, snap) {
    var at = 0;
    el.innerHTML = html(snap, { step: at });
    function step(d) {
      if (!snap) return at;
      var next = PQ.clampStep(at + d, snap.trace.length);
      if (next === at) return at;
      at = next;
      var doc = el.ownerDocument;
      var walk = doc.getElementById("home-walk");
      /* AC-82: redrawing the walk takes the step button that was pressed with
         it. Give the focus to the same direction's button, or the other one
         once this one is disabled at the end. */
      var active = doc.activeElement;
      var pressed = walk && active && walk.contains && walk.contains(active) && active.tagName === "BUTTON";
      if (walk) walk.innerHTML = walkInnerHtml(snap, at);
      if (pressed && walk.querySelectorAll) {
        var buttons = walk.querySelectorAll(".trace-nav button");
        var want = buttons[d < 0 ? 0 : 1];
        if (!want || want.disabled) want = buttons[d < 0 ? 1 : 0];
        if (want && !want.disabled) want.focus({ preventScroll: true });
      }
      if (PQ.announce) PQ.announce(PQ.traceSay(traceQuestion(snap), at));
      return at;
    }
    return { step: step, at: function () { return at; } };
  }

  /* Read the snapshot the room wrote into the page. */
  function readSnapshot(doc) {
    var slot = doc.getElementById("take-home");
    if (!slot) return null;
    try { return JSON.parse(slot.textContent || "null"); } catch (e) { return null; }
  }

  function boot() {
    var doc = root.document;
    var el = doc && doc.getElementById("home");
    if (!el) return null;
    var page = mount(el, readSnapshot(doc));
    /* trace.js's buttons call a global handler by name. */
    root.PopQuizHomeStep = page.step;
    doc.addEventListener("keydown", function (ev) {
      if (ev.altKey || ev.ctrlKey || ev.metaKey || ev.shiftKey) return;
      /* A focused code well scrolls sideways on the arrow keys; leave it be. */
      var t = ev.target;
      if (t && t.closest && t.closest(".rn-src-scroll, .vv, input, textarea")) return;
      if (ev.key === "ArrowRight" || ev.key === "Right") { page.step(1); }
      else if (ev.key === "ArrowLeft" || ev.key === "Left") { page.step(-1); }
    });
    return page;
  }

  PQ.Home = {
    html: html,
    mount: mount,
    boot: boot,
    spokenDate: spokenDate
  };
  /* The PROPOSED-§11 words by this page's own names, read from copy.js. */
  Object.defineProperty(PQ.Home, "PROPOSED", {
    enumerable: true,
    get: function () {
      var out = {};
      Object.keys(PROPOSED).forEach(function (k) { out[k] = p(k); });
      return out;
    }
  });

  if (root.document && root.document.getElementById && root.document.getElementById("home")) {
    boot();
  }
})(typeof window !== "undefined" ? window : globalThis);
