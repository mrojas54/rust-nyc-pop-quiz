/* ===========================================================================
   The source well — SPEC §5.3, ported from prototypes/_shared/proto.js.

   White well, line numbers in a 2ch gutter, Cascadia Mono, 1.6 line height
   (measured 2026-09-18, and the type model depends on that number being the
   one components.css actually sets).

   WHAT THIS PORT CHANGES: the prototype took `opts.syntax` from its caller, so
   any caller could colour any phase. Here colour is decided by the PHASE and
   only by the phase (AC-99), through phase.js. A caller cannot ask for colour
   in `work`, and asking for a well in `idle` gets nothing rather than an empty
   frame.

   The font size is NEVER read off this file or off the prototype. typemodel.js
   derives it from the room's measurements (SPEC §5.2) and the caller passes it
   in. components.css's 13px is the default for surfaces read at a human
   distance, not the wall's.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* Ported verbatim. The word list is Rust's keywords as the prototype had
     them — deliberately not the full reserved list, because colouring `async`
     or `await` in a bank that has never used them buys nothing and risks
     colouring an identifier. */
  var RUST_KW = ("as|break|const|continue|crate|dyn|else|enum|extern|false|fn|for|if|impl|in|let|" +
    "loop|match|mod|move|mut|pub|ref|return|self|static|struct|super|trait|true|type|unsafe|use|" +
    "where|while").split("|");

  /* Runs on already-escaped text. Comments and strings win over everything, so
     they are matched first and their contents are left alone — otherwise a
     keyword inside a string literal would colour, and the room would be reading
     a lie about what the program says.

     Ported verbatim from proto.js. Muted values live in components.css. */
  function rustColour(escaped) {
    return escaped.replace(
      /(\/\/[^\n]*)|("(?:[^"\\]|\\.)*")|(\b\d[\d_]*(?:\.\d+)?\b)|(\b[a-zA-Z_][a-zA-Z0-9_]*!)|(\b[A-Z][a-zA-Z0-9_]*\b)|(\b[a-zA-Z_][a-zA-Z0-9_]*\b)/g,
      function (m, comment, str, num, mac, type, word) {
        if (comment) return '<span class="tk-c">' + comment + "</span>";
        if (str)     return '<span class="tk-s">' + str + "</span>";
        if (num)     return '<span class="tk-n">' + num + "</span>";
        if (mac)     return '<span class="tk-m">' + mac + "</span>";
        if (type)    return '<span class="tk-t">' + type + "</span>";
        if (word && RUST_KW.indexOf(word) >= 0) return '<span class="tk-k">' + word + "</span>";
        return m;
      });
  }

  /* Split a program into its lines the way the type model counts them: a single
     trailing newline is not a line. typemodel.js calls this too, so the two can
     never disagree about how many lines a source has. */
  function sourceLines(code) {
    return String(code).replace(/\n$/, "").split("\n");
  }

  /* The well's markup for a phase.

     Returns "" when the phase renders no source (idle, released) — AC-99's
     third value. An empty string rather than an empty frame: the title and
     released walls have their own designs and a stray bordered box would be
     visible on the projector.

     opts.phase    required — decides colour, and whether anything renders
     opts.hl       1-based line numbers drawn as executing now
     opts.focus    [first, last] 1-based inclusive; everything outside is dimmed
     opts.size     the derived font size, e.g. "22.1px" — from typemodel.js
     opts.label    the well header; SPEC §11 gives "What does this program print?"
     opts.fit      a fit verdict from typemodel.js; adds the clipped edge
     opts.meta     right-hand header slot — TRUSTED MARKUP, see below
     opts.style    inline style on the outer .rn-src — TRUSTED, see below

     TRUST BOUNDARY. Everything that can carry bank content — the source, the
     label — goes through escapeHtml. `meta` and `style` are interpolated raw,
     because the prototype's callers pass markup through them, so they are the
     WALL'S OWN strings and never a question's. Do not pass bank data, a trace
     note, an option text or anything from a payload into them: it would both
     bypass the escaping and put question content somewhere G-3's canary does
     not look.
  */
  function sourceWellHtml(code, opts) {
    opts = opts || {};
    var phase = PQ.assertPhase(opts.phase);
    if (!PQ.rendersSource(phase)) return "";

    var esc = PQ.escapeHtml;
    var escAttr = PQ.escapeAttr || esc;
    var hl = opts.hl || [];
    var focus = opts.focus || null;
    var colour = PQ.colourAllowed(phase);
    var lines = sourceLines(code);
    var width = String(lines.length).length + "ch";
    var label = opts.label || PQ.t("proposed_well_label");

    var body = lines.map(function (line, i) {
      var n = i + 1;
      var cls = ["rn-src-line"];
      if (hl.indexOf(n) >= 0) cls.push("hl");
      else if (focus && (n < focus[0] || n > focus[1])) cls.push("dim");
      /* A blank line still needs a box to sit in, or the gutter number floats. */
      var text = esc(line || " ");
      return '<span class="' + cls.join(" ") + '">' +
               '<span class="rn-src-ln" aria-hidden="true" style="width:' + width + '">' + n + "</span>" +
               (colour ? rustColour(text) : text) +
             "</span>";
    }).join("");

    var edge = PQ.fitEdgeClasses ? PQ.fitEdgeClasses(opts.fit) : "";
    var described = edge ? ' aria-describedby="rn-src-fit"' : "";

    return '<div class="rn-src' + edge + '"' + (opts.style ? ' style="' + opts.style + '"' : "") + ">" +
      '<div class="rn-src-head"><span>' + esc(label) + "</span>" +
        (opts.meta ? "<span>" + opts.meta + "</span>" : "") +
      "</div>" +
      '<div class="rn-src-scroll" role="region" aria-label="' + escAttr(label) + '" tabindex="0"' + described + ">" +
        "<pre" + (opts.size ? ' style="font-size:' + opts.size + '"' : "") + "><code>" + body + "</code></pre>" +
      "</div></div>";
  }

  /* Render into an element. Returns true if anything was rendered, false for a
     phase that carries no source — so a caller can branch without re-asking
     phase.js and without parsing the empty string. */
  function renderSource(el, code, opts) {
    if (!el) return false;
    var html = sourceWellHtml(code, opts);
    el.innerHTML = html;
    return html !== "";
  }

  PQ.RUST_KW = RUST_KW.slice();
  PQ.rustColour = rustColour;
  PQ.sourceLines = sourceLines;
  PQ.sourceWellHtml = sourceWellHtml;
  PQ.renderSource = renderSource;
})(typeof window !== "undefined" ? window : globalThis);
