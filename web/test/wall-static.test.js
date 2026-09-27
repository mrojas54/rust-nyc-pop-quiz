// The static fallback (T-26, SPEC §12): the driver, the phase rules, and parity
// with the live wall. AC-102, AC-97, AC-99.
//
// Two generated fixtures, neither typed by hand:
//   web/wall/fixtures/q3-phases.json  the room's view::wall frames for q3
//                                      (room/tests/wall_page.rs)
//   web/wall/fixtures/q3-static.json  the pipeline's bake of q3
//                                      (popquiz.fallback --bake; test_fallback.py
//                                      holds it equal)
// Parity is the claim that matters: fed the same counts, the static frame builder
// produces the room's frames, and the one renderer draws both the same, minus the
// strip at the bottom (the join strip, which a room-less file cannot have).

const test = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { load } = require('./_load');

const WALL = path.join(__dirname, '..', 'wall');

function loaded() {
  const l = load('all', { timers: false });
  for (const f of ['qr.js', 'wall.js', 'fallback/static.js']) {
    vm.runInContext(fs.readFileSync(path.join(WALL, f), 'utf8'), l.sandbox, { filename: `web/wall/${f}` });
  }
  return l;
}

const { PQ } = loaded();
const S = PQ.Static;
const W = PQ.Wall;
const FIX = JSON.parse(fs.readFileSync(path.join(WALL, 'fixtures', 'q3-phases.json'), 'utf8'));
const F = FIX.frames;
const U = FIX.unanimous.frames;
const BAKE = JSON.parse(fs.readFileSync(path.join(WALL, 'fixtures', 'q3-static.json'), 'utf8'));
const M = BAKE.trace.length;

// Host-realm plain data (vm objects carry another realm's prototypes).
const plain = (o) => JSON.parse(JSON.stringify(o));
const counts = (fix, frames) => ({ totals: fix.totals, present: frames.split.split.present });
const TOKEN_SPAN = /<span class="tk-[kstmnc]">/g;
const count = (s, re) => (s.match(re) || []).length;
const noStrip = (h) => h.replace(/<div class="joinstrip">[\s\S]*?<\/div><\/div><\/div>$/, '</div></div>');

// The room's frame for one static state.
function roomFrame(frames, s) {
  if (s.phase === 'work') return frames.work.find((f) => f.trace.at === s.at);
  if (s.phase === 'reveal') return frames.reveal.find((f) => f.trace.at === s.at);
  return frames[s.phase];
}

// Every state the driver can reach, in the order a host walks it.
function every() {
  const out = [];
  for (const phase of S.PHASES) {
    if (phase === 'work') for (let at = 0; at <= M - 2; at++) out.push({ phase, at });
    else if (phase === 'reveal') for (let at = M - 1; at >= 0; at--) out.push({ phase, at });
    else out.push({ phase, at: 0 });
  }
  return out;
}

// --------------------------------------------------------------------------
// The driver: Space, Esc, ← → (AC-102)
// --------------------------------------------------------------------------

test('Space walks the seven views in the room\'s order, entering work at 0 and reveal at M-1', () => {
  let s = S.enter(BAKE, 'idle');
  const seen = [plain(s)];
  for (let i = 0; i < 6; i++) { s = S.press(BAKE, s, ' '); seen.push(plain(s)); }
  assert.deepStrictEqual(seen, [
    { phase: 'idle', at: 0 }, { phase: 'live', at: 0 }, { phase: 'closed', at: 0 },
    { phase: 'split', at: 0 }, { phase: 'work', at: 0 }, { phase: 'reveal', at: M - 1 },
    { phase: 'released', at: 0 },
  ]);
  assert.deepStrictEqual(plain(S.press(BAKE, s, ' ')), plain(s), 'nothing follows released');
  // Space as every engine spells it: key " ", code "Space", old "Spacebar".
  for (const k of [' ', 'Space', 'Spacebar']) {
    assert.strictEqual(S.press(BAKE, S.enter(BAKE, 'idle'), k).phase, 'live', k);
  }
  const st = fs.readFileSync(path.join(WALL, 'fallback', 'static.js'), 'utf8');
  assert.match(st, /press\(bake, state, ev\.key\) \|\| \(ev\.code \? press\(bake, state, ev\.code\) : null\)/);
});

test('Esc goes back one phase at a time, to the start; idle is the start', () => {
  let s = S.enter(BAKE, 'released');
  const seen = [];
  for (let i = 0; i < 7; i++) { s = S.press(BAKE, s, 'Escape'); seen.push(s.phase); }
  assert.deepStrictEqual(seen, ['reveal', 'work', 'split', 'closed', 'live', 'idle', 'idle']);
  // back into reveal lands on the resolving step; back into work on its first
  assert.strictEqual(S.back(BAKE, S.enter(BAKE, 'released')).at, M - 1);
  assert.strictEqual(S.back(BAKE, { phase: 'reveal', at: 2 }).at, 0);
});

test('← → step the trace only in work and reveal, and never past their bounds (AC-97)', () => {
  for (const phase of ['idle', 'live', 'closed', 'split', 'released']) {
    const s = S.enter(BAKE, phase);
    assert.deepStrictEqual(plain(S.press(BAKE, s, 'ArrowRight')), plain(s), `${phase} →`);
    assert.deepStrictEqual(plain(S.press(BAKE, s, 'ArrowLeft')), plain(s), `${phase} ←`);
  }
  let w = S.enter(BAKE, 'work');
  for (let i = 0; i < M + 3; i++) w = S.press(BAKE, w, 'ArrowRight');
  assert.strictEqual(w.at, M - 2, 'work stops at M-2: the resolving step is never walked (D-10)');
  for (let i = 0; i < M + 3; i++) w = S.press(BAKE, w, 'ArrowLeft');
  assert.strictEqual(w.at, 0);
  let r = S.enter(BAKE, 'reveal');
  assert.strictEqual(r.at, M - 1, 'reveal enters at M-1');
  r = S.press(BAKE, r, 'ArrowRight');
  assert.strictEqual(r.at, M - 1, 'nothing past M-1');
  for (let i = 0; i < M + 3; i++) r = S.press(BAKE, r, 'ArrowLeft');
  assert.strictEqual(r.at, 0, 'reveal steps back through all of it');
  assert.strictEqual(S.press(BAKE, r, 'q'), null, 'a key the driver does not take');
});

// --------------------------------------------------------------------------
// The phase rules on the rendered views (AC-97, AC-99)
// --------------------------------------------------------------------------

test('work carries no ✓, no receipt, no colour, no stdout and no step beyond M-2', () => {
  for (let at = 0; at <= M - 2; at++) {
    const f = S.frame(BAKE, { phase: 'work', at });
    const h = W.html(f);
    assert.ok(!h.includes('✓'), `step ${at}: ✓`);
    assert.ok(!h.includes('receipt'), `step ${at}: receipt`);
    assert.strictEqual(count(h, TOKEN_SPAN), 0, `step ${at}: colour`);
    assert.ok(!/>stdout</.test(h) && !JSON.stringify(f).includes('"stdout"'), `step ${at}: stdout`);
    assert.ok(h.includes(`Step ${at + 1} of ${M}`));
    assert.ok(!h.includes(`Step ${M} of ${M}`));
    assert.strictEqual(f.reveal, undefined);
  }
  // asked for a step past M-2, the frame still shows M-2
  assert.strictEqual(S.frame(BAKE, { phase: 'work', at: M - 1 }).trace.at, M - 2);
});

test('colour in live, closed and split; none in work and reveal; no source in idle and released (AC-99)', () => {
  for (const s of every()) {
    const h = W.html(S.frame(BAKE, s));
    const n = count(h, TOKEN_SPAN);
    if (['live', 'closed', 'split'].includes(s.phase)) assert.ok(n > 0, `${s.phase}: no colour`);
    else assert.strictEqual(n, 0, `${s.phase}@${s.at}: ${n} coloured spans`);
    assert.strictEqual(h.includes('class="rn-src'), !['idle', 'released'].includes(s.phase), s.phase);
    if (s.phase !== 'reveal') assert.ok(!h.includes('✓'), `${s.phase}: ✓ before reveal`);
  }
});

test('reveal enters at M-1 with the ✓, the baked receipt and the stdout step', () => {
  const h = W.html(S.frame(BAKE, S.enter(BAKE, 'reveal')));
  assert.ok(h.includes(`Step ${M} of ${M}`));
  assert.ok(h.includes('<span class="rn-check" aria-hidden="true">✓</span>'));
  const got = [...h.matchAll(/<span class="receipt-line">([^<]*)<\/span>/g)].map((m) => m[1]);
  assert.deepStrictEqual(got, BAKE.receipt.lines, 'G-7: the pipeline\'s lines, printed verbatim');
});

// --------------------------------------------------------------------------
// With no phones: the room's own rendering of a room where nobody answered
// --------------------------------------------------------------------------

test('no counts: empty bars at 0 · 0%, "Nobody read it another way.", no answered line, no join strip', () => {
  const split = W.html(S.frame(BAKE, { phase: 'split', at: 0 }));
  assert.strictEqual(count(split, /<span class="bar-n">0 · 0%<\/span>/g), 5);
  assert.ok(!split.includes('in the room answered'));
  assert.ok(split.includes('<div class="joinstrip"></div>'));
  const reveal = W.html(S.frame(BAKE, S.enter(BAKE, 'reveal')));
  assert.ok(reveal.includes(`<div class="wall-middle">${PQ.COPY.wall_reveal_nobody_else}</div>`));
  for (const s of every()) {
    const h = W.html(S.frame(BAKE, s));
    assert.ok(!h.includes('join @'), `${s.phase}: a join strip`);
  }
  // idle's strip holds §11's key legend where the join strip was
  const idle = W.html(S.frame(BAKE, { phase: 'idle', at: 0 }));
  assert.ok(idle.includes(`<div class="joinstrip"><span>${PQ.escapeHtml(S.legend())}</span></div>`));
  assert.strictEqual(S.legend(), 'Space next phase · ← → step the trace · Esc back a phase');
});

// --------------------------------------------------------------------------
// Parity with the room (SPEC §12: "never a second design")
// --------------------------------------------------------------------------

function parity(fix, frames, label) {
  for (const s of every()) {
    const room = plain(roomFrame(frames, s));
    const mine = plain(S.frame(BAKE, s, counts(fix, frames)));
    // The frame: the room's, minus what only a room has (its code, its join
    // strip, its own take-it-home host) and plus the idle key legend.
    const expect = { ...room };
    delete expect.code; delete expect.join;
    const got = { ...mine };
    if (s.phase === 'idle') { assert.strictEqual(got.strip, S.legend()); delete got.strip; }
    if (s.phase === 'released') {
      assert.strictEqual(got.released.link, BAKE.home_link);
      got.released = { ...got.released, link: room.released.link };
    }
    assert.deepStrictEqual(got, expect, `${label} ${s.phase}@${s.at}: frame`);
    // The rendered DOM, minus the strip at the bottom.
    const a = W.html(roomFrame(frames, s), { fontPx: 22.1 });
    const b = W.html(S.frame(BAKE, s, counts(fix, frames)), { fontPx: 22.1 });
    if (s.phase === 'released') continue; // the link differs by host; the frame check above covers the rest
    assert.strictEqual(noStrip(b), noStrip(a), `${label} ${s.phase}@${s.at}: DOM`);
  }
}

test('parity: the static frames are the room\'s q3 frames, and render the same DOM minus the strip', () => {
  parity(FIX, F, 'split room');
});

test('parity: the unanimous room too (the "nobody read it another way" variant)', () => {
  parity(FIX.unanimous, U, 'unanimous');
});

test('parity check is not vacuous: the strip removal leaves the canvas', () => {
  const h = W.html(F.live, { fontPx: 22.1 });
  const cut = noStrip(h);
  assert.notStrictEqual(cut, h);
  assert.ok(cut.includes('wall-options') && cut.endsWith('</div></div>') && !cut.includes('joinstrip'));
});

// --------------------------------------------------------------------------
// The live wall is untouched by static mode
// --------------------------------------------------------------------------

test('the live wall still boots itself; a static wrap is left to static.js', () => {
  const src = fs.readFileSync(path.join(WALL, 'wall.js'), 'utf8');
  assert.match(src, /getAttribute\("data-mode"\) !== "static"/);
  const st = fs.readFileSync(path.join(WALL, 'fallback', 'static.js'), 'utf8');
  assert.ok(!/WebSocket|fetch\(|XMLHttpRequest|EventSource/.test(st), 'static.js opens no connection');
  const page = fs.readFileSync(path.join(WALL, 'index.html'), 'utf8');
  assert.ok(!page.includes('static.js'), 'the live page does not load the static driver');
});
