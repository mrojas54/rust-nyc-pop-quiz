/* ===========================================================================
   The two copy lints — SPEC §11's Forbidden row (G-5, AC-98) and the §11.1
   trope check (D-23, AC-42) — as one source.

   Test-only, so it lives here and not in web/shared/, which holds what the
   pages load and the room serves (room/tests/wall_page.rs). Not named
   *.test.js, so the `test-web` glob does not run it as a suite; the suites
   load it through _load.js.

   The patterns are held as SPEC's own text, character for character: the
   Forbidden row's nine (the markdown table's `\|` read as `|`) and §11.1's
   sixteen in its five groups. web/test/copylint.test.js reads SPEC.md and
   fails if either list differs from it; pipeline/src/popquiz/copylint.py holds
   the same text and its suite pins it to SPEC.md the same way, so the two
   languages cannot drift from SPEC or from each other. Both run the fixture set
   in bank/fixtures/copy-lint/.

   Compiled case-insensitive, with `i` and without `u`: `\b` and `\w` are ASCII,
   `\s` is JS's own whitespace set. The Python twin compiles to the same
   behaviour; where the two engines cannot agree (a `{0,N}` span counts UTF-16
   units here and code points there) the fixtures say so.

   The two consequences live with the callers, not here:
     - over the copy module (web/shared/copy.js), any match of either lint
       fails `just test`;
     - over question prose (`explains`, `why_tempting`, `hint`), a match is a
       warning beside the text on the review screen (§7.4) — the Python twin's
       check_prose() is the hook T-18 calls.

   Text is read with U+2018/U+2019 taken as `'` before matching: SPEC's
   patterns spell the apostrophe straight (`that'?s`), and a curly one in
   generated prose would otherwise walk past every contraction pattern.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* SPEC §11, the Forbidden row. The one normative list. */
  var FORBIDDEN = [
    String.raw`turn to`,
    String.raw`ask (someone|the person|your neighbou?r)`,
    String.raw`find someone`,
    String.raw`volunteer`,
    String.raw`who (said|picked|chose)`,
    String.raw`\bwrong\b`,
    String.raw`\bincorrect\b`,
    String.raw`✗`,
    String.raw`argu`
  ];

  /* SPEC §11.1, grouped as the spec groups them. */
  var TROPES = {
    contrast: [
      String.raw`\b(it'?s|it is|that'?s|that is|this is) not\b[^.!?]{0,90}(—|;|,|:)\s*(it'?s|it is|that'?s|that is|this is|but)\b`,
      String.raw`\bnot (just|only|merely|simply)\b`,
      String.raw`(^|[.!?]\s+)not (the|a|an|your|our|every|any)\b`,
      String.raw`\bnot\b[^.!?]{0,40},\s*not\b[^.!?]{0,40},\s*not\b`,
      String.raw`\b(listen|look|read|think),? (don'?t|do not)\b`
    ],
    filler: [
      String.raw`\b(genuinely|truly|honestly|quietly|extremely|deeply|fundamentally|literally)\b`,
      String.raw`\bdoing (all|the) (the )?work\b`
    ],
    signpost: [
      String.raw`\bworth (a|the|stopping|talking|noting|remembering)\b`,
      String.raw`\b(here'?s|here is) the (thing|whole idea)\b`,
      String.raw`\bthe (important|key) (word|thing|part)\b`,
      String.raw`\bif you remember one thing\b`
    ],
    reassurance: [
      String.raw`\bthat'?s (fine|okay|ok|totally fine)\b`,
      String.raw`\bit'?s (fine|okay|ok|normal) to\b`,
      String.raw`\bnothing is missing\b`,
      String.raw`\bdon'?t worry\b`
    ],
    flattery: [
      String.raw`\bmost (interesting|impressive|clever|insightful)\b`
    ]
  };

  /* Every pattern, compiled once, with the id the fixtures use: `forbidden/N`
     or `<group>/N`, N counted from 1 in SPEC's order. */
  var RULES = [];
  FORBIDDEN.forEach(function (src, i) {
    RULES.push({ lint: "forbidden", group: "forbidden", id: "forbidden/" + (i + 1), source: src, re: new RegExp(src, "i") });
  });
  Object.keys(TROPES).forEach(function (group) {
    TROPES[group].forEach(function (src, i) {
      RULES.push({ lint: "trope", group: group, id: group + "/" + (i + 1), source: src, re: new RegExp(src, "i") });
    });
  });

  /* Curly single quotes read as the straight one. Same length, so a match's
     index in the normalized text is its index in the original. */
  function normalize(text) {
    return String(text).replace(/[‘’]/g, "'");
  }

  /* Every rule that matches `text`: [{lint, group, id, pattern, match}], in
     SPEC's order, `match` quoted from the original text. */
  function check(text) {
    var original = String(text);
    var norm = normalize(original);
    var out = [];
    RULES.forEach(function (r) {
      var m = r.re.exec(norm);
      if (m) {
        out.push({ lint: r.lint, group: r.group, id: r.id, pattern: r.source,
          match: original.slice(m.index, m.index + m[0].length) });
      }
    });
    return out;
  }

  function forbidden(text) {
    return check(text).filter(function (h) { return h.lint === "forbidden"; });
  }

  function tropes(text) {
    return check(text).filter(function (h) { return h.lint === "trope"; });
  }

  PQ.COPYLINT = {
    FORBIDDEN: FORBIDDEN,
    TROPES: TROPES,
    RULES: RULES,
    normalize: normalize,
    check: check,
    forbidden: forbidden,
    tropes: tropes
  };
})(typeof window !== "undefined" ? window : globalThis);
