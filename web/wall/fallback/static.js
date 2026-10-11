/* ===========================================================================
   The static fallback — SPEC §12, AC-102 (with AC-97, AC-99).

   The wall with no room: no socket, the phase in a local variable, the question
   baked into the page by the pipeline (`popquiz.fallback`), driven from the
   keyboard. It draws through the SAME renderer as the live wall
   (PopQuiz.Wall.html / mount), so the fallback is that design, never a second one.

   TWO PIECES, both pure except boot():

     frame(bake, state, counts)  the wall frame `view::wall` would send for this
                                 phase and step — a line-for-line mirror of
                                 room/src/view.rs (sealed::wall, revealed::wall),
                                 every string from copy.js. The parity test holds
                                 it equal to the room's generated q3 frames.

     next / back / step          the driver, as transitions over {phase, at}.
                                 Space next phase, Esc back a phase, ← → step the
                                 trace inside work and reveal and nowhere else.

   The phase rules are the room's (AC-97, AC-99): `work` walks steps 0..M-2 with
   no `stdout` row and no ✓, no receipt, no colour; `reveal` enters at M-1.

   What the page does not have, because there is no room:
     - the join strip (SPEC §12). In `idle` its place holds §11's key legend.
     - counts. Nobody answered on a phone, so the split is §4.4's empty bars and
       the reveal's middle line is §4.5's "nobody read it another way" — the
       room's own rendering of a room where nobody answered. The
       `‹answered› of ‹present›` line is about phones in a room and goes with the
       join strip. `counts` exists only so the parity test can feed the fixture's.

   The receipt lines and the correct letter are baked by the pipeline
   (`receipt.receipt_lines`, `bank.correct_index` — the twins of the room's
   functions, G-7). This file prints them; it derives neither.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  var PHASES = ["idle", "live", "closed", "split", "work", "reveal", "released"];

  function options(bake) {
    return bake.options.map(function (o) { return { letter: o.letter, text: o.text }; });
  }

  function copyStep(s) {
    return {
      lines: s.lines.slice(),
      focus: s.focus.slice(),
      note: s.note,
      values: s.values.map(function (v) { return { name: v.name, was: v.was, now: v.now }; }),
      pivot: !!s.pivot
    };
  }

  /* The walk-through's half: steps 0..M-2, no `stdout` row in any of them
     (answers::load; G-3). */
  function walk(bake) {
    return bake.trace.slice(0, -1).map(function (s) {
      var c = copyStep(s);
      c.values = c.values.filter(function (v) { return v.name !== "stdout"; });
      return c;
    });
  }

  function traceView(at, m, step) {
    var t = {
      at: at,
      m: m,
      label: PQ.t("wall_trace_step", { N: at + 1, M: m }),
      step: step
    };
    if (step.pivot) t.pause = PQ.t("wall_trace_pivot");
    return t;
  }

  /* §4.4: whole percent of `answered`, rounded half up; 0 when nobody answered. */
  function splitView(counts) {
    var totals = counts ? counts.totals : [0, 0, 0, 0, 0];
    var answered = totals.reduce(function (a, b) { return a + b; }, 0);
    var s = {
      bars: ["A", "B", "C", "D", "E"].map(function (letter, i) {
        var n = totals[i];
        return {
          letter: letter,
          count: n,
          percent: answered === 0 ? 0 : Math.floor((n * 100 + Math.floor(answered / 2)) / answered)
        };
      }),
      answered: answered,
      present: counts ? counts.present : 0
    };
    if (counts) s.line = PQ.t("wall_split_answered", { answered: answered, present: counts.present });
    return s;
  }

  /* §4.5: the most-chosen incorrect option, ties to the lower letter; none when
     no incorrect option has a vote. The twin of the room's Vault::judge. */
  function middle(bake, split) {
    var best = null;
    split.bars.forEach(function (b) {
      if (b.letter === bake.correct) return;
      if (b.count > 0 && (!best || b.count > best.count)) best = b;
    });
    if (!best) return { line: PQ.t("wall_reveal_nobody_else") };
    return {
      letter: best.letter,
      count: best.count,
      line: PQ.t("wall_reveal_most_chosen", { n: best.count, X: best.letter })
    };
  }

  function legend() {
    return [PQ.t("static_next_phase"), PQ.t("static_step_trace"), PQ.t("static_back_phase")].join(" · ");
  }

  /* The frame for one {phase, at}. Throws on an unknown phase. */
  function frame(bake, state, counts) {
    var phase = PQ.assertPhase(state.phase);
    var m = bake.trace.length;
    var f = { phase: phase };
    if (phase === "idle") {
      f.title = PQ.t("wall_idle_title");
      f.strip = legend();
    } else if (phase === "live" || phase === "closed" || phase === "split") {
      f.source = bake.source;
      f.colour = true;
      f.options = options(bake);
      if (phase === "live") f.well_header = PQ.t("wall_live_well_header");
      else if (phase === "closed") f.strip = PQ.t("wall_closed");
      else f.split = splitView(counts);
    } else if (phase === "work") {
      f.source = bake.source;
      f.colour = false;
      f.options = options(bake);
      var steps = walk(bake);
      var at = clamp(state.at, 0, steps.length - 1);
      f.trace = traceView(at, m, steps[at]);
    } else if (phase === "reveal") {
      var split = splitView(counts);
      var rat = clamp(state.at, 0, m - 1);
      f.source = bake.source;
      f.colour = false;
      f.options = options(bake);
      f.split = split;
      f.trace = traceView(rat, m, copyStep(bake.trace[rat]));
      f.reveal = {
        correct: bake.correct,
        mark: "✓",
        middle: middle(bake, split),
        receipt: { heading: bake.receipt.heading, lines: bake.receipt.lines.slice() }
      };
    } else { /* released */
      f.released = {
        title: PQ.t("wall_released_title"),
        link: bake.home_link
      };
    }
    return f;
  }

  function clamp(n, lo, hi) { return Math.max(lo, Math.min(hi, n | 0)); }

  /* --- The driver --------------------------------------------------------- */

  /* Where a phase is entered: work at its first step, reveal at the resolving
     step M-1 (AC-97), every other phase has no step. */
  function enter(bake, phase) {
    if (phase === "work") return { phase: phase, at: 0 };
    if (phase === "reveal") return { phase: phase, at: bake.trace.length - 1 };
    return { phase: phase, at: 0 };
  }

  /* Space. `released` is the end; nothing follows it. */
  function next(bake, state) {
    var i = PHASES.indexOf(state.phase);
    return i < PHASES.length - 1 ? enter(bake, PHASES[i + 1]) : state;
  }

  /* Esc. `idle` is the start. */
  function back(bake, state) {
    var i = PHASES.indexOf(state.phase);
    return i > 0 ? enter(bake, PHASES[i - 1]) : state;
  }

  /* ← (-1) and → (+1): inside work (0..M-2) and reveal (0..M-1) only. */
  function step(bake, state, dir) {
    var m = bake.trace.length;
    var hi = state.phase === "work" ? m - 2 : state.phase === "reveal" ? m - 1 : null;
    if (hi === null) return state;
    return { phase: state.phase, at: clamp(state.at + dir, 0, hi) };
  }

  var KEYS = {
    /* A real keyboard sends key " " with code "Space"; old engines "Spacebar";
       synthetic drivers (the test-full run's c11 browser) the word "Space". */
    " ": function (b, s) { return next(b, s); },
    Space: function (b, s) { return next(b, s); },
    Spacebar: function (b, s) { return next(b, s); },
    Escape: function (b, s) { return back(b, s); },
    Esc: function (b, s) { return back(b, s); },
    ArrowRight: function (b, s) { return step(b, s, 1); },
    Right: function (b, s) { return step(b, s, 1); },
    ArrowLeft: function (b, s) { return step(b, s, -1); },
    Left: function (b, s) { return step(b, s, -1); }
  };

  /* The key's new state, or null for a key the driver does not take. */
  function press(bake, state, key) {
    var f = KEYS[key];
    return f ? f(bake, state) : null;
  }

  /* --- The browser --------------------------------------------------------- */

  function boot() {
    var doc = root.document;
    var wrap = doc && doc.getElementById("wallWrap");
    var data = doc && doc.getElementById("pq-static");
    if (!wrap || !data) return null;
    var bake = JSON.parse(data.textContent);
    var wall = PQ.Wall.mount(wrap, {});
    var state = enter(bake, "idle");
    wall.show(frame(bake, state));
    doc.addEventListener("keydown", function (ev) {
      if (ev.altKey || ev.ctrlKey || ev.metaKey) return;
      var s = press(bake, state, ev.key) || (ev.code ? press(bake, state, ev.code) : null);
      if (!s) return;
      ev.preventDefault();
      if (s.phase === state.phase && s.at === state.at) return;
      state = s;
      wall.show(frame(bake, state));
    });
    var api = { state: function () { return state; }, wall: wall };
    PQ.staticWall = api;
    return api;
  }

  PQ.Static = {
    PHASES: PHASES,
    frame: frame,
    enter: enter,
    next: next,
    back: back,
    step: step,
    press: press,
    legend: legend,
    boot: boot
  };

  var w = root.document && root.document.getElementById && root.document.getElementById("wallWrap");
  if (w && w.getAttribute("data-mode") === "static") boot();
})(typeof window !== "undefined" ? window : globalThis);
