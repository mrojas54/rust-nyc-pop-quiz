// The copy freeze (T-22, G-5): every participant-facing string under web/ is a
// key in web/shared/copy.js and reaches a page only through it — and the proof
// of AC-98's second half and AC-59 over the same rendered screens.
//
// HOW THE FREEZE IS PROVEN. The surfaces are loaded with copy.js rewritten at
// load so that every value is a marker naming its key, `⟦key ‹p›…⟧` (the
// placeholders stay, so fill() works, and the rewrite happens in the source so
// load-time captures such as HOST_PHASE_LABEL are markers too). Every screen
// the a11y suite draws is drawn again: the wall in every phase and step, its
// static fallback at every {phase, step} it can reach, the buzzer through its
// reducer, the host phone, and take-it-home. From each, the text of every
// element and every participant-facing attribute (aria-label, title, alt,
// placeholder, aria-description) is read, every marker is removed, and what
// is left may hold no word with a letter in it that did not come from the
// screen's own data (the question, the room's code, the fixture's notes). A
// word that is left came from a string literal in a surface — the thing G-5
// and §11's "authored here and only here" forbid. Live-region strings get the
// same treatment.
//
// The wall's words come from the room, not from PQ.t: the room fills §11's
// templates in room/src/view.rs from room/src/copy.rs, which twins.rs pins to
// copy.js. So each text field of a wall frame fixture must fully match one of
// copy.js's real templates — which is checked — and is then swapped for the
// same key's marker.
//
// WHAT IT CANNOT SEE. A literal word that also occurs in that screen's own data
// passes (the data vocabulary is per screen, which keeps that small). Glyphs
// with no letter in them (← → ● ↑ ✎ · ✓ —) are not words and are out of scope.
// The room's own strings outside copy.rs (its refusal `reason`s) are data here;
// where one reaches a page is the allowlist below.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { load, SHARED } = require('./_load.js');
const D = require('./a11y/dom.js');

const WEB = path.join(__dirname, '..');
const read = (f) => fs.readFileSync(path.join(WEB, f), 'utf8');
const json = (f) => JSON.parse(read(f));

const REAL = load('copy').PQ;

// --- The marker copy module ------------------------------------------------------

const MARKER = /⟦([a-z0-9_]+)[^⟦⟧]*⟧/g;

function markerCopySource() {
  const src = fs.readFileSync(path.join(SHARED, 'copy.js'), 'utf8');
  const a = src.indexOf('var COPY = {');
  const b = src.indexOf('var COPY_ROWS');
  let n = 0;
  const body = src.slice(a, b).replace(/^(\s+)([a-z0-9_]+): "([^"]*)"(,?)$/gm, (_, ind, key, val, comma) => {
    n++;
    const ph = [...val.matchAll(/‹[^›]+›/g)].map((m) => m[0]);
    return `${ind}${key}: ${JSON.stringify('⟦' + [key, ...ph].join(' ') + '⟧')}${comma}`;
  });
  assert.equal(n, Object.keys(REAL.COPY).length, 'the marker rewrite missed a copy.js entry');
  return src.slice(0, a) + body + src.slice(b);
}
const MARKED_COPY = markerCopySource();

function env(extra, opts = {}) {
  const l = load('dom', { timers: false });
  const run = (rel, code) => vm.runInContext(code === undefined ? read(rel) : code, l.sandbox, { filename: `web/${rel}` });
  for (const f of ['phase.js', 'check.js', 'well.js', 'trace.js', 'typemodel.js']) run(`shared/${f}`);
  run('shared/copy.js', MARKED_COPY);
  if (opts.noMount) l.sandbox.__POPQUIZ_NO_MOUNT__ = true;
  for (const f of extra) run(f);
  return l.PQ;
}

const WALL = env(['wall/qr.js', 'wall/wall.js', 'wall/fallback/static.js']);
const BUZZ = env(['buzzer/buzzer.js'], { noMount: true });
const HOST = env(['host/host.js']);
const HOME = env(['home/home.js']);
const M = WALL.COPY;   // the marker values, the same in every env

// --- The room's filled strings, back to their keys --------------------------------

// Fields of a wall frame or a take-it-home snapshot that carry §11 strings the
// room filled. Everything else in a frame is data.
const COPY_FIELDS = new Set(['title', 'strip', 'join', 'joined', 'answered', 'well_header', 'line', 'heading', 'label', 'pause', 'lines']);

const TEMPLATES = Object.entries(REAL.COPY)
  .filter(([, v]) => v.replace(/‹[^›]+›/g, '').trim() !== '')      // `‹link›` alone would match anything
  .map(([key, v]) => {
    const names = [...v.matchAll(/‹([^›]+)›/g)].map((m) => m[1]);
    const re = new RegExp('^' + v.split(/‹[^›]+›/).map((s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('(.+?)') + '$');
    return { key, re, names };
  });

function toMarker(s, where) {
  for (const t of TEMPLATES) {
    const m = t.re.exec(s);
    if (m) return WALL.fill(M[t.key], Object.fromEntries(t.names.map((n, i) => [n, m[i + 1]])));
  }
  assert.fail(`${where}: the room sent ${JSON.stringify(s)}, which is no §11 template`);
}

function marked(obj, where = '') {
  if (Array.isArray(obj)) return obj.map((x, i) => marked(x, `${where}[${i}]`));
  if (!obj || typeof obj !== 'object') return obj;
  const out = {};
  for (const [k, v] of Object.entries(obj)) {
    if (COPY_FIELDS.has(k) && typeof v === 'string') out[k] = toMarker(v, `${where}.${k}`);
    else if (k === 'lines' && Array.isArray(v) && v.every((x) => typeof x === 'string')) out[k] = v.map((s, i) => toMarker(s, `${where}.lines[${i}]`));
    else out[k] = marked(v, `${where}.${k}`);
  }
  return out;
}

// --- Words ------------------------------------------------------------------------

const WORD = /\p{L}[\p{L}\p{M}]*/gu;
const words = (s) => String(s).replace(MARKER, ' ').match(WORD) || [];
// Keys whose values drive rendering but are never shown as words.
const NOT_DATA = new Set(['phase', 't', 'type', 'do', 'refusal', 'search', 'primary', 'action', 'colour', 'mark', 'status']);

function vocabulary(...inputs) {
  const v = new Set();
  const walk = (o, k) => {
    if (NOT_DATA.has(k)) return;
    if (typeof o === 'string') { for (const w of words(o)) v.add(w); return; }
    if (Array.isArray(o)) { o.forEach((x) => walk(x)); return; }
    if (o && typeof o === 'object') for (const [kk, vv] of Object.entries(o)) walk(vv, kk);
  };
  inputs.forEach((i) => walk(i));
  return v;
}

const ATTRS = ['aria-label', 'title', 'alt', 'placeholder', 'aria-description', 'aria-valuetext',
  'aria-roledescription', 'aria-placeholder'];

// Every piece of text a screen shows or names: [{where, text, node}].
function texts(html) {
  const root = D.parseInto(new D.Node('div', {}), html);
  const out = [];
  for (const n of [root, ...root.walk()]) {
    if (n.text.trim()) out.push({ where: n.tag + (n.classes.length ? '.' + n.classes.join('.') : ''), text: n.text, node: n });
    for (const a of ATTRS) if (n.attrs[a]) out.push({ where: `${n.tag}[${a}]`, text: n.attrs[a], node: n });
  }
  return out;
}

// --- The screens ------------------------------------------------------------------

const WALL_FIX = json('wall/fixtures/q3-phases.json');
const BAKE = json('wall/fixtures/q3-static.json');
const HOME_FIX = json('home/fixtures/take-home.json');

function wallScreens() {
  const F = WALL_FIX.frames;
  const U = WALL_FIX.unanimous.frames;
  const raw = [
    ['idle', F.idle], ['live', F.live], ['closed', F.closed], ['split', F.split],
    ...F.work.map((f, i) => [`work[${i}]`, f]),
    ...F.reveal.map((f, i) => [`reveal[${i}]`, f]),
    ['released', F.released],
    ...U.reveal.map((f, i) => [`reveal[${i}] unanimous`, f]),
  ];
  const out = raw.map(([name, f]) => {
    const frame = marked(f, name);
    return { surface: 'wall', name, html: WALL.Wall.html(frame), vocab: vocabulary(frame), said: [WALL.Wall.announcement(frame)] };
  });
  const S = WALL.Static;
  const bake = marked(BAKE, 'bake');
  const vocab = vocabulary(bake);
  const seen = new Set();
  let st = S.enter(bake, 'idle');
  for (;;) {
    let s = st;
    for (;;) {
      const key = `${s.phase}[${s.at}]`;
      if (!seen.has(key)) {
        seen.add(key);
        const frame = S.frame(bake, s);
        out.push({ surface: 'wall', name: `static ${key}`, html: WALL.Wall.html(frame), vocab, said: [WALL.Wall.announcement(frame)] });
      }
      const n = S.step(bake, s, 1);
      if (n.at === s.at) break;
      s = n;
    }
    const n = S.next(bake, st);
    if (n.phase === st.phase) break;
    st = n;
  }
  return out;
}

const B = BUZZ.Buzzer;
const LETTERS = ['A', 'B', 'C', 'D', 'E'];
const BARS = LETTERS.map((letter, i) => ({ letter, count: [3, 1, 7, 0, 2][i], percent: [23, 8, 54, 0, 15][i] }));
const counts = { totals: [3, 1, 7, 0, 2], answered: 13, present: 16 };
const base = { t: 'state', code: 'ABC234', lines: [] };
const HINT = 'Hinted fixture prose.';
const BF = {
  idle: { ...base, revision: 1, phase: 'idle' },
  live: { ...base, revision: 2, phase: 'live', letters: LETTERS, locked: false, hint: { text: HINT, action: M.buzzer_hint_action } },
  closed: { ...base, revision: 3, phase: 'closed', letters: LETTERS, locked: true },
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
  const said = [];
  for (const e of events) {
    const r = B.reduce(state, e);
    state = r.state;
    for (const fx of r.effects) if (fx.do === 'announce') said.push(fx.text);
  }
  return { state, said, html: B.view(state) };
}

function buzzerScreens() {
  const s = (name, events) => {
    const d = drive(events);
    return { surface: 'buzzer', name, html: d.html, said: d.said, state: d.state, vocab: vocabulary(events) };
  };
  const live = [...IN_ROOM, { type: 'frame', frame: attach(BF.live) }];
  const saved = [...live, { type: 'tap', letter: 'C' }, { type: 'answered', status: 200, body: { saved: 'C' } }];
  const join = { type: 'boot', search: '', stored: null };
  return [
    s('join', [join]),
    s('joining', [join, { type: 'submit', code: 'ABC234' }]),
    ...Object.keys(B.REFUSAL_KEY).map((slug) => s(`join refused: ${slug}`,
      [join, { type: 'submit', code: 'ABC234' }, { type: 'joined', status: 404, body: { refusal: slug } }])),
    s('join failed (network)', [join, { type: 'submit', code: 'X' }, { type: 'joined', status: 0, body: null }]),
    s('attaching', IN_ROOM),
    s('idle', [...IN_ROOM, { type: 'frame', frame: attach(BF.idle) }]),
    s('live, no answer', live),
    s('live, saving', [...live, { type: 'tap', letter: 'C' }]),
    s('live, saved', saved),
    s('live, failed', [...live, { type: 'tap', letter: 'C' }, { type: 'answered', status: 500, body: null }]),
    s('live, hint shown', [...live, { type: 'hint' }]),
    s('live, paused', [...saved, { type: 'closed', code: 1006 }]),
    s('closed, answered', [...IN_ROOM, { type: 'frame', frame: attach(BF.closed, 'B') }]),
    s('closed, no answer', [...IN_ROOM, { type: 'frame', frame: attach(BF.closed) }]),
    s('split', [...IN_ROOM, { type: 'frame', frame: attach(BF.split, 'B') }]),
    s('work', [...IN_ROOM, { type: 'frame', frame: attach(BF.work, 'B') }]),
    s('reveal, answered', [...IN_ROOM, { type: 'frame', frame: attach(BF.reveal, 'B') }]),
    s('reveal, no answer', [...IN_ROOM, { type: 'frame', frame: attach(BF.reveal) }]),
    s('released', [...IN_ROOM, { type: 'frame', frame: attach(BF.released, 'B') }]),
  ];
}

const H = HOST.host;
const PHASES = ['idle', 'live', 'closed', 'split', 'work', 'reveal', 'released'];
const NEXT = { idle: 'put-on-screen', live: 'close-answers', closed: 'show-split', split: 'walk-it', work: 'reveal', reveal: 'release', released: 'run-it-again' };
function hostPayload(phase, extra = {}) {
  const p = { phase, code: 'ABC234', primary: { action: NEXT[phase] }, present: 12 };
  if (phase !== 'idle') p.answered = 7;
  if (phase === 'work' || phase === 'reveal') {
    const at = phase === 'work' ? 1 : 5;
    p.step = { at, m: 6, note: `Fixture step note ${at}.`, can_back: true, can_forward: phase === 'work' };
  }
  if (phase === 'reveal') {
    p.read_aloud = {
      what: { text: 'Fixture what beat.' },
      middle: { heading: HOST.fill(M.host_reveal_beat_why, { n: 4, X: 'B' }), text: 'Fixture why beat.' },
      takeaway: { text: 'Fixture takeaway beat.' },
    };
  }
  return Object.assign(p, extra);
}

// The host's status line shows what the room or the network said, verbatim.
const REASON = 'Sentinel refusal reason text';

function hostScreens() {
  const h = (name, html, payload) => ({ surface: 'host', name, html, said: [], vocab: vocabulary(payload || {}) });
  return [
    h('sign in', H.renderSignIn({ status: null, busy: false }, 'q3')),
    h('create', H.renderCreate({ status: null, busy: false })),
    h('create, refused', H.renderCreate({ status: REASON, busy: false })),
    ...PHASES.map((ph) => h(ph, H.render(hostPayload(ph), { status: null, busy: false }), hostPayload(ph))),
    h('live, refused', H.render(hostPayload('live'), { status: REASON, busy: false }), hostPayload('live')),
    h('work, in flight', H.render(hostPayload('work'), { status: null, busy: true }), hostPayload('work')),
    h('reveal, first step', H.render(hostPayload('reveal', { step: { at: 0, m: 6, note: 'N', can_back: false, can_forward: true } }), {}),
      hostPayload('reveal', { step: { note: 'N' } })),
  ];
}

function homeScreens() {
  const out = [{ surface: 'home', name: 'nothing released yet', html: HOME.Home.html(null), said: [], vocab: new Set() }];
  for (const key of ['q3', 'complete', 'dnc']) {
    const snap = marked(HOME_FIX[key], `home ${key}`);
    const vocab = vocabulary(snap);
    const q = { source: snap.source, trace: { steps: snap.trace } };
    for (let i = 0; i < snap.trace.length; i++) {
      out.push({ surface: 'home', name: `${key} step ${i + 1}`, html: HOME.Home.html(snap, { step: i }), vocab, said: [HOME.traceSay(q, i)], snap });
    }
  }
  return out;
}

const SCREENS = [...wallScreens(), ...buzzerScreens(), ...hostScreens(), ...homeScreens()];

// --- The allowlist ----------------------------------------------------------------

// Where a string that is not a copy key may reach a page, entry by entry, each
// with its reason. Empty but for one finding, which is not T-22's to fix.
const ALLOWLIST = [
  {
    surface: 'host', where: 'p.host-status.meta',
    why: 'QUESTION Q2 (PQ-27): host.js statusLine shows the room\'s `reason` verbatim, or the HTTP status text, '
      + 'an exception message or a close code. Two reasons are §11 keys (the denials); others are room diagnostics '
      + 'not in §11. Organizer-facing; changing it is a behaviour change and a string to author.',
  },
  {
    surface: 'home', where: 'span.v', words: ['opt', 'level', 'overflow', 'checks', 'debug', 'assertions', 'on', 'off'],
    why: 'home.js flagsText: the machine row spells the record\'s three flags as rustc takes them '
      + '(`-C opt-level=0 -C overflow-checks=on …`, §13, AC-87). Compiler syntax, not prose; these words only.',
  },
];
const allowed = (screen, t, w) => ALLOWLIST.some((a) => a.surface === screen.surface && a.where === t.where
  && (!a.words || a.words.includes(w)));

test.after(() => {
  console.log(`\ncopy freeze: ${SCREENS.length} screens, allowlist ${ALLOWLIST.length} entr${ALLOWLIST.length === 1 ? 'y' : 'ies'}:`);
  for (const a of ALLOWLIST) console.log(`  ${a.surface} ${a.where} — ${a.why}`);
});

// --- The freeze -------------------------------------------------------------------

test('every screen was drawn: each surface, each phase', () => {
  // Exact, so a screen that drops out of the set is loud rather than quiet.
  const by = (s) => SCREENS.filter((x) => x.surface === s).length;
  assert.deepEqual({ wall: by('wall'), buzzer: by('buzzer'), host: by('host'), home: by('home') },
    { wall: 33, buzzer: 24, host: 13, home: 19 });
  // And each surface reaches every phase it has.
  const wallPhases = new Set(SCREENS.filter((x) => x.surface === 'wall').map((x) => x.name.replace(/^static /, '').replace(/[[ ].*$/, '')));
  for (const ph of PHASES) assert.ok(wallPhases.has(ph), `wall never drew ${ph}`);
  for (const ph of PHASES) {
    assert.ok(SCREENS.some((x) => x.surface === 'host' && x.name === ph), `host never drew ${ph}`);
    assert.ok(SCREENS.some((x) => x.surface === 'buzzer' && x.name.startsWith(ph)), `buzzer never drew ${ph}`);
  }
});

test('no screen shows a word that is neither a copy key nor its own data (G-5, §11)', () => {
  const leaks = [];
  for (const s of SCREENS) {
    for (const t of texts(s.html)) {
      for (const w of words(t.text)) if (!s.vocab.has(w) && !allowed(s, t, w)) leaks.push(`${s.surface} / ${s.name} / ${t.where}: "${w}" in ${JSON.stringify(t.text)}`);
    }
  }
  assert.deepEqual([...new Set(leaks)], []);
});

test('every live-region string is a copy key, filled with data (AC-83)', () => {
  const leaks = [];
  let n = 0;
  for (const s of SCREENS) {
    for (const said of s.said.filter(Boolean)) {
      n++;
      assert.ok(MARKER.test(said), `${s.name}: announced ${JSON.stringify(said)} through no copy key`);
      MARKER.lastIndex = 0;
      for (const w of words(said)) if (!s.vocab.has(w)) leaks.push(`${s.surface} / ${s.name}: "${w}" in ${JSON.stringify(said)}`);
    }
  }
  for (const [phase, label] of Object.entries(HOST.HOST_PHASE_LABEL)) {
    n++;
    assert.match(label, /^⟦host_phase_[a-z]+⟧$/, phase);
  }
  assert.ok(n > 40, `only ${n} announcements checked`);
  assert.deepEqual(leaks, []);
});

test('the allowlisted host status line is the one place a room reason shows, and it does show there', () => {
  // Pinning the finding: if a second path appears, or this one closes, this
  // test says so and the allowlist has to change with it.
  for (const s of SCREENS) {
    const where = texts(s.html).filter((t) => t.text.includes(REASON)).map((t) => t.where);
    if (s.surface === 'host' && /refused/.test(s.name)) assert.deepEqual(where, ['p.host-status.meta'], s.name);
    else assert.deepEqual(where, [], `${s.surface} / ${s.name}`);
  }
});

test('the page shells\' own text is §11\'s title row, composed from the keys', () => {
  // Static HTML cannot call PQ.t. SPEC §11's Title row: `Rust NYC Pop Quiz · Host`
  // / `… · Guest` / `Rust NYC Pop Quiz` (wall, and take-it-home, which is not a
  // phone surface). measure.html and test/a11y/browser.html are dev harnesses,
  // never served to a participant.
  const C = REAL.COPY;
  const want = {
    'wall/index.html': C.title_wordmark,
    'buzzer/index.html': `${C.title_wordmark} · ${C.buzzer_title_role}`,
    'host/index.html': `${C.title_wordmark} · ${C.host_title_role}`,
    'home/index.html': C.title_wordmark,
  };
  for (const [file, title] of Object.entries(want)) {
    const html = read(file);
    assert.equal(/<title>([^<]*)<\/title>/.exec(html)[1], title, file);
    const body = html.slice(html.indexOf('<body')).replace(/<script[\s\S]*?<\/script>/g, '').replace(/<!--[\s\S]*?-->/g, '');
    const doc = D.parseInto(new D.Node('div', {}), body);
    const stray = [doc, ...doc.walk()].flatMap((n) => [n.text, ...ATTRS.map((a) => n.attrs[a] || '')]).filter((t) => WORD.test(t));
    WORD.lastIndex = 0;
    assert.deepEqual(stray, [], `${file} has literal text in its body`);
  }
});

test('no surface retypes a §11 string as a literal of its own', () => {
  // The render test cannot see a literal on a path no fixture reaches; this
  // reads the source. Multi-word values only: a single word ("Host", "Start")
  // is also an identifier's worth of text and is left to the render test.
  const values = new Set(Object.values(REAL.COPY).filter((v) => (v.match(WORD) || []).length >= 2));
  const files = [...fs.readdirSync(SHARED).filter((f) => f.endsWith('.js') && f !== 'copy.js').map((f) => `shared/${f}`),
    'wall/wall.js', 'wall/fallback/static.js', 'buzzer/buzzer.js', 'host/host.js', 'home/home.js'];
  const found = [];
  for (const f of files) {
    const code = read(f).replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
    for (const m of code.matchAll(/"((?:[^"\\\n]|\\.)*)"|'((?:[^'\\\n]|\\.)*)'/g)) {
      const lit = m[1] !== undefined ? m[1] : m[2];
      if (values.has(lit)) found.push(`${f}: ${JSON.stringify(lit)}`);
    }
  }
  assert.deepEqual(found, []);
});

// --- AC-98: nothing asks for a contribution ---------------------------------------

const keysIn = (s) => [...String(s).matchAll(MARKER)].map((m) => m[1]);
const CONTROL_KEYS = new Set([...Object.keys(REAL.COPY).filter((k) => k.startsWith('host_action_')),
  'buzzer_join_button', 'buzzer_hint_action', 'buzzer_save_retry', 'proposed_trace_previous_step', 'proposed_trace_next_step']);

test('AC-98: every control is an action, an answer letter or a step — none asks anyone to speak', () => {
  // Controls are what a person can press. Each is named by a copy key from
  // the host's actions, the join, the hint, the retry or the trace's steps, or
  // it is an answer letter A–E or a step arrow. The code field is labelled by
  // its <label>, room code.
  const seen = new Set();
  for (const s of SCREENS) {
    const root = D.parseInto(new D.Node('div', {}), s.html);
    for (const n of root.walk()) {
      if (!['button', 'a', 'input', 'select', 'textarea'].includes(n.tag) && n.attrs.role !== 'button') continue;
      const label = n.attrs['aria-label'] || n.textContent;
      const keys = keysIn(label);
      const bare = label.replace(MARKER, '').trim();
      if (n.tag === 'input') { assert.equal(n.attrs.id, 'pq-code', `${s.name}: an unlabelled input`); continue; }
      if (keys.length) {
        for (const k of keys) { assert.ok(CONTROL_KEYS.has(k), `${s.surface} / ${s.name}: a control keyed ${k}`); seen.add(k); }
      } else {
        assert.match(bare, /^([A-E]|[←→])$/, `${s.surface} / ${s.name}: a control labelled ${JSON.stringify(label)}`);
      }
    }
  }
  for (const k of ['host_action_create', 'host_action_sign_in', 'buzzer_join_button', 'buzzer_hint_action', 'buzzer_save_retry',
    'proposed_trace_previous_step']) assert.ok(seen.has(k), `no screen drew ${k}`);
});

test('AC-98: no string in the copy module asks anyone to speak, compare or volunteer', () => {
  // The Forbidden row (copylint.test.js) is the normative lint; this is the
  // criterion's own words, as a second net over the real strings.
  // AC-98's own verbs, aimed at a person: speak, compare, volunteer, be called
  // on, turn to someone. (The screen may "say so"; a person is never asked to.)
  const ask = /\b(speak|talk|tell (us|someone|the room|your)|share (your|with)|discuss|compare|volunteer|ask (someone|the|your)|turn to|raise your hand|neighbou?rs?|partner|call(ed)? on)\b/i;
  const hits = Object.entries(REAL.COPY).filter(([, v]) => ask.test(v)).map(([k, v]) => `${k}: ${v}`);
  assert.deepEqual(hits, []);
});

test('AC-98: the host is shown totals, never who — even when the payload carries names', () => {
  // The room's host payload carries `present` and `answered` only (view.rs).
  // If it ever carried people, the phone still would not show them.
  const who = { who: ['Personalpha'], names: ['Personbeta'], answers: { Persongamma: 'B' }, silent: ['Persondelta'] };
  for (const ph of PHASES) {
    const html = H.render(hostPayload(ph, who), { status: null, busy: false });
    assert.ok(!/Person(alpha|beta|gamma|delta)/.test(html), ph);
    const shown = keysIn(texts(html).map((t) => t.text).join(' '));
    const counts = shown.filter((k) => /^count_/.test(k));
    assert.deepEqual(counts, [ph === 'idle' ? 'count_joined' : 'count_answered'], ph);
  }
});

test('AC-98: nothing waits for a contribution — the host moves on with nobody in the room', () => {
  for (const ph of PHASES) {
    const html = H.render(hostPayload(ph, { present: 0, answered: 0 }), { status: null, busy: false });
    const root = D.parseInto(new D.Node('div', {}), html);
    const primary = root.querySelector('[data-primary]');
    assert.ok(primary && !primary.disabled, `${ph}: the host's next action waits`);
  }
});

test('AC-98: only the participant\'s own phone ever says they did not answer', () => {
  const NO_ANSWER = ['buzzer_noanswer_count_one', 'buzzer_closed_you_didnt'];
  for (const s of SCREENS) {
    if (s.surface === 'buzzer') continue;
    const keys = keysIn(texts(s.html).map((t) => t.text).join(' '));
    assert.deepEqual(keys.filter((k) => NO_ANSWER.includes(k)), [], `${s.surface} / ${s.name}`);
  }
});

test('AC-98: take-it-home carries no count', () => {
  const COUNTS = ['count_joined', 'count_answered', 'wall_split_answered', 'wall_reveal_most_chosen', 'host_reveal_beat_why'];
  for (const s of SCREENS.filter((x) => x.surface === 'home')) {
    const keys = keysIn(texts(s.html).map((t) => t.text).join(' '));
    assert.deepEqual(keys.filter((k) => COUNTS.includes(k)), [], s.name);
  }
  for (const [name, snap] of Object.entries(HOME_FIX)) {
    if (name.startsWith('_')) continue;
    const keys = JSON.stringify(Object.keys(snap));
    assert.ok(!/count|answered|present|split|vote|chosen|totals/i.test(keys), `${name}: ${keys}`);
  }
});

// --- AC-59: what the released buzzer serves ---------------------------------------

test('AC-59: the released buzzer says the room is released and shows nothing about anyone', () => {
  // The line SPEC's brief names for release — "Nothing about you was
  // recorded." — was cut by the client at HC-0 (PQ-34; copy.test.js pins its
  // absence) and §11 has no row for it: QUESTION Q1 (PQ-27). What release does
  // serve is asserted here: its announcement, and a screen with the heading
  // and the code and no line, no count, no letter — even with an answer saved.
  const s = SCREENS.find((x) => x.surface === 'buzzer' && x.name === 'released');
  assert.ok(s.said.includes(M.live_released), 'release is not announced');
  const shown = keysIn(texts(s.html).map((t) => t.text).join(' '));
  assert.deepEqual(shown.sort(), ['buzzer_join_label', 'buzzer_title_role', 'title_wordmark']);
  const bare = texts(s.html).map((t) => t.text.replace(MARKER, '').trim()).filter(Boolean);
  assert.deepEqual(bare, ['ABC234']);
  assert.ok(!Object.values(REAL.COPY).includes('Nothing about you was recorded.'));
});

// --- PQ-4: the room's refusal reasons are not copy -------------------------------

test('the buzzer renders a refusal from its §11 key by code, never the room\'s reason', () => {
  const join = { type: 'boot', search: '', stored: null };
  const live = [...IN_ROOM, { type: 'frame', frame: attach(BF.live) }];
  const cases = [
    ...Object.keys(B.REFUSAL_KEY).map((slug) => [join, { type: 'submit', code: 'ABC234' },
      { type: 'joined', status: 404, body: { refusal: slug, reason: REASON } }]),
    [join, { type: 'submit', code: 'ABC234' }, { type: 'joined', status: 409, body: { refusal: 'no_such_slug', reason: REASON } }],
    [join, { type: 'submit', code: 'ABC234' }, { type: 'joined', status: 500, body: { reason: REASON } }],
    [...live, { type: 'tap', letter: 'C' }, { type: 'answered', status: 409, body: { reason: REASON, phase: 'closed', saved: 'C' } }],
    [...live, { type: 'tap', letter: 'C' }, { type: 'answered', status: 422, body: { reason: REASON } }],
  ];
  for (const events of cases) {
    const d = drive(events);
    assert.ok(!d.html.includes(REASON), JSON.stringify(events.at(-1)));
    assert.ok(!d.said.some((x) => String(x).includes(REASON)), JSON.stringify(events.at(-1)));
  }
  for (const slug of Object.keys(B.REFUSAL_KEY)) {
    const d = drive([join, { type: 'submit', code: 'ABC234' }, { type: 'joined', status: 404, body: { refusal: slug, reason: REASON } }]);
    if (slug === 'unknown_error') continue;
    assert.deepEqual(keysIn(d.html).filter((k) => k.startsWith('join_fail_')), [B.REFUSAL_KEY[slug]], slug);
  }
});
