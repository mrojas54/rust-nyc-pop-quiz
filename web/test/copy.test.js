// The copy module: completeness against SPEC §11, and cleanliness against the
// two lints that police it.
//
// SCOPE. The lints themselves (the Forbidden row, G-5 and AC-98; the §11.1
// trope check, AC-42) are T-22's one source, web/shared/copylint.js, and run over
// this module in copylint.test.js. This file is about the module's shape: every
// §11 row has its keys, every key has a row, placeholders, lookup, and the
// PROPOSED-§11 block's limits.

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

test('the host page is headed by the wordmark, Host beneath', () => {
  assert.strictEqual(COPY.title_wordmark, 'Rust NYC Pop Quiz');
  assert.strictEqual(COPY.host_title_role, 'Host');
});

// PQ-34, HC-0 (2026-09-27): "remove any text that is absolutely unnecessary.
// no need to narrate the demo." These keys are gone and their strings with
// them; a line that comes back has to come back through SPEC §11.
const REMOVED = {
  buzzer_join_beneath: 'or open the link on the screen',
  buzzer_foot: 'no account · no name · no score',
  buzzer_split_lookup: 'Look up.',
  buzzer_split_where: 'Where the room landed.',
  buzzer_split_said: 'people said ‹X›, including you.',
  buzzer_split_readings: 'Five different readings. Nobody knows what anyone picked.',
  buzzer_work_walking: "We're walking it through.",
  buzzer_work_nothing: 'Nothing to do. Nobody knows the answer yet.',
  buzzer_reveal_on_screen: 'The answer is on the screen.',
  buzzer_reveal_company: 'You and ‹n−1› other people read it the same way.',
  buzzer_reveal_company_one: 'You and 1 other person read it the same way.',
  buzzer_reveal_host_reading: 'The host is reading out the why now.',
  buzzer_reveal_only_one: 'You were the only one who read it that way.',
  buzzer_noanswer_count: "‹k› people didn't answer, you included.",
  buzzer_noanswer_reveal: '✓ It was ‹Y›. The host is reading out the why now.',
  buzzer_foot_computed: 'computed on this phone · never sent anywhere',
  buzzer_released: 'Nothing about you was recorded.',
  buzzer_hint_shown: 'Only you can see this. Nobody is told you looked.',
  wall_work_lead: "Let's walk it.",
  wall_work_no_answer: 'Still no answer.',
  wall_work_nobody: 'Nobody has to say anything.',
  wall_released_line: 'the question, the walk-through and the why — at your own pace.',
  host_first_resume: 'If you lose this phone, open this on another one: ‹resume link›',
  not_a_guarantee_options_public: 'Option text is public — the correct answer is always one of the five visible options.',
  not_a_guarantee_host_honest: "A host who reads Rust can work out the answer from the source; the host's not being shown it keeps the host honest, it is not a security guarantee.",
};

test('PQ-34: the removed keys and their strings are gone', () => {
  for (const [key, str] of Object.entries(REMOVED)) {
    assert.ok(!Object.prototype.hasOwnProperty.call(COPY, key), `${key} is back`);
    assert.ok(!Object.values(COPY).includes(str), `"${str}" is back under another key`);
  }
  assert.strictEqual(COPY.buzzer_idle, "You're in.");
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

// --- the two lints --------------------------------------------------------------

// The Forbidden row and the §11.1 trope check run over this module from their
// one source, web/shared/copylint.js, in web/test/copylint.test.js: patterns
// pinned to SPEC.md, the shared fixtures, and a failure on any match in any
// entry here. This file keeps only what is about the module's own shape.

test('no ✗ appears anywhere', () => {
  // DESIGN.md: "No ✗, anywhere." AC-94. (Also forbidden/8; kept as its own
  // line because it is a design rule as much as a lint.)
  for (const [key, value] of entries()) {
    assert.ok(!value.includes('✗'), key);
  }
});

// --- PROPOSED-§11 -----------------------------------------------------------------

const PROPOSED_ROW = 'PROPOSED-§11 (not yet in SPEC)';

test('the PROPOSED-§11 row holds exactly the proposed_ keys, and no §11 row holds one', () => {
  // The block is for strings a page already said before T-22 moved them here,
  // waiting for §11 to adopt them. It must not become a place to author copy.
  const proposed = Object.keys(COPY).filter((k) => k.startsWith('proposed_'));
  assert.deepStrictEqual([...ROWS[PROPOSED_ROW]].sort(), proposed.sort());
  for (const [row, keys] of Object.entries(ROWS)) {
    if (row === PROPOSED_ROW) continue;
    assert.ok(!keys.some((k) => k.startsWith('proposed_')), row);
  }
});

test('the PROPOSED-§11 strings are the words the pages used, unchanged', () => {
  // F-34 (home.js), F-38 (trace.js), and the three T-22 found. A change of
  // words is a §11 decision, not an edit here.
  assert.deepStrictEqual(Object.fromEntries(ROWS[PROPOSED_ROW].map((k) => [k, COPY[k]])), {
    proposed_home_nothing_yet: 'Welcome to the POP QUIZ',
    proposed_home_miri_separately: 'run separately',
    proposed_home_row_compiler: 'compiler',
    proposed_home_row_edition: 'edition',
    proposed_home_row_target: 'target',
    proposed_home_row_flags: 'flags',
    proposed_home_row_miri: 'miri',
    proposed_home_miri_seeds: 'seeds',
    proposed_trace_previous_step: 'previous step',
    proposed_trace_next_step: 'next step',
    proposed_trace_value_now: '‹name› is now ‹now›',
    proposed_check_correct: 'Correct',
    proposed_well_label: 'Source code',
  });
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
