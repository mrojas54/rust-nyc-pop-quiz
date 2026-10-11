// Take it home — web/home/home.js, SPEC §13, T-12.
//
// Every snapshot here comes from web/home/fixtures/take-home.json, which
// room/tests/take_home.rs GENERATES from AppState::take_home() and checks on
// every `just test` — so these are the pages the room serves, and nothing here
// writes down what a program prints. `q3` is a legacy record that ran;
// `complete` and `dnc` are SYNTHETIC records built from q3's prose (the
// fixture's _note says which are which and why).
//
// There is no layout engine here (no headless browser on this machine or in
// CI; justfile). AC-33 is proven in two halves: this suite asserts the rules
// that make it true, and web/home/measure.html measures the rendered page at
// 360 px and 375 px in a real browser (room/README.md, Pages, records the run).

const test = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { load } = require('./_load');

const HOME = path.join(__dirname, '..', 'home');
const FIX = JSON.parse(fs.readFileSync(path.join(HOME, 'fixtures', 'take-home.json'), 'utf8'));
// Comments stripped: the stylesheet's header names what it refuses to do.
const CSS = fs.readFileSync(path.join(HOME, 'home.css'), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
const PAGE = fs.readFileSync(path.join(HOME, 'index.html'), 'utf8');

function loadHome() {
  const env = load('all', { timers: false });
  vm.runInContext(fs.readFileSync(path.join(HOME, 'home.js'), 'utf8'), env.sandbox, { filename: 'web/home/home.js' });
  return env;
}
const { PQ } = loadHome();
const render = (snap, step) => PQ.Home.html(snap, { step });

// Text with the tags stripped and the five entities undone, for reading what a
// person reads.
function text(html) {
  return html.replace(/<[^>]*>/g, '')
    .replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'").replace(/&amp;/g, '&');
}
function headings(html) {
  return [...html.matchAll(/<h([1-3])[^>]*>([\s\S]*?)<\/h\1>/g)].map((m) => text(m[2]));
}
function section(html, cls) {
  const at = html.indexOf('<section class="sec ' + cls);
  assert.ok(at >= 0, `no ${cls} section`);
  const end = html.indexOf('</section>', at);
  return html.slice(at, end);
}

// --------------------------------------------------------------------------
// §11: the headings, in order.
// --------------------------------------------------------------------------

test('§11: the headings are the take-it-home row, in its order, one Why per incorrect option', () => {
  const snap = FIX.q3;
  const why = snap.why.map((w) => PQ.t('home_heading_why', { X: w.letter }) + ': ' + w.text);
  assert.deepStrictEqual(headings(render(snap)), [
    PQ.t('home_heading_question', { date: 'October 14' }),
    PQ.t('home_heading_what'),
    ...why,
    PQ.t('home_heading_remember'),
    PQ.t('home_heading_walk'),
    PQ.t('home_heading_how_we_know'),
  ]);
  assert.strictEqual(snap.meetup_date, '2026-10-14');
  assert.strictEqual(PQ.Home.spokenDate('2026-01-05'), 'January 5');
});

test('before the first release the page says so and carries nothing of a question', () => {
  const html = render(null);
  assert.ok(text(html).includes(PQ.Home.PROPOSED.nothing_yet));
  assert.ok(!html.includes('rn-src') && !html.includes('opt') && !html.includes('receipt'));
});

// --------------------------------------------------------------------------
// AC-95 / D-12 / AC-56: every incorrect option's beat, and no room state.
// --------------------------------------------------------------------------

test('AC-95 on this page: every incorrect option gets its why_tempting, the correct one none', () => {
  for (const name of ['q3', 'complete', 'dnc']) {
    const snap = FIX[name];
    const beats = section(render(snap), 'beats');
    const incorrect = snap.options.filter((o) => !o.correct);
    assert.strictEqual(snap.why.length, 4, name);
    assert.deepStrictEqual(snap.why.map((w) => w.letter), incorrect.map((o) => o.letter), name);
    for (const w of snap.why) {
      assert.ok(text(beats).includes(w.why_tempting), `${name}: ${w.letter}'s why_tempting`);
    }
    const right = snap.options.find((o) => o.correct).letter;
    assert.ok(!text(beats).includes(PQ.t('home_heading_why', { X: right })), `${name}: no beat for the answer`);
    assert.ok(text(beats).includes(snap.what) && text(beats).includes(snap.takeaway), name);
  }
});

test('D-12, AC-56: no count, no split, no most-chosen option anywhere on the page', () => {
  const countShapes = [
    /of us said/i, /Joined:/, /Answered:/, /in the room answered/, /\d+\s*%/, /Nobody read it another way/,
    /Why nobody said anything else/, /bar-fill|bar-n|wall-middle|most.chosen/,
  ];
  for (const name of ['q3', 'complete', 'dnc']) {
    const snap = FIX[name];
    for (let step = 0; step < snap.trace.length; step++) {
      const html = render(snap, step);
      for (const re of countShapes) assert.ok(!re.test(html), `${name} step ${step}: ${re}`);
    }
    for (const key of ['count', 'counts', 'totals', 'split', 'middle', 'most_chosen', 'answered', 'present']) {
      assert.ok(!(key in snap), `${name}: the snapshot has ${key}`);
    }
  }
});

// --------------------------------------------------------------------------
// AC-40, AC-94, AC-74.
// --------------------------------------------------------------------------

test('AC-40: exactly one ✓, on the correct option, glyph and colour together; no ✗', () => {
  for (const name of ['q3', 'complete', 'dnc']) {
    const snap = FIX[name];
    const opts = [...render(snap).matchAll(/<div class="opt">([\s\S]*?)<\/div>/g)].map((m) => m[1]);
    assert.strictEqual(opts.length, 5, name);
    opts.forEach((o, i) => {
      const correct = snap.options[i].correct;
      assert.strictEqual(o.includes('✓'), correct, `${name} ${snap.options[i].letter}`);
      assert.strictEqual(o.includes('rn-correct'), correct, `${name}: the colour class goes with the glyph`);
    });
    assert.ok(!render(snap).includes('\u2717'), 'no ballot x (AC-94, §11 Forbidden)');
  }
  // The green edge keys off check.js's element, so it cannot appear without the ✓.
  assert.match(CSS, /\.opt:has\(\.tick\)\s*\{[^}]*--success/);
  assert.ok(!/\.opt\.correct/.test(CSS));
});

test('AC-74: the beats carry the human provenance markup, the answer and How we know the machine\'s', () => {
  const html = render(FIX.q3);
  assert.match(section(html, 'beats'), /by-human[\s\S]*data-provenance="human"[\s\S]*provenance human/);
  assert.match(section(html, 'how'), /class="receipt by-machine" data-provenance="machine"/);
  assert.match(section(html, 'program'), /class="opts" data-provenance="machine"/);
});

// --------------------------------------------------------------------------
// §13: How we know — AC-87 and AC-71.
// --------------------------------------------------------------------------

function rows(html) {
  const how = section(html, 'how');
  return Object.fromEntries([...how.matchAll(/<div class="row"><span class="k">([^<]*)<\/span><span class="v">([\s\S]*?)<\/span><\/div>/g)]
    .map((m) => [m[1], text(m[2])]));
}

test('How we know: the wall\'s list verbatim, above the machine', () => {
  for (const name of ['q3', 'complete', 'dnc']) {
    const snap = FIX[name];
    const how = section(render(snap), 'how');
    const lines = [...how.matchAll(/<span class="receipt-line">([^<]*)<\/span>/g)].map((m) => text(m[1]));
    assert.deepStrictEqual(lines, snap.receipt.lines, name);
    assert.ok(how.indexOf('receipt-line') < how.indexOf('class="machine"'), name);
  }
  assert.strictEqual(FIX.q3.receipt.lines.length, 4);
  assert.strictEqual(FIX.dnc.receipt.lines.length, 3);
});

test('AC-87, a complete record: the full -Vv, edition, target triple, flags and Miri configuration', () => {
  const m = FIX.complete.machine;
  const r = rows(render(FIX.complete));
  const P = PQ.Home.PROPOSED;
  assert.deepStrictEqual(Object.keys(r), [P.row_compiler, P.row_edition, P.row_target, P.row_flags, P.row_miri]);
  assert.strictEqual(r[P.row_compiler], m.compiler, 'the -Vv, every line');
  assert.ok(m.compiler.split('\n').length > 1);
  assert.strictEqual(r[P.row_edition], m.edition);
  assert.strictEqual(r[P.row_target], m.target);
  assert.strictEqual(r[P.row_flags],
    `-C opt-level=${m.flags.opt_level} -C overflow-checks=on -C debug-assertions=on`);
  assert.ok(r[P.row_miri].includes(m.miri.version));
  for (const c of m.miri.configs) assert.ok(r[P.row_miri].includes(c));
  assert.ok(r[P.row_miri].includes(m.miri.seeds.join(', ')));
  assert.ok(!r[P.row_miri].includes(P.miri_separately));
});

test('AC-87, a legacy record: target not recorded, Miri run separately, nothing back-filled', () => {
  const r = rows(render(FIX.q3));
  const P = PQ.Home.PROPOSED;
  assert.deepStrictEqual(Object.keys(r), [P.row_compiler, P.row_edition, P.row_target, P.row_miri]);
  assert.strictEqual(r[P.row_compiler], FIX.q3.machine.compiler);
  assert.strictEqual(r[P.row_target], PQ.t('home_not_recorded'));
  assert.strictEqual(r[P.row_miri], P.miri_separately);
  assert.ok(!(P.row_flags in r), 'no flags row: the MVP recorded none');
});

test('AC-87, a does-not-compile record: no Miri row, because nothing ran', () => {
  const r = rows(render(FIX.dnc));
  const P = PQ.Home.PROPOSED;
  assert.deepStrictEqual(Object.keys(r), [P.row_compiler, P.row_edition, P.row_target]);
  assert.strictEqual(r[P.row_target], PQ.t('home_not_recorded'));
});

test('AC-71: How we know ends with the machine-checked-the-answer-only sentence', () => {
  for (const name of ['q3', 'complete', 'dnc']) {
    const how = text(section(render(FIX[name]), 'how'));
    assert.ok(how.trim().endsWith(PQ.t('home_machine_only')), name);
  }
});

// --------------------------------------------------------------------------
// The source and the trace.
// --------------------------------------------------------------------------

test('§13: the program has colour; the trace well does not (it signals by highlight-and-dim)', () => {
  const html = render(FIX.q3);
  assert.match(section(html, 'program'), /class="tk-/);
  assert.ok(!/class="tk-/.test(section(html, 'walk')));
  assert.match(section(html, 'walk'), /rn-src-line hl/);
});

test('the trace steps both ways over the whole of it, from the first step, clamped at both ends', () => {
  const snap = FIX.q3;
  const m = snap.trace.length;
  assert.ok(m >= 2);
  const doc = { getElementById: (id) => (id === 'home-walk' ? walk : null) };
  const walk = { innerHTML: '' };
  const el = { innerHTML: '', ownerDocument: doc };
  const page = PQ.Home.mount(el, snap);
  assert.strictEqual(page.at(), 0);
  assert.match(el.innerHTML, new RegExp(`Step 1 of ${m}`));
  assert.strictEqual(page.step(-1), 0, 'no step before the first');
  for (let i = 1; i < m; i++) {
    assert.strictEqual(page.step(1), i);
    assert.match(walk.innerHTML, new RegExp(`Step ${i + 1} of ${m}`));
  }
  assert.strictEqual(page.step(1), m - 1, 'no step after the last');
  // The resolving step is reachable: its values name stdout (§5.3, D-10).
  assert.ok(walk.innerHTML.includes('>stdout<'));
  for (let i = m - 2; i >= 0; i--) assert.strictEqual(page.step(-1), i);
});

test('the trace has buttons both ways and the §11 key hint', () => {
  const walk = section(render(FIX.q3, 1), 'walk');
  assert.match(walk, /onclick="PopQuizHomeStep\(-1\)"/);
  assert.match(walk, /onclick="PopQuizHomeStep\(1\)"/);
  assert.ok(text(walk).includes(PQ.t('static_step_trace')));
});

// --------------------------------------------------------------------------
// AC-33, the rules half: code scrolls in its container, the page never does.
// --------------------------------------------------------------------------

test('AC-33: code wells scroll in their own containers; nothing hides a page overflow', () => {
  const shared = fs.readFileSync(path.join(__dirname, '..', 'shared', 'components.css'), 'utf8');
  assert.match(shared, /\.rn-src-scroll\s*\{[^}]*overflow-x:\s*auto/);
  assert.match(CSS, /\.receipt \.vv\s*\{[^}]*white-space:\s*pre[^}]*overflow-x:\s*auto[^}]*max-width:\s*100%/);
  // Hiding the overflow would pass a scroll check and still cut the program off.
  assert.ok(!/overflow-x:\s*hidden/.test(CSS) && !/overflow:\s*hidden/.test(CSS));
  // Every flex child that holds program text or prose may shrink, and wraps.
  for (const rule of ['.opt .otext', '.receipt .row .v', '.beat p', '.beat h3']) {
    const body = CSS.slice(CSS.indexOf(rule + ' {'));
    assert.ok(CSS.includes(rule + ' {'), rule);
    assert.match(body.slice(0, body.indexOf('}')), /overflow-wrap:\s*anywhere/, rule);
  }
  for (const rule of ['.sheet', '.opt', '.receipt .row', '.opt .otext', '.receipt .row .v', '.walk-body']) {
    const at = CSS.indexOf(rule + ' {');
    assert.ok(at >= 0, rule);
    assert.match(CSS.slice(at, CSS.indexOf('}', at)), /min-width:\s*0/, rule);
  }
  // The -Vv lines and the source render inside those containers.
  const html = render(FIX.complete);
  assert.match(section(html, 'how'), /<pre class="vv"[ >]/);
  assert.match(section(html, 'program'), /class="rn-src-scroll"/);
  assert.match(PAGE, /<meta name="viewport" content="width=device-width, initial-scale=1">/);
});

// --------------------------------------------------------------------------
// Copy: the page authors nothing §11 forbids, and escapes what it prints.
// --------------------------------------------------------------------------

test('§11: the proposed strings match neither the Forbidden row nor the trope check', () => {
  const forbidden = [/turn to/i, /ask (someone|the person|your neighbou?r)/i, /find someone/i, /volunteer/i,
    /who (said|picked|chose)/i, /\bwrong\b/i, /\bincorrect\b/i, /✗/, /argu/i];
  const tropes = [/\bnot (just|only|merely|simply)\b/i, /\b(genuinely|truly|honestly|quietly|extremely|deeply|fundamentally|literally)\b/i,
    /\bworth (a|the|stopping|talking|noting|remembering)\b/i, /\bdon'?t worry\b/i, /\bnothing is missing\b/i];
  for (const [key, s] of Object.entries(PQ.Home.PROPOSED)) {
    for (const re of [...forbidden, ...tropes]) assert.ok(!re.test(s), `${key}: ${re}`);
  }
  for (const name of ['q3', 'complete', 'dnc']) {
    const shown = text(render(FIX[name]).replace(/<pre[\s\S]*?<\/pre>|<code>[\s\S]*?<\/code>/g, ''));
    assert.ok(!/\bwrong\b|\bincorrect\b|✗/i.test(shown), name);
  }
});

test('everything from the snapshot is escaped', () => {
  const snap = JSON.parse(JSON.stringify(FIX.complete));
  const evil = '<img src=x onerror=alert(1)>';
  snap.why[0].why_tempting = evil;
  snap.what = evil;
  snap.options[0].text = evil;
  snap.machine.compiler = evil;
  snap.machine.target = evil;
  snap.receipt.lines[0] = evil;
  snap.trace[0].note = evil;
  const html = render(snap);
  assert.ok(!html.includes('<img'), 'no raw markup from the snapshot');
  assert.ok(html.includes('&lt;img src=x onerror=alert(1)&gt;'));
});

test('the served page has the snapshot slot the room writes into, and loads what it uses', () => {
  assert.ok(PAGE.includes('<script type="application/json" id="take-home">null</script>'));
  for (const src of ['/shared/dom.js', '/shared/phase.js', '/shared/check.js', '/shared/well.js',
    '/shared/trace.js', '/shared/copy.js', '/home/home.js']) {
    assert.ok(PAGE.includes(`<script src="${src}"></script>`), src);
  }
  assert.ok(PAGE.indexOf('<script src="/shared/copy.js">') < PAGE.indexOf('<script src="/home/home.js">'));
});
