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

// Text cases: {text, expect}. notes.json holds whole records for checkProse.
const TEXT_FIXTURES = ['retired.json', 'forbidden.json', 'tropes.json', 'near-misses.json', 'clean.json', 'dialect.json',
  'normalization.json'];
const FIXTURE_FILES = [...TEXT_FIXTURES, 'notes.json'];

test('the fixture set is the one the Python suite reads, and nothing else', () => {
  assert.deepEqual(fs.readdirSync(FIXTURES).filter((f) => f.endsWith('.json')).sort(), [...FIXTURE_FILES].sort());
});

for (const name of TEXT_FIXTURES) {
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

test('a match is quoted from the normalized text', () => {
  const [hit] = L.check('Well. That\u2019s\u00a0fine, then.');
  assert.equal(hit.id, 'reassurance/1');
  assert.equal(hit.match, "That's fine");
});

test('the engine alone behaves as dialect.json says (expect_raw, before normalize)', () => {
  for (const c of fixture('dialect.json')) {
    const want = c.expect_raw !== undefined ? c.expect_raw : c.expect !== undefined ? c.expect : c.expect_js;
    assert.deepEqual([...L.checkRaw(c.text)], want, c.note);
  }
});

test('every normalization probe walked past the rules before normalize, and is caught after', () => {
  for (const c of fixture('normalization.json')) {
    assert.notDeepEqual([...L.checkRaw(c.text)], c.expect, c.note);
    assert.deepEqual(ids(c.text), c.expect, c.note);
  }
});

test('normalize is idempotent', () => {
  for (const c of [...fixture('normalization.json'), ...fixture('dialect.json')]) {
    const once = L.normalize(c.text);
    assert.equal(L.normalize(once), once, c.note);
  }
});

function records() {
  const f = JSON.parse(fs.readFileSync(path.join(FIXTURES, 'notes.json'), 'utf8'));
  assert.ok(f.records.length > 0, 'notes.json has no records');
  return f.records;
}

test('trace notes are read and warned on as hint is (F-47, AC-98)', () => {
  for (const r of records()) {
    assert.deepEqual([...L.checkProse(r.record)].map((w) => [w.field, w.id]), r.expect, r.note);
  }
});

test('the notes fixture trips every rule family in a note', () => {
  const fired = new Set(records().flatMap((r) => r.expect).filter(([f]) => f.startsWith('trace.steps[')).map(([, id]) => id.split('/')[0]));
  assert.deepEqual([...fired].sort(), ['forbidden', ...Object.keys(L.TROPES)].sort());
});

test('a record that crashes the lint is one lint-error warning, never clean', () => {
  const SECRET = 'secret prose from a private bank';
  const record = { get explains() { throw new TypeError(SECRET); } };
  const got = [...L.checkProse(record)].map((w) => ({ ...w }));
  assert.deepEqual(got, [{ field: 'record', id: 'lint-error', group: 'lint-error', pattern: '', matched: 'TypeError' }]);
  const steps = [{ note: 'Fine.' }, { get note() { throw new RangeError(SECRET); } }];
  const onNote = [...L.checkProse({ trace: { steps } })].map((w) => ({ ...w }));
  assert.equal(onNote.length, 1);
  assert.equal(onNote[0].id, 'lint-error');
  assert.equal(onNote[0].matched, 'RangeError');
  assert.ok(!JSON.stringify(onNote).includes(SECRET));
  // A crash inside one field's check names that field.
  const real = L.check;
  L.check = (text) => { if (text === 'hint text') throw new RangeError(SECRET); return real(text); };
  try {
    const onHint = [...L.checkProse({ explains: { what: "Don't worry." }, hint: 'hint text' })].map((w) => ({ ...w }));
    assert.deepEqual(onHint, [{ field: 'hint', id: 'lint-error', group: 'lint-error', pattern: '', matched: 'RangeError' }]);
  } finally {
    L.check = real;
  }
});

// --- 2a. The Python twin, run over the same files --------------------------------

// Both suites assert against the same `expect`, so a rule dropped in one
// language fails that language's own check. This test also runs the Python
// twin itself over every fixture and compares case by case, so a disagreement
// fails here whichever side moved. One direction only: the pipeline suite may
// start no process (tests/test_runner.py guards it), so it does not run node.
// Needs pipeline/.venv (`just setup`); a missing venv fails, never skips.
const PY_DUMP = String.raw`
import json, pathlib, sys, unicodedata
from popquiz import copylint
d = pathlib.Path(sys.argv[1])
out = {"unicode": unicodedata.unidata_version, "files": {}}
for p in sorted(d.glob("*.json")):
    f = json.loads(p.read_text(encoding="utf-8"))
    if "records" in f:
        out["files"][p.name] = [[[w.field, w.id] for w in copylint.check_prose(r["record"])] for r in f["records"]]
    else:
        out["files"][p.name] = [{"check": [h.id for h in copylint.check(c["text"])], "raw": copylint.check_raw(c["text"])} for c in f["cases"]]
print(json.dumps(out))
`;

test('the Python twin reports what this one does on every fixture, bar the one named divergence', () => {
  const { spawnSync } = require('node:child_process');
  const r = spawnSync('uv', ['run', '--offline', '--no-sync', 'python', '-c', PY_DUMP, FIXTURES],
    { cwd: path.join(REPO, 'pipeline'), encoding: 'utf8' });
  assert.equal(r.status, 0, `the Python twin did not run: ${r.error || ''}${r.stderr || ''}`);
  const py = JSON.parse(r.stdout);
  const versions = `Unicode: python ${py.unicode}, node ${process.versions.unicode}`;
  assert.deepEqual(Object.keys(py.files).sort(), [...FIXTURE_FILES].sort());
  let compared = 0;
  for (const name of TEXT_FIXTURES) {
    fixture(name).forEach((c, i) => {
      const theirs = py.files[name][i];
      const mine = { check: ids(c.text), raw: [...L.checkRaw(c.text)] };
      if (c.expect_js !== undefined) {
        // dialect.json's one documented case: each engine holds its own side.
        assert.deepEqual(mine.check, c.expect_js, c.note);
        assert.deepEqual(theirs.check, c.expect_py, c.note);
      } else {
        assert.deepEqual(theirs, mine, `${name}[${i}] ${c.note || ''} (${versions})`);
      }
      compared++;
    });
  }
  records().forEach((rec, i) => {
    assert.deepEqual(py.files['notes.json'][i], [...L.checkProse(rec.record)].map((w) => [w.field, w.id]), `notes.json[${i}] ${rec.note}`);
    compared++;
  });
  assert.ok(compared >= 80,`only ${compared} fixture cases compared`);
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
