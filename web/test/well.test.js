// AC-99 as rendered — coloured token spans present in live/closed/split, ZERO
// in work/reveal, and no source element at all in idle/released.
//
// The fixture is written to contain one of every token class the colourer
// knows, so a regression that drops (say) macros is caught rather than hidden
// behind the five that still work.

const test = require('node:test');
const assert = require('node:assert');
const { load } = require('./_load.js');

const { PQ } = load('well');

// keyword (let, fn), type (Vec, String), macro (println!), string ("hi"),
// number (42), comment (// ...). Not a real bank question and not run: the
// type model only reads shape, and nothing here writes down what a program
// prints (CLAUDE.md).
const FIXTURE = [
  '// a comment',
  'fn main() {',
  '    let v: Vec<String> = Vec::new();',
  '    println!("hi {}", 42);',
  '}',
].join('\n');

const TOKEN_CLASSES = ['tk-k', 'tk-t', 'tk-m', 'tk-s', 'tk-n', 'tk-c'];
const countSpans = (html) => (html.match(/class="tk-/g) || []).length;

test('every token class the colourer knows appears in a reading phase', () => {
  const html = PQ.sourceWellHtml(FIXTURE, { phase: 'live' });
  for (const cls of TOKEN_CLASSES) {
    assert.ok(html.includes(`class="${cls}"`), `${cls} is missing — colour regressed`);
  }
});

test('colour spans are present in live, closed and split', () => {
  for (const phase of ['live', 'closed', 'split']) {
    const html = PQ.sourceWellHtml(FIXTURE, { phase });
    assert.ok(countSpans(html) > 0, `${phase} should carry colour`);
  }
});

test('colour spans are exactly zero in work and reveal', () => {
  // The trace signals by highlight-and-dim and nothing may compete with it.
  for (const phase of ['work', 'reveal']) {
    const html = PQ.sourceWellHtml(FIXTURE, { phase });
    assert.strictEqual(countSpans(html), 0, `${phase} must carry no colour`);
    assert.ok(html.includes('rn-src'), `${phase} still renders the well`);
  }
});

test('idle and released render no source at all', () => {
  for (const phase of ['idle', 'released']) {
    assert.strictEqual(PQ.sourceWellHtml(FIXTURE, { phase }), '', phase);
  }
});

test('renderSource reports whether it rendered, so a caller need not re-ask', () => {
  const el = { innerHTML: 'stale' };
  assert.strictEqual(PQ.renderSource(el, FIXTURE, { phase: 'live' }), true);
  assert.ok(el.innerHTML.includes('rn-src'));

  // An empty string, not an empty bordered box: the title and released walls
  // have their own designs and a stray frame is visible on a projector.
  assert.strictEqual(PQ.renderSource(el, FIXTURE, { phase: 'idle' }), false);
  assert.strictEqual(el.innerHTML, '');
});

test('a caller cannot ask for colour in a phase that forbids it', () => {
  // The prototype took opts.syntax from its caller, so any caller could colour
  // any phase. Here the phase decides and only the phase.
  const html = PQ.sourceWellHtml(FIXTURE, { phase: 'work', syntax: true, colour: true });
  assert.strictEqual(countSpans(html), 0);
});

test('an unknown phase throws rather than rendering something', () => {
  assert.throws(() => PQ.sourceWellHtml(FIXTURE, { phase: 'walkthrough' }), /unknown phase/);
});

test('the well has line numbers in a 2ch gutter and a focusable scroll region', () => {
  const html = PQ.sourceWellHtml(FIXTURE, { phase: 'live' });
  assert.ok(html.includes('class="rn-src-ln"'), 'line numbers');
  assert.ok(html.includes('style="width:1ch"'), 'gutter sized to the line count');
  assert.ok(html.includes('class="rn-src-scroll"'));
  assert.ok(html.includes('tabindex="0"'), 'keyboard reachable (AC-82)');
  assert.ok(html.includes('role="region"'));
  // one line element per source line
  assert.strictEqual((html.match(/class="rn-src-line/g) || []).length, 5);
});

test('the gutter widens with the line count', () => {
  const long = Array.from({ length: 12 }, (_, i) => `let x${i} = ${i};`).join('\n');
  assert.ok(PQ.sourceWellHtml(long, { phase: 'live' }).includes('style="width:2ch"'));
});

test('the derived font size is applied, and never read off this file', () => {
  const html = PQ.sourceWellHtml(FIXTURE, { phase: 'live', size: '22.1px' });
  assert.ok(html.includes('style="font-size:22.1px"'));
});

test('highlight and dim mark the trace step', () => {
  const html = PQ.sourceWellHtml(FIXTURE, { phase: 'work', hl: [3], focus: [2, 4] });
  assert.ok(html.includes('rn-src-line hl'), 'the executing line');
  assert.ok(html.includes('rn-src-line dim'), 'outside the focus region');
  // Line 3 is highlighted, so it must not also be dimmed.
  assert.ok(!html.includes('rn-src-line hl dim'));
});

test('source text is escaped — it is program text, not markup', () => {
  const html = PQ.sourceWellHtml('let v: Vec<&str> = vec![];', { phase: 'live' });
  assert.ok(!html.includes('<&str>'), 'raw angle brackets would break the markup');
  assert.ok(html.includes('&lt;') && html.includes('&gt;'));
});

test('a string literal\'s contents are not coloured as keywords', () => {
  // Comments and strings win over everything, or the room reads a lie about
  // what the program says.
  const html = PQ.sourceWellHtml('let s = "let fn match";', { phase: 'live' });
  const inString = html.slice(html.indexOf('class="tk-s"'));
  assert.ok(!inString.slice(0, 60).includes('tk-k'),
    'keywords inside a string literal must stay plain');
});

test('a blank line still gets a gutter number', () => {
  const html = PQ.sourceWellHtml('fn a() {}\n\nfn b() {}', { phase: 'live' });
  assert.strictEqual((html.match(/class="rn-src-line/g) || []).length, 3);
});

test('a single trailing newline is not a line', () => {
  // The type model counts lines the same way, through the same splitter.
  assert.deepStrictEqual([...PQ.sourceLines('a\nb\n')], ['a', 'b']);
  assert.deepStrictEqual([...PQ.sourceLines('a\nb')], ['a', 'b']);
});

test('the clipped edge renders on the well when the verdict says so', () => {
  const { PQ: full } = load('typemodel');
  const clipped = full.sourceWellHtml(FIXTURE, { phase: 'live', fit: 'clipped_y' });
  assert.ok(clipped.includes('clipped-y'));
  assert.ok(!clipped.includes('clipped-x'));
  assert.ok(clipped.includes('aria-describedby'), 'colour is never the only signal');

  const fits = full.sourceWellHtml(FIXTURE, { phase: 'live', fit: 'fits' });
  assert.ok(!fits.includes('clipped-'));
});

test('a label with a quote in it cannot break out of its attribute', () => {
  // escapeHtml is safe for text content but not for an attribute value, and
  // aria-label is an attribute. T-05/T-06/T-07 may pass a bank-derived label.
  const html = PQ.sourceWellHtml('fn main() {}', {
    phase: 'live', label: 'What does "this" print?',
  });
  assert.ok(!html.includes('aria-label="What does "this" print?"'),
    'the raw quote would end the attribute early');
  assert.ok(html.includes('&quot;'), 'quotes are entity-escaped in the attribute');
  // The header, which is text content, still reads naturally.
  assert.ok(html.includes('What does &quot;this&quot; print?'));
});

test('escapeAttr covers both quote characters', () => {
  assert.strictEqual(PQ.escapeAttr(`a"b'c<d&e`), 'a&quot;b&#39;c&lt;d&amp;e');
});
