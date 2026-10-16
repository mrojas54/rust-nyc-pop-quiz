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
     - over question prose (`explains`, `why_tempting`, `hint` and every
       `trace.steps[].note` — the last is F-47's: AC-98 outranks SPEC's
       four-field list), a match is a warning beside the text on the review
       screen (§7.4) — the Python twin's check_prose() is the hook T-18 calls;
       checkProse() here is its twin, held to the same fixtures.

   Text is normalized before matching (normalize(), below): SPEC's patterns
   spell one space, a straight apostrophe (`that'?s`) and an em dash, and a
   newline, a curly or modifier apostrophe, a zero-width character, fullwidth
   letters or `--` in generated prose would otherwise walk past them.
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

  /* The text as SPEC's patterns spell it, so a variant cannot walk past them.
     In order, the same as the Python twin: NFKC (fullwidth letters,
     look-alikes and NBSP to their plain forms); zero-width characters
     (U+200B..U+200D, U+FEFF) removed; every run of `\s`, newlines included, to
     one space, and the ends trimmed; U+2018, U+2019, U+02BC and the backtick to
     `'`; `--` (and longer), U+2013, U+2014 and a spaced hyphen ` - ` to U+2014,
     the dash the contrast pattern spells. Lengths change: a match is quoted
     from this text. */
  function normalize(text) {
    return String(text)
      .normalize("NFKC")
      .replace(/[​-‍﻿]/g, "")
      .replace(/\s+/g, " ")
      .replace(/^ | $/g, "")
      .replace(/[‘’ʼ`]/g, "'")
      .replace(/ - /g, " — ")
      .replace(/-{2,}|[–—]/g, "—");
  }

  /* Every rule that matches `text`: [{lint, group, id, pattern, match}], in
     SPEC's order, `match` quoted from the normalized text. */
  function check(text) {
    var norm = normalize(text);
    var out = [];
    RULES.forEach(function (r) {
      var m = r.re.exec(norm);
      if (m) out.push({ lint: r.lint, group: r.group, id: r.id, pattern: r.source, match: m[0] });
    });
    return out;
  }

  /* The ids of every rule whose compiled pattern matches `text` as given, with
     no normalizing: the engine alone, for the dialect fixtures. */
  function checkRaw(text) {
    return RULES.filter(function (r) { return r.re.test(String(text)); }).map(function (r) { return r.id; });
  }

  function forbidden(text) {
    return check(text).filter(function (h) { return h.lint === "forbidden"; });
  }

  function tropes(text) {
    return check(text).filter(function (h) { return h.lint === "trope"; });
  }

  function get(obj, name) {
    return obj !== null && typeof obj === "object" ? obj[name] : undefined;
  }

  /* The lint's homes in one question, in the Python twin's order (prose_fields):
     explains.what, explains.takeaway, each option's why_tempting, hint, and
     every trace step's note — the fifth home is F-47's (AC-98 outranks SPEC
     §11's four-field list). Never explains.legacy, source or option text.
     Skips what is missing or not a string. */
  function proseFields(record) {
    var fields = [];
    var explains = get(record, "explains");
    ["what", "takeaway"].forEach(function (name) {
      var value = get(explains, name);
      if (typeof value === "string") fields.push(["explains." + name, value]);
    });
    var options = get(record, "options");
    if (Array.isArray(options)) {
      options.forEach(function (option, i) {
        var value = get(option, "why_tempting");
        if (typeof value === "string") fields.push(["options[" + i + "].why_tempting", value]);
      });
    }
    var hint = get(record, "hint");
    if (typeof hint === "string") fields.push(["hint", hint]);
    var steps = get(get(record, "trace"), "steps");
    if (Array.isArray(steps)) {
      steps.forEach(function (step, i) {
        var value = get(step, "note");
        if (typeof value === "string") fields.push(["trace.steps[" + i + "].note", value]);
      });
    }
    return fields;
  }

  /* Every match in a question's prose, as warnings:
     [{field, id, group, pattern, matched}] — check_prose's shape. */
  function checkProse(record) {
    var out = [];
    proseFields(record).forEach(function (f) {
      check(f[1]).forEach(function (h) {
        out.push({ field: f[0], id: h.id, group: h.group, pattern: h.pattern, matched: h.match });
      });
    });
    return out;
  }

  PQ.COPYLINT = {
    FORBIDDEN: FORBIDDEN,
    TROPES: TROPES,
    RULES: RULES,
    normalize: normalize,
    check: check,
    checkRaw: checkRaw,
    forbidden: forbidden,
    tropes: tropes,
    proseFields: proseFields,
    checkProse: checkProse
  };
})(typeof window !== "undefined" ? window : globalThis);
