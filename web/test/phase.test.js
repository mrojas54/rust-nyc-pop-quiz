// AC-99, the pure-function half — which phases render source, and which of
// those may colour it. well.test.js proves the rendering follows.

const test = require('node:test');
const assert = require('node:assert');
const { load } = require('./_load.js');

const { PQ } = load('phase');

test('the seven phases are G-6\'s, in G-6\'s order', () => {
  assert.deepStrictEqual([...PQ.PHASES],
    ['idle', 'live', 'closed', 'split', 'work', 'reveal', 'released']);
});

test('the source renders in the middle five phases and no others', () => {
  // idle is FIRST and renders nothing — the title wall is one line. released
  // is last and renders nothing — the link and the QR own that screen.
  for (const p of ['live', 'closed', 'split', 'work', 'reveal']) {
    assert.strictEqual(PQ.rendersSource(p), true, p);
  }
  for (const p of ['idle', 'released']) {
    assert.strictEqual(PQ.rendersSource(p), false, p);
  }
});

test('colour is on in the reading phases and off wherever a trace runs', () => {
  for (const p of ['live', 'closed', 'split']) {
    assert.strictEqual(PQ.colourAllowed(p), true, p);
  }
  for (const p of ['work', 'reveal']) {
    assert.strictEqual(PQ.colourAllowed(p), false, p);
  }
});

test('a phase that renders no source allows no colour either', () => {
  // AC-99 is three-valued. idle and released are not "colour off" — they are
  // "no source", and a caller asking either question gets a safe answer.
  for (const p of ['idle', 'released']) {
    assert.strictEqual(PQ.colourAllowed(p), false, p);
    assert.strictEqual(PQ.rendersSource(p), false, p);
  }
});

test('colour is allowed in exactly the phases where no trace runs', () => {
  for (const p of PQ.PHASES) {
    if (!PQ.rendersSource(p)) continue;
    assert.strictEqual(PQ.colourAllowed(p), !PQ.tracing(p),
      `${p}: colour and the trace must never both be on`);
  }
});

test('tracing is true for work and reveal only', () => {
  assert.strictEqual(PQ.tracing('work'), true);
  assert.strictEqual(PQ.tracing('reveal'), true);
  for (const p of ['idle', 'live', 'closed', 'split', 'released']) {
    assert.strictEqual(PQ.tracing(p), false, p);
  }
});

test('an unknown phase throws rather than falling through to colour on', () => {
  // A typo that silently permitted colour would be an AC-99 failure that looks
  // like a render bug, and it would reach the room before anyone noticed.
  for (const bad of ['Live', 'reveal ', 'walkthrough', '', null, undefined, 42]) {
    assert.throws(() => PQ.colourAllowed(bad), /unknown phase/, String(bad));
    assert.throws(() => PQ.rendersSource(bad), /unknown phase/, String(bad));
  }
});

test('PHASES is frozen, so a caller cannot reorder the phase machine', () => {
  assert.throws(() => PQ.PHASES.push('bonus-round'));
  assert.strictEqual(PQ.PHASES.length, 7);
  assert.strictEqual(PQ.PHASES[0], 'idle');
});
