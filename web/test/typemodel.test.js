// AC-100 — the wall's type model.
//
// SCOPE. EVALUATION.md puts AC-100's headline proof in `test-full` (the wall's
// measured overflow is zero where it reports "fits") and in `bank-audit` (the
// flag on a question that will not fit at the floor). Neither hook exists yet:
// T-05 builds the wall, T-19 builds the audit. What is provable here, under
// `just test`, is the arithmetic and the verdict logic those two hooks will
// call — SPEC §5.2's derivation, its six-pass measured refit, and the four fit
// verdicts. This file claims that and no more.
//
// T-19 reimplements floorPx / sourceMetrics / desiredFontPx in Python and
// tests against the same §5.2 worked examples. If a number below changes, that
// mirror changes with it.

const test = require('node:test');
const assert = require('node:assert');
const { load, to1, shapedSource, plain, makeElement } = require('./_load.js');

const { PQ } = load('typemodel');
const K = PQ.TYPE_CONSTANTS;

test('the room defaults are SPEC §5.2\'s, and give a 14.2px floor', () => {
  assert.deepStrictEqual(plain(PQ.ROOM_DEFAULTS), {
    screen_width_ft: 15,
    screen_height_ft: 8.44,
    back_row_ft: 20,
  });
  assert.strictEqual(to1(PQ.floorPx()), 14.2);
  assert.strictEqual(to1(PQ.floorPx(PQ.ROOM_DEFAULTS)), 14.2);
});

test('the floor moves with the room, which is the point of HC-1', () => {
  // Twice the distance wants twice the cap height.
  const far = { screen_width_ft: 15, screen_height_ft: 8.44, back_row_ft: 40 };
  assert.ok(PQ.floorPx(far) > PQ.floorPx() * 1.99);
  // A bigger screen makes the same distance easier.
  const big = { screen_width_ft: 30, screen_height_ft: 16.88, back_row_ft: 20 };
  assert.ok(PQ.floorPx(big) < PQ.floorPx());
});

test('the code area is the phase\'s, and "split" is a phase, not a layout', () => {
  // SPEC §5.2: reading layout for live/closed/split, trace layout for
  // work/reveal. The prototype also had a `split` LAYOUT (564x426, source
  // beside options) that the client rejected at T-20; it must not be here.
  for (const phase of ['live', 'closed', 'split']) {
    assert.deepStrictEqual(plain(PQ.codeAreaFor(phase)), { w: 994, h: 177 }, phase);
  }
  for (const phase of ['work', 'reveal']) {
    assert.deepStrictEqual(plain(PQ.codeAreaFor(phase)), { w: 994, h: 190 }, phase);
  }
  for (const phase of ['idle', 'released']) {
    assert.strictEqual(PQ.codeAreaFor(phase), null, phase);
  }
  const areas = JSON.stringify(PQ.CODE_AREA);
  assert.ok(!areas.includes('564') && !areas.includes('426'),
    'the rejected split-column layout constants must not be ported');
});

test('the constants are the ones SPEC §5.2 states', () => {
  assert.strictEqual(K.CANVAS_W, 1120);
  assert.strictEqual(K.CANVAS_H, 630);
  assert.strictEqual(K.LINE_HEIGHT, 1.6);
  assert.strictEqual(K.MAX_FONT, 46);
  assert.strictEqual(K.CAP_RATIO, 0.7);
  assert.strictEqual(K.READ_RATIO, 150);
  assert.strictEqual(K.GUTTER_CH, 3.5);
  assert.strictEqual(K.CHAR_ADVANCE, 0.6);
  assert.strictEqual(K.OPTIONS_BLOCK_PX, 190);
  assert.strictEqual(K.BEAT_BLOCK_PX, 203);
  assert.strictEqual(K.OPTION_MAX_CHARS, 29);
  assert.strictEqual(K.REFIT_MAX_PASSES, 6);
  assert.strictEqual(K.REFIT_SHRINK, 0.97);
});

// SPEC §5.2's worked table, at the 15ft / 8.44ft / 20ft guess (floor 14.2px).
// `wants` is what the source asks for before the floor is applied — which is
// the number SPEC reports, including for the four that are below the floor.
const WORKED = [
  { q: 'q3', lines: 5, chars: 42, wants: 22.1, fits: true },
  { q: 'q4', lines: 6, chars: 56, wants: 18.4, fits: true },
  { q: 'q7', lines: 5, chars: 69, wants: 22.1, fits: true },
  { q: 'q8', lines: 6, chars: 30, wants: 18.4, fits: true },
  { q: 'q5', lines: 9, chars: 50, wants: 12.3, fits: false },
  { q: 'q6', lines: 9, chars: 32, wants: 12.3, fits: false },
  { q: 'q2', lines: 10, chars: 38, wants: 11.1, fits: false },
  { q: 'q1', lines: 16, chars: 36, wants: 6.9, fits: false },
];

test('SPEC §5.2\'s eight worked examples reproduce to one decimal', () => {
  const floor = PQ.floorPx();
  for (const w of WORKED) {
    const src = shapedSource(w.lines, w.chars);
    const m = PQ.sourceMetrics(src);
    assert.strictEqual(m.lines, w.lines, `${w.q} line count`);
    assert.strictEqual(m.widestChars, w.chars, `${w.q} widest line`);

    const d = PQ.derivedFontPx(src, 'live');
    assert.strictEqual(to1(d.wants), w.wants, `${w.q} wants ${w.wants}px`);
    assert.strictEqual(d.belowFloor, !w.fits, `${w.q} below the floor?`);

    if (w.fits) {
      // Above the floor: the wall renders what the source wants.
      assert.strictEqual(to1(d.fontPx), w.wants, `${w.q} renders at ${w.wants}px`);
    } else {
      // Below the floor: legibility wins, and the question is too long for
      // this room. That is the room's limit, not the generator's constant.
      assert.strictEqual(to1(d.fontPx), to1(floor), `${w.q} clamps to the floor`);
      assert.ok(d.wants < floor, `${w.q} wants less than the floor`);
    }
  }
});

test('by source length only q3, q4, q7 and q8 fit — SPEC §5.2\'s own conclusion', () => {
  const fitting = WORKED
    .filter((w) => !PQ.derivedFontPx(shapedSource(w.lines, w.chars), 'live').belowFloor)
    .map((w) => w.q);
  assert.deepStrictEqual(fitting, ['q3', 'q4', 'q7', 'q8']);
});

test('the trace layout is more generous than the reading one', () => {
  // 190px of trace text box against 177px of reading text box: the same source
  // may fit while walking and not while reading. bank-audit fits against the
  // reading layout, the smaller (SPEC §5.2).
  const src = shapedSource(9, 50); // q5's shape
  const reading = PQ.derivedFontPx(src, 'live');
  const trace = PQ.derivedFontPx(src, 'work');
  assert.ok(trace.wants > reading.wants);
});

test('the 46px cap binds on a short source', () => {
  const d = PQ.derivedFontPx('fn main() {}', 'live');
  assert.strictEqual(d.wants, 46);
  assert.strictEqual(d.fontPx, 46);
  assert.strictEqual(d.cappedAtMax, true);
});

test('the floor beats the 46px cap, because legibility is not negotiable', () => {
  // A room far enough back that even 46px is illegible: SPEC §5.2 says the
  // floor wins and the wall reports it.
  const tiny = { screen_width_ft: 6, screen_height_ft: 3.38, back_row_ft: 40 };
  const floor = PQ.floorPx(tiny);
  assert.ok(floor > K.MAX_FONT, 'fixture must put the floor above the cap');
  const d = PQ.derivedFontPx('fn main() {}', 'live', tiny);
  assert.strictEqual(d.fontPx, floor);
  assert.ok(d.fontPx > K.MAX_FONT);
  assert.strictEqual(d.floorAboveCap, true);
});

// --- the measured refit -----------------------------------------------------

// A measurer that reports overflow proportional to how far the font exceeds a
// notional fitting size. refit() takes its measurer as a parameter, which is
// what lets the loop be proven with no layout engine.
function measurerThatFitsAt(fitPx, axis = 'y') {
  return (fontPx) => {
    const over = Math.max(0, Math.round((fontPx - fitPx) * 10));
    const box = 200;
    return {
      overX: axis === 'x' || axis === 'xy' ? over : 0,
      overY: axis === 'y' || axis === 'xy' ? over : 0,
      cw: box, ch: box,
      sw: box + (axis === 'x' || axis === 'xy' ? over : 0),
      sh: box + (axis === 'y' || axis === 'xy' ? over : 0),
    };
  };
}

test('refit shrinks an overflowing well and stops once it fits', () => {
  const calls = [];
  const measure = measurerThatFitsAt(20);
  const r = PQ.refit(40, 14.2, (px) => { calls.push(px); return measure(px); });

  assert.ok(r.passes >= 1, 'it should have shrunk at least once');
  assert.ok(r.passes <= K.REFIT_MAX_PASSES);
  assert.ok(r.fontPx < 40, 'the font came down');
  assert.strictEqual(calls[0], 40, 'the first render is at the derived size');
  assert.ok(!PQ.overflows(r.measurement), 'it stops fitting, not guessing');
});

test('refit never runs more than six passes', () => {
  // A well that always overflows by a little, so each pass shrinks by only
  // ~3.5% and the floor is never the thing that stops it. The pass cap is.
  let renders = 0;
  const r = PQ.refit(40, 0.001, () => {
    renders += 1;
    return { overX: 0, overY: 5, cw: 1000, ch: 1000, sw: 1000, sh: 1005 };
  });
  assert.strictEqual(r.passes, K.REFIT_MAX_PASSES);
  assert.ok(r.fontPx > 1, 'this fixture must stop on the pass cap, not the floor');
  // one initial render, then one per pass
  assert.strictEqual(renders, K.REFIT_MAX_PASSES + 1);
});

test('refit stops at the floor even when passes remain', () => {
  // The other stop condition: a well overflowing badly enough that k drives
  // the size to the floor in fewer than six passes. Legibility ends the loop.
  let renders = 0;
  const r = PQ.refit(40, 1, () => {
    renders += 1;
    return { overX: 0, overY: 500, cw: 100, ch: 100, sw: 100, sh: 600 };
  });
  assert.ok(r.passes < K.REFIT_MAX_PASSES, 'it reached the floor early');
  assert.strictEqual(r.fontPx, 1);
  assert.strictEqual(r.atFloor, true);
  // Still overflowing at the floor: this is the question that is too long for
  // the room, and the wall must say so rather than shrink further.
  assert.strictEqual(PQ.fitVerdict(r.measurement), 'clipped_y');
});

test('refit never goes below the floor', () => {
  const floor = 14.2;
  const r = PQ.refit(40, floor, () => ({
    overX: 0, overY: 500, cw: 100, ch: 100, sw: 100, sh: 600,
  }));
  assert.ok(r.fontPx >= floor, `${r.fontPx} >= ${floor}`);
  assert.strictEqual(r.atFloor, true);
});

test('refit does not shrink a well that already fits', () => {
  let renders = 0;
  const r = PQ.refit(22.1, 14.2, () => {
    renders += 1;
    return { overX: 0, overY: 0, cw: 200, ch: 200, sw: 200, sh: 200 };
  });
  assert.strictEqual(r.passes, 0);
  assert.strictEqual(r.fontPx, 22.1);
  assert.strictEqual(renders, 1);
});

test('refit tolerates sub-pixel overflow rather than shrinking on rounding', () => {
  // scrollWidth/clientWidth are integers rounded from fractional layout, so a
  // well that fits exactly can report 1px. Shrinking on that would make the
  // wall smaller than the room needs.
  const r = PQ.refit(22.1, 14.2, () => ({
    overX: 2, overY: 1, cw: 200, ch: 200, sw: 202, sh: 201,
  }));
  assert.strictEqual(r.passes, 0);
  assert.strictEqual(r.fontPx, 22.1);
});

test('refit applies k x 0.97 as SPEC §5.2 writes it', () => {
  const sizes = [];
  PQ.refit(40, 1, (px) => {
    sizes.push(px);
    return { overX: 0, overY: 100, cw: 100, ch: 100, sw: 100, sh: 200 };
  });
  // k = ch/sh = 100/200 = 0.5, so each pass multiplies by 0.5 * 0.97.
  assert.strictEqual(sizes[1], 40 * 0.5 * K.REFIT_SHRINK);
  assert.strictEqual(sizes[2], sizes[1] * 0.5 * K.REFIT_SHRINK);
});

// --- the verdict ------------------------------------------------------------

const M = (overX, overY) => ({ overX, overY, cw: 100, ch: 100, sw: 100 + overX, sh: 100 + overY });

test('fitVerdict returns each of the four values', () => {
  assert.strictEqual(PQ.fitVerdict(M(0, 0)), 'fits');
  assert.strictEqual(PQ.fitVerdict(M(40, 0)), 'clipped_x');
  assert.strictEqual(PQ.fitVerdict(M(0, 40)), 'clipped_y');
  assert.strictEqual(PQ.fitVerdict(M(40, 40)), 'clipped_xy');
});

test('the tolerance is applied per axis, so the host is sent to the right edge', () => {
  // 1px of horizontal slop beside real vertical overflow is clipped_y, not
  // clipped_xy — the verdict becomes words telling the host which edge lost
  // content, and naming both would send them to the wrong side of the screen.
  assert.strictEqual(PQ.fitVerdict(M(1, 40)), 'clipped_y');
  assert.strictEqual(PQ.fitVerdict(M(40, 2)), 'clipped_x');
  assert.strictEqual(PQ.fitVerdict(M(2, 2)), 'fits');
  assert.strictEqual(PQ.fitVerdict(M(3, 3)), 'clipped_xy');
});

test('an unmeasured well has no verdict, and does not get "fits"', () => {
  // AC-100: "fits" is a measured claim after layout, not a computed guess, and
  // DESIGN.md says nothing is silently clipped. Returning "fits" here would
  // assert the measurement without making it.
  assert.strictEqual(PQ.fitVerdict(null), null);
  assert.strictEqual(PQ.fitVerdict(undefined), null);
  assert.strictEqual(PQ.measureWell(null), null);
  assert.strictEqual(PQ.measureWell({}), null);
});

test('measureWell reports overflow on both axes', () => {
  const m = PQ.measureWell({ scrollWidth: 120, clientWidth: 100, scrollHeight: 250, clientHeight: 200 });
  assert.strictEqual(m.overX, 20);
  assert.strictEqual(m.overY, 50);
  assert.strictEqual(PQ.fitVerdict(m), 'clipped_xy');
});

// --- the clipped edge -------------------------------------------------------

test('the red edge lands on the side that lost content, and only that side', () => {
  assert.strictEqual(PQ.fitEdgeClasses('fits'), '');
  assert.strictEqual(PQ.fitEdgeClasses(null), '');
  assert.strictEqual(PQ.fitEdgeClasses('clipped_x'), ' clipped-x');
  assert.strictEqual(PQ.fitEdgeClasses('clipped_y'), ' clipped-y');
  assert.strictEqual(PQ.fitEdgeClasses('clipped_xy'), ' clipped-x clipped-y');
});

test('applyClippedEdge sets and clears the edge on an element', () => {
  const el = makeElement('div');

  assert.strictEqual(PQ.applyClippedEdge(el, 'clipped_y'), true);
  assert.strictEqual(el.classList.contains('clipped-y'), true);
  assert.strictEqual(el.classList.contains('clipped-x'), false);

  // A refit that now fits must take the edge back off.
  assert.strictEqual(PQ.applyClippedEdge(el, 'fits'), false);
  assert.strictEqual(el.classList.contains('clipped-y'), false);

  assert.strictEqual(PQ.applyClippedEdge(el, 'clipped_xy'), true);
  assert.strictEqual(el.classList.contains('clipped-x'), true);
  assert.strictEqual(el.classList.contains('clipped-y'), true);
});

// --- the bank's side of AC-100 ---------------------------------------------

test('an option is one line of at most 29 characters (D-15)', () => {
  assert.strictEqual(PQ.optionFits('x'.repeat(29)), true);
  assert.strictEqual(PQ.optionFits('x'.repeat(30)), false);
  assert.strictEqual(PQ.optionFits('short\nbut two lines'), false,
    'the wall has no design for a wrapped option');
});
