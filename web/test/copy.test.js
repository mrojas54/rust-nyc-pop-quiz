// The copy module: completeness against SPEC §11, and cleanliness against the
// two lints that police it.
//
// SCOPE. T-22 owns the forbidden-copy lint (G-5, AC-98) and the trope check
// (§11.1) as reusable tools, and owns AC-42's row. What is here is the
// patterns applied to THIS module's own strings, because shipping a copy module
// nobody has checked against the lints written to police it would hand T-22 a
// red suite on arrival. The retired-strings fixture below is what makes the
// clean assertion mean anything: without it, "no entry matches" could pass
// because the patterns were transcribed wrong.

const test = require('node:test');
const assert = require('node:assert');
const { load } = require('./_load.js');

const { PQ } = load('copy');
const COPY = PQ.COPY;
const ROWS = PQ.COPY_ROWS;

const entries = () => Object.keys(COPY).map((k) => [k, COPY[k]]);

// --- completeness -----------------------------------------------------------

test('every key in the row index resolves to a non-empty string', () => {
  for (const [row, keys] of Object.entries(ROWS)) {
    for (const key of keys) {
      assert.ok(Object.prototype.hasOwnProperty.call(COPY, key),
        `${row}: no entry for ${key}`);
      assert.strictEqual(typeof COPY[key], 'string', `${row}: ${key}`);
      assert.ok(COPY[key].length > 0, `${row}: ${key} is empty`);
    }
  }
});

test('every entry belongs to a §11 row — nothing is authored here', () => {
  // §11: "Authored here and only here." A string in the module that belongs to
  // no row was written somewhere other than the spec.
  const indexed = new Set(Object.values(ROWS).flat());
  const orphans = Object.keys(COPY).filter((k) => !indexed.has(k));
  assert.deepStrictEqual(orphans, [],
    'these entries belong to no §11 row — author them in SPEC.md §11 first');
});

test('no key is claimed by two rows', () => {
  const seen = new Map();
  for (const [row, keys] of Object.entries(ROWS)) {
    for (const key of keys) {
      assert.ok(!seen.has(key), `${key} is in both "${seen.get(key)}" and "${row}"`);
      seen.set(key, row);
    }
  }
});

test('the rows SPEC §11 gives a fixed count for have that count', () => {
  assert.strictEqual(ROWS['Live region (AC-83), verbatim'].length, 10);
  assert.strictEqual(ROWS['Host, phase labels (AC-49)'].length, 7);
  assert.strictEqual(ROWS['Host, actions'].length, 8);
  assert.strictEqual(ROWS['Buzzer, join failures (AC-29)'].length, 6);
  assert.strictEqual(ROWS['Take it home, headings in order'].length, 6);
  assert.strictEqual(ROWS['Host, fit line (AC-100)'].length, 4);
  // §7.5: the heading plus seven step-line templates, Miri having two wordings.
  assert.strictEqual(ROWS['Receipt (§7.5)'].length, 9);
});

test('there is a phase label for each of the seven phases', () => {
  const { PQ: p } = load('all');
  for (const phase of p.PHASES) {
    assert.strictEqual(typeof p.HOST_PHASE_LABEL[phase], 'string', phase);
  }
});

test('there is a fit line for each verdict, and none for an unmeasured well', () => {
  for (const v of ['fits', 'clipped_x', 'clipped_y', 'clipped_xy']) {
    assert.strictEqual(typeof PQ.hostFitLine(v), 'string', v);
  }
  assert.strictEqual(PQ.hostFitLine(null), null);
  assert.strictEqual(PQ.hostFitLine('maybe'), null);
});

// --- a few strings, verbatim ------------------------------------------------

test('the client\'s own lines are hers, exactly', () => {
  assert.strictEqual(COPY.wall_idle_title, 'Time for a pop quiz.');
  assert.strictEqual(COPY.wall_released_title, "Let's go to the bar.");
});

test('the released buzzer says nothing about you was recorded', () => {
  assert.strictEqual(COPY.buzzer_released, 'Nothing about you was recorded.');
});

test('the receipt heading and its two lists are §7.5\'s', () => {
  assert.strictEqual(COPY.receipt_heading, 'How we know');
  for (const k of ['receipt_compiled', 'receipt_ran_n_times', 'receipt_output_never_varied',
                   'receipt_miri_clean', 'receipt_miri_ub', 'receipt_compiler_refused',
                   'receipt_error_codes', 'receipt_nothing_ran']) {
    assert.ok(COPY[k].startsWith('✓ '), `${k} carries the done mark`);
  }
});

test('no wall receipt line ends in terminal punctuation or claims a result', () => {
  // AC-43. The list shows the steps taken; it never asserts a verdict.
  const claims = /\b(verified|established|proves|always|guaranteed)\b/i;
  for (const k of ROWS['Receipt (§7.5)']) {
    if (k === 'receipt_heading') continue;
    assert.ok(!/[.!?]$/.test(COPY[k]), `${k} ends in terminal punctuation`);
    assert.ok(!claims.test(COPY[k]), `${k} contains a claim word`);
  }
});

// --- the forbidden-copy lint (G-5, AC-98) -----------------------------------

// SPEC §11's Forbidden row, verbatim. The one normative list.
const FORBIDDEN = [
  /turn to/i,
  /ask (someone|the person|your neighbou?r)/i,
  /find someone/i,
  /volunteer/i,
  /who (said|picked|chose)/i,
  /\bwrong\b/i,
  /\bincorrect\b/i,
  /✗/,
  /argu/i,
];

test('no string obliges anyone to speak, and none says wrong (AC-98)', () => {
  for (const [key, value] of entries()) {
    for (const pattern of FORBIDDEN) {
      assert.ok(!pattern.test(value), `${key} matches ${pattern}: ${JSON.stringify(value)}`);
    }
  }
});

test('the forbidden patterns actually fire', () => {
  // Otherwise the assertion above passes on a broken transcription.
  const shouldFail = [
    'turn to the person beside you',
    'ask your neighbour what they picked',
    'find someone who chose differently',
    'any volunteers?',
    'who said B?',
    'that answer is wrong',
    'that answer is incorrect',
    '✗ not that one',
    'the argument is subtle',
  ];
  for (const s of shouldFail) {
    assert.ok(FORBIDDEN.some((p) => p.test(s)), `nothing caught: ${s}`);
  }
});

test('the two allowed near-misses stay allowed', () => {
  // §11: the wall's "‹n› of us said ‹X›" and the host's "Why ‹n› of us said
  // ‹X›" do not match who (said|picked|chose) and are allowed.
  for (const s of [COPY.wall_reveal_most_chosen, COPY.host_reveal_beat_why]) {
    assert.ok(!/who (said|picked|chose)/i.test(s), s);
  }
});

test('no ✗ appears anywhere', () => {
  // DESIGN.md: "No ✗, anywhere." AC-94.
  for (const [key, value] of entries()) {
    assert.ok(!value.includes('✗'), key);
  }
});

// --- the trope check (§11.1) ------------------------------------------------

// SPEC §11.1's patterns, grouped as the spec groups them. A match in the copy
// module fails the build.
const TROPES = {
  contrast: [
    /\b(it'?s|it is|that'?s|that is|this is) not\b[^.!?]{0,90}(—|;|,|:)\s*(it'?s|it is|that'?s|that is|this is|but)\b/i,
    /\bnot (just|only|merely|simply)\b/i,
    /(^|[.!?]\s+)not (the|a|an|your|our|every|any)\b/i,
    /\bnot\b[^.!?]{0,40},\s*not\b[^.!?]{0,40},\s*not\b/i,
    /\b(listen|look|read|think),? (don'?t|do not)\b/i,
  ],
  filler: [
    /\b(genuinely|truly|honestly|quietly|extremely|deeply|fundamentally|literally)\b/i,
    /\bdoing (all|the) (the )?work\b/i,
  ],
  signpost: [
    /\bworth (a|the|stopping|talking|noting|remembering)\b/i,
    /\b(here'?s|here is) the (thing|whole idea)\b/i,
    /\bthe (important|key) (word|thing|part)\b/i,
    /\bif you remember one thing\b/i,
  ],
  reassurance: [
    /\bthat'?s (fine|okay|ok|totally fine)\b/i,
    /\bit'?s (fine|okay|ok|normal) to\b/i,
    /\bnothing is missing\b/i,
    /\bdon'?t worry\b/i,
  ],
  flattery: [
    /\bmost (interesting|impressive|clever|insightful)\b/i,
  ],
};

test('no entry reads as generated (§11.1)', () => {
  for (const [key, value] of entries()) {
    for (const [group, patterns] of Object.entries(TROPES)) {
      for (const p of patterns) {
        assert.ok(!p.test(value), `${key} matches ${group} ${p}: ${JSON.stringify(value)}`);
      }
    }
  }
});

test('every trope group fires on the retired strings (AC-42)', () => {
  // EVALUATION.md AC-42's fixture: the strings this project actually wrote and
  // then cut. If a group does not fire here, its patterns are transcribed
  // wrong and the clean assertion above is vacuous.
  const RETIRED = [
    'That is not a room getting it wrong — that is a room…',
    'Not the explanation — that one is human',
    "listen, don't read",
    "that's fine",
    'the most interesting answer in the room',
    'The word doing all the work',
    'The bit worth talking about',
  ];
  for (const [group, patterns] of Object.entries(TROPES)) {
    const fired = RETIRED.some((s) => patterns.some((p) => p.test(s)));
    assert.ok(fired, `the ${group} group fires on none of the retired strings`);
  }
  // And every retired string is caught by something.
  const all = Object.values(TROPES).flat();
  for (const s of RETIRED) {
    assert.ok(all.some((p) => p.test(s)), `nothing caught: ${s}`);
  }
});

// --- placeholders and lookup ------------------------------------------------

test('fill substitutes ‹placeholders›', () => {
  assert.strictEqual(PQ.fill(COPY.wall_reveal_most_chosen, { n: 24, X: 'A' }), '24 of us said A');
  assert.strictEqual(PQ.fill(COPY.wall_split_answered, { answered: 31, present: 40 }),
    '31 of 40 in the room answered');
});

test('a missing placeholder throws rather than reaching the wall', () => {
  // "‹n› of us said A" on a projector is worse than a loud failure at the call
  // site, where a test can catch it.
  assert.throws(() => PQ.fill(COPY.wall_reveal_most_chosen, { n: 24 }), /no value for ‹X›/);
  assert.throws(() => PQ.t('wall_reveal_most_chosen', {}), /no value for/);
});

test('every placeholder in every entry is a bare name', () => {
  for (const [key, value] of entries()) {
    for (const m of value.matchAll(/‹([^›]*)›/g)) {
      assert.ok(m[1].length > 0, `${key} has an empty placeholder`);
      assert.ok(!m[1].includes('‹'), `${key} has a nested placeholder`);
    }
  }
});

test('an unknown key throws rather than rendering its own name', () => {
  assert.throws(() => PQ.t('buzzer_relased'), /no entry for/);
});

test('t returns the string unchanged when there is nothing to fill', () => {
  assert.strictEqual(PQ.t('wall_closed'), 'answers are closed');
});

test('no entry carries SPEC\'s markdown formatting as literal text', () => {
  // SPEC writes key names and templates in backtick code fencing. That is its
  // formatting, not punctuation anyone reads off a screen, and it must not
  // survive into a string the organizer or the room sees.
  for (const [key, value] of entries()) {
    assert.ok(!value.includes('`'), `${key} kept SPEC's backticks: ${JSON.stringify(value)}`);
    assert.ok(!/\*\*/.test(value), `${key} kept SPEC's bold markers`);
  }
});

test('the static fallback names its keys in plain words (AC-102)', () => {
  assert.strictEqual(COPY.static_next_phase, 'Space next phase');
  assert.strictEqual(COPY.static_step_trace, '← → step the trace');
  assert.strictEqual(COPY.static_back_phase, 'Esc back a phase');
});
