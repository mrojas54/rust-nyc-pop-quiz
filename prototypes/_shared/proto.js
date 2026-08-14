/* ===========================================================================
   Shared prototype plumbing: Rust highlighting, the PROTOTYPE badge, and the
   polite live region that AC-83 asks every state change to go through.
   =========================================================================== */

const KEYWORDS = [
  "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
  "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
  "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
  "super", "trait", "true", "type", "unsafe", "use", "where", "while"
];

function escapeHtml(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/* Deliberately muted: keywords, strings, numbers, macros, comments. No rainbow.
   PHILOSOPHY.md asks for "a well-made terminal, not a SaaS dashboard". */
function rustHighlight(src) {
  const kw = KEYWORDS.join("|");
  const re = new RegExp(
    "(//[^\\n]*)"                        + "|" +  // 1 comment
    "(\"(?:\\\\.|[^\"\\\\])*\")"          + "|" +  // 2 string
    "([A-Za-z_][A-Za-z0-9_]*!)"          + "|" +  // 3 macro
    "\\b(" + kw + ")\\b"                 + "|" +  // 4 keyword
    "\\b(\\d[\\d_]*(?:\\.\\d+)?)\\b",             // 5 number
    "g"
  );

  let out = "";
  let last = 0;
  let m;
  while ((m = re.exec(src)) !== null) {
    out += escapeHtml(src.slice(last, m.index));
    const cls = m[1] ? "c" : m[2] ? "s" : m[3] ? "m" : m[4] ? "k" : "n";
    out += '<span class="' + cls + '">' + escapeHtml(m[0]) + "</span>";
    last = m.index + m[0].length;
  }
  out += escapeHtml(src.slice(last));

  /* fn names read better bold; done after tokenising so it can't eat a keyword. */
  return out.replace(
    /(<span class="k">fn<\/span>\s+)([A-Za-z_][A-Za-z0-9_]*)/g,
    '$1<span class="f">$2</span>'
  );
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
function traceHtml(q, i, opts) {
  opts = opts || {};
  const t = q.trace;
  if (!t) return "";
  const step = t.steps[Math.max(0, Math.min(i, t.steps.length - 1))];
  const kept = step.kept || [];
  const size = opts.big ? "font-size:22px;min-width:56px;padding:12px 14px" : "";

  const tape = t.cells.map(function (v, idx) {
    const cls = ["tcell"];
    /* A cell is "gone" once the walk has passed it and it was not kept. */
    const passed = step.done || (step.cursor !== null && idx <= step.cursor);
    if (kept.indexOf(idx) >= 0) cls.push("kept");
    else if (passed) cls.push("gone");
    if (idx === step.cursor) cls.push("cursor");
    if (idx === step.against) cls.push("against");
    return '<span class="' + cls.join(" ") + '" style="' + size + '">' + v +
           '<span class="tidx">' + idx + '</span></span>';
  }).join("");

  const vtag = step.verdict === "keep" ? '<span class="vtag keep">keep</span>'
             : step.verdict === "drop" ? '<span class="vtag drop">drop</span>'
             : '<span class="vtag none">' + (step.done ? "done" : "start") + '</span>';

  const dots = t.steps.map(function (_, n) {
    return '<i class="' + (n === i ? "on" : n < i ? "past" : "") + '"></i>';
  }).join("");

  return '<div class="trace">' +
    (opts.noHead ? '' :
      '<div class="trace-head"><h3>Step it through</h3><code>' + escapeHtml(t.call) + '</code>' +
      '<span class="meta">' + escapeHtml(t.subtitle) + '</span></div>') +
    '<div class="trace-tape">' + tape + '</div>' +
    (opts.cellsOnly ? '' :
      '<div class="trace-verdict">' + vtag + '<span>' + escapeHtml(step.say) + '</span></div>') +
    (step.pivot && !opts.cellsOnly
      ? '<div class="trace-pivot"><b>This is the frame worth stopping on.</b> ' +
        'The 2 survives because its neighbour is a 3 — not because it is unique. ' +
        'Everyone who read <code>dedup</code> as <code>unique</code> was reading a ' +
        'reasonable thing that Rust does not do.</div>'
      : '') +
    /* AC-79: the room display takes no interaction beyond host controls, so the
       projector renders the tape with no nav and the host's phone drives it. */
    (opts.noNav
      ? '<div class="trace-nav"><span class="trace-dots" aria-hidden="true">' + dots + '</span>' +
        '<span class="pos">step ' + i + ' of ' + (t.steps.length - 1) + '</span></div>'
      : '<div class="trace-nav">' +
        '<button class="btn" style="min-height:34px;padding:4px 12px" onclick="traceStep(-1)"' +
          (i === 0 ? " disabled" : "") + ' aria-label="previous step">←</button>' +
        '<button class="btn' + (i < t.steps.length - 1 ? " btn-primary" : "") +
          '" style="min-height:34px;padding:4px 12px" onclick="traceStep(1)"' +
          (i >= t.steps.length - 1 ? " disabled" : "") + ' aria-label="next step">→</button>' +
        '<span class="trace-dots" aria-hidden="true">' + dots + '</span>' +
        '<span class="pos">step ' + i + ' of ' + (t.steps.length - 1) + '</span>' +
      '</div>') +
  '</div>';
}

/* Announcement text for a step, so the tape is not a visual-only channel. */
function traceSay(q, i) {
  const s = q.trace.steps[Math.max(0, Math.min(i, q.trace.steps.length - 1))];
  return (s.verdict ? s.verdict + ". " : "") + s.say;
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
