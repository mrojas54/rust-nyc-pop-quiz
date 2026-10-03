// The copy lints (T-22): SPEC §11's Forbidden row (G-5, AC-98) and the §11.1
// trope check (D-23, AC-42), from their one source, web/test/copylint.js.
//
// Three things are proven here:
//   1. the patterns are SPEC's, character for character — read out of SPEC.md,
//      not retyped into this file;
//   2. they behave as the hand-written fixtures in bank/fixtures/copy-lint/
//      say, the same files pipeline/tests/test_copylint.py runs against the
//      Python twin, so the two engines are held to one expectation;
//   3. the copy module is clean of both: a match of either lint in any entry
//      fails the build (§11.1: "Those strings are fixed and ours").
// Question prose takes the other consequence — a warning on the review screen,
// never a failure — and is the Python twin's (copylint.check_prose).

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { load } = require('./_load.js');

const REPO = path.join(__dirname, '..', '..');
const FIXTURES = path.join(REPO, 'bank', 'fixtures', 'copy-lint');

const { PQ } = load('copylint');
const L = PQ.COPYLINT;
const COPY = load('copy').PQ.COPY;

// --- 1. SPEC's patterns, verbatim ----------------------------------------------

function specPatterns() {
  const spec = fs.readFileSync(path.join(REPO, 'SPEC.md'), 'utf8');
  // The Forbidden row: the backticked list between "cites this row |" and the
  // note on the near-misses, with the markdown table's `\|` read as `|`.
  const row = spec.split('\n').find((l) => l.startsWith('| **Forbidden**'));
  assert.ok(row, 'SPEC.md has no Forbidden row');
  const list = row.slice(row.indexOf('cites this row |') + 'cites this row |'.length, row.indexOf("The wall's"));
  const forbidden = [...list.matchAll(/`((?:[^`\\]|\\.)*)`/g)].map((m) => m[1].replace(/\\\|/g, '|'));
  // §11.1: the fenced block; a line opening at column 0 names its group.
  const at = spec.indexOf('### 11.1 The trope check');
  const open = spec.indexOf('```', at);
  const close = spec.indexOf('```', open + 3);
  const tropes = {};
  let group = null;
  for (const line of spec.slice(open + 3, close).split('\n')) {
    if (!line.trim()) continue;
    const m = /^(\S+)?\s+(\S.*)$/.exec(line);
    if (m[1]) group = m[1];
    (tropes[group] = tropes[group] || []).push(m[2].trim());
  }
  return { forbidden, tropes };
}

test('the Forbidden patterns are SPEC §11\'s row, verbatim and in order', () => {
  const { forbidden } = specPatterns();
  assert.equal(forbidden.length, 9);
  assert.deepEqual([...L.FORBIDDEN], forbidden);
});

test('the trope patterns are SPEC §11.1\'s, verbatim, grouped and in order', () => {
  const { tropes } = specPatterns();
  assert.deepEqual(Object.keys(tropes), ['contrast', 'filler', 'signpost', 'reassurance', 'flattery']);
  assert.equal(Object.values(tropes).flat().length, 16);
  assert.deepEqual(JSON.parse(JSON.stringify(L.TROPES)), tropes);
});

test('every rule compiles case-insensitive, without the u flag', () => {
  assert.equal(L.RULES.length, 25);
  for (const r of L.RULES) assert.equal(r.re.flags, 'i', r.id);
});

// --- 2. The shared fixtures ----------------------------------------------------

function fixture(name) {
  const f = JSON.parse(fs.readFileSync(path.join(FIXTURES, name), 'utf8'));
  assert.ok(f.cases.length > 0, `${name} has no cases`);
  return f.cases;
}
// Spread into this realm: arrays built inside node:vm fail deepStrictEqual
// against a literal here on their prototype alone (_load.js, plain()).
const ids = (text) => [...L.check(text)].map((h) => h.id);

test('the fixture set is the one the Python suite reads, and nothing else', () => {
  assert.deepEqual(fs.readdirSync(FIXTURES).filter((f) => f.endsWith('.json')).sort(),
    ['clean.json', 'dialect.json', 'forbidden.json', 'near-misses.json', 'retired.json', 'tropes.json']);
});

for (const name of ['retired.json', 'forbidden.json', 'tropes.json', 'near-misses.json', 'clean.json', 'dialect.json']) {
  test(`${name}: every case matches exactly the rules it names`, () => {
    for (const c of fixture(name)) {
      const want = c.expect !== undefined ? c.expect : c.expect_js;
      assert.deepEqual(ids(c.text), want, `${JSON.stringify(c.text)}${c.note ? ' — ' + c.note : ''}`);
    }
  });
}

test('every Forbidden pattern and every trope pattern has a positive case of its own', () => {
  const fired = new Set([...fixture('forbidden.json'), ...fixture('tropes.json')].flatMap((c) => c.expect));
  for (const r of L.RULES) assert.ok(fired.has(r.id), `no fixture fires ${r.id}`);
});

test('every trope group fires on the retired strings, and each retired string is caught (AC-42)', () => {
  const retired = fixture('retired.json');
  for (const group of Object.keys(L.TROPES)) {
    assert.ok(retired.some((c) => L.tropes(c.text).some((h) => h.group === group)),
      `the ${group} group fires on none of the retired strings`);
  }
  for (const c of retired) assert.ok(L.tropes(c.text).length > 0, `no trope caught: ${c.text}`);
});

test('the two allowed near-misses stay allowed', () => {
  for (const c of fixture('near-misses.json')) assert.deepEqual([...L.forbidden(c.text)], [], c.text);
  // And they are the copy module's own templates, filled.
  const copy = load('copy').PQ;
  assert.equal(copy.fill(copy.COPY.wall_reveal_most_chosen, { n: 24, X: 'B' }), fixture('near-misses.json')[0].text);
  assert.equal(copy.fill(copy.COPY.host_reveal_beat_why, { n: 24, X: 'B' }), fixture('near-misses.json')[1].text);
});

test('a match is quoted from the original text, curly apostrophe and all', () => {
  const [hit] = L.check('Well. That’s fine, then.');
  assert.equal(hit.id, 'reassurance/1');
  assert.equal(hit.match, 'That’s fine');
});

// --- 3. The copy module: any match fails the build -----------------------------

test('no copy module entry matches the Forbidden row (G-5, AC-98)', () => {
  const hits = Object.entries(COPY).flatMap(([k, v]) => L.forbidden(v).map((h) => `${k}: ${h.id} "${h.match}"`));
  assert.deepEqual(hits, []);
});

test('no copy module entry matches the trope check — a match fails the build (§11.1, D-23)', () => {
  const hits = Object.entries(COPY).flatMap(([k, v]) => L.tropes(v).map((h) => `${k}: ${h.id} "${h.match}"`));
  assert.deepEqual(hits, []);
});

test('the lint runs over every entry, the PROPOSED-§11 ones included', () => {
  const proposed = load('copy').PQ.COPY_ROWS['PROPOSED-§11 (not yet in SPEC)'];
  assert.ok(proposed && proposed.length > 0, 'no PROPOSED-§11 row');
  for (const k of proposed) assert.equal(typeof COPY[k], 'string', k);
});
