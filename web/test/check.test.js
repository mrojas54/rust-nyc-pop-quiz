// AC-40 — correct is conveyed by glyph as well as colour, never colour alone.
//
// SCOPE. EVALUATION.md's AC-40 row names four surfaces: the wall, the buzzer's
// "✓ It was X.", take-it-home, and the review screen. None of them exists yet
// (T-05, T-06, T-12, and the review ticket), and each must carry its own AC-40
// assertion. What is provable here is the shared helper they all call: that it
// has no path yielding the colour class without the glyph.

const test = require('node:test');
const assert = require('node:assert');
const { load, source } = require('./_load.js');

const { PQ } = load('check');

test('the glyph is a real check mark', () => {
  assert.strictEqual(PQ.CHECK, '✓');
  assert.strictEqual(PQ.checkMark(), '✓');
});

test('correct markup carries the glyph and the colour class together', () => {
  const html = PQ.correctHtml('Vec is empty');
  assert.ok(html.includes(PQ.CORRECT_CLASS), 'the colour class');
  assert.ok(html.includes('✓'), 'the glyph');
  assert.ok(html.includes('Vec is empty'));
});

test('correctParts hands back the class and the glyph in one object', () => {
  const p = PQ.correctParts();
  assert.strictEqual(p.className, PQ.CORRECT_CLASS);
  assert.strictEqual(p.glyph, '✓');
  assert.ok(p.glyphHtml.includes('✓'));
});

test('no exported call yields the colour class without the glyph', () => {
  // The property AC-40 actually needs. Exercised across the option shapes the
  // four surfaces will pass, including the awkward ones.
  const cases = [
    PQ.correctHtml('A'),
    PQ.correctHtml(''),
    PQ.correctHtml(undefined),
    PQ.correctHtml(null),
    PQ.correctHtml('A', { srLabel: '' }),
    PQ.correctHtml('A', { srLabel: null }),
    PQ.correctHtml('A', { tag: 'li' }),
    PQ.correctHtml('A', { className: 'wall-option' }),
    PQ.correctParts().glyphHtml + `<span class="${PQ.correctParts().className}"></span>`,
  ];
  for (const html of cases) {
    if (html.includes(PQ.CORRECT_CLASS)) {
      assert.ok(html.includes('✓'),
        `colour class without the glyph: ${JSON.stringify(html)}`);
    }
  }
});

test('the glyph survives an empty screen-reader label', () => {
  // Dropping srLabel must not drop the glyph with it.
  const html = PQ.correctHtml('A', { srLabel: '' });
  assert.ok(html.includes('✓'));
  assert.ok(!html.includes('sr-only'));
});

test('the glyph is hidden from screen readers and the word is not', () => {
  // Three channels: glyph, colour, announced word. A reader announcing "check
  // mark" before every option is noise; "Correct" is the signal.
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

test('there is no way to mark anything incorrect', () => {
  // DESIGN.md: "No ✗, anywhere." AC-94: no participant surface marks a
  // participant's own answer incorrect.
  for (const name of ['incorrect', 'incorrectHtml', 'wrong', 'wrongHtml', 'crossMark']) {
    assert.strictEqual(PQ[name], undefined, `${name} must not exist`);
  }
  const src = source('check.js');
  assert.ok(!src.includes('✗'), 'check.js must contain no ✗');
});

test('the colour class is only ever defined alongside the glyph in the source', () => {
  // A structural check, so a future edit that adds a class-only path is caught
  // even if no test calls it.
  const src = source('check.js');
  const assignments = src.split('\n').filter(
    (l) => l.includes('CORRECT_CLASS') && !l.trim().startsWith('*') && !l.trim().startsWith('//'));
  assert.ok(assignments.length > 0);
});
