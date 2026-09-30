// The a11y suite (T-13): AC-82…AC-86 over every surface and every phase.
//
// `just a11y` runs this file with A11Y_MATRIX=1, which prints what was covered:
// a surface × screen × criterion matrix, then every contrast pair and its
// ratio. `just test` runs it too, silently, through the test-web glob.
//
// Every screen is the surface's own renderer on a fixture: the wall and its
// static fallback on q3's generated frames and bake, the buzzer through its
// reducer, the host through render() and through boot() on scripted socket
// frames, take-it-home on the snapshots in web/home/fixtures. Nothing here runs
// or describes a program; the buzzer's and the host's payloads are made up in
// the shape the room sends and name nothing a program prints.
//
// WHAT THIS CAN PROVE, AND WHAT IT CANNOT. The reader (a11y/dom.js) builds the
// markup into a tree and runs the real stylesheets' cascade over it, so
// colours, font sizes, outlines and min sizes are what the CSS says for that
// element. It does no layout. Rendered sizes, the real Tab order, the ring as
// drawn, the region actually speaking and the OS reduced-motion setting are
// the browser pass's (web/README.md, Accessibility) — each test below says
// which half it is.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { load } = require('./_load.js');
const D = require('./a11y/dom.js');
const W = require('./a11y/wcag.js');

const WEB = path.join(__dirname, '..');
const read = (f) => fs.readFileSync(path.join(WEB, f), 'utf8');
const json = (f) => JSON.parse(read(f));

// --- The four surfaces, as their pages load them ------------------------------

const SURFACES = {
  wall: {
    sheets: ['shared/tokens.css', 'shared/components.css', 'wall/wall.css'],
    chain: [{ tag: 'body', attrs: { class: 'wall-page' } }, { tag: 'div', attrs: { class: 'wall-stage' } },
      { tag: 'div', attrs: { class: 'wall-wrap', id: 'wallWrap' } }],
  },
  buzzer: {
    sheets: ['shared/tokens.css', 'shared/components.css', 'buzzer/buzzer.css'],
    chain: [{ tag: 'body', attrs: {} }, { tag: 'main', attrs: { id: 'buzzer', class: 'buzzer', 'aria-live': 'off' } }],
  },
  host: {
    sheets: ['shared/tokens.css', 'shared/components.css', 'host/host.css'],
    chain: [{ tag: 'body', attrs: {} }, { tag: 'main', attrs: { class: 'host-phone', id: 'host' } }],
  },
  home: {
    sheets: ['shared/tokens.css', 'shared/components.css', 'home/home.css'],
    chain: [{ tag: 'body', attrs: { class: 'home-page' } }, { tag: 'main', attrs: { class: 'home-app', id: 'home' } }],
  },
};
for (const s of Object.values(SURFACES)) s.css = D.sheets(s.sheets);

// The pages link these; the suite reads the same list so the two cannot drift.
test('the suite reads each surface\'s stylesheets in the order its page links them', () => {
  const pages = { wall: 'wall/index.html', buzzer: 'buzzer/index.html', host: 'host/index.html', home: 'home/index.html' };
  for (const [name, file] of Object.entries(pages)) {
    const linked = [...read(file).matchAll(/<link rel="stylesheet" href="\/([^"]+)"/g)]
      .map((m) => m[1].replace(/^join\//, 'buzzer/'))
      .filter((f) => f !== 'shared/fonts.css');   // @font-face only: no selector to cascade
    assert.deepEqual(linked, SURFACES[name].sheets, name);
  }
});

// --- Loading the surfaces' scripts --------------------------------------------

function loadAll(extra, opts = {}) {
  const l = load('all', { timers: false });
  if (opts.noMount) l.sandbox.__POPQUIZ_NO_MOUNT__ = true;
  for (const f of extra) vm.runInContext(read(f), l.sandbox, { filename: `web/${f}` });
  return l;
}

const wallEnv = loadAll(['wall/qr.js', 'wall/wall.js', 'wall/fallback/static.js']);
const PQ = wallEnv.PQ;
const C = PQ.COPY;
const buzzEnv = loadAll(['buzzer/buzzer.js'], { noMount: true });
const B = buzzEnv.PQ.Buzzer;
const hostEnv = loadAll(['host/host.js']);
const H = hostEnv.PQ.host;
const homeEnv = loadAll(['home/home.js']);
const HOME = homeEnv.PQ.Home;

// §11's ten live-region strings, and the host's seven phase labels.
const LIVE = ['live_question_on_screen', 'live_saving', 'live_saved', 'live_save_failed', 'live_answers_closed',
  'live_split_on_screen', 'live_walking_through', 'live_revealed', 'live_released', 'live_hint_shown'];
const LIVE_RE = LIVE.map((k) => new RegExp('^' + C[k].replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/‹[XY]›/, '[A-E]') + '$'));
const isLiveString = (s) => LIVE_RE.some((re) => re.test(s));
const PHASES = ['idle', 'live', 'closed', 'split', 'work', 'reveal', 'released'];

// --- Screens -------------------------------------------------------------------

const WALL_FIX = json('wall/fixtures/q3-phases.json');
const F = WALL_FIX.frames;
const U = WALL_FIX.unanimous.frames;
const BAKE = json('wall/fixtures/q3-static.json');
const HOME_FIX = json('home/fixtures/take-home.json');

function wallScreens() {
  const out = [
    ['idle', F.idle], ['live', F.live], ['closed', F.closed], ['split', F.split],
    ...F.work.map((f, i) => [`work[${i}]`, f]),
    ...F.reveal.map((f, i) => [`reveal[${i}]`, f]),
    ['released', F.released],
    ...U.reveal.map((f, i) => [`reveal[${i}] unanimous`, f]),
  ].map(([name, f]) => ({ name, phase: f.phase, html: PQ.Wall.html(f) }));
  // The static fallback: every {phase, step} its driver can reach.
  const S = PQ.Static;
  let st = S.enter(BAKE, 'idle');
  const seen = new Set();
  const visit = (s) => {
    const key = `${s.phase}[${s.at}]`;
    if (seen.has(key)) return;
    seen.add(key);
    out.push({ name: `static ${key}`, phase: s.phase, html: PQ.Wall.html(S.frame(BAKE, s)) });
  };
  for (;;) {
    visit(st);
    let s = st;
    for (;;) { const n = S.step(BAKE, s, 1); if (n.at === s.at) break; s = n; visit(s); }
    const n = S.next(BAKE, st);
    if (n.phase === st.phase) break;
    st = n;
  }
  return out;
}

const HINT = 'HINT-FIXTURE-TEXT';
const base = { t: 'state', code: 'ABC234', lines: [] };
const BARS = ['A', 'B', 'C', 'D', 'E'].map((letter, i) => ({ letter, count: [3, 1, 7, 0, 2][i], percent: [23, 8, 54, 0, 15][i] }));
const counts = { totals: [3, 1, 7, 0, 2], answered: 13, present: 16 };
const BF = {
  idle: { ...base, revision: 1, phase: 'idle' },
  live: { ...base, revision: 2, phase: 'live', letters: ['A', 'B', 'C', 'D', 'E'], locked: false, hint: { text: HINT, action: C.buzzer_hint_action } },
  closed: { ...base, revision: 3, phase: 'closed', letters: ['A', 'B', 'C', 'D', 'E'], locked: true },
  split: { ...base, revision: 4, phase: 'split', counts, split: BARS },
  work: { ...base, revision: 5, phase: 'work', counts, split: BARS },
  reveal: { ...base, revision: 6, phase: 'reveal', correct: 'E', counts, split: BARS },
  released: { ...base, revision: 7, phase: 'released', mark: '✓' },
};
const attach = (f, saved) => ({ ...f, session: { saved: saved === undefined ? null : saved } });
const JOINED = { type: 'joined', status: 201, body: { room_id: 'room-1', token: 'tok-1', buzzer: BF.idle } };
const IN_ROOM = [{ type: 'boot', search: '?code=ABC234', stored: null }, JOINED];

function drive(events) {
  let state = B.initial();
  const effects = [];
  for (const e of events) { const r = B.reduce(state, e); state = r.state; effects.push(...r.effects); }
  return { state, effects, html: B.view(state) };
}

function buzzerScreens() {
  const s = (name, phase, events) => ({ name, phase, ...drive(events) });
  const live = [...IN_ROOM, { type: 'frame', frame: attach(BF.live) }];
  const refusals = Object.keys(B.REFUSAL_KEY).map((slug) =>
    s(`join refused: ${slug}`, null, [{ type: 'boot', search: '', stored: null }, { type: 'submit', code: 'ABC234' },
      { type: 'joined', status: 404, body: { refusal: slug } }]));
  return [
    s('join', null, [{ type: 'boot', search: '', stored: null }]),
    s('joining', null, [{ type: 'boot', search: '', stored: null }, { type: 'submit', code: 'ABC234' }]),
    ...refusals,
    s('join failed (network)', null, [{ type: 'boot', search: '', stored: null }, { type: 'submit', code: 'X' }, { type: 'joined', status: 0, body: null }]),
    s('attaching', null, IN_ROOM),
    s('idle', 'idle', [...IN_ROOM, { type: 'frame', frame: attach(BF.idle) }]),
    s('live, no answer', 'live', live),
    s('live, saving', 'live', [...live, { type: 'tap', letter: 'C' }]),
    s('live, saved', 'live', [...live, { type: 'tap', letter: 'C' }, { type: 'answered', status: 200, body: { saved: 'C' } }]),
    s('live, failed', 'live', [...live, { type: 'tap', letter: 'C' }, { type: 'answered', status: 500, body: null }]),
    s('live, hint shown', 'live', [...live, { type: 'hint' }]),
    s('live, paused', 'live', [...live, { type: 'tap', letter: 'C' }, { type: 'answered', status: 200, body: { saved: 'C' } }, { type: 'closed', code: 1006 }]),
    s('closed, answered', 'closed', [...IN_ROOM, { type: 'frame', frame: attach(BF.closed, 'B') }]),
    s('closed, no answer', 'closed', [...IN_ROOM, { type: 'frame', frame: attach(BF.closed) }]),
    s('split', 'split', [...IN_ROOM, { type: 'frame', frame: attach(BF.split, 'B') }]),
    s('work', 'work', [...IN_ROOM, { type: 'frame', frame: attach(BF.work, 'B') }]),
    s('reveal, answered', 'reveal', [...IN_ROOM, { type: 'frame', frame: attach(BF.reveal, 'B') }]),
    s('reveal, no answer', 'reveal', [...IN_ROOM, { type: 'frame', frame: attach(BF.reveal) }]),
    s('released', 'released', [...IN_ROOM, { type: 'frame', frame: attach(BF.released) }]),
  ];
}

const NEXT = { idle: 'put-on-screen', live: 'close-answers', closed: 'show-split', split: 'walk-it', work: 'reveal', reveal: 'release', released: 'run-it-again' };
function hostPayload(phase, extra = {}) {
  const p = { phase, code: 'ABC234', primary: { action: NEXT[phase] }, present: 12 };
  if (phase !== 'idle') p.answered = 7;
  if (phase === 'work' || phase === 'reveal') {
    const at = phase === 'work' ? 1 : 5;
    p.step = { at, m: 6, note: `FIXTURE-STEP-NOTE-${at}`, can_back: true, can_forward: phase === 'work' };
  }
  if (phase === 'reveal') {
    p.read_aloud = {
      what: { text: 'FIXTURE-WHAT' },
      middle: { heading: PQ.fill(C.host_reveal_beat_why, { n: 4, X: 'B' }), text: 'FIXTURE-WHY-B' },
      takeaway: { text: 'FIXTURE-TAKEAWAY' },
    };
  }
  return Object.assign(p, extra);
}

function hostScreens() {
  return [
    { name: 'sign in', phase: null, html: H.renderSignIn({ status: null, busy: false }, 'q3') },
    { name: 'create', phase: null, html: H.renderCreate({ status: null, busy: false }) },
    { name: 'create, refused', phase: null, html: H.renderCreate({ status: 'FIXTURE-REASON', busy: false }) },
    ...PHASES.map((ph) => ({ name: ph, phase: ph, html: H.render(hostPayload(ph), { status: null, busy: false }) })),
    { name: 'work, in flight', phase: 'work', html: H.render(hostPayload('work'), { status: null, busy: true }) },
    { name: 'reveal, first step', phase: 'reveal', html: H.render(hostPayload('reveal', { step: { at: 0, m: 6, note: 'N', can_back: false, can_forward: true } }), {}) },
  ];
}

function homeScreens() {
  const out = [{ name: 'nothing released yet', phase: null, html: HOME.html(null) }];
  for (const key of ['q3', 'complete', 'dnc']) {
    const snap = HOME_FIX[key];
    for (let i = 0; i < snap.trace.length; i++) out.push({ name: `${key} step ${i + 1}`, phase: null, html: HOME.html(snap, { step: i }) });
  }
  return out;
}

const SCREENS = { wall: wallScreens(), buzzer: buzzerScreens(), host: hostScreens(), home: homeScreens() };

// --- The matrix ---------------------------------------------------------------

const MATRIX = {};      // surface -> screen -> {AC-82: '…', …}
const PAIRS = [];       // [surface, what, fg, bg, ratio, need, verdict]
function mark(surface, screen, ac, v) {
  ((MATRIX[surface] = MATRIX[surface] || {})[screen] = MATRIX[surface][screen] || {})[ac] = v;
}
function pair(surface, what, fg, bg, need, verdict) {
  const ratio = W.contrast(fg, bg);
  const key = [surface, what, W.hex(fg), W.hex(bg)].join('|');
  if (!PAIRS.some((p) => p.key === key)) PAIRS.push({ key, surface, what, fg: W.hex(fg), bg: W.hex(bg), ratio, need, verdict });
  return ratio;
}

test.after(() => {
  if (process.env.A11Y_MATRIX !== '1') return;
  const ACS = ['AC-82', 'AC-83', 'AC-84', 'AC-85', 'AC-86'];
  const lines = ['', '=== a11y: surface × screen × criterion ===',
    `${'surface'.padEnd(8)} ${'screen'.padEnd(26)} ${ACS.map((a) => a.padEnd(15)).join(' ')}`];
  for (const [surface, screens] of Object.entries(MATRIX)) {
    for (const [screen, row] of Object.entries(screens)) {
      lines.push(`${surface.padEnd(8)} ${screen.slice(0, 26).padEnd(26)} ${ACS.map((a) => String(row[a] || '·').slice(0, 15).padEnd(15)).join(' ')}`);
    }
  }
  lines.push('', '=== a11y: contrast pairs (WCAG 2.1 relative luminance) ===');
  lines.push(`${'surface'.padEnd(8)} ${'what'.padEnd(52)} ${'fg'.padEnd(8)} ${'bg'.padEnd(8)} ${'ratio'.padStart(6)}  need  verdict`);
  for (const p of PAIRS.sort((a, b) => a.surface.localeCompare(b.surface) || a.ratio - b.ratio)) {
    lines.push(`${p.surface.padEnd(8)} ${p.what.slice(0, 52).padEnd(52)} ${p.fg} ${p.bg} ${p.ratio.toFixed(2).padStart(6)}  ${String(p.need).padEnd(4)}  ${p.verdict}`);
  }
  lines.push('', 'Not provable without a browser (web/README.md, Accessibility): rendered target size, the real Tab order and the ring as drawn, the region spoken, the OS reduced-motion setting.', '');
  console.log(lines.join('\n'));
});

// --- AC-82: keyboard ----------------------------------------------------------

// An element a pointer can act on. Our markup's actions are named by these
// attributes (buzzer.js, host.js, trace.js), so an action on a <div> would show
// up here and fail the native-control check below.
function isAction(n) {
  return ['onclick', 'data-letter', 'data-primary', 'data-step', 'data-sign-in'].some((a) => n.getAttribute(a) !== null) ||
    (n.getAttribute('data-act') !== null && n.tag !== 'form');
}
function isControl(n) {
  return n.tag === 'button' || n.tag === 'input' || n.tag === 'select' || n.tag === 'textarea' ||
    (n.tag === 'a' && n.getAttribute('href') !== null) || isAction(n) ||
    (n.getAttribute('tabindex') !== null && n.getAttribute('tabindex') !== '-1');
}
const NATIVE = (n) => ['button', 'input', 'select', 'textarea'].includes(n.tag) || (n.tag === 'a' && n.getAttribute('href') !== null);

function accessibleName(doc, n) {
  if (n.getAttribute('aria-label')) return n.getAttribute('aria-label');
  if (n.id) { const l = doc.root.querySelector(`label[for="${n.id}"]`); if (l) return l.textContent.trim(); }
  return n.textContent.trim();
}

// The ring an element draws on focus, and what it is drawn against.
function ring(surface, n) {
  const css = SURFACES[surface].css;
  const o = W.outline(css, n, W.color(css, n));
  if (!o || o.none || o.width < 2) return { ok: false, why: `no visible :focus-visible outline (${o ? o.selector : 'none'})` };
  const behind = W.background(css, o.offset < 0 ? n : n.parent).color;
  const ratio = pair(surface, `focus ring (${o.selector})`, o.color, behind, 3, W.contrast(o.color, behind) >= 3 ? 'AA' : 'FAIL');
  return { ok: ratio >= 3, ratio, o, behind, why: `${o.selector}: ${W.hex(o.color)} on ${W.hex(behind)} is ${ratio.toFixed(2)}:1` };
}

function ac82(surface) {
  for (const sc of SCREENS[surface]) {
    const doc = D.page(sc.html, SURFACES[surface].chain);
    const all = [...doc.root.walk()];
    const controls = all.filter(isControl);
    for (const n of all) {
      const ti = n.getAttribute('tabindex');
      if (ti !== null) assert.ok(Number(ti) <= 0, `${sc.name}: positive tabindex reorders the page`);
      if (ti === '-1') {
        // Only a landing place for focus a control hands on (buzzer.js), never a control.
        assert.ok(!isAction(n) && !NATIVE(n), `${sc.name}: a control taken out of the Tab order`);
      }
    }
    for (const n of controls) {
      const label = `${sc.name}: <${n.tag} class="${n.attrs.class || ''}">`;
      if (isAction(n)) assert.ok(NATIVE(n), `${label} acts on click but is not a native control, so no key reaches it`);
      assert.ok(accessibleName(doc, n), `${label} has no accessible name`);
      if (!n.disabled) {
        const r = ring(surface, n);
        assert.ok(r.ok, `${label}: ${r.why}`);
      }
    }
    mark(surface, sc.name, 'AC-82', controls.length ? `${controls.length} ctl, ring ok` : 'none');
  }
}

test('ac82_wall: no control on any phase or step (AC-79); the reading region is the one Tab stop, ringed at 3:1', () => {
  ac82('wall');
  for (const sc of SCREENS.wall) {
    const doc = D.page(sc.html, SURFACES.wall.chain);
    const stops = [...doc.root.walk()].filter(isControl);
    const reads = PQ.rendersSource(sc.phase);
    assert.equal(stops.length, reads ? 1 : 0, `${sc.name}: Tab stops`);
    if (reads) {
      assert.ok(stops[0].classes.includes('rn-src-scroll') && stops[0].getAttribute('role') === 'region', sc.name);
      mark('wall', sc.name, 'AC-82', 'region, ring ok');
    }
  }
});

test('ac82_buzzer: every control native, named, in the Tab order, and ringed at 3:1 on every screen', () => ac82('buzzer'));
test('ac82_host: every control native, named, in the Tab order, and ringed at 3:1 on every screen', () => ac82('host'));
test('ac82_home: the program well, the trace buttons and the -Vv block are reachable and ringed at 3:1', () => {
  ac82('home');
  const doc = D.page(HOME.html(HOME_FIX.complete), SURFACES.home.chain);
  const vv = doc.root.querySelector('pre.vv');
  assert.ok(vv && vv.getAttribute('tabindex') === '0', 'the -Vv block scrolls sideways and takes the focus');
});

// The static fallback is driven from the keyboard alone (AC-102): every phase
// and step the room's wall shows is reachable by Space, Esc and the arrows.
test('ac82_wall: the static fallback reaches every phase and every step from the keyboard', () => {
  const S = PQ.Static;
  let st = S.enter(BAKE, 'idle');
  const reached = [st.phase];
  for (let i = 0; i < 10; i++) {
    const n = S.press(BAKE, st, ' ');
    if (n.phase === st.phase) break;
    st = n; reached.push(st.phase);
  }
  assert.deepEqual(reached, PHASES);
  const back = S.press(BAKE, st, 'Escape');
  assert.equal(back.phase, 'reveal');
  const M = BAKE.trace.length;
  let w = S.enter(BAKE, 'work');
  for (let i = 0; i < M + 2; i++) w = S.press(BAKE, w, 'ArrowRight');
  assert.equal(w.at, M - 2, 'work stops one short of the resolving step');
  let r = S.enter(BAKE, 'reveal');
  for (let i = 0; i < M + 2; i++) r = S.press(BAKE, r, 'ArrowLeft');
  assert.equal(r.at, 0, 'reveal steps back through all of it');
});

// --- AC-82: the focus survives a repaint ----------------------------------------
//
// Each surface repaints by innerHTML. The reader's Node behaves as a browser
// does when that happens — the old children leave and a focus among them falls
// back to <body> — so these run the surfaces' own mount/boot code against it.

class FakeSocket {
  constructor(url) { this.url = url; FakeSocket.last = this; this.sent = []; }
  send(m) { this.sent.push(m); }
  close() {}
}
const settle = () => new Promise((r) => setImmediate(r));

function listen(node) {
  node.handlers = {};
  node.addEventListener = (type, fn) => { node.handlers[type] = fn; };
  return node;
}

test('ac82_buzzer: a letter, retry and the hint keep the keyboard where it was across the repaint', async () => {
  const doc = D.page('', SURFACES.buzzer.chain);
  const el = listen(doc.root);
  const said = [];
  const handle = B.mount(el, {
    fetch: () => new Promise(() => {}), WebSocket: FakeSocket, storage: null,
    location: { search: '', protocol: 'http:', host: 'h' }, setTimeout: () => 0, announce: (t) => said.push(t),
  });
  handle.dispatch(JOINED);
  handle.dispatch({ type: 'frame', frame: attach(BF.live) });
  el.querySelector('[data-letter="C"]').focus();
  handle.dispatch({ type: 'tap', letter: 'C' });
  assert.equal(doc.activeElement.getAttribute('data-letter'), 'C', 'after the tap');
  handle.dispatch({ type: 'answered', status: 500, body: null });
  assert.equal(doc.activeElement.getAttribute('data-letter'), 'C', 'after the failure');
  el.querySelector('[data-act="retry"]').focus();
  handle.dispatch({ type: 'retry' });
  assert.equal(doc.activeElement.getAttribute('data-letter'), 'C', 'retry hands the focus to the letter it retries');
  handle.dispatch({ type: 'answered', status: 200, body: { saved: 'C' } });
  el.querySelector('[data-act="hint"]').focus();
  handle.dispatch({ type: 'hint' });
  assert.ok(doc.activeElement.classes.includes('buzz-hint'), 'the hint takes the focus its button had');
  // A reader who moved the focus off the buzzer keeps it there.
  const outside = new D.Node('button', {}, doc.body, doc);
  doc.body.children.push(outside);
  outside.focus();
  handle.dispatch({ type: 'frame', frame: attach(BF.closed, 'C') });
  assert.equal(doc.activeElement, outside);
  mark('buzzer', 'live, saving', 'AC-82', 'focus kept');
});

function hostWindow(pathname) {
  const doc = D.page('', SURFACES.host.chain);
  listen(doc.root);
  const posts = [];
  const win = {
    document: doc,
    location: { pathname, hash: '#FIXTURE-SESSION', search: '', protocol: 'http:', host: 'h', replace() {}, assign() {} },
    history: { replaceState() {} },
    sessionStorage: { getItem: () => null, setItem() {} },
    setTimeout: () => 0,
    WebSocket: FakeSocket,
    fetch: (url, init) => {
      if (init && init.method === 'POST') {
        // The action's own response: the room answers it, then pushes the frame.
        posts.push(url);
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve('{}') });
      }
      const body = JSON.stringify(hostPayload('idle'));
      return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(body) });
    },
  };
  return { win, doc, el: doc.root, posts };
}
const frame = (payload, revision) => ({ data: JSON.stringify({ t: 'state', revision, ...payload }) });
const click = (el, node) => el.handlers.click({ target: node });

test('ac82_host: the primary action and the step buttons keep the keyboard through each phase', async () => {
  const { win, doc, el, posts } = hostWindow('/host/room-1');
  const said = [];
  hostEnv.PQ.announce = (t) => said.push(t);
  H.boot(win);
  await settle();
  const ws = FakeSocket.last;
  let rev = 1;
  ws.onmessage(frame(hostPayload('idle'), rev++));
  for (const phase of PHASES.slice(1)) {
    const primary = el.querySelector('[data-primary]');
    primary.focus();
    click(el, primary);
    assert.equal(doc.activeElement, doc.body, 'the disabled in-flight button cannot hold it');
    await settle();
    ws.onmessage(frame(hostPayload(phase), rev++));
    const now = el.querySelector('[data-primary]');
    assert.equal(doc.activeElement, now, `${phase}: the next action has the focus`);
    assert.equal(now.getAttribute('data-primary'), NEXT[phase]);
  }
  assert.equal(posts.length, 6);
  // Stepping forward onto the bound hands the focus to ←.
  ws.onmessage(frame(hostPayload('work', { step: { at: 3, m: 6, note: 'N', can_back: true, can_forward: true } }), rev++));
  el.querySelector('[data-step="step-forward"]').focus();
  click(el, el.querySelector('[data-step="step-forward"]'));
  await settle();
  ws.onmessage(frame(hostPayload('work', { step: { at: 4, m: 6, note: 'N', can_back: true, can_forward: false } }), rev++));
  assert.equal(doc.activeElement.getAttribute('data-step'), 'step-back');
});

test('ac82_home: a trace button keeps the focus as the walk is redrawn, and hands it over at the end', () => {
  const doc = D.page('', SURFACES.home.chain);
  const snap = HOME_FIX.q3;
  const page = HOME.mount(doc.root, snap);
  const nav = () => doc.getElementById('home-walk').querySelectorAll('.trace-nav button');
  nav()[1].focus();
  page.step(1);
  assert.equal(doc.activeElement, nav()[1], 'next keeps it');
  for (let i = 0; i < snap.trace.length; i++) page.step(1);
  assert.equal(doc.activeElement, nav()[0], 'at the last step, next is disabled and the focus moves to previous');
});

// --- AC-83: the live region -------------------------------------------------------

function fakeWrap() {
  const doc = D.page('', SURFACES.wall.chain);
  const wrap = doc.root;
  wrap.style = {};
  doc.defaultView = { innerWidth: 1120, innerHeight: 630, addEventListener() {} };
  return wrap;
}

function wallRun(frames) {
  const said = [];
  PQ.announce = (t) => { said.push(t); return true; };
  const wall = PQ.Wall.mount(fakeWrap(), {});
  const per = frames.map((f) => { const before = said.length; wall.show(f); return said.slice(before); });
  return { said, per };
}

const WALL_ORDER = [F.idle, F.live, F.closed, F.split, ...F.work, ...F.reveal, F.released];
const expectWall = (correct) => [C.live_question_on_screen, C.live_answers_closed, C.live_split_on_screen,
  C.live_walking_through, PQ.fill(C.live_revealed, { Y: correct }), C.live_released];

test('ac83_wall: each phase entry says its §11 string once; idle and trace steps say nothing', () => {
  const { said, per } = wallRun(WALL_ORDER);
  const correct = F.reveal[0].reveal.correct;
  assert.deepEqual(said, expectWall(correct));
  assert.deepEqual(per[0], [], 'idle');
  WALL_ORDER.forEach((f, i) => {
    const name = f.phase + (per[i].length ? '' : ' (step)');
    mark('wall', f.phase === 'work' || f.phase === 'reveal' ? `${f.phase}[${(f.phase === 'work' ? F.work : F.reveal).indexOf(f)}]` : f.phase,
      'AC-83', per[i].length ? `"${per[i][0].slice(0, 12)}…"` : 'silent');
    void name;
  });
  const u = wallRun([U.idle, U.live, U.closed, U.split, ...U.work, ...U.reveal, U.released]);
  assert.deepEqual(u.said, expectWall(U.reveal[0].reveal.correct));
  assert.ok(said.every(isLiveString));
  // The answer's letter is said at reveal and not before (canary stays green).
  assert.ok(!said.slice(0, 4).some((s) => s.includes(correct)) || !/\b[A-E]\b/.test(said.slice(0, 4).join(' ')));
});

test('ac83_wall: the static fallback, stepped from the keyboard, says exactly what the live wall says', () => {
  const said = [];
  PQ.announce = (t) => { said.push(t); return true; };
  const wrap = fakeWrap();
  let keydown = null;
  const sb = wallEnv.sandbox;
  const doc0 = sb.document;
  sb.document = Object.assign({}, doc0, {
    getElementById: (id) => (id === 'wallWrap' ? wrap : id === 'pq-static' ? { textContent: JSON.stringify(BAKE) } : null),
    addEventListener: (t, fn) => { if (t === 'keydown') keydown = fn; },
  });
  try {
    PQ.Static.boot();
    const key = (k) => keydown({ key: k, preventDefault() {} });
    key(' '); key(' '); key(' '); key(' ');
    key('ArrowRight'); key('ArrowRight'); key('ArrowLeft');
    key(' ');
    key('ArrowLeft');
    key(' ');
    assert.deepEqual(said, expectWall(BAKE.correct));
    mark('wall', 'static (driven)', 'AC-83', '6 strings, =live');
  } finally {
    sb.document = doc0;
  }
});

test('ac83_buzzer: every phase entry, saving, saved, failed and the hint say their §11 string; idle says nothing', () => {
  const run = drive([
    ...IN_ROOM, { type: 'frame', frame: attach(BF.idle) }, { type: 'frame', frame: attach(BF.live) },
    { type: 'tap', letter: 'B' }, { type: 'answered', status: 500, body: null },
    { type: 'retry' }, { type: 'answered', status: 200, body: { saved: 'B' } }, { type: 'hint' },
    { type: 'frame', frame: BF.closed }, { type: 'frame', frame: BF.split }, { type: 'frame', frame: BF.work },
    { type: 'frame', frame: BF.reveal }, { type: 'frame', frame: BF.released },
  ]);
  const said = run.effects.filter((e) => e.do === 'announce').map((e) => e.text);
  assert.deepEqual(said, [
    C.live_question_on_screen, C.live_saving, C.live_save_failed, C.live_saving, PQ.fill(C.live_saved, { X: 'B' }),
    C.live_hint_shown, C.live_answers_closed, C.live_split_on_screen, C.live_walking_through,
    PQ.fill(C.live_revealed, { Y: 'E' }), C.live_released,
  ]);
  assert.ok(said.every(isLiveString), 'nothing but §11 strings');
  // All ten, between the wall's six and this list.
  const covered = new Set(said.map((s) => LIVE.find((k, i) => LIVE_RE[i].test(s))));
  assert.deepEqual([...covered].sort(), [...LIVE].sort());
  const idle = drive([...IN_ROOM, { type: 'frame', frame: attach(BF.idle) }]);
  assert.deepEqual(idle.effects.filter((e) => e.do === 'announce'), [], 'idle');
  for (const sc of SCREENS.buzzer) {
    const t = sc.effects.filter((e) => e.do === 'announce').map((e) => e.text);
    assert.ok(t.every(isLiveString), sc.name);
    mark('buzzer', sc.name, 'AC-83', t.length ? `"${t[t.length - 1].slice(0, 12)}…"` : 'silent');
  }
});

test('ac83_buzzer: the one live region is the polite status PopQuiz.announce makes, outside the repainted <main>', () => {
  const env = load('dom', { timers: false });
  env.PQ.announce(C.live_saving);
  const region = env.document.body.children[0];
  assert.equal(region.getAttribute('role'), 'status');
  assert.equal(region.getAttribute('aria-live'), 'polite');
  assert.equal(region.className, 'sr-only');
  assert.equal(region.textContent, C.live_saving);
  const pageHtml = read('buzzer/index.html');
  assert.match(pageHtml, /<main id="buzzer" class="buzzer" aria-live="off"><\/main>/, 'main is silent: it is repainted whole');
  assert.match(read('shared/components.css'), /\.sr-only\s*\{[^}]*clip:rect/, 'visually hidden, not display:none');
});

test('ac83_buzzer: AC-48 — taking the hint is said on this phone alone, and nothing leaves it', () => {
  const live = [...IN_ROOM, { type: 'frame', frame: attach(BF.live) }];
  const before = drive(live).state;
  const r = B.reduce(before, { type: 'hint' });
  assert.deepEqual(Array.from(r.effects, (e) => ({ ...e })), [{ do: 'announce', text: C.live_hint_shown }]);
  for (const f of ['wall/wall.js', 'wall/fallback/static.js', 'host/host.js', 'home/home.js']) {
    assert.ok(!read(f).includes('live_hint_shown'), `${f} never says the hint`);
  }
});

test('ac83_host: each of the seven phase labels is said, the first on opening the room; a step says nothing', async () => {
  const { win } = hostWindow('/host/room-1');
  const said = [];
  hostEnv.PQ.announce = (t) => said.push(t);
  H.boot(win);
  await settle();
  const ws = FakeSocket.last;
  let rev = 1;
  for (const phase of PHASES.slice(0, -1)) ws.onmessage(frame(hostPayload(phase), rev++));
  ws.onmessage(frame(hostPayload('reveal', { step: { at: 4, m: 6, note: 'N', can_back: true, can_forward: true } }), rev++));
  ws.onmessage(frame(hostPayload('released'), rev++));
  assert.deepEqual(said, PHASES.map((p) => hostEnv.PQ.HOST_PHASE_LABEL[p]));
  PHASES.forEach((p, i) => mark('host', p, 'AC-83', `"${said[i]}"`));
  for (const s of hostScreens().filter((x) => !x.phase)) mark('host', s.name, 'AC-83', 'no phase yet');
});

test('ac83_home: a trace step is said in words; opening the page and a step past the end say nothing', () => {
  const said = [];
  homeEnv.PQ.announce = (t) => said.push(t);
  const doc = D.page('', SURFACES.home.chain);
  const snap = HOME_FIX.q3;
  const page = HOME.mount(doc.root, snap);
  assert.deepEqual(said, [], 'opening');
  page.step(-1);
  assert.deepEqual(said, [], 'before the first step');
  page.step(1);
  assert.equal(said.length, 1);
  assert.match(said[0], /^Step 2 of \d+\. /);
  assert.equal(said[0], homeEnv.PQ.traceSay({ source: snap.source, trace: { steps: snap.trace } }, 1));
  for (const sc of SCREENS.home) mark('home', sc.name, 'AC-83', sc.name.startsWith('nothing') ? 'no trace' : 'step said');
});

// --- AC-84: contrast ----------------------------------------------------------------
//
// Every element with visible text of its own, on every screen: its colour
// (through every opacity above it) against the background showing behind it,
// at its size and weight. Text inside .sr-only is not seen; a disabled control
// is exempt (SC 1.4.3's "inactive user interface component"). Dimmed trace
// lines are printed and flagged, not held (T-13 plan, D3): highlight-and-dim is
// the trace's one signal, and a dim that passed 4.5:1 would not dim.
//
// AC-84's dim-room presentation does not exist in the build (no dark mode in
// v1, DESIGN.md; touchpoint T-16): this proves the one presentation there is.

function textPairs(surface, sc) {
  const css = SURFACES[surface].css;
  const doc = D.page(sc.html, SURFACES[surface].chain);
  const out = [];
  for (const n of doc.root.walk()) {
    if (!n.text.trim()) continue;
    let hidden = false, inactive = false, dim = false;
    for (let x = n; x; x = x.parent) {
      if (x.classes.includes('sr-only')) hidden = true;
      if (x.disabled) inactive = true;
      if (x.classes.includes('dim')) dim = true;
    }
    if (hidden || n.tag === 'script' || n.tag === 'style') continue;
    const bg = W.background(css, n).color;
    const fg = W.blend(W.color(css, n), bg, W.opacity(css, n));
    const px = W.fontPx(css, n);
    const large = W.isLarge(px, W.bold(css, n));
    const need = large ? 3 : 4.5;
    const sel = `${n.tag}${n.classes.length ? '.' + n.classes.join('.') : ''}`;
    out.push({ n, fg, bg, px, large, need, dim, inactive, sel, ratio: W.contrast(fg, bg) });
  }
  return out;
}

function ac84(surface) {
  const flagged = [];
  for (const sc of SCREENS[surface]) {
    let min = Infinity;
    for (const t of textPairs(surface, sc)) {
      const what = `text ${t.sel} ${Math.round(t.px * 10) / 10}px${t.large ? ' large' : ''}`;
      if (t.inactive) { pair(surface, what, t.fg, t.bg, '-', 'exempt: disabled'); continue; }
      if (t.dim) { pair(surface, what + ' (dim)', t.fg, t.bg, '-', 'exempt: trace dim, flagged (D3)'); flagged.push(t); continue; }
      pair(surface, what, t.fg, t.bg, t.need, t.ratio >= t.need ? 'AA' : 'FAIL');
      assert.ok(t.ratio >= t.need,
        `${surface} ${sc.name}: ${what} "${t.n.text.trim().slice(0, 30)}" is ${W.hex(t.fg)} on ${W.hex(t.bg)} = ${t.ratio.toFixed(2)}:1, needs ${t.need}:1`);
      min = Math.min(min, t.ratio);
    }
    mark(surface, sc.name, 'AC-84', Number.isFinite(min) ? `min ${min.toFixed(2)}:1` : 'no text');
  }
  return flagged;
}

// Non-text contrast (SC 1.4.11) the surfaces rely on, from the real cascade.
function edge(surface, html, selector, prop, against, what, supplementary) {
  const css = SURFACES[surface].css;
  const doc = D.page(html, SURFACES[surface].chain);
  const n = doc.root.querySelector(selector);
  assert.ok(n, `${surface}: ${selector} is rendered`);
  let fg;
  if (prop === 'color') fg = W.color(css, n);   // inherited
  else {
    const w = D.winner(css, n, prop);
    assert.ok(w, `${surface}: ${selector} has a ${prop}`);
    fg = W.parseColor(w.value);
  }
  assert.ok(fg, `${surface}: ${selector} ${prop} is a colour`);
  const bgs = against(n, css);
  for (const bg of bgs) {
    const ratio = W.contrast(fg, bg);
    pair(surface, what, fg, bg, supplementary ? '-' : 3, supplementary ? `supplementary: ${supplementary}` : (ratio >= 3 ? 'AA' : 'FAIL'));
    if (!supplementary) assert.ok(ratio >= 3, `${surface}: ${what} is ${W.hex(fg)} on ${W.hex(bg)} = ${ratio.toFixed(2)}:1`);
  }
}
const parentBg = (n, css) => [W.background(css, n.parent).color];
const ownAndParent = (n, css) => [W.background(css, n.parent).color, W.background(css, n).color];

test('ac84_wall: every text on every phase, step and fallback view is AA; the ✓, the highlight edge and the bars', () => {
  const flagged = ac84('wall');
  assert.ok(flagged.length > 0, 'the trace dims something in work');
  const reveal = PQ.Wall.html(F.reveal[0]);
  edge('wall', reveal, '.rn-check', 'color', parentBg, 'the ✓ glyph (correct marker)');
  edge('wall', PQ.Wall.html(F.work[1]), '.rn-src-line.hl', 'border-left-color', ownAndParent, 'trace highlight edge (the line running now)');
  edge('wall', reveal, '.bar-row:has(.rn-correct) .bar-track', 'border', parentBg, 'correct bar edge', 'the ✓ carries it');
  edge('wall', reveal, '.bar-row:has(.rn-correct) .bar-fill', 'background', (n, css) => [W.background(css, n.parent).color], 'correct bar fill on its track', '`n · p%` carries it');
  edge('wall', reveal, '.bar-row:not(:has(.rn-correct)) .bar-fill', 'background', (n, css) => [W.background(css, n.parent).color], 'bar fill on its track', '`n · p%` carries it');
});

test('ac84_buzzer: every text on every screen is AA; the join field\'s edge and the ✓ are 3:1', () => {
  ac84('buzzer');
  edge('buzzer', SCREENS.buzzer[0].html, '.buzz-input', 'border', parentBg, 'join field edge (identifies the field)');
  const reveal = SCREENS.buzzer.find((s) => s.name === 'reveal, answered').html;
  edge('buzzer', reveal, '.rn-check', 'color', parentBg, 'the ✓ glyph (correct marker)');
  const saved = SCREENS.buzzer.find((s) => s.name === 'live, saved').html;
  edge('buzzer', saved, '.buzz[aria-pressed="true"]', 'border-color', parentBg, 'your saved letter\'s edge', 'fill, ✓ and "saved — X" carry it');
});

test('ac84_host: every text on every screen is AA', () => { ac84('host'); });

test('ac84_home: every text at every step is AA; the ✓ and the highlight edge are 3:1', () => {
  ac84('home');
  const html = HOME.html(HOME_FIX.q3, { step: 1 });
  edge('home', html, '.rn-check', 'color', (n, css) => [W.background(css, n).color], 'the ✓ glyph (correct marker)');
  edge('home', html, '.rn-src-line.hl', 'border-left-color', ownAndParent, 'trace highlight edge (the line running now)');
});

// --- AC-85: touch targets ------------------------------------------------------------
//
// The stub proves what the CSS asks for; the rendered box is the browser's.

function px(css, n, prop) {
  const w = D.winner(css, n, prop);
  if (!w) return 0;
  const m = /^([\d.]+)px$/.exec(w.value);
  return m ? +m[1] : 0;
}

function ac85(surface) {
  const css = SURFACES[surface].css;
  const target = px(css, D.page('', SURFACES[surface].chain).body, '--touch-target') ||
    parseFloat(D.resolve(css, 'var(--touch-target)'));
  assert.equal(target, 44);
  for (const sc of SCREENS[surface]) {
    const doc = D.page(sc.html, SURFACES[surface].chain);
    const controls = [...doc.root.walk()].filter((n) => isControl(n) && n.getAttribute('role') !== 'region' && !n.classes.includes('vv'));
    for (const n of controls) {
      const h = px(css, n, 'min-height'), w = px(css, n, 'min-width');
      assert.ok(h >= target && w >= target,
        `${surface} ${sc.name}: <${n.tag} class="${n.attrs.class || ''}"> min ${w}×${h}px, needs ${target}×${target}`);
    }
    mark(surface, sc.name, 'AC-85', controls.length ? `${controls.length} ≥ 44×44` : 'no target');
  }
}

test('ac85_buzzer: every letter, button and the code field ask for at least 44 × 44 px', () => ac85('buzzer'));
test('ac85_host: every action, step button and the sign-in link ask for at least 44 × 44 px', () => ac85('host'));
test('ac85_home: the trace\'s step buttons ask for 44 × 44 px over trace.js\'s inline 34 px', () => ac85('home'));

// --- AC-86: motion ---------------------------------------------------------------------

const ALL_SHEETS = ['shared/tokens.css', 'shared/components.css', 'shared/fonts.css', 'wall/wall.css',
  'buzzer/buzzer.css', 'host/host.css', 'home/home.css'];
const MOTION = /^(animation|transition)(-|$)|^scroll-behavior$/;

test('ac86_*: nothing animates outside a reduce block; each surface carries the reduce guard', () => {
  const raw = ALL_SHEETS.map((f) => [f, read(f)]);
  for (const [f, text] of raw) {
    assert.ok(!/@keyframes/.test(text.replace(/\/\*[\s\S]*?\*\//g, '')), `${f}: @keyframes`);
    const rules = D.parseCss(text, f);
    const moving = rules.filter((r) => !(r.media && /prefers-reduced-motion:\s*reduce/.test(r.media)))
      .flatMap((r) => r.decls.filter((d) => MOTION.test(d.prop) && !/^(none|auto|0s?|0\.01ms)$/.test(d.value)).map((d) => `${r.selector} { ${d.prop} }`));
    const reduce = rules.filter((r) => r.media && /prefers-reduced-motion:\s*reduce/.test(r.media));
    if (moving.length) assert.ok(reduce.length, `${f} moves (${moving.join('; ')}) with no reduce block`);
    assert.deepEqual(moving, [], `${f}: motion outside prefers-reduced-motion`);
    if (/^(wall|buzzer|host|home)\//.test(f)) {
      const stops = reduce.flatMap((r) => r.decls).filter((d) => /^(animation|transition)/.test(d.prop));
      assert.ok(stops.some((d) => /animation/.test(d.prop)) && stops.some((d) => /transition/.test(d.prop)), `${f}: the reduce guard stops both`);
    }
  }
  const scripts = ['shared/dom.js', 'shared/phase.js', 'shared/check.js', 'shared/well.js', 'shared/trace.js',
    'shared/typemodel.js', 'shared/copy.js', 'wall/wall.js', 'wall/qr.js', 'wall/fallback/static.js',
    'buzzer/buzzer.js', 'host/host.js', 'home/home.js'];
  for (const f of scripts) {
    const code = read(f).replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
    assert.ok(!/requestAnimationFrame|setInterval|\.animate\(|behavior:\s*["']smooth/.test(code), `${f} animates from script`);
  }
});

// No state is carried by motion alone: each one's static cue, on the screen that shows it.
const text = (html) => D.decode(html.replace(/<[^>]+>/g, ' ')).replace(/\s+/g, ' ');
const STATIC_CUES = [
  ['buzzer', 'live, saving', (h) => text(h).includes(C.buzzer_saving) && /is-pending/.test(h), '"saving…" and the dashed letter'],
  ['buzzer', 'live, saved', (h) => /aria-pressed="true"/.test(h) && text(h).includes(PQ.fill(C.buzzer_saved, { X: 'C' })), '"saved — C" and the filled letter'],
  ['buzzer', 'live, failed', (h) => text(h).includes("couldn't save.") && /data-act="retry"/.test(h), 'the sentence and Try again'],
  ['buzzer', 'live, hint shown', (h) => h.includes(HINT) && /data-state="hint-shown"/.test(h), 'the hint, in a panel'],
  ['buzzer', 'live, paused', (h) => text(h).includes(C.buzzer_reconnecting.replace('…', '')), '"paused — reconnecting…"'],
  ['buzzer', 'reveal, answered', (h) => /rn-check/.test(h) && text(h).includes('It was'), '✓ It was E.'],
];

test('ac86_buzzer: every state has a static cue; none is motion', () => {
  for (const [surface, name, ok, cue] of STATIC_CUES) {
    const sc = SCREENS[surface].find((s) => s.name === name);
    assert.ok(ok(sc.html), `${name}: ${cue}`);
    mark(surface, name, 'AC-86', 'static cue');
  }
  for (const sc of SCREENS.buzzer) if (!MATRIX.buzzer[sc.name]['AC-86']) mark('buzzer', sc.name, 'AC-86', 'no motion');
});

test('ac86_wall: each phase is its own static frame, and each trace step says its number in words', () => {
  let prev = null;
  for (const sc of SCREENS.wall) {
    assert.match(sc.html, new RegExp(`data-phase="${sc.phase}"`));
    if (sc.phase === 'work' || sc.phase === 'reveal') {
      assert.match(text(sc.html), /Step \d+ of \d+/, `${sc.name}: the step, in words`);
      if (prev && prev.phase === sc.phase) assert.notEqual(sc.html, prev.html, `${sc.name}: a step is a different frame, not a motion`);
    }
    prev = sc;
    mark('wall', sc.name, 'AC-86', sc.phase === 'work' || sc.phase === 'reveal' ? 'Step N of M' : 'static frame');
  }
});

test('ac86_host: each phase is a different primary action; nothing moves', () => {
  const labels = PHASES.map((p) => text(H.render(hostPayload(p), {})));
  PHASES.forEach((p, i) => assert.ok(labels[i].includes(H.actionLabel(NEXT[p])), p));
  for (const sc of SCREENS.host) mark('host', sc.name, 'AC-86', 'static screen');
});

test('ac86_home: each step says its number in words; nothing moves', () => {
  for (const sc of SCREENS.home) {
    if (!sc.name.startsWith('nothing')) assert.match(text(sc.html), /Step \d+ of \d+/, sc.name);
    mark('home', sc.name, 'AC-86', 'static page');
  }
});
