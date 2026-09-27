/* ===========================================================================
   The wall — SPEC §4 (the wall column), §5, AC-33/39/40/74/79/99/100, G-7.

   Seven phases on the 1120x630 design canvas, scaled to the projector as one
   unit (§5.1). What the wall shows is decided by ONE THING: the frame the room
   sends on `GET /rooms/{id}/ws/wall`, which is `view::wall` — the wall's whole
   state, never a delta (room/src/ws.rs). Nothing is carried from one frame to
   the next, so a reload shows exactly what was on the screen.

   TWO LAYERS, and the split is load-bearing:

     html(frame, opts)  PURE. A frame in, the canvas's markup out. No DOM, no
                        socket, no clock. The tests drive it in node:vm, and the
                        static fallback (T-26, SPEC §12) drives it from a local
                        state object with `mode: "static"` — so the fallback is
                        this design, never a second one.

     mount / boot       THE BROWSER. Scale the canvas to the viewport, render
                        each frame, and on entering every phase that renders the
                        source run the type model: derive, render, MEASURE,
                        refit (typemodel.js), then draw the clipped edge on the
                        side that lost content. "Fits" is a measured claim.

   What the wall never does: take input (AC-79 — there is no control on it; the
   host phone drives it), colour the source while a trace runs (AC-99 —
   phase.js decides, not this file), show the explanation (AC-39 — the host
   reads it aloud), or compute a receipt (G-7 — the lines arrive in the frame
   from `answers::receipt_lines`, the one function, and are printed verbatim).

   Every participant-facing string is either payload data (built from
   room/src/copy.rs, the mirror of web/shared/copy.js) or read from copy.js
   here. The one exception is the brand line, which SPEC §4/§5.1 require and
   §11 does not list — flagged to the Orchestrator rather than invented into
   copy.js, which this ticket may not edit.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* SPEC §4 and §5.1: "brand line top-left". Not a §11 string — see above. */
  var BRAND = "Rust NYC · Pop Quiz";

  function esc(s) { return PQ.escapeHtml(s); }

  /* --- Pieces ------------------------------------------------------------ */

  /* The join strip: `join @ ‹link›` with the link set large (prototype
     `.joinstrip b`). The frame carries the filled string; the template from
     copy.js says where the link sits in it, so no string is retyped. */
  function joinHtml(line, key) {
    var tpl = PQ.t(key);
    var at = tpl.indexOf("‹link›");
    var pre = tpl.slice(0, at);
    var post = tpl.slice(at + "‹link›".length);
    var s = String(line);
    if (at >= 0 && s.length > pre.length + post.length &&
        s.slice(0, pre.length) === pre && s.slice(s.length - post.length) === post) {
      return "<span>" + esc(pre) + "<b>" + esc(displayLink(s.slice(pre.length, s.length - post.length))) +
             "</b>" + esc(post) + "</span>";
    }
    return "<span>" + esc(s) + "</span>";
  }

  /* A question-shaped object for trace.js, built from the one step the frame
     carries. trace.js wants `q.trace.steps` for the dots and the step at `at`;
     the frame has only that step (the room never sends the others, and in
     `work` never the resolving one — D-10), so the rest are empty slots that
     only count toward "Step N of M". */
  function traceQuestion(frame) {
    var t = frame.trace;
    var steps = [];
    for (var i = 0; i < t.m; i++) steps.push(i === t.at ? t.step : { note: "" });
    return { source: frame.source, trace: { steps: steps } };
  }

  function wellHtml(frame, opts) {
    var label = PQ.t("wall_live_well_header");
    if (PQ.tracing(frame.phase) && frame.trace) {
      return PQ.traceSourceHtml(traceQuestion(frame), frame.trace.at, {
        phase: frame.phase, label: label, size: opts.size, fit: opts.fit
      });
    }
    return PQ.sourceWellHtml(frame.source, {
      phase: frame.phase, label: label, size: opts.size, fit: opts.fit
    });
  }

  function optionsHtml(options) {
    return '<div class="wall-block reading wall-options">' + options.map(function (o) {
      return '<div class="opt"><span class="letter">' + esc(o.letter) + "</span>" +
             '<span class="otext">' + esc(o.text) + "</span></div>";
    }).join("") + "</div>";
  }

  /* §5.4: five bars, `display: block` fills, `n · p%` beside each, the correct
     option unmarked until reveal. At reveal the correct bar's letter goes
     through check.js, which emits the ✓ and the colour class together (AC-40);
     the green fill keys off that element in wall.css, so there is no way to
     get the colour without the glyph. No bar is ever marked wrong. */
  function barsHtml(split, correct) {
    return '<div class="bars">' + split.bars.map(function (b) {
      var letter = b.letter === correct
        ? PQ.correctHtml(b.letter, { className: "letter" })
        : '<span class="letter">' + esc(b.letter) + "</span>";
      return '<div class="bar-row">' + letter +
        '<span class="bar-track"><span class="bar-fill" style="width:' + Number(b.percent) + '%"></span></span>' +
        '<span class="bar-n">' + Number(b.count) + " · " + Number(b.percent) + "%</span></div>";
    }).join("") + "</div>";
  }

  function traceNoteHtml(frame) {
    return PQ.traceNoteHtml(traceQuestion(frame), frame.trace.at, { noNav: true });
  }

  /* The machine column at reveal (AC-74): the correct option and the receipt,
     together under the machine provenance marker, because both are what the
     machine established. The wall renders no human-reviewed prose at all —
     AC-39 keeps the explanation on the host phone and take-it-home. */
  function machineHtml(frame) {
    var r = frame.reveal;
    var opt = (frame.options || []).filter(function (o) { return o.letter === r.correct; })[0];
    var chip = opt
      ? '<div class="opt">' + PQ.correctHtml(opt.letter, { className: "letter" }) +
        '<span class="otext">' + esc(opt.text) + "</span></div>"
      : "";
    return '<div class="wall-machine by-machine">' + chip +
      '<div class="receipt">' +
        '<span class="provenance machine">' + esc(r.receipt.heading) + "</span>" +
        r.receipt.lines.map(function (l) { return '<span class="receipt-line">' + esc(l) + "</span>"; }).join("") +
      "</div></div>";
  }

  function titleCard(inner) {
    return '<div class="title-card">' + inner + "</div>";
  }

  /* The released link without its scheme, as the prototype shows it
     (`popquiz.rustnyc.org/last`). The QR carries the whole link. */
  function displayLink(link) {
    return String(link).replace(/^https?:\/\//, "");
  }

  /* --- The one pure renderer -------------------------------------------- */

  /* The canvas's inner markup for one frame.

       frame   a wall frame (view::wall, flattened into the socket's envelope)
       opts.fontPx  the well's size from the type model; omitted before refit
       opts.fit     a fit verdict, for the clipped edge

     Throws on an unknown phase (phase.js), rather than rendering a guess. */
  function html(frame, opts) {
    opts = opts || {};
    var phase = PQ.assertPhase(frame.phase);
    var size = opts.fontPx ? (Math.round(opts.fontPx * 100) / 100) + "px" : undefined;
    var well = { size: size, fit: opts.fit };
    var main, mainClass = "proj-main full", strip = "";

    if (phase === "idle") {
      mainClass = "title-card-main";
      main = titleCard("<div><h2>" + esc(frame.title) + "</h2></div>");
      /* The static fallback has no room to join; its idle strip carries the
         §11 key legend instead (fallback/static.js, SPEC §12). */
      strip = frame.join ? joinHtml(frame.join, "wall_idle_join")
        : frame.strip ? "<span>" + esc(frame.strip) + "</span>" : "";
    } else if (phase === "released") {
      var rel = frame.released;
      mainClass = "title-card-main";
      main = titleCard('<div class="endcard"><h2>' + esc(rel.title) + "</h2>" +
        '<div class="endlink">' + esc(displayLink(rel.link)) + "</div>" +
        PQ.qrSvg(rel.link, { className: "endqr" }) + "</div>");
    } else if (phase === "live" || phase === "closed") {
      main = wellHtml(frame, well) + optionsHtml(frame.options);
      strip = phase === "live"
        ? (frame.join ? joinHtml(frame.join, "wall_live_join") : "")
        : "<span>" + esc(frame.strip) + "</span>";
    } else if (phase === "split") {
      main = wellHtml(frame, well) +
        '<div class="wall-block reading wall-split">' + barsHtml(frame.split, null) + "</div>";
      strip = frame.split.line ? "<span>" + esc(frame.split.line) + "</span>" : "";
    } else if (phase === "work") {
      main = wellHtml(frame, well) +
        '<div class="wall-block trace work">' +
          '<div class="wall-trace">' + (frame.trace ? traceNoteHtml(frame) : "") + "</div>" +
        "</div>";
    } else { /* reveal */
      var r = frame.reveal;
      main = wellHtml(frame, well) +
        '<div class="wall-block trace reveal">' +
          '<div class="wall-trace">' + traceNoteHtml(frame) + "</div>" +
          '<div class="wall-room">' + (frame.split ? barsHtml(frame.split, r.correct) : "") +
            '<div class="wall-middle">' + esc(r.middle.line) + "</div></div>" +
          machineHtml(frame) +
        "</div>";
      strip = frame.split && frame.split.line ? "<span>" + esc(frame.split.line) + "</span>" : "";
    }

    return '<div class="projector wall" data-phase="' + phase + '">' +
      '<div class="proj-body">' +
        '<div class="proj-top"><span class="proj-brand">' + esc(BRAND) + "</span></div>" +
        '<div class="' + mainClass + '">' + main + "</div>" +
        (phase === "released" ? "" : '<div class="joinstrip">' + strip + "</div>") +
      "</div></div>";
  }

  /* --- The browser ------------------------------------------------------- */

  var CANVAS_W = 1120, CANVAS_H = 630;

  /* §11's live-region line for entering a phase (AC-83). `idle` has none. */
  function announcement(frame) {
    switch (frame.phase) {
      case "live": return PQ.t("live_question_on_screen");
      case "closed": return PQ.t("live_answers_closed");
      case "split": return PQ.t("live_split_on_screen");
      case "work": return PQ.t("live_walking_through");
      case "reveal": return frame.reveal ? PQ.t("live_revealed", { Y: frame.reveal.correct }) : null;
      case "released": return PQ.t("live_released");
      default: return null;
    }
  }

  /* One wall, mounted into `wrap` (the 1120x630 box). Returns a `show(frame)`
     that renders a frame; the socket, or T-26's static mode, calls it.

     opts.room      the room's measurements for the floor (§5.2); defaults to
                    PQ.ROOM_DEFAULTS, a hypothesis until HC-1
     opts.onFit     called with each measured verdict (AC-100) */
  function mount(wrap, opts) {
    opts = opts || {};
    var doc = wrap.ownerDocument;
    var win = doc.defaultView;
    var room = opts.room || PQ.ROOM_DEFAULTS;
    var current = null;   /* the frame on screen */
    var fitted = null;    /* {phase, fontPx} — the size refitted on entering the phase */
    var lastFit = null;

    function scale() {
      var s = Math.min(win.innerWidth / CANVAS_W, win.innerHeight / CANVAS_H);
      wrap.style.transform = "scale(" + s + ")";
      wrap.style.left = Math.round((win.innerWidth - CANVAS_W * s) / 2) + "px";
      wrap.style.top = Math.round((win.innerHeight - CANVAS_H * s) / 2) + "px";
    }

    function scroller() { return wrap.querySelector(".rn-src-scroll"); }

    function paint(frame, refitNow) {
      var phase = frame.phase;
      if (!PQ.rendersSource(phase)) {
        wrap.innerHTML = html(frame);
        fitted = null;
        return;
      }
      var fontPx;
      if (refitNow || !fitted || fitted.phase !== phase) {
        var d = PQ.derivedFontPx(frame.source, phase, room);
        var r = PQ.refit(d.fontPx, d.floorPx, function (px) {
          wrap.innerHTML = html(frame, { fontPx: px });
          return PQ.measureWell(scroller());
        });
        fontPx = r.fontPx;
        fitted = { phase: phase, fontPx: fontPx, floorPx: d.floorPx, passes: r.passes };
      } else {
        fontPx = fitted.fontPx;
        wrap.innerHTML = html(frame, { fontPx: fontPx });
      }
      var verdict = PQ.fitVerdict(PQ.measureWell(scroller()));
      PQ.applyClippedEdge(wrap.querySelector(".rn-src"), verdict);
      fitted.verdict = verdict;
      if (verdict && verdict !== lastFit) {
        lastFit = verdict;
        if (opts.onFit) opts.onFit(verdict, fitted);
      }
    }

    function show(frame) {
      var entering = !current || current.phase !== frame.phase;
      current = frame;
      paint(frame, false);
      if (entering) {
        var say = announcement(frame);
        if (say) PQ.announce(say);
      }
    }

    scale();
    win.addEventListener("resize", scale);
    /* The webfont lands late, and a well measured before it does reports a fit
       it does not have. Refit once it is in. */
    if (doc.fonts && doc.fonts.ready) {
      doc.fonts.ready.then(function () { if (current) paint(current, true); });
    }

    return {
      show: show,
      /* For the measured-layout run and the tests: what the wall last fitted. */
      fitted: function () { return fitted; }
    };
  }

  /* The live room: the wall socket for the room in the page's path,
     `/wall/{room_id}`. No attach message — the wall's projection is public
     (ws.rs). Every frame replaces the whole state. On a dropped socket the
     last frame stays up and the wall reconnects with backoff; there is no
     error text, because §11 authors none for the wall and a projector is not
     where a network fault is fixed. A room that is gone (4404) stops it. */
  function connect(wall, roomId, loc) {
    var delay = 500;
    var proto = loc.protocol === "https:" ? "wss:" : "ws:";
    var url = proto + "//" + loc.host + "/rooms/" + encodeURIComponent(roomId) + "/ws/wall";
    function open() {
      var ws = new root.WebSocket(url);
      ws.onopen = function () { delay = 500; };
      ws.onmessage = function (ev) {
        var frame;
        try { frame = JSON.parse(ev.data); } catch (e) { return; }
        if (frame && frame.t === "state") wall.show(frame);
      };
      ws.onclose = function (ev) {
        if (ev && ev.code === 4404) return;
        root.setTimeout(open, delay);
        delay = Math.min(delay * 2, 5000);
      };
    }
    open();
  }

  function boot() {
    var doc = root.document;
    var wrap = doc && doc.getElementById("wallWrap");
    if (!wrap) return null;
    var parts = root.location.pathname.split("/");
    var roomId = decodeURIComponent(parts[2] || "");
    /* AC-100: the verdict goes to the room, which shows the host the matching
       §11 fit line. Only a change is sent; the room ignores a repeat anyway. */
    var wall = mount(wrap, {
      onFit: function (verdict) {
        if (typeof root.fetch !== "function") return;
        root.fetch("/rooms/" + encodeURIComponent(roomId) + "/fit", {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ fit: verdict })
        }).catch(function () { /* the next refit reports again */ });
      }
    });
    connect(wall, roomId, root.location);
    return wall;
  }

  PQ.Wall = {
    BRAND: BRAND,
    html: html,
    mount: mount,
    connect: connect,
    boot: boot,
    announcement: announcement
  };

  /* The static fallback (fallback/static.js) mounts the same wall itself, with no socket. */
  var auto = root.document && root.document.getElementById && root.document.getElementById("wallWrap");
  if (auto && auto.getAttribute("data-mode") !== "static") {
    boot();
  }
})(typeof window !== "undefined" ? window : globalThis);
