// The wall (T-05): phase -> rendered structure, on the real q3 payloads.
//
// Every frame here comes from web/wall/fixtures/q3-phases.json, which
// room/tests/wall_page.rs generates from view::wall and holds equal to what the
// room serves — so nothing below renders a payload somebody typed, and nothing
// writes down what a program prints (CLAUDE.md). Wall.html() is pure, so this
// suite needs no browser; the measured half of AC-100 and AC-33 is the
// measured-layout run (web/wall/measure.html, `just wall-layout`).
//
// Criteria: AC-33, AC-39, AC-40, AC-74, AC-79, AC-99, AC-100 (the edge), G-7.

const test = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { load, plain } = require('./_load');

const WEB = path.join(__dirname, '..');
const REPO = path.join(WEB, '..');
const WALL = path.join(WEB, 'wall');

function wall() {
  const loaded = load('all', { timers: false });
  for (const f of ['qr.js', 'wall.js']) {
    vm.runInContext(fs.readFileSync(path.join(WALL, f), 'utf8'), loaded.sandbox, {
      filename: `web/wall/${f}`,
    });
  }
  return loaded;
}

const { PQ, document } = wall();
const W = PQ.Wall;
const FIXTURE = JSON.parse(fs.readFileSync(path.join(WALL, 'fixtures', 'q3-phases.json'), 'utf8'));
const F = FIXTURE.frames;
const U = FIXTURE.unanimous.frames;
const Q3 = JSON.parse(fs.readFileSync(path.join(REPO, 'bank', 'questions', 'q3.json'), 'utf8'));

// One frame per phase, and every work/reveal step.
const ALL = [
  ['idle', F.idle], ['live', F.live], ['closed', F.closed], ['split', F.split],
  ...F.work.map((f, i) => [`work[${i}]`, f]),
  ...F.reveal.map((f, i) => [`reveal[${i}]`, f]),
  ['released', F.released],
  ...U.reveal.map((f, i) => [`unanimous reveal[${i}]`, f]),
];

const count = (s, re) => (s.match(re) || []).length;
const TOKEN_SPAN = /<span class="tk-[kstmnc]">/g;

// Visible text nodes, entity-decoded.
function texts(html) {
  const out = [];
  const re = />([^<]+)</g;
  let m;
  while ((m = re.exec(html))) {
    const t = m[1]
      .replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'").replace(/&amp;/g, '&').trim();
    if (t) out.push(t);
  }
  return out;
}

// Every string value in a frame, at any depth.
function strings(v, out = new Set()) {
  if (typeof v === 'string') out.add(v);
  else if (Array.isArray(v)) v.forEach((x) => strings(x, out));
  else if (v && typeof v === 'object') Object.values(v).forEach((x) => strings(x, out));
  return out;
}

// --------------------------------------------------------------------------
// The fixture is what the tests think it is
// --------------------------------------------------------------------------

test('the fixture walks q3 through all seven phases', () => {
  assert.deepStrictEqual(
    Object.keys(F).sort(),
    ['closed', 'idle', 'live', 'released', 'reveal', 'split', 'work'],
  );
  const m = F.reveal[0].trace.m;
  assert.strictEqual(m, Q3.trace.steps.length);
  assert.strictEqual(F.work.length, m - 1, 'work steps 0..M-2 (D-10)');
  assert.deepStrictEqual(F.work.map((f) => f.trace.at), [...Array(m - 1).keys()]);
  assert.strictEqual(F.reveal[0].trace.at, m - 1, 'reveal enters at M-1');
});

// --------------------------------------------------------------------------
// Phase -> structure
// --------------------------------------------------------------------------

test('idle: the title card and the join strip, no source', () => {
  const h = W.html(F.idle);
  assert.match(h, /data-phase="idle"/);
  assert.ok(h.includes(`<h2>${PQ.COPY.wall_idle_title}</h2>`));
  assert.ok(!h.includes('rn-src'), 'idle renders no source (AC-99)');
  // join @ ‹link›, the link set apart and shown without its scheme.
  const link = F.idle.join.slice('join @ '.length);
  assert.ok(h.includes(`<span>join @ <b>${link.replace(/^https?:\/\//, '')}</b></span>`), h);
});

test('live: the source in colour, five options beneath, the join strip and nothing after the link', () => {
  const h = W.html(F.live, { fontPx: 22.1 });
  assert.ok(h.includes('<pre style="font-size:22.1px">'), 'the well takes the type model\'s size');
  assert.ok(h.includes(PQ.COPY.wall_live_well_header));
  assert.strictEqual(count(h, /<div class="opt">/g), 5);
  for (const o of F.live.options) {
    assert.ok(h.includes(`<span class="letter">${o.letter}</span><span class="otext">${PQ.escapeHtml(o.text)}</span>`), o.letter);
  }
  assert.match(h, /class="wall-block reading wall-options"/);
  // HC-0 (2026-09-27): `join @ ‹link›`, the same line as idle, nothing after it.
  assert.strictEqual(F.live.join, F.idle.join);
  const link = F.live.join.slice('join @ '.length).replace(/^https?:\/\//, '');
  assert.ok(h.includes(`<div class="joinstrip"><span>join @ <b>${link}</b></span>`), h);
  assert.ok(!h.includes('still open'));
  assert.ok(!h.includes('bar-row'), 'no bars before the split');
});

test('closed: same source and options; the strip says answers are closed and nothing else', () => {
  const h = W.html(F.closed);
  assert.strictEqual(count(h, /<div class="opt">/g), 5);
  assert.ok(h.includes(`<div class="joinstrip"><span>${PQ.COPY.wall_closed}</span></div>`));
});

test('split: five bars with n · p%, the answered line, and no answer', () => {
  const h = W.html(F.split);
  assert.strictEqual(count(h, /<div class="bar-row">/g), 5);
  for (const b of F.split.split.bars) {
    assert.ok(h.includes(`<span class="bar-fill" style="width:${b.percent}%"></span>`));
    assert.ok(h.includes(`<span class="bar-n">${b.count} · ${b.percent}%</span>`));
  }
  assert.ok(h.includes(`<span>${F.split.split.line}</span>`));
  assert.ok(!h.includes('<div class="opt">'), 'the bars take the options\' place (prototype)');
});

test('work: the trace without colour, the step within 0..M-2, and no beat panel', () => {
  const m = F.work[0].trace.m;
  F.work.forEach((f, i) => {
    const h = W.html(f);
    assert.ok(h.includes(`Step ${i + 1} of ${m}`), `step ${i}`);
    assert.ok(!h.includes(`Step ${m} of ${m}`), 'the resolving step is never shown in work (D-10)');
    assert.ok(!h.includes('workbeat'), 'no beat panel (PQ-34)');
    // highlight-and-dim from the step
    for (const n of f.trace.step.lines) {
      assert.match(h, new RegExp(`<span class="rn-src-line hl"><span class="rn-src-ln" aria-hidden="true" style="width:1ch">${n}</span>`));
    }
    assert.ok(!/"name":"stdout"|>stdout</.test(h), 'no stdout value in work');
    assert.ok(!h.includes('trace-nav"><button'), 'no stepping buttons on the wall');
  });
  const pivot = F.work.find((f) => f.trace.step.pivot);
  assert.ok(pivot, 'q3 has a pivot step in work');
  assert.ok(W.html(pivot).includes(` · ${PQ.COPY.wall_trace_pivot}`));
});

test('reveal: ✓ on the correct option, the totals, the named option, the receipt — and no explanation (AC-39)', () => {
  const f = F.reveal[0];
  const h = W.html(f);
  const r = f.reveal;
  assert.ok(h.includes(`Step ${f.trace.m} of ${f.trace.m}`), 'enters at the resolving step');
  // totals
  for (const b of f.split.bars) assert.ok(h.includes(`<span class="bar-n">${b.count} · ${b.percent}%</span>`));
  // the most-chosen incorrect option, named and counted
  assert.strictEqual(r.middle.letter, 'A');
  assert.ok(h.includes(`<div class="wall-middle">${r.middle.line}</div>`));
  // the receipt, every line
  for (const line of r.receipt.lines) assert.ok(h.includes(`<span class="receipt-line">${line}</span>`), line);
  // no explanation text: none of the three beats' prose, no why_tempting
  const prose = [Q3.explains.what, Q3.explains.takeaway,
    ...Q3.options.map((o) => o.why_tempting).filter(Boolean)];
  for (const p of prose) {
    assert.ok(!h.includes(PQ.escapeHtml(p)), `explanation on the wall: ${p.slice(0, 40)}…`);
    for (const piece of p.split(/(?<=[.!?])\s+/)) {
      if (piece.length > 24) assert.ok(!h.includes(PQ.escapeHtml(piece)), `a sentence of the explanation: ${piece}`);
    }
  }
});

test('reveal steps back through the whole trace, without colour', () => {
  const m = F.reveal[0].trace.m;
  assert.deepStrictEqual(F.reveal.map((f) => f.trace.at), [...Array(m).keys()].reverse());
  for (const f of F.reveal) {
    const h = W.html(f);
    assert.ok(h.includes(`Step ${f.trace.at + 1} of ${m}`));
    assert.strictEqual(count(h, TOKEN_SPAN), 0);
  }
});

test('reveal with no incorrect votes: "Nobody read it another way."', () => {
  const f = U.reveal[0];
  assert.strictEqual(f.reveal.middle.letter, undefined);
  const h = W.html(f);
  assert.ok(h.includes(`<div class="wall-middle">${PQ.COPY.wall_reveal_nobody_else}</div>`));
});

test('released: the bar line, the link at 40px, a QR of the same link — nothing else', () => {
  const f = F.released;
  const h = W.html(f);
  assert.ok(h.includes(`<h2>${PQ.escapeHtml(PQ.COPY.wall_released_title)}</h2>`));
  assert.ok(h.includes(`<div class="endlink">${f.released.link.replace(/^https?:\/\//, '')}</div>`));
  assert.ok(!h.includes('endsub'), 'no line under the link (PQ-34)');
  assert.ok(h.includes(`<svg class="endqr"`) && h.includes(`aria-label="${f.released.link}"`));
  // the QR in the page is the QR of the whole link
  const expected = PQ.qrSvg(f.released.link, { className: 'endqr' });
  assert.ok(h.includes(expected));
  assert.ok(!h.includes('joinstrip'), 'no join strip on the released wall');
  assert.ok(!h.includes('rn-src'), 'no source on the released wall (AC-99)');
});

// --------------------------------------------------------------------------
// AC-99: colour in the reading phases, none while a trace runs
// --------------------------------------------------------------------------

test('AC-99: token spans in live/closed/split, zero in work/reveal, no source in idle/released', () => {
  for (const [name, f] of ALL) {
    const h = W.html(f);
    const n = count(h, TOKEN_SPAN);
    if (['live', 'closed', 'split'].includes(f.phase)) assert.ok(n > 0, `${name}: no colour`);
    else assert.strictEqual(n, 0, `${name}: ${n} coloured token spans`);
    if (['idle', 'released'].includes(f.phase)) assert.ok(!h.includes('class="rn-src'), `${name}: renders source`);
    else assert.ok(h.includes('class="rn-src'), `${name}: no source`);
  }
});

// --------------------------------------------------------------------------
// AC-40: the ✓ glyph with the colour, never colour alone; no ✗ anywhere
// --------------------------------------------------------------------------

test('AC-40: at reveal the correct bar and the correct option carry the ✓ glyph with the class', () => {
  for (const f of [...F.reveal, ...U.reveal]) {
    const h = W.html(f);
    const c = f.reveal.correct;
    const marked = `<span class="rn-correct letter"><span class="rn-check" aria-hidden="true">✓</span><span class="sr-only">Correct</span> ${c}</span>`;
    assert.strictEqual(count(h, new RegExp(marked.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g')), 2,
      'the correct letter is marked on its bar and on the option chip');
    // the class never appears without the glyph right inside it
    const classes = count(h, /class="rn-correct/g);
    const glyphs = count(h, /class="rn-correct[^"]*"><span class="rn-check" aria-hidden="true">✓<\/span>/g);
    assert.strictEqual(classes, glyphs);
    // only one letter is marked
    for (const other of ['A', 'B', 'C', 'D', 'E'].filter((l) => l !== c)) {
      assert.ok(!h.includes(`<span class="sr-only">Correct</span> ${other}</span>`));
    }
    const text = f.options.find((o) => o.letter === c).text;
    assert.ok(h.includes(`${marked}<span class="otext">${PQ.escapeHtml(text)}</span>`), 'the option text beside the ✓');
  }
  // The green keys off check.js's element (which always holds the glyph) —
  // the stylesheet has no other way to colour a bar or an option correct.
  const css = fs.readFileSync(path.join(WALL, 'wall.css'), 'utf8');
  assert.match(css, /\.bar-row:has\(\.rn-correct\) \.bar-fill \{ background: var\(--success\); \}/);
  assert.ok(!/is-correct|\.correct\b/.test(css), 'a correct-colour class that is not check.js\'s');
});

test('no ✓ anywhere before reveal, and no ✗ in any phase', () => {
  for (const [name, f] of ALL) {
    const h = W.html(f);
    if (f.phase !== 'reveal') assert.ok(!h.includes('✓'), `${name}: carries a ✓`);
    assert.ok(!h.includes('✗') && !h.includes('✘'), `${name}: carries a ✗`);
  }
});

// --------------------------------------------------------------------------
// AC-74: machine fact under the machine marker; no human prose on the wall
// --------------------------------------------------------------------------

test('AC-74: the answer and the receipt sit under the machine provenance marker', () => {
  for (const f of F.reveal) {
    const h = W.html(f);
    const i = h.indexOf('<div class="wall-machine by-machine">');
    assert.ok(i >= 0);
    const machine = h.slice(i);
    assert.ok(machine.includes(`<span class="provenance machine">${f.reveal.receipt.heading}</span>`));
    assert.ok(machine.includes('rn-correct'), 'the answer is in the machine column');
    assert.ok(!h.includes('provenance human') && !h.includes('by-human'),
      'the wall renders no human-reviewed prose, so it carries no human marker');
  }
});

// --------------------------------------------------------------------------
// G-7: the receipt is the one function's lines, printed verbatim
// --------------------------------------------------------------------------

test('G-7: for every receipt fixture, the wall prints exactly the lines the record renders', () => {
  const dir = path.join(REPO, 'bank', 'fixtures', 'receipts');
  const fixtures = fs.readdirSync(dir).filter((n) => n.endsWith('.json'));
  assert.ok(fixtures.length >= 8);
  let rendered = 0;
  for (const name of fixtures) {
    const rec = JSON.parse(fs.readFileSync(path.join(dir, name), 'utf8'));
    if (!Array.isArray(rec.expected_lines)) continue; // a record that renders no receipt cannot be scheduled
    const f = JSON.parse(JSON.stringify(F.reveal[0]));
    f.reveal.receipt.lines = rec.expected_lines;
    const h = W.html(f);
    const got = [...h.matchAll(/<span class="receipt-line">([^<]*)<\/span>/g)].map((m) => m[1]);
    assert.deepStrictEqual(got, rec.expected_lines.map((l) => PQ.escapeHtml(l)), name);
    rendered++;
  }
  assert.ok(rendered >= 7, `only ${rendered} receipt fixtures rendered`);
  // …and the wall has no receipt function of its own: nothing in it spells a
  // receipt line or reads the verified record.
  const src = fs.readFileSync(path.join(WALL, 'wall.js'), 'utf8');
  for (const k of ['receipt_compiled', 'receipt_ran_n_times', 'receipt_miri_clean', 'receipt_nothing_ran',
    'Miri', 'Compiled', 'verified']) {
    assert.ok(!src.includes(k), `wall.js mentions ${k}`);
  }
});

// --------------------------------------------------------------------------
// AC-79: no interactive control, in any phase or in the page
// --------------------------------------------------------------------------

const CONTROLS = ['<button', '<input', '<select', '<textarea', '<a ', '<form', 'onclick', 'onkeydown', 'contenteditable'];

test('AC-79: the wall has no interactive control in any phase', () => {
  for (const [name, f] of ALL) {
    const h = W.html(f);
    for (const c of CONTROLS) assert.ok(!h.includes(c), `${name}: ${c}`);
    // The only focusable thing is the source well's region (well.js), which
    // is a reading affordance, not a control; it is the only tabindex.
    const tabbable = count(h, /tabindex=/g);
    assert.strictEqual(tabbable, f.source === undefined ? 0 : 1, `${name}: tabindex count`);
    if (tabbable) assert.match(h, /class="rn-src-scroll" role="region" aria-label="[^"]*" tabindex="0"/);
  }
  const page = fs.readFileSync(path.join(WALL, 'index.html'), 'utf8');
  for (const c of CONTROLS) assert.ok(!page.includes(c), `index.html: ${c}`);
  assert.ok(!/<script>/.test(page), 'no inline script in the page');
});

// --------------------------------------------------------------------------
// Copy: every visible string is the payload's or copy.js's
// --------------------------------------------------------------------------

// SPEC §11's Forbidden row and the §11.1 trope patterns, for the one string
// the wall authors itself (the brand line, which §11 does not list).
const FORBIDDEN = [/turn to/i, /ask (someone|the person|your neighbou?r)/i, /find someone/i, /volunteer/i,
  /who (said|picked|chose)/i, /\bwrong\b/i, /\bincorrect\b/i, /✗/, /argu/i];
const TROPES = [/\bnot (just|only|merely|simply)\b/i,
  /\b(genuinely|truly|honestly|quietly|extremely|deeply|fundamentally|literally)\b/i,
  /\bworth (a|the|stopping|talking|noting|remembering)\b/i, /\bdon'?t worry\b/i,
  /\bthat'?s (fine|okay|ok|totally fine)\b/i, /\bit'?s (fine|okay|ok|normal) to\b/i];

test('AC-46: Joined in idle, Joined and Answered in live, at the join strip\'s end; neither after', () => {
  const idle = W.html({ ...F.idle, joined: 'Joined: 12' });
  assert.ok(idle.includes('<span class="wall-counts"><span>Joined: 12</span></span></div>'));
  const live = W.html({ ...F.live, joined: 'Joined: 12', answered: 'Answered: 7' });
  assert.ok(live.includes('<span class="wall-counts"><span>Joined: 12</span><span>Answered: 7</span></span></div>'));
  assert.strictEqual(F.idle.joined, PQ.fill(PQ.COPY.count_joined, { n: 0 }), 'the room fills it');
  assert.strictEqual(F.live.answered, PQ.fill(PQ.COPY.count_answered, { n: 0 }));
  for (const p of ['closed', 'split', 'released']) {
    assert.ok(!('joined' in F[p]) && !('answered' in F[p]), `${p}: no counts`);
  }
  for (const f of [...F.work, ...F.reveal]) assert.ok(!('joined' in f) && !('answered' in f), f.phase);
});

test('every visible string on the wall is payload data or a copy.js string', () => {
  const copyPieces = new Set();
  for (const v of Object.values(PQ.COPY)) {
    copyPieces.add(v);
    for (const piece of v.split(/‹[^›]+›/)) if (piece.trim()) copyPieces.add(piece.trim());
  }
  const structural = [/^Step \d+ of \d+( · Pause here\.)?$/, /^\d+ · \d+%$/, /^[A-E]$/, /^\d+$/, /^[→—]$/,
    /^Correct$/ /* check.js's screen-reader word beside the ✓ */];
  for (const [name, f] of ALL) {
    const data = strings(f);
    const lines = new Set();
    for (const s of data) {
      for (const l of s.split('\n')) if (l.trim()) lines.add(l.trim());
      lines.add(s.replace(/^https?:\/\//, ''));
    }
    for (const t of texts(W.html(f))) {
      // A link the payload carries, shown without its scheme.
      const link = [...data].some((s) => s.includes(`http://${t}`) || s.includes(`https://${t}`));
      // Program text, cut into tokens by the syntax colour.
      const program = f.source !== undefined && f.source.includes(t);
      const ok = t === W.BRAND || data.has(t) || lines.has(t) || copyPieces.has(t) || link || program ||
        structural.some((re) => re.test(t));
      assert.ok(ok, `${name}: "${t}" is neither payload nor copy.js`);
    }
  }
});

test('the brand line passes the Forbidden and trope patterns (SPEC §11, §11.1)', () => {
  for (const re of [...FORBIDDEN, ...TROPES]) assert.ok(!re.test(W.BRAND), `${W.BRAND} matches ${re}`);
});

// --------------------------------------------------------------------------
// AC-33 and AC-100: the page never scrolls; the well clips and says so
// --------------------------------------------------------------------------

test('AC-33: the page and the well never scroll; the canvas is scaled, never width:100%', () => {
  const css = fs.readFileSync(path.join(WALL, 'wall.css'), 'utf8');
  assert.match(css, /body\.wall-page \{[^}]*overflow: hidden;/);
  assert.match(css, /\.wall-stage \{[^}]*overflow: hidden;/);
  assert.match(css, /\.wall \.rn-src-scroll \{[^}]*overflow: hidden; \}/);
  assert.match(css, /\.wall-wrap \{[^}]*width: 1120px; height: 630px;/);
  assert.ok(!/width:\s*100%/.test(css.replace(/\/\*[\s\S]*?\*\//g, '')), 'a box sized to 100% width');
  const js = fs.readFileSync(path.join(WALL, 'wall.js'), 'utf8');
  assert.match(js, /wrap\.style\.transform = "scale\(" \+ s \+ "\)"/);
});

test('AC-100: a clipped verdict draws the red edge on the side that lost content', () => {
  const cases = { fits: '', clipped_x: ' clipped-x', clipped_y: ' clipped-y', clipped_xy: ' clipped-x clipped-y' };
  for (const [verdict, cls] of Object.entries(cases)) {
    const h = W.html(F.live, { fontPx: 14.2, fit: verdict });
    assert.ok(h.includes(`<div class="rn-src${cls}">`), verdict);
  }
});

test('the type model sizes the well: the derived size is what the well is drawn at', () => {
  for (const phase of ['live', 'closed', 'split', 'work', 'reveal']) {
    const f = Array.isArray(F[phase]) ? F[phase][0] : F[phase];
    const d = PQ.derivedFontPx(f.source, phase);
    const px = Math.round(d.fontPx * 100) / 100;
    assert.ok(W.html(f, { fontPx: d.fontPx }).includes(`<pre style="font-size:${px}px">`), phase);
  }
  // SPEC §5.2's worked example: q3 at 22.1px in the reading layout.
  assert.strictEqual(Math.round(PQ.derivedFontPx(F.live.source, 'live').fontPx * 10) / 10, 22.1);
});

// --------------------------------------------------------------------------
// Pure, and a phase it does not know is an error
// --------------------------------------------------------------------------

test('html() is pure: same frame, same markup, and no document touched', () => {
  const before = document.body.children.length;
  for (const [, f] of ALL) assert.strictEqual(W.html(f), W.html(JSON.parse(JSON.stringify(f))));
  assert.strictEqual(document.body.children.length, before);
  assert.throws(() => W.html({ phase: 'lights-down' }), /unknown phase/);
});

test('each phase entry announces its §11 live-region line (AC-83)', () => {
  assert.strictEqual(W.announcement(F.idle), null);
  assert.strictEqual(W.announcement(F.live), PQ.COPY.live_question_on_screen);
  assert.strictEqual(W.announcement(F.closed), PQ.COPY.live_answers_closed);
  assert.strictEqual(W.announcement(F.split), PQ.COPY.live_split_on_screen);
  assert.strictEqual(W.announcement(F.work[0]), PQ.COPY.live_walking_through);
  assert.strictEqual(W.announcement(F.reveal[0]), `Revealed: it was ${F.reveal[0].reveal.correct}.`);
  assert.strictEqual(W.announcement(F.released), PQ.COPY.live_released);
});

// --------------------------------------------------------------------------
// The QR (SPEC §5.5)
// --------------------------------------------------------------------------

// Produced by web/wall/qr.js and DECODED by an independent reader (macOS
// CoreImage CIDetector, 2026-09-26) back to exactly this URL — so a change
// here is a change to what phones read, not a refactor.
const GOLDEN_URL = 'https://popquiz.rustnyc.org/last';
const GOLDEN = [
  '#######.#..#####..###.#######',
  '#.....#.#.#.##..#.....#.....#',
  '#.###.#...#.....#.##..#.###.#',
  '#.###.#.####..##.####.#.###.#',
  '#.###.#..#.#..#.#..#..#.###.#',
  '#.....#..#.###.#####..#.....#',
  '#######.#.#.#.#.#.#.#.#######',
  '........##.##..###.#.........',
  '#.##.###.....#.#.##...#..#.##',
  '.##.##.###..####.####.###...#',
  '..##.###....##..#.#.##....##.',
  '##..##..##.#....#.#####.....#',
  '..#.###..##.#.##.#..#..#.##..',
  '##..#..#..##..#.####..#...###',
  '.####.#####.##.###.##.....###',
  '...###.##..##.#.#.##....#..#.',
  '#...#.#.##....##..#..#..##.#.',
  '.##.##.....#..#.#...##.#.###.',
  '#.#######.#.####.#..####..#..',
  '...#.#..###..##.###...#.#.#..',
  '.#..#.###.#.###..#.########..',
  '........##.#....#####...#####',
  '#######.#...#.#.#.###.#.##.#.',
  '#.....#.#.##.####.#.#...##..#',
  '#.###.#..#.....###..#####.##.',
  '#.###.#.###....##......###..#',
  '#.###.#.#..#.#....##...#..#.#',
  '#.....#...##...#..###.#.##.#.',
  '#######.#..#..#...###.##.#.#.',
];

const rows = (q) => Array.from(q.modules, (r) => Array.from(r)).map((r) => r.map((b) => (b ? '#' : '.')).join(''));

test('QR: the golden matrix a real decoder read back', () => {
  const q = plain(PQ.qrMatrix(GOLDEN_URL));
  assert.strictEqual(q.version, 3);
  assert.deepStrictEqual(rows(q), GOLDEN);
});

test('QR: finders, timing, and both copies of the format bits agree with the mask', () => {
  for (const url of [GOLDEN_URL, F.released.released.link, 'http://127.0.0.1:3000/home?x=' + 'y'.repeat(120)]) {
    const q = PQ.qrMatrix(url);
    const n = q.size;
    const d = (x, y) => q.modules[y][x];
    for (const [ox, oy] of [[0, 0], [n - 7, 0], [0, n - 7]]) {
      for (let y = 0; y < 7; y++) for (let x = 0; x < 7; x++) {
        const ring = Math.max(Math.abs(x - 3), Math.abs(y - 3));
        assert.strictEqual(d(ox + x, oy + y), ring !== 2, `finder at ${ox},${oy}`);
      }
    }
    for (let i = 8; i < n - 8; i++) {
      assert.strictEqual(d(i, 6), i % 2 === 0);
      assert.strictEqual(d(6, i), i % 2 === 0);
    }
    const want = PQ._qrFormatBits(q.mask);
    let a = 0, b = 0;
    const bitA = [[8, 0], [8, 1], [8, 2], [8, 3], [8, 4], [8, 5], [8, 7], [8, 8], [7, 8], [5, 8], [4, 8], [3, 8], [2, 8], [1, 8], [0, 8]];
    bitA.forEach(([x, y], i) => { if (d(x, y)) a |= 1 << i; });
    for (let i = 0; i < 8; i++) if (d(n - 1 - i, 8)) b |= 1 << i;
    for (let i = 8; i < 15; i++) if (d(8, n - 15 + i)) b |= 1 << i;
    assert.strictEqual(a, want, `${url}: format bits by the top-left finder`);
    assert.strictEqual(b, want, `${url}: format bits split across the other two`);
    assert.strictEqual(d(8, n - 8), true, 'the dark module');
  }
});

test('QR: a link that does not fit version 10 fails loudly rather than drawing a wrong code', () => {
  assert.throws(() => PQ.qrMatrix('x'.repeat(400)), /do not fit/);
});
