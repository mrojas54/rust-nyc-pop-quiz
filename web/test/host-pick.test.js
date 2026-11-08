// Choosing the question on the host phone (SPEC §8.5, PQ-80).
//
// Signed in on /host, the page asks the room for the questions the club can
// run today (GET /organizer/questions, id and topic only), shows them as one
// radio group headed **Question**, and creates the room on the one picked
// through POST /rooms as before. The room-side twin, which drives the real
// router, is popquiz/tests/organizer_questions.rs. The topics below are
// placeholders; nothing here writes down what a program prints.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { load } = require('./_load.js');

const HOST_JS = path.join(__dirname, '..', 'host', 'host.js');

function loadHost() {
  const { PQ, sandbox } = load('all', { timers: false });
  vm.runInContext(fs.readFileSync(HOST_JS, 'utf8'), sandbox, { filename: 'web/host/host.js' });
  return PQ;
}

const PQ = loadHost();
const C = PQ.COPY;

const text = (html) => html.replace(/<[^>]+>/g, ' ')
  .replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&amp;/g, '&')
  .replace(/\s+/g, ' ').trim();
const radios = (html) => [...html.matchAll(/<input\b[^>]*>/g)].map((m) => m[0]).filter((t) => /type="radio"/.test(t));
const primary = (html) => (html.match(/<button\b[^>]*data-primary="create"[^>]*>/) || [null])[0];
const TWO = [{ id: 'q3', topic: 'FIXTURE-TOPIC-A' }, { id: 'q7', topic: 'FIXTURE-TOPIC-B' }];

// --------------------------------------------------------------------------
// The words (AC-98: new keys, each with its Rust twin in popquiz/src/copy.rs).
// --------------------------------------------------------------------------

test('the picker\'s sentences are the approved ones', () => {
  assert.equal(C.host_pick_legend, 'Question');
  assert.equal(C.host_pick_choice, '‹id› · ‹topic›');
  assert.equal(C.host_pick_loading, 'Loading questions…');
  assert.equal(C.host_pick_empty, 'No question is ready for this club today.');
  assert.equal(C.host_pick_reload, 'Reload');
  assert.equal(C.host_pick_load_failed, "Couldn't load the questions.");
});

// --------------------------------------------------------------------------
// The screens.
// --------------------------------------------------------------------------

test('the list is asked for the club in the link, or for the default club', () => {
  assert.equal(PQ.host.questionsHref(null), '/organizer/questions');
  assert.equal(PQ.host.questionsHref('la'), '/organizer/questions?club=la');
  assert.equal(PQ.host.questionsHref('a&b'), '/organizer/questions?club=a%26b');
});

test('loading: the wordmark, Host, the loading line, and no way to create yet', () => {
  const html = PQ.host.renderPick({}, { state: 'loading' });
  assert.equal(text(html), `${C.title_wordmark} ${C.host_title_role} ${C.host_pick_loading}`);
  assert.equal(primary(html), null);
});

test('ready: one radio group headed Question, a choice per question, then Create a room', () => {
  const html = PQ.host.renderPick({}, { state: 'ready', questions: TWO, chosen: null });
  assert.ok(/<fieldset\b[^>]*>\s*<legend\b[^>]*>Question<\/legend>/.test(html), html);
  const rs = radios(html);
  assert.equal(rs.length, 2);
  for (const [i, q] of TWO.entries()) {
    assert.ok(rs[i].includes('name="question"') && rs[i].includes(`value="${q.id}"`), rs[i]);
    assert.ok(text(html).includes(PQ.fill(C.host_pick_choice, q)), q.id);
  }
  // Each radio is named by its own <label>, so the row is the target.
  assert.equal((html.match(/<label\b/g) || []).length, 2);
  // Nothing chosen: Create a room is there and disabled.
  assert.ok(primary(html) && / disabled/.test(primary(html)), primary(html));
  assert.equal((html.match(/data-primary=/g) || []).length, 1, 'one primary action');
});

test('ready: the chosen question is checked, natively, and Create a room is enabled', () => {
  const html = PQ.host.renderPick({}, { state: 'ready', questions: TWO, chosen: 'q7' });
  const rs = radios(html);
  assert.ok(!/ checked/.test(rs[0]) && / checked/.test(rs[1]), rs.join('\n'));
  assert.ok(!/ disabled/.test(primary(html)), primary(html));
  // AC-40: the checked state is the radio's own mark, not a colour class.
  assert.ok(!/class="[^"]*(selected|active|chosen)/.test(html), 'no colour-only state');
});

test('the link\'s question is preselected when it is offered, or the only one', () => {
  assert.equal(PQ.host.preselect(TWO, 'q7'), 'q7');
  assert.equal(PQ.host.preselect(TWO, 'q9'), null, 'not offered: nothing chosen');
  assert.equal(PQ.host.preselect(TWO, null), null);
  assert.equal(PQ.host.preselect(TWO.slice(0, 1), null), 'q3', 'the only one');
  assert.equal(PQ.host.preselect([], 'q3'), null);
});

test('empty: the readiness line and Reload, nothing to create', () => {
  const html = PQ.host.renderPick({}, { state: 'empty' });
  assert.equal(text(html), `${C.title_wordmark} ${C.host_title_role} ${C.host_pick_empty} ${C.host_pick_reload}`);
  assert.ok(/<button\b[^>]*data-reload[^>]*>/.test(html));
  assert.equal(primary(html), null);
  assert.equal(radios(html).length, 0);
});

test('failed: the failure line and Reload', () => {
  const html = PQ.host.renderPick({}, { state: 'failed' });
  assert.equal(text(html), `${C.title_wordmark} ${C.host_title_role} ${C.host_pick_load_failed} ${C.host_pick_reload}`);
  assert.ok(/<button\b[^>]*data-reload[^>]*>/.test(html));
});

test('a refusal stays on the screen above the list', () => {
  const html = PQ.host.renderPick({ status: 'FIXTURE-REASON' }, { state: 'ready', questions: TWO, chosen: null });
  assert.ok(text(html).includes('FIXTURE-REASON'));
});

test('ids and topics are escaped before they reach the page', () => {
  const html = PQ.host.renderPick({}, { state: 'ready', questions: [{ id: 'q"x', topic: '<script>alert(1)</script>' }], chosen: null });
  assert.ok(!html.includes('<script>'));
  assert.ok(!html.includes('value="q"x"'));
});

// --------------------------------------------------------------------------
// The flow, on a scripted window: load, pick, create, refusals, sign-in.
// --------------------------------------------------------------------------

function scripted(href, replies) {
  const [pathname, rest = ''] = href.split('#')[0].split(/(?=\?)/);
  const calls = [];
  const handlers = {};
  const el = {
    innerHTML: '',
    addEventListener: (type, fn) => { (handlers[type] = handlers[type] || []).push(fn); },
    querySelector: () => null,
    contains: () => false,
  };
  const win = {
    document: { getElementById: () => el, activeElement: null, body: {} },
    location: {
      pathname, search: rest, hash: href.includes('#') ? '#' + href.split('#')[1] : '',
      protocol: 'http:', host: 'h',
      replace: (u) => calls.push({ navigate: u }), assign: (u) => calls.push({ navigate: u }),
    },
    history: { replaceState: (_s, _t, u) => calls.push({ history: u }) },
    sessionStorage: { getItem: () => null, setItem: () => {} },
    setTimeout: () => 0,
    fetch: (url, opts = {}) => {
      calls.push({ url, method: opts.method || 'GET', auth: opts.headers && opts.headers.Authorization, body: opts.body && JSON.parse(opts.body) });
      const r = replies.shift();
      if (!r) return Promise.reject(new Error('no scripted reply for ' + url));
      if (r.network) return Promise.reject(new Error('offline'));
      return Promise.resolve({
        status: r.status, ok: r.status >= 200 && r.status < 300, statusText: '',
        text: () => Promise.resolve(r.body === undefined ? '' : JSON.stringify(r.body)),
      });
    },
  };
  const fire = (type, target) => (handlers[type] || []).forEach((fn) => fn({ target, preventDefault() {} }));
  const control = (attr, value) => ({
    closest: (sel) => (sel.split(',').some((s) => s.trim() === `[${attr}]`) ? { getAttribute: (a) => (a === attr ? value : null), disabled: false } : null),
    getAttribute: () => null,
  });
  return {
    win, el, calls,
    pick: (id) => fire('change', { name: 'question', value: id, checked: true, closest: () => null }),
    press: (attr, value = '') => fire('click', control(attr, value)),
    settle: () => new Promise((r) => setImmediate(r)),
  };
}

test('signed in: the page loads the club\'s list with the organizer session, then shows it', async () => {
  const s = scripted('/host?club=la#tok', [{ status: 200, body: { questions: TWO } }]);
  PQ.host.boot(s.win);
  assert.ok(text(s.el.innerHTML).includes(C.host_pick_loading), 'loading first');
  await s.settle(); await s.settle();
  assert.deepEqual(s.calls[0], { url: '/organizer/questions?club=la', method: 'GET', auth: 'Bearer tok', body: undefined });
  assert.equal(radios(s.el.innerHTML).length, 2);
});

test('pick, then Create a room: POST /rooms with the picked id and the club', async () => {
  const s = scripted('/host?club=la#tok', [
    { status: 200, body: { questions: TWO } },
    { status: 201, body: { id: 'r1', host_session: 'hs', host_resume_url: '/host/r1#hs' } },
  ]);
  PQ.host.boot(s.win);
  await s.settle(); await s.settle();
  s.pick('q7');
  assert.ok(!/ disabled/.test(primary(s.el.innerHTML)), 'chosen: Create a room is enabled');
  s.press('data-primary', 'create');
  await s.settle(); await s.settle();
  const post = s.calls.find((c) => c.method === 'POST');
  assert.deepEqual(post, { url: '/rooms', method: 'POST', auth: 'Bearer tok', body: { question_id: 'q7', club: 'la' } });
  assert.ok(s.calls.some((c) => c.navigate === '/host/r1#hs'), 'on to the room');
});

test('a refusal on create keeps its reason on screen across the reload of the list', async () => {
  const s = scripted('/host#tok', [
    { status: 200, body: { questions: TWO } },
    { status: 409, body: { reason: 'FIXTURE-REASON' } },
    { status: 200, body: { questions: TWO.slice(1) } },
  ]);
  PQ.host.boot(s.win);
  await s.settle(); await s.settle();
  s.pick('q3');
  s.press('data-primary', 'create');
  for (let i = 0; i < 6; i++) await s.settle();
  assert.equal(s.calls.filter((c) => c.url === '/organizer/questions').length, 2, 'the list was reloaded');
  assert.ok(text(s.el.innerHTML).includes('FIXTURE-REASON'), s.el.innerHTML);
  assert.deepEqual(radios(s.el.innerHTML).length, 1, 'the reloaded list');
});

test('an empty list, then Reload asks again', async () => {
  const s = scripted('/host#tok', [{ status: 200, body: { questions: [] } }, { status: 200, body: { questions: TWO } }]);
  PQ.host.boot(s.win);
  await s.settle(); await s.settle();
  assert.ok(text(s.el.innerHTML).includes(C.host_pick_empty));
  s.press('data-reload');
  await s.settle(); await s.settle();
  assert.equal(radios(s.el.innerHTML).length, 2);
});

test('a failed load (server or network) says so and offers Reload', async () => {
  for (const reply of [{ status: 503 }, { network: true }]) {
    const s = scripted('/host#tok', [reply]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    assert.ok(text(s.el.innerHTML).includes(C.host_pick_load_failed), JSON.stringify(reply));
  }
});

test('a 403 shows the room\'s reason, not a list', async () => {
  const s = scripted('/host?club=la#tok', [{ status: 403, body: { refusal: 'wrong_role', reason: C.host_denied_wrong_role } }]);
  PQ.host.boot(s.win);
  await s.settle(); await s.settle();
  assert.ok(text(s.el.innerHTML).includes(C.host_denied_wrong_role));
  assert.equal(radios(s.el.innerHTML).length, 0);
});

test('a 401 on the list returns to Sign in with Discord, carrying the club', async () => {
  const s = scripted('/host?club=la#tok', [{ status: 401 }]);
  PQ.host.boot(s.win);
  await s.settle(); await s.settle();
  assert.ok(s.el.innerHTML.includes('data-sign-in'), s.el.innerHTML);
  assert.ok(s.el.innerHTML.includes('href="/auth/discord?question=&amp;club=la"'), s.el.innerHTML);
});

// --------------------------------------------------------------------------
// What a screen reader hears (SPEC §10, AC-83 pattern, PQ-87): each change of
// the picker's state says its own line once through the live region. Opening
// says nothing (the loading line is the page as it loads, read with it); the
// list arriving says its legend, the one existing line that names it. A
// repaint that leaves the state as it was says nothing again.
// --------------------------------------------------------------------------

function hearing() {
  const said = [];
  const was = PQ.announce;
  PQ.announce = (t) => { said.push(t); return true; };
  return { said, done: () => { PQ.announce = was; } };
}

test('announced: opening says nothing; the list arriving says Question, once', async () => {
  const h = hearing();
  try {
    const s = scripted('/host?club=la#tok', [{ status: 200, body: { questions: TWO } }]);
    PQ.host.boot(s.win);
    assert.deepEqual(h.said, [], 'opening');
    await s.settle(); await s.settle();
    assert.deepEqual(h.said, [C.host_pick_legend]);
  } finally { h.done(); }
});

test('announced: an empty list says the readiness line', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 200, body: { questions: [] } }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    assert.deepEqual(h.said, [C.host_pick_empty]);
  } finally { h.done(); }
});

test('announced: a failed load (server or network) says the failure line', async () => {
  for (const reply of [{ status: 503 }, { network: true }]) {
    const h = hearing();
    try {
      const s = scripted('/host#tok', [reply]);
      PQ.host.boot(s.win);
      await s.settle(); await s.settle();
      assert.deepEqual(h.said, [C.host_pick_load_failed], JSON.stringify(reply));
    } finally { h.done(); }
  }
});

test('announced: Reload says the loading line, then what the list came back as', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 200, body: { questions: [] } }, { status: 200, body: { questions: TWO } }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    s.press('data-reload');
    assert.deepEqual(h.said, [C.host_pick_empty, C.host_pick_loading], 'Reload → loading');
    await s.settle(); await s.settle();
    assert.deepEqual(h.said, [C.host_pick_empty, C.host_pick_loading, C.host_pick_legend]);
  } finally { h.done(); }
});

test('announced: a repaint that keeps the state (a pick, Create a room in flight) says nothing again', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [
      { status: 200, body: { questions: TWO } },
      { status: 201, body: { id: 'r1', host_session: 'hs', host_resume_url: '/host/r1#hs' } },
    ]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    s.pick('q7');
    s.pick('q3');
    s.press('data-primary', 'create');
    await s.settle(); await s.settle();
    assert.ok(s.calls.some((c) => c.navigate === '/host/r1#hs'), 'on to the room');
    assert.deepEqual(h.said, [C.host_pick_legend], 'the list, said once');
  } finally { h.done(); }
});

// PQ-88: a 403 on the list, and a refused *Create a room*, say the status
// line's own text, the room's reason, once. Nothing new is written for them.

test('announced: a 403 on the list says the room\'s reason, once', async () => {
  const h = hearing();
  try {
    const s = scripted('/host?club=la#tok', [{ status: 403, body: { refusal: 'wrong_role', reason: C.host_denied_wrong_role } }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    assert.deepEqual(h.said, [C.host_denied_wrong_role]);
  } finally { h.done(); }
});

test('announced: a 403 with no reason says the status line as drawn', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 403 }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    assert.ok(text(s.el.innerHTML).includes('403'), s.el.innerHTML);
    assert.deepEqual(h.said, ['403']);
  } finally { h.done(); }
});

test('announced: once denied, the page\'s own events say the reason no more', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 403, body: { reason: 'FIXTURE-REASON' } }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    s.pick('q7');
    s.press('data-primary', 'create');
    await s.settle(); await s.settle();
    assert.deepEqual(h.said, ['FIXTURE-REASON'], 'said once');
    assert.equal(s.calls.length, 1, 'nothing else was asked for');
  } finally { h.done(); }
});

test('announced: a refused create says its reason in place of the loading line, then the list', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [
      { status: 200, body: { questions: TWO } },
      { status: 409, body: { reason: 'FIXTURE-REASON' } },
      { status: 200, body: { questions: TWO.slice(1) } },
    ]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    s.pick('q3');
    s.press('data-primary', 'create');
    for (let i = 0; i < 6; i++) await s.settle();
    assert.deepEqual(h.said, [C.host_pick_legend, 'FIXTURE-REASON', C.host_pick_legend]);
  } finally { h.done(); }
});

// PQ-89: a *Create a room* that fails at the network leaves the list as it
// was (still ready), so no state changes; what changes is the status line,
// and that line, the error as drawn, is what is said. Each failed press is a
// new attempt and says it again; a repaint in between says nothing.

test('announced: a network error on create says the status line as drawn, once', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 200, body: { questions: TWO } }, { network: true }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    s.pick('q3');
    s.press('data-primary', 'create');
    await s.settle(); await s.settle();
    assert.ok(text(s.el.innerHTML).includes('offline'), s.el.innerHTML);
    assert.equal(radios(s.el.innerHTML).length, 2, 'the list stays');
    assert.deepEqual(h.said, [C.host_pick_legend, 'offline']);
  } finally { h.done(); }
});

test('announced: after a network error, a repaint that keeps the state and the line says nothing again', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 200, body: { questions: TWO } }, { network: true }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    s.pick('q3');
    s.press('data-primary', 'create');
    await s.settle(); await s.settle();
    s.pick('q7');
    s.pick('q3');
    await s.settle();
    assert.ok(text(s.el.innerHTML).includes('offline'), 'the line is still drawn');
    assert.deepEqual(h.said, [C.host_pick_legend, 'offline']);
  } finally { h.done(); }
});

test('announced: a second press that fails at the network says it again', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 200, body: { questions: TWO } }, { network: true }, { network: true }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    s.pick('q3');
    s.press('data-primary', 'create');
    await s.settle(); await s.settle();
    s.press('data-primary', 'create');
    await s.settle(); await s.settle();
    assert.equal(s.calls.filter((c) => c.method === 'POST').length, 2, 'two attempts');
    assert.deepEqual(h.said, [C.host_pick_legend, 'offline', 'offline']);
  } finally { h.done(); }
});

// PQ-89 (reopened on the client's ruling): a 401 sends the host back to Sign
// in, and that says the client's line, verbatim, once. Opening the page signed
// out still says nothing (PQ-87): the sign-in screen is the page as it loads.

test('the signed-out line is the client\'s, verbatim', () => {
  assert.equal(C.host_pick_signed_out, 'Recompiling, please sign in again');
});

// The page's clock and timers, held by the test: `t` is now, in ms, and
// `run()` fires what is waiting, moving the clock to each one's time.
function clocked(s) {
  const c = { t: 0, timers: [] };
  s.win.performance = { now: () => c.t };
  s.win.setTimeout = (fn, ms) => { c.timers.push({ fn, at: c.t + ms, ms }); return c.timers.length; };
  c.run = () => { for (const x of c.timers.splice(0)) { c.t = Math.max(c.t, x.at); x.fn(); } };
  return c;
}

test('announced: a 401 on Create a room says the signed-out line once and shows Sign in', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 200, body: { questions: TWO } }, { status: 401 }]);
    const c = clocked(s);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    c.t += 1000;   // a person picks and presses a while after the list arrives
    s.pick('q3');
    s.press('data-primary', 'create');
    await s.settle(); await s.settle();
    assert.ok(s.el.innerHTML.includes('data-sign-in'), s.el.innerHTML);
    assert.deepEqual(h.said, [C.host_pick_legend, C.host_pick_signed_out]);
    assert.ok(C.host_pick_signed_out, 'the line exists');
  } finally { h.done(); }
});

test('announced: a 401 on the list says the signed-out line once and shows Sign in', async () => {
  const h = hearing();
  try {
    const s = scripted('/host?club=la#tok', [{ status: 401 }]);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    assert.ok(s.el.innerHTML.includes('data-sign-in'), s.el.innerHTML);
    assert.deepEqual(h.said, [C.host_pick_signed_out]);
    assert.ok(C.host_pick_signed_out, 'the line exists');
  } finally { h.done(); }
});

// PQ-89 review pass 2: a line said within 40 ms of the one before it can
// collapse that one (PQ.announce writes 40 ms after the call). A list 401 can
// come back that fast, so the signed-out line waits until the line before it
// has been written and has stood for a moment; then it is said.

test('announced: a fast 401 after Reload holds the signed-out line until the loading line has been written', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [{ status: 200, body: { questions: [] } }, { status: 401 }]);
    const c = clocked(s);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    c.t += 1000;
    s.press('data-reload');
    assert.deepEqual(h.said, [C.host_pick_empty, C.host_pick_loading], 'Reload → loading');
    c.t += 5;   // the 401 is back 5 ms later
    await s.settle(); await s.settle();
    assert.ok(s.el.innerHTML.includes('data-sign-in'), 'Sign in is back at once');
    assert.deepEqual(h.said, [C.host_pick_empty, C.host_pick_loading], 'held, not said over the loading line');
    assert.equal(c.timers.length, 1, 'one held line');
    assert.ok(c.timers[0].ms > 40, `held past the loading line's write (${c.timers[0].ms} ms)`);
    c.run();
    assert.deepEqual(h.said, [C.host_pick_empty, C.host_pick_loading, C.host_pick_signed_out]);
  } finally { h.done(); }
});

test('announced: a fast list 401 after a refused create holds the signed-out line until the reason has been written', async () => {
  const h = hearing();
  try {
    const s = scripted('/host#tok', [
      { status: 200, body: { questions: TWO } },
      { status: 409, body: { reason: 'FIXTURE-REASON' } },
      { status: 401 },
    ]);
    const c = clocked(s);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    c.t += 1000;
    s.pick('q3');
    s.press('data-primary', 'create');
    for (let i = 0; i < 6; i++) await s.settle();
    assert.ok(s.el.innerHTML.includes('data-sign-in'), 'Sign in is back at once');
    assert.deepEqual(h.said, [C.host_pick_legend, 'FIXTURE-REASON'], 'held, not said over the reason');
    assert.equal(c.timers.length, 1, 'one held line');
    c.run();
    assert.deepEqual(h.said, [C.host_pick_legend, 'FIXTURE-REASON', C.host_pick_signed_out]);
  } finally { h.done(); }
});

test('announced: opening the page signed out says nothing', async () => {
  const h = hearing();
  try {
    const s = scripted('/host?question=q3', []);
    PQ.host.boot(s.win);
    await s.settle(); await s.settle();
    assert.ok(s.el.innerHTML.includes('data-sign-in'), s.el.innerHTML);
    assert.deepEqual(h.said, []);
    assert.equal(s.calls.length, 0, 'nothing was asked for');
  } finally { h.done(); }
});
