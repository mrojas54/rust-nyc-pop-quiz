// AC-40 — correct is conveyed by glyph as well as colour, never colour alone.
//
// SCOPE. EVALUATION.md's AC-40 row names four surfaces: the wall, the buzzer's
// "✓ It was X.", take-it-home, and the review screen. None exists yet (T-05,
// T-06, T-12, the review ticket), and each must carry its own AC-40 assertion.
// What is provable here is the shared helper they all call.
//
// A NOTE ON WHAT THESE TESTS CAN AND CANNOT DO, because an earlier version of
// this file got it wrong. A review defeated the first safety net by dropping a
// field of an object the module had already returned — no new reference to the
// class, so identifier-counting never saw it, and all ten tests passed on a
// `correctHtml` that emitted colour without the glyph. The fix was to the API,
// not to the test: nothing in check.js now carries the class separately from
// the glyph, and the class is not exported at all. These tests verify that
// shape. They do NOT claim that openTag/applyCorrect could never themselves be
// edited to drop the glyph — no source-level test can promise that.

const test = require('node:test');
const assert = require('node:assert');
const { load, source, makeElement } = require('./_load.js');

const { PQ } = load('check');

const GLYPH = '✓';
const CLASS = 'rn-correct';

test('the glyph is a real check mark', () => {
  assert.strictEqual(PQ.CHECK, GLYPH);
  assert.strictEqual(PQ.checkMark(), GLYPH);
});

test('correctHtml carries the glyph and the colour class together', () => {
  const html = PQ.correctHtml('Vec is empty');
  assert.ok(html.includes(CLASS), 'the colour class');
  assert.ok(html.includes(GLYPH), 'the glyph');
  assert.ok(html.includes('Vec is empty'));
});

test('applyCorrect marks an element with both, in one call', () => {
  const el = makeElement('div');
  el.innerHTML = 'option C';
  assert.strictEqual(PQ.applyCorrect(el), true);
  assert.strictEqual(el.classList.contains(CLASS), true);
  assert.ok(el.innerHTML.includes(GLYPH));
  assert.ok(el.innerHTML.includes('option C'), 'existing content is kept');
});

test('applyCorrect refuses an element it cannot mark', () => {
  assert.strictEqual(PQ.applyCorrect(null), false);
  assert.strictEqual(PQ.applyCorrect({}), false);
});

// --- the property AC-40 actually needs --------------------------------------

test('no exported call yields the colour class without the glyph', () => {
  // Every exported function, over the option shapes the four surfaces will
  // realistically pass — including ones that suppress the screen-reader label,
  // change the tag, or add a class of their own.
  const shapes = [
    undefined, {}, null,
    { srLabel: '' }, { srLabel: null }, { srLabel: 'Correct answer' },
    { tag: 'li' }, { tag: 'div' },
    { className: 'wall-option' }, { className: '' },
    { tag: 'li', className: 'bar', srLabel: '' },
  ];
  const texts = ['A', '', undefined, null, 'Vec<&str>', 'a & b'];

  for (const opts of shapes) {
    for (const text of texts) {
      const html = PQ.correctHtml(text, opts);
      if (html.includes(CLASS)) {
        assert.ok(html.includes(GLYPH),
          `correctHtml(${JSON.stringify(text)}, ${JSON.stringify(opts)}) → colour without glyph`);
      }
    }
    const el = makeElement('div');
    PQ.applyCorrect(el, opts);
    if (el.classList.contains(CLASS)) {
      assert.ok(el.innerHTML.includes(GLYPH),
        `applyCorrect(${JSON.stringify(opts)}) → colour without glyph`);
    }
  }
});

test('no OPTION the module reads can separate the class from the glyph', () => {
  // The fixed shapes above can only cover options someone thought to list. This
  // discovers the option surface instead of guessing it: a Proxy records every
  // key the module actually reads, then each recorded key is driven through a
  // range of values and the invariant re-checked.
  //
  // This is what defeats the mutation class that beat the previous version — an
  // opt-in flag inside the glyph-emitting path. The flag has to be read from
  // opts to fire, reading it puts it in the recorded set, and the sweep then
  // turns it on. It is still not a proof for a flag only read when a SECOND
  // flag is already set; nothing short of exhaustive search covers that.
  function keysReadBy(call) {
    const seen = new Set();
    const probe = new Proxy({}, {
      get(_t, k) { if (typeof k === 'string') seen.add(k); return undefined; },
      has(_t, k) { if (typeof k === 'string') seen.add(k); return false; },
    });
    call(probe);
    return [...seen];
  }

  const VALUES = [true, false, '', 0, 1, 'x', null, undefined, {}, []];

  const htmlKeys = keysReadBy((o) => PQ.correctHtml('A', o));
  assert.ok(htmlKeys.length > 0, 'the probe should observe some option reads');
  for (const k of htmlKeys) {
    for (const v of VALUES) {
      const html = PQ.correctHtml('A', { [k]: v });
      if (html.includes(CLASS)) {
        assert.ok(html.includes(GLYPH),
          `correctHtml with {${k}: ${JSON.stringify(v)}} → colour without glyph`);
      }
    }
  }

  const applyKeys = keysReadBy((o) => PQ.applyCorrect(makeElement('div'), o));
  for (const k of applyKeys) {
    for (const v of VALUES) {
      const el = makeElement('div');
      PQ.applyCorrect(el, { [k]: v });
      if (el.classList.contains(CLASS)) {
        assert.ok(el.innerHTML.includes(GLYPH),
          `applyCorrect with {${k}: ${JSON.stringify(v)}} → colour without glyph`);
      }
    }
  }
});

test('the bare class name is not reachable from outside', () => {
  // This is what makes the property hold rather than merely being true today:
  // anything that could obtain "rn-correct" on its own could apply it without
  // the glyph, so nothing can.
  assert.strictEqual(PQ.CORRECT_CLASS, undefined, 'the class must not be exported');
  assert.strictEqual(PQ.correctParts, undefined,
    'correctParts handed out separable pieces and is gone');

  for (const [key, value] of Object.entries(PQ)) {
    if (typeof value === 'string') {
      assert.notStrictEqual(value, CLASS, `PQ.${key} exposes the bare class name`);
    }
  }
});

// --- structural: where the class may appear at all --------------------------

function codeOf(file) {
  return source(file)
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .split('\n')
    .filter((l) => !l.trim().startsWith('//'))
    .join('\n');
}

function bodyOf(code, name) {
  const start = code.indexOf(`function ${name}(`);
  assert.ok(start >= 0, `${name} not found`);
  const end = code.indexOf('\n  }', start);
  assert.ok(end > start, `${name} has no two-space closing brace`);
  return code.slice(start, end);
}

test('the class is used in exactly two functions, both of which emit the glyph', () => {
  const code = codeOf('check.js');

  const lines = code.split('\n').filter((l) => l.includes('CORRECT_CLASS'));
  assert.strictEqual(lines.length, 3,
    `CORRECT_CLASS should appear on exactly 3 lines (declaration, openTag, applyCorrect); found:\n${lines.join('\n')}`);

  for (const fn of ['openTag', 'applyCorrect']) {
    const body = bodyOf(code, fn);
    assert.ok(body.includes('CORRECT_CLASS'), `${fn} uses the class`);
    assert.ok(body.includes('glyphHtml'), `${fn} emits the glyph in the same call`);
  }

  // correctHtml must not touch the class: it receives openTag()'s finished
  // string, which already contains the glyph, so it has nothing to separate.
  assert.ok(!bodyOf(code, 'correctHtml').includes('CORRECT_CLASS'),
    'correctHtml must go through openTag rather than naming the class');
});

// --- no ✗, anywhere ---------------------------------------------------------

test('there is no way to mark anything incorrect', () => {
  // DESIGN.md: "No ✗, anywhere." AC-94: no participant surface marks a
  // participant's own answer incorrect.
  for (const name of ['incorrect', 'incorrectHtml', 'wrong', 'wrongHtml', 'crossMark', 'applyIncorrect']) {
    assert.strictEqual(PQ[name], undefined, `${name} must not exist`);
  }
  assert.ok(!source('check.js').includes('✗'), 'check.js must contain no ✗');
});

test('the glyph is hidden from screen readers and the word is not', () => {
  const html = PQ.correctHtml('A');
  assert.ok(html.includes('aria-hidden="true"'));
  assert.ok(html.includes('class="sr-only"'));
  assert.ok(html.includes('Correct'));
});

test('option text is escaped — it is program text', () => {
  const html = PQ.correctHtml('Vec<&str>');
  assert.ok(!html.includes('<&str>'));
  assert.ok(html.includes('&lt;'));
});
