// The buzzer (T-06): every phase's render, the one-of-three submission state
// over the answer route's response table, the six refusals, the count line,
// reconnect, the private hint, and what the phone must never show.
//
// The fixtures below are shaped like `view::buzzer` payloads and T-04c's state
// frames (room/README.md, *Frames*). No program is run or described here; the
// letters and counts are made up and name nothing a program prints.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { makeElement } = require('./_load.js');

const WEB = path.join(__dirname, '..');
const FILES = ['shared/dom.js', 'shared/copy.js', 'shared/check.js', 'buzzer/buzzer.js'];
const read = (f) => fs.readFileSync(path.join(WEB, f), 'utf8');

function load() {
  const sandbox = { console, __POPQUIZ_NO_MOUNT__: true };
  sandbox.window = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.document = { createElement: makeElement, body: makeElement('body') };
  sandbox.setTimeout = (fn) => { fn(); return 0; };
  vm.createContext(sandbox);
  for (const f of FILES) vm.runInContext(read(f), sandbox, { filename: `web/${f}` });
  return sandbox.window.PopQuiz;
}

const PQ = load();
const B = PQ.Buzzer;
const C = PQ.COPY;
// The wordmark and Guest head every buzzer screen, the join form included.
const TITLE = `${C.title_wordmark} ${C.buzzer_title_role}`;

// --- replay mode: room/tests/buzzer_page.rs ----------------------------------
//
// With BUZZER_REPLAY=1 this file is not a suite but a driver. The Rust test
// drives the wired room, records what a phone received — the join response,
// each socket frame, each answer response, each socket close — as buzzer.js
// events, pipes them in on stdin, and reads back what the page would show
// after each one, so the page's state is held to the server's.
if (process.env.BUZZER_REPLAY === '1') {
  const events = JSON.parse(fs.readFileSync(0, 'utf8'));
  let state = B.initial();
  const out = [];
  for (const event of events) {
    const r = B.reduce(state, event);
    state = r.state;
    const html = B.view(state);
    const count = /data-count="(\d+)"/.exec(html);
    out.push({
      event: event.type,
      screen: state.screen,
      phase: state.frame ? state.frame.phase : null,
      conn: state.conn,
      saved: state.saved,
      submit: state.submit,
      count: count ? Number(count[1]) : null,
      effects: r.effects.map((e) => e.do),
      puts: r.effects.filter((e) => e.do === 'put').map((e) => e.letter),
      html,
    });
  }
  process.stdout.write(JSON.stringify(out));
  return;
}

// --- fixtures ---------------------------------------------------------------

const HINT = 'HINT-FIXTURE-TEXT';
const base = { t: 'state', code: 'ABC234', lines: [] };
// PQ-37: the wall's `split.bars`, as the room sends them to the phone from the
// split on. Made-up counts of made-up votes; percents rounded as view.rs does.
const BARS = [
  { letter: 'A', count: 3, percent: 23 }, { letter: 'B', count: 1, percent: 8 },
  { letter: 'C', count: 7, percent: 54 }, { letter: 'D', count: 0, percent: 0 },
  { letter: 'E', count: 2, percent: 15 },
];
const frames = {
  idle: { ...base, revision: 1, phase: 'idle' },
  live: { ...base, revision: 2, phase: 'live', letters: ['A', 'B', 'C', 'D', 'E'], locked: false,
          hint: { text: HINT, action: C.buzzer_hint_action } },
  closed: { ...base, revision: 3, phase: 'closed', letters: ['A', 'B', 'C', 'D', 'E'], locked: true },
  split: { ...base, revision: 4, phase: 'split',
           counts: { totals: [3, 1, 7, 0, 2], answered: 13, present: 16 }, split: BARS },
  work: { ...base, revision: 5, phase: 'work',
          counts: { totals: [3, 1, 7, 0, 2], answered: 13, present: 16 }, split: BARS },
  reveal: { ...base, revision: 6, phase: 'reveal', correct: 'E',
            counts: { totals: [3, 1, 7, 0, 2], answered: 13, present: 16 }, split: BARS },
  released: { ...base, revision: 7, phase: 'released', mark: '✓' },
};
const attach = (f, saved) => ({ ...f, session: { saved: saved === undefined ? null : saved } });

// Drive the pure core through events; collect every effect on the way.
function drive(events, start) {
  let state = start || B.initial();
  const effects = [];
  for (const e of events) {
    const r = B.reduce(state, e);
    state = r.state;
    effects.push(...r.effects);
  }
  return { state, effects, html: B.view(state) };
}

const joined = { type: 'joined', status: 201, body: { room_id: 'room-1', token: 'tok-1', buzzer: frames.idle } };
const inRoom = (f, saved) => [
  { type: 'boot', search: '?code=ABC234', stored: null }, joined, { type: 'frame', frame: attach(f, saved) },
];

const announced = (effects) => [...effects].filter((e) => e.do === 'announce').map((e) => e.text);
const requests = (effects) => effects.filter((e) => e.do === 'put' || e.do === 'join');
const text = (html) => html.replace(/<\/?b>/g, '').replace(/<[^>]+>/g, ' ').replace(/&amp;/g, '&').replace(/\s+/g, ' ').trim();
const SUBMISSION = [
  C.buzzer_no_answer_yet, C.buzzer_saving,
  (x) => PQ.fill(C.buzzer_saved, { X: x }), (x) => PQ.fill(C.buzzer_save_failed, { X: x }),
];
function submissionShown(html) {
  const t = text(html);
  const found = [];
  if (t.includes(C.buzzer_no_answer_yet)) found.push('none');
  if (t.includes(C.buzzer_saving)) found.push('saving');
  if (/saved — [A-E]/.test(t)) found.push('saved');
  if (t.includes("couldn't save.")) found.push('failed');
  return found;
}

// --- AC-28: one field, by link or typed ----------------------------------------

test('AC-28: the join screen is one field and one button, and nothing beneath', () => {
  const html = B.view(B.initial());
  assert.equal((html.match(/<input\b/g) || []).length, 1, 'exactly one input');
  assert.equal((html.match(/<button\b/g) || []).length, 1, 'exactly one button');
  assert.equal((html.match(/<(select|textarea)\b/g) || []).length, 0);
  assert.ok(html.includes(`>${C.buzzer_join_label}</label>`));
  assert.ok(html.includes(`>${C.buzzer_join_button}</button>`));
  assert.equal(text(html), `${TITLE} ${C.buzzer_join_label} ${C.buzzer_join_button}`, 'no other line (PQ-34)');
});

test('AC-28: the link carries the code — ?code= joins once, with no typing', () => {
  const { state, effects } = drive([{ type: 'boot', search: '?code=abc234', stored: null }]);
  assert.deepEqual(effects.map((e) => ({ ...e })), [{ do: 'join', code: 'ABC234' }]);
  assert.equal(state.code, 'ABC234');
  assert.equal(B.codeFromSearch('?x=1&code=%20xyz789%20'), 'XYZ789');
  assert.equal(B.codeFromSearch(''), '');
});

test('AC-28: a typed code joins; the join keeps the token and attaches', () => {
  const { state, effects } = drive([
    { type: 'boot', search: '', stored: null }, { type: 'submit', code: ' abc234 ' }, joined,
  ]);
  assert.deepEqual(effects.map((e) => e.do), ['join', 'store', 'attach']);
  assert.equal(effects[0].code, 'ABC234');
  assert.deepEqual({ ...effects[1].value }, { code: 'ABC234', room_id: 'room-1', token: 'tok-1' });
  assert.equal(state.screen, 'room');
  assert.equal(state.conn, 'paused', 'paused until the attach frame arrives');
});

test('a reload in the same tab re-attaches the stored session instead of joining again', () => {
  const stored = { code: 'ABC234', room_id: 'room-1', token: 'tok-1' };
  const { effects, state } = drive([{ type: 'boot', search: '?code=ABC234', stored }]);
  assert.deepEqual(effects.map((e) => e.do), ['attach']);
  assert.equal(state.token, 'tok-1');
  // A stored session for another room is not reused.
  const other = drive([{ type: 'boot', search: '?code=XYZ789', stored }]);
  assert.deepEqual(other.effects.map((e) => e.do), ['join']);
});

// --- AC-29 / AC-30: six refusals, six sentences --------------------------------

test('AC-29, AC-30: each refusal shows its own §11 sentence and keeps the field', () => {
  const slugs = ['malformed', 'unknown', 'not_yet_open', 'already_ended', 'closed_for_inactivity', 'full'];
  const keys = ['join_fail_malformed', 'join_fail_unknown', 'join_fail_not_yet_open',
                'join_fail_already_ended', 'join_fail_closed_inactivity', 'join_fail_full'];
  const shown = slugs.map((slug, i) => {
    const status = slug === 'unknown' ? 404 : 409;
    const { html, state } = drive([
      { type: 'submit', code: 'ABC234' },
      { type: 'joined', status, body: { refusal: slug, reason: 'server sentence' } },
    ]);
    assert.equal(state.screen, 'join');
    assert.ok(html.includes(PQ.escapeHtml(C[keys[i]])), `${slug} shows ${keys[i]}`);
    assert.ok(!html.includes('server sentence'), 'rendered from the slug, not the reason');
    assert.equal((html.match(/<input\b/g) || []).length, 1, `${slug}: the field is still there`);
    return C[keys[i]];
  });
  assert.equal(new Set(shown).size, 6, 'six distinct messages');
});

// --- the phases ---------------------------------------------------------------

test('idle: the code in the header, You’re in, and no foot', () => {
  const { html } = drive(inRoom(frames.idle));
  assert.ok(html.includes('ABC234'));
  assert.equal(text(html), `${TITLE} ${C.buzzer_join_label} ABC234 ↑ ${C.buzzer_idle}`, 'nothing else (PQ-34)');
  assert.ok(!html.includes('<footer'));
});

test('live: five letter buttons, the hint behind a tap, Vote', () => {
  const { html } = drive(inRoom(frames.live));
  const letters = html.match(/<button class="buzz[^"]*" type="button" data-letter="([A-E])"/g);
  assert.equal(letters.length, 5);
  assert.ok(!/data-letter="[A-E]"[^>]*disabled/.test(html), 'letters are live');
  assert.ok(text(html).includes(C.buzzer_hint_action));
  assert.ok(!html.includes(HINT), 'the hint is hidden until tapped');
  assert.deepEqual(submissionShown(html), ['none']);
  // HC-0 (2026-09-27): the client's word for the line under the letters.
  assert.equal(C.buzzer_no_answer_yet, 'Vote');
});

test('closed: letters locked, the saved answer restated — or none given', () => {
  let { html } = drive(inRoom(frames.closed, 'B'));
  assert.equal((html.match(/data-letter="[A-E]"[^>]*disabled/g) || []).length, 5);
  assert.ok(text(html).includes(C.buzzer_closed));
  assert.ok(text(html).includes(PQ.fill(C.buzzer_closed_you_said, { X: 'B' })));
  ({ html } = drive(inRoom(frames.closed, null)));
  assert.ok(text(html).includes(C.buzzer_closed_you_didnt));
});

// PQ-37 (HC-0, "guest view should have stats for answers"): from the split
// on, the phone shows the room's five bars under its lines — the wall's bars,
// the wall's `n · p%`, in the wall's order. Still no data-count: the phone
// computes nothing about the room; it draws what the room sent.
const barsText = (yours, correct) => BARS.map((b) =>
  `${b.letter === yours ? '● ' : ''}${b.letter === correct ? '✓ Correct ' : ''}${b.letter} ${b.count} · ${b.percent}%`).join(' ');
const BARS_TEXT = barsText(null, null);
const YOURS_LINE = (x) => `● ${PQ.fill(C.buzzer_closed_you_said, { X: x })}`;
const rows = (html) => [...html.matchAll(/<div class="bar-row([^"]*)" data-bar="([A-E])">/g)]
  .map((m) => ({ letter: m[2], yours: m[1].includes('is-yours') }));

test('split and work: the room\'s five bars, the reader\'s own letter marked, nothing else', () => {
  for (const p of ['split', 'work']) {
    let { html } = drive(inRoom(frames[p], 'C'));
    assert.equal(text(html), `${TITLE} ${C.buzzer_join_label} ABC234 ${barsText('C', null)} ${YOURS_LINE('C')}`, p);
    assert.deepEqual(rows(html), BARS.map((b) => ({ letter: b.letter, yours: b.letter === 'C' })), `${p}: wall order, C is yours`);
    for (const b of BARS) {
      assert.ok(html.includes(`<span class="bar-fill" style="width:${b.percent}%"></span>`), `${p}: ${b.letter}'s bar is ${b.percent}% wide`);
    }
    assert.ok(!html.includes('✓') && !html.includes('rn-correct'), `${p}: no ✓ before reveal`);
    assert.ok(!html.includes('data-count'), `${p}: the phone counts nothing itself`);
    ({ html } = drive(inRoom(frames[p], null)));
    assert.equal(text(html), `${TITLE} ${C.buzzer_join_label} ABC234 ${BARS_TEXT}`, `${p}, no answer: no legend`);
    assert.ok(rows(html).every((r) => !r.yours), `${p}, no answer: no row is marked`);
  }
});

test('AC-40: the reader\'s row is marked by a glyph and a label, never by colour alone', () => {
  const html = drive(inRoom(frames.split, 'B')).html;
  assert.deepEqual(rows(html).filter((r) => r.yours).map((r) => r.letter), ['B'], 'B is the marked row');
  assert.ok(html.includes('<div class="bar-row is-yours" data-bar="B"><span class="bar-yours" aria-hidden="true">●</span>'), 'the glyph is on the row');
  assert.ok(text(html).endsWith(YOURS_LINE('B')), 'the glyph is keyed by *you said B*');
  const css = read('buzzer/buzzer.css');
  const rule = css.slice(css.indexOf('.buzzer .bar-row.is-yours'), css.indexOf('}', css.indexOf('.buzzer .bar-row.is-yours')));
  assert.ok(!/color|background|--accent|--success/.test(rule), 'the mark sets no colour');
});

test('reveal: ✓ It was Y., then the bars with the ✓ letter marked as the wall marks it', () => {
  const html = drive(inRoom(frames.reveal, 'C')).html;
  const t = text(html);
  assert.equal(t, `${TITLE} ${C.buzzer_join_label} ABC234 ✓ It was E. ${barsText('C', 'E')} ${YOURS_LINE('C')}`);
  // The wall's own markup for the correct bar: check.js's element, glyph inside.
  assert.ok(html.includes(`<div class="bar-row" data-bar="E"><span class="bar-yours" aria-hidden="true"></span>${PQ.correctHtml('E', { className: 'letter' })}`));
  assert.deepEqual(rows(html).filter((r) => r.yours).map((r) => r.letter), ['C'], 'C stays the reader\'s, unmarked otherwise');
  assert.equal((html.match(/rn-correct/g) || []).length, 2, 'the ✓ line and the E bar, nothing else');
});

test('reveal with no answer given: You didn’t answer, then ✓ It was Y., then the bars', () => {
  const t = text(drive(inRoom(frames.reveal, null)).html);
  assert.equal(t, `${TITLE} ${C.buzzer_join_label} ABC234 ${C.buzzer_noanswer_count_one} ✓ It was E. ${barsText(null, 'E')}`);
});

test('a frame without `split` draws no bars — the screen is what it was', () => {
  for (const p of ['split', 'work']) {
    const { split, ...bare } = frames[p];
    assert.equal(text(drive(inRoom(bare, 'C')).html), `${TITLE} ${C.buzzer_join_label} ABC234`);
  }
  const { split, ...bare } = frames.reveal;
  assert.equal(text(drive(inRoom(bare, 'C')).html), `${TITLE} ${C.buzzer_join_label} ABC234 ✓ It was E.`);
});

test('the bars read only letter, count and percent; nothing else in an entry reaches the page', () => {
  const planted = { ...frames.split, split: BARS.map((b) => ({ ...b, text: 'PLANTED-TEXT', note: 'PLANTED-NOTE' })) };
  assert.ok(!/PLANTED/.test(drive(inRoom(planted, 'A')).html));
});

test('released: the code and nothing else', () => {
  const { html } = drive(inRoom(frames.released, 'A'));
  assert.equal(text(html), `${TITLE} ${C.buzzer_join_label} ABC234`);
  assert.ok(!html.includes('data-letter'));
});

// --- AC-40 / AC-94: the ✓ is about the answer, never a verdict on the person ------

test('AC-40: ✓ It was X. carries the glyph and the colour together', () => {
  for (const saved of ['E', 'C', null]) {
    const html = drive(inRoom(frames.reveal, saved)).html;
    const m = /<p class="buzz-itwas"><span class="rn-correct">(.*?)<\/span><\/p>/.exec(html);
    assert.ok(m, `the ✓ line is marked (saved ${saved})`);
    assert.ok(m[1].startsWith('<span class="rn-check" aria-hidden="true">✓</span>'), 'glyph inside the class');
    assert.equal((text(m[1]).match(/✓/g) || []).length, 1, 'one ✓, not two');
  }
});

test('AC-94: no ✗, no red, no "wrong" against the participant’s own choice, in any phase', () => {
  for (const p of Object.keys(frames)) {
    for (const saved of ['A', 'B', 'C', 'D', 'E', null]) {
      const html = drive(inRoom(frames[p], saved)).html;
      assert.ok(!html.includes('✗') && !html.includes('✘'), `${p}/${saved}: no ✗`);
      assert.ok(!/\b(wrong|incorrect)\b/i.test(html), `${p}/${saved}: no wrong`);
      assert.ok(!/(red|error|mine-wrong)/i.test(html.replace(/reading|read it|read the/gi, '')), `${p}/${saved}: no red class`);
    }
  }
  for (const f of ['buzzer/buzzer.js', 'buzzer/buzzer.css', 'buzzer/index.html']) {
    const src = read(f);
    assert.ok(!src.includes('✗'), `${f}: no ✗`);
    assert.ok(!/--(red|error)|mine-wrong|var\(--red/.test(src), `${f}: no red token`);
  }
});

// --- AC-35 / AC-36 / AC-34: saving, saved, failed --------------------------------

const RESPONSES = {
  ok: (letter) => ({ type: 'answered', status: 200, body: { saved: letter } }),
  bad: () => ({ type: 'answered', status: 400, body: { reason: 'x' } }),
  server: () => ({ type: 'answered', status: 503, body: null }),
  network: () => ({ type: 'answered', status: 0, body: null }),
};

test('AC-35: exactly one of saving / saved / failed at all times while live', () => {
  // Every sequence of up to three taps, each answered by every response class,
  // with a retry after each failure. The screen shows exactly one state after
  // every single event.
  const outcomes = Object.keys(RESPONSES);
  let checked = 0;
  for (const a of outcomes) for (const b of outcomes) for (const c of outcomes) {
    let state = drive(inRoom(frames.live)).state;
    const script = [
      { type: 'tap', letter: 'A' }, RESPONSES[a]('A'),
      { type: 'tap', letter: 'B' }, { type: 'tap', letter: 'D' }, RESPONSES[b]('B'), RESPONSES[b]('D'),
      { type: 'retry' }, RESPONSES[c]('D'),
    ];
    for (const e of script) {
      state = B.reduce(state, e).state;
      const shown = submissionShown(B.view(state));
      assert.equal(shown.length, 1, `${a}/${b}/${c} after ${e.type}: ${shown}`);
      checked++;
    }
  }
  assert.ok(checked > 400);
});

test('AC-36: a failed write names the last saved answer as safe and offers a retry', () => {
  const { state, html } = drive([...inRoom(frames.live), { type: 'tap', letter: 'B' }, RESPONSES.ok('B'),
    { type: 'tap', letter: 'D' }, RESPONSES.server()]);
  assert.equal(state.saved, 'B', 'the stored answer is untouched');
  assert.ok(text(html).includes(PQ.fill(C.buzzer_save_failed, { X: 'B' })));
  assert.ok(html.includes('data-act="retry"') && text(html).includes(C.buzzer_save_retry));
  const retried = B.reduce(state, { type: 'retry' });
  assert.deepEqual([...retried.effects.filter((e) => e.do === 'put').map((e) => e.letter)], ['D'], 'retry re-sends the choice');
});

test('AC-34: taps while a write is in flight — the last one wins', () => {
  const { effects, state } = drive([...inRoom(frames.live),
    { type: 'tap', letter: 'A' }, { type: 'tap', letter: 'B' }, { type: 'tap', letter: 'C' },
    RESPONSES.ok('A'), RESPONSES.ok('C')]);
  assert.deepEqual(requests(effects).filter((e) => e.do === 'put').map((e) => e.letter), ['A', 'C']);
  assert.equal(state.saved, 'C');
});

test('a 409 restates the saved answer from phase and saved, never the reason', () => {
  const { state, html } = drive([...inRoom(frames.live), { type: 'tap', letter: 'A' },
    { type: 'answered', status: 409, body: { reason: 'SERVER REASON', phase: 'closed', saved: 'E' } },
    { type: 'frame', frame: frames.closed }]);
  assert.equal(state.saved, 'E');
  assert.ok(!html.includes('SERVER REASON'));
  assert.ok(text(html).includes(PQ.fill(C.buzzer_closed_you_said, { X: 'E' })));
});

test('a 401 on the answer route goes back to joining', () => {
  const { state, effects } = drive([...inRoom(frames.live), { type: 'tap', letter: 'A' },
    { type: 'answered', status: 401, body: null }]);
  assert.equal(state.screen, 'join');
  assert.ok(effects.some((e) => e.do === 'store' && e.value === null), 'the dead token is forgotten');
});

// --- AC-58: nothing about the answer leaves the phone after close ------------------

test('AC-58: after close, taps send nothing; the count needs no request', () => {
  const { effects } = drive([...inRoom(frames.live), { type: 'tap', letter: 'C' }, RESPONSES.ok('C'),
    { type: 'frame', frame: frames.closed }, { type: 'tap', letter: 'A' }, { type: 'retry' },
    { type: 'frame', frame: frames.split }, { type: 'tap', letter: 'B' },
    { type: 'frame', frame: frames.work }, { type: 'frame', frame: frames.reveal }]);
  const puts = effects.filter((e) => e.do === 'put');
  assert.deepEqual(puts.map((e) => e.letter), ['C'], 'the only write is the one made while live');
});

// --- AC-37: reconnect ---------------------------------------------------------

test('AC-37: a drop shows paused and locks the letters until the attach frame', () => {
  let { state } = drive([...inRoom(frames.live), { type: 'tap', letter: 'D' }, RESPONSES.ok('D')]);
  let r = B.reduce(state, { type: 'closed', code: 1006 });
  assert.ok(r.effects.some((e) => e.do === 'reconnect'));
  let html = B.view(r.state);
  assert.ok(text(html).includes(PQ.fill(C.buzzer_reconnecting_with_answer, { X: 'D' })));
  assert.equal((html.match(/data-letter="[A-E]"[^>]*disabled/g) || []).length, 5, 'controls paused');
  assert.deepEqual(submissionShown(html), [], 'paused replaces the submission line');
  assert.equal(B.reduce(r.state, { type: 'tap', letter: 'A' }).effects.length, 0, 'a tap while paused sends nothing');
  // Back, in a later phase, with the saved answer from the server's map.
  state = B.reduce(r.state, { type: 'frame', frame: attach(frames.split, 'D') }).state;
  html = B.view(state);
  assert.equal(state.conn, 'attached');
  assert.ok(!text(html).includes(C.buzzer_reconnecting));
  assert.equal(state.saved, 'D', 'the saved answer survives the drop');
});

test('AC-37: paused with no answer yet says only paused', () => {
  const r = B.reduce(drive(inRoom(frames.live)).state, { type: 'closed', code: 1006 });
  const t = text(B.view(r.state));
  assert.ok(t.includes(C.buzzer_reconnecting));
  assert.ok(!t.includes('is safe'));
});

test('the attach frame’s saved answer is the truth after a reconnect', () => {
  let { state } = drive(inRoom(frames.live, null));
  state = B.reduce(state, { type: 'closed', code: 1006 }).state;
  state = B.reduce(state, { type: 'frame', frame: attach(frames.live, 'B') }).state;
  assert.equal(state.saved, 'B');
  assert.deepEqual(submissionShown(B.view(state)), ['saved']);
  // A later broadcast carries no `session` and does not touch it.
  state = B.reduce(state, { type: 'frame', frame: frames.live }).state;
  assert.equal(state.saved, 'B');
});

test('close codes: 4401 re-joins once, 4404 has ended, 4000 does not reconnect', () => {
  let r = B.reduce(drive(inRoom(frames.live)).state, { type: 'closed', code: 4401 });
  assert.equal(r.state.screen, 'join');
  assert.deepEqual([...r.effects.filter((e) => e.do === 'join').map((e) => e.code)], ['ABC234']);
  r = B.reduce(drive(inRoom(frames.live)).state, { type: 'closed', code: 4404 });
  assert.equal(r.state.refusal, 'already_ended');
  r = B.reduce(drive(inRoom(frames.live)).state, { type: 'closed', code: 4000 });
  assert.ok(!r.effects.some((e) => e.do === 'reconnect'));
  assert.equal(r.state.conn, 'paused');
});

// --- AC-48: the hint --------------------------------------------------------

test('AC-48: the hint is read from the frame already held — no request of any kind', () => {
  const { state, html, effects } = drive([...inRoom(frames.live), { type: 'hint' }]);
  assert.ok(html.includes(HINT));
  assert.ok(text(html).endsWith(HINT), 'the hint and nothing after it (PQ-34)');
  assert.deepEqual(effects.filter((e) => e.do !== 'announce' && e.do !== 'store' && e.do !== 'attach' && e.do !== 'join'), []);
  assert.equal(B.reduce(state, { type: 'hint' }).effects.length, 0, 'a second tap does nothing');
});

test('AC-48: through the shell, taking the hint calls neither fetch nor the socket', () => {
  const calls = [];
  const sent = [];
  const listeners = {};
  const el = { innerHTML: '', addEventListener: (k, fn) => { listeners[k] = fn; }, querySelector: () => null };
  class FakeSocket {
    constructor(url) { this.url = url; FakeSocket.last = this; }
    send(m) { sent.push(m); }
    close() {}
  }
  const io = {
    fetch: (url, init) => { calls.push([url, init && init.method]); return new Promise(() => {}); },
    WebSocket: FakeSocket,
    storage: { getItem: () => JSON.stringify({ code: 'ABC234', room_id: 'room-1', token: 'tok-1' }), setItem() {}, removeItem() {} },
    location: { protocol: 'http:', host: 'x', search: '?code=ABC234' },
    setTimeout: () => 0,
    announce: () => true,
  };
  const handle = B.mount(el, io);
  const ws = FakeSocket.last;
  assert.equal(ws.url, 'ws://x/rooms/room-1/ws/buzzer');
  ws.onopen();
  assert.deepEqual(JSON.parse(sent[0]), { t: 'attach', token: 'tok-1' });
  ws.onmessage({ data: JSON.stringify(attach(frames.live, null)) });
  const before = [calls.length, sent.length];
  const hintButton = { getAttribute: (k) => (k === 'data-act' ? 'hint' : null) };
  listeners.click({ target: { closest: () => hintButton } });
  assert.ok(el.innerHTML.includes(HINT));
  assert.deepEqual([calls.length, sent.length], before, 'no fetch, no socket message');
  assert.ok(handle.state().hintShown);
});

// --- AC-83: the live region ------------------------------------------------------

test('AC-83: each phase and each save state is announced verbatim, once', () => {
  const { effects } = drive([...inRoom(frames.live), { type: 'hint' },
    { type: 'tap', letter: 'A' }, RESPONSES.ok('A'), { type: 'tap', letter: 'B' }, RESPONSES.network(),
    { type: 'frame', frame: frames.live },
    { type: 'frame', frame: frames.closed }, { type: 'frame', frame: frames.split },
    { type: 'frame', frame: frames.work }, { type: 'frame', frame: frames.reveal },
    { type: 'frame', frame: frames.released }]);
  assert.deepEqual(announced(effects), [
    C.live_question_on_screen, C.live_hint_shown,
    C.live_saving, PQ.fill(C.live_saved, { X: 'A' }), C.live_saving, C.live_save_failed,
    C.live_answers_closed, C.live_split_on_screen, C.live_walking_through,
    PQ.fill(C.live_revealed, { Y: 'E' }), C.live_released,
  ]);
});

test('AC-83: a reconnect into the same phase does not re-announce it', () => {
  let { state } = drive(inRoom(frames.split, 'A'));
  state = B.reduce(state, { type: 'closed', code: 1006 }).state;
  const r = B.reduce(state, { type: 'frame', frame: attach(frames.split, 'A') });
  assert.deepEqual(announced(r.effects), []);
});

// --- AC-85: 44 px targets ------------------------------------------------------

test('AC-85: every control on the buzzer is at least 44 px tall', () => {
  const tokens = fs.readFileSync(path.join(WEB, 'shared/tokens.css'), 'utf8');
  assert.match(tokens, /--touch-target:\s*44px/);
  const css = read('buzzer/buzzer.css');
  for (const sel of ['.buzz {', '.buzz-input {', '.buzzer .btn {', '.buzz-submit {']) {
    const at = css.indexOf(sel);
    assert.ok(at !== -1, `${sel} is styled`);
    const block = css.slice(at, css.indexOf('}', at));
    assert.match(block, /min-height:\s*var\(--touch-target\)/, `${sel} is a 44 px target`);
  }
});

// --- G-8 / AC-32: never source, never trace ----------------------------------------

test('G-8, AC-32: the buzzer reads no source, trace or option text, even if a payload carried it', () => {
  const src = read('buzzer/buzzer.js');
  assert.ok(!/\.(source|trace|options)\b/.test(src), 'no read of source, trace or options');
  const planted = { ...frames.live, source: 'PLANTED-SOURCE', trace: { note: 'PLANTED-TRACE' },
                    options: [{ letter: 'A', text: 'PLANTED-OPTION' }] };
  for (const f of [planted, { ...planted, phase: 'reveal', correct: 'A', counts: frames.reveal.counts }]) {
    const html = drive(inRoom(f, 'A')).html;
    assert.ok(!/PLANTED/.test(html));
  }
});

test('the page shell types no string of its own', () => {
  const html = read('buzzer/index.html');
  const body = html.slice(html.indexOf('<body>'), html.indexOf('</body>')).replace(/<!--[\s\S]*?-->/g, '');
  assert.equal(text(body.replace(/<script[^>]*><\/script>/g, '')), '');
});

// Hardening: these catch overlapping joins, invisible transport failures,
// and a request that never settles (including a late success after retry).
test('joining ignores a second submit without changing the pending room code', () => {
  const first = B.reduce(B.initial(), { type: 'submit', code: 'ABC234' });
  const second = B.reduce(first.state, { type: 'submit', code: 'XYZ567' });
  assert.equal(second.state.code, 'ABC234');
  assert.equal(second.effects.length, 0);
});

test('transport failures show accessible recovery guidance and preserve the room code', () => {
  for (const status of [0, 429, 500, 201]) {
    const pending = B.reduce(B.initial(), { type: 'submit', code: 'ABC234' }).state;
    const failed = B.reduce(pending, { type: 'joined', status, body: null }).state;
    const html = B.view(failed);
    assert.match(html, /id="pq-refusal" role="status"/);
    assert.match(html, /aria-describedby="pq-refusal"/);
    assert.match(html, /Try again/);
    assert.match(html, /value="ABC234"/);
    assert.doesNotMatch(html, /type="submit" disabled/);
  }
});

test('a hung join becomes retryable and its late response cannot replace a newer join', async () => {
  const timers = [];
  const requests = [];
  const el = { innerHTML: '', addEventListener() {}, querySelector: () => null };
  const handle = B.mount(el, {
    fetch: () => new Promise(resolve => requests.push(resolve)),
    location: { search: '', protocol: 'http:', host: 'x' },
    setTimeout: fn => { timers.push(fn); return timers.length; },
    clearTimeout() {},
  });
  handle.dispatch({ type: 'submit', code: 'ABC234' });
  assert.equal(timers.length, 1, 'the pending request has a deadline');
  timers[0]();
  assert.equal(handle.state().joining, false);
  assert.match(el.innerHTML, /Try again/);
  handle.dispatch({ type: 'submit', code: 'XYZ567' });
  requests[0]({ status: 201, text: () => Promise.resolve(JSON.stringify({ room_id: 'old', token: 'old' })) });
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(handle.state().screen, 'join');
  assert.equal(handle.state().code, 'XYZ567');
  assert.equal(handle.state().joining, true);
});
