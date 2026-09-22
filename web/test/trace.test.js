// The trace renderer and, more importantly, the bound.
//
// SPEC §3.4: "trace_step — in `work` bounded to 0..M-2; `reveal` enters at M-1
// and may step the whole trace." Those are 0-based indices. The last step is
// the resolving one — its `values` names `stdout` — and D-10 withholds it until
// reveal, so `work` tops out at M-2. AC-97 and AC-102 both restate it.

const test = require('node:test');
const assert = require('node:assert');
const { load } = require('./_load.js');

const { PQ } = load('trace');

// Six steps, shaped like SPEC §5.3's model. The last step names stdout and is
// the resolving one (D-10) — the thing `work` must never reach.
const Q = {
  source: 'fn main() {\n    let v = 1;\n    let w = 2;\n    println!("{}", v + w);\n}',
  trace: {
    steps: [
      { lines: [2], focus: [1, 5], note: 'first', values: [{ name: 'v', was: '—', now: '1' }] },
      { lines: [3], focus: [2, 4], note: 'second', values: [] },
      { lines: [3], focus: [3, 3], note: 'third', values: [{ name: 'w', was: '—', now: '2' }] },
      { lines: [3], focus: [3, 3], note: 'fourth', values: [] },
      { lines: [3], focus: [3, 3], pivot: true, note: 'the turn', values: [] },
      { lines: [4], focus: [1, 5], note: 'last', values: [{ name: 'stdout', was: '—', now: '3' }] },
    ],
  },
};
const M = Q.trace.steps.length; // 6

// --- the bound --------------------------------------------------------------

test('the bound is 0-based: work stops at M-2, reveal enters at M-1', () => {
  assert.strictEqual(PQ.workMaxIndex(M), M - 2); // 4
  assert.strictEqual(PQ.revealEntryIndex(M), M - 1); // 5
});

test('work cannot reach the resolving step, which is D-10\'s whole point', () => {
  const bound = PQ.workMaxIndex(M);
  const reached = [];
  for (let i = 0; i < M + 3; i++) reached.push(PQ.clampStep(i, M, bound));
  assert.strictEqual(Math.max(...reached), M - 2);

  // The step work tops out on carries no stdout; the one it cannot reach does.
  const top = PQ.traceStepOf(Q, 99, bound);
  assert.ok(!(top.values || []).some((v) => v.name === 'stdout'));
  const resolving = Q.trace.steps[M - 1];
  assert.ok(resolving.values.some((v) => v.name === 'stdout'));
});

test('reveal may step the whole trace, including the resolving step', () => {
  const at = PQ.clampStep(PQ.revealEntryIndex(M), M);
  assert.strictEqual(at, M - 1);
  assert.ok(PQ.traceStepOf(Q, at).values.some((v) => v.name === 'stdout'));
});

test('the trace is steppable both ways', () => {
  const bound = PQ.workMaxIndex(M);
  let at = 0;
  at = PQ.clampStep(at + 1, M, bound); assert.strictEqual(at, 1);
  at = PQ.clampStep(at + 1, M, bound); assert.strictEqual(at, 2);
  at = PQ.clampStep(at - 1, M, bound); assert.strictEqual(at, 1);
  at = PQ.clampStep(at - 1, M, bound); assert.strictEqual(at, 0);
  at = PQ.clampStep(at - 1, M, bound); assert.strictEqual(at, 0, 'never below zero');
});

test('degenerate traces clamp to 0 rather than going negative', () => {
  // SPEC §7.4 refuses to affirm a trace with fewer than two steps, so M >= 2
  // in the bank; this is for fixtures and the static fallback.
  assert.strictEqual(PQ.workMaxIndex(2), 0);
  assert.strictEqual(PQ.workMaxIndex(1), 0);
  assert.strictEqual(PQ.workMaxIndex(0), 0);
  assert.strictEqual(PQ.revealEntryIndex(1), 0);
  assert.strictEqual(PQ.revealEntryIndex(0), 0);
  assert.strictEqual(PQ.clampStep(5, 1, PQ.workMaxIndex(1)), 0);
});

test('an absent bound means the whole trace', () => {
  assert.strictEqual(PQ.clampStep(99, M), M - 1);
  assert.strictEqual(PQ.clampStep(99, M, null), M - 1);
  assert.strictEqual(PQ.clampStep(99, M, undefined), M - 1);
});

test('a bound past the end cannot walk off the array', () => {
  assert.strictEqual(PQ.clampStep(99, M, 999), M - 1);
  assert.ok(PQ.traceStepOf(Q, 99, 999));
});

// --- what a step renders ----------------------------------------------------

test('the step carries its number, its words and its values', () => {
  const html = PQ.traceNoteHtml(Q, 0, { phase: 'work' });
  assert.ok(html.includes('Step 1 of 6'), 'the number is the accessible signal');
  assert.ok(html.includes('first'));
  assert.ok(html.includes('class="tname"'));
  assert.ok(html.includes('>v<'));
  assert.ok(html.includes('>1<'));
});

test('an absent value reads as an em dash, not as empty', () => {
  const html = PQ.traceNoteHtml(Q, 0, { phase: 'work' });
  assert.ok(html.includes('—'), 'was: "—" for absent');

  const nulls = { trace: { steps: [{ lines: [1], focus: [1, 1], note: 'n', values: [{ name: 'x' }] }] } };
  assert.ok(PQ.traceNoteHtml(nulls, 0, { phase: 'work' }).includes('—'));
});

test('a step with no values renders no table', () => {
  const html = PQ.traceNoteHtml(Q, 1, { phase: 'work' });
  assert.ok(!html.includes('trace-values'));
});

test('a pivot step says Pause here.', () => {
  assert.ok(PQ.traceNoteHtml(Q, 4, { phase: 'work' }).includes('Pause here.'));
  assert.ok(!PQ.traceNoteHtml(Q, 0, { phase: 'work' }).includes('Pause here.'));
});

test('the dots span the whole trace even when stepping is bounded', () => {
  // The room should see how far there is to go; only the stepping is bounded.
  const html = PQ.traceNoteHtml(Q, 0, { phase: 'work', maxIndex: PQ.workMaxIndex(M) });
  assert.strictEqual((html.match(/<i class=/g) || []).length, M);
  assert.ok(html.includes('aria-hidden="true"'), 'dots are supplementary, never the signal');
});

test('the wall renders progress only — it takes no interaction (AC-79)', () => {
  const html = PQ.traceNoteHtml(Q, 2, { phase: 'work', noNav: true });
  assert.ok(!html.includes('<button'));
  assert.ok(html.includes('Step 3 of 6'));
});

test('next is disabled at the bound, and previous at the start', () => {
  const bound = PQ.workMaxIndex(M);
  const atBound = PQ.traceNoteHtml(Q, bound, { phase: 'work', maxIndex: bound });
  assert.ok(/next step"\s+disabled|disabled[^>]*aria-label="next step"/.test(atBound)
    || atBound.includes('disabled aria-label="next step"'),
    'next must be disabled at M-2 in work');

  const atStart = PQ.traceNoteHtml(Q, 0, { phase: 'work' });
  assert.ok(atStart.includes('disabled aria-label="previous step"'));
});

test('the trace source well carries no colour in work or reveal', () => {
  for (const phase of ['work', 'reveal']) {
    const html = PQ.traceSourceHtml(Q, 0, { phase });
    assert.strictEqual((html.match(/class="tk-/g) || []).length, 0, phase);
    assert.ok(html.includes('rn-src-line hl'), 'highlight is the signal instead');
  }
});

test('the spoken form carries the number, the words and the values (AC-83)', () => {
  const said = PQ.traceSay(Q, 0);
  assert.ok(said.startsWith('Step 1 of 6.'));
  assert.ok(said.includes('first'));
  assert.ok(said.includes('v is now 1'));
  assert.ok(!said.includes('<'), 'spoken, not rendered');
});

test('the spoken form honours the bound too', () => {
  const bound = PQ.workMaxIndex(M);
  assert.ok(!PQ.traceSay(Q, 99, bound).includes('stdout'),
    'the resolving step must not be announced during work');
});

test('notes and values are escaped', () => {
  const nasty = {
    trace: { steps: [{ lines: [1], focus: [1, 1], note: '<img src=x>', values: [{ name: '<b>', was: '&', now: '<i>' }] }] },
  };
  const html = PQ.traceNoteHtml(nasty, 0, { phase: 'work' });
  assert.ok(!html.includes('<img'));
  assert.ok(html.includes('&lt;img'));
  assert.ok(html.includes('&amp;'));
});
