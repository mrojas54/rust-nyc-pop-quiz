// The host phone (T-07): AC-45, AC-46, AC-47, AC-49, AC-50, AC-74.
//
// web/host/host.js is a classic script like web/shared, so it is evaluated into
// the same node:vm sandbox _load.js builds for the shared modules. The payloads
// below are hand-built in the shape of room/src/view.rs HostPayload; they hold
// placeholder prose and no program text, so nothing here writes down what a
// program prints. The room-side twin, which drives the real router through
// every phase, is room/tests/host_page.rs.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { load } = require('./_load.js');

const HOST_JS = path.join(__dirname, '..', 'host', 'host.js');
const HOST_HTML = path.join(__dirname, '..', 'host', 'index.html');

function loadHost() {
  const { PQ, sandbox } = load('all', { timers: false });
  vm.runInContext(fs.readFileSync(HOST_JS, 'utf8'), sandbox, { filename: 'web/host/host.js' });
  return PQ;
}

const PQ = loadHost();

// The room's phase → next action table (room/src/phase.rs Phase::next_action).
const NEXT = {
  idle: 'put-on-screen',
  live: 'close-answers',
  closed: 'show-split',
  split: 'walk-it',
  work: 'reveal',
  reveal: 'release',
  released: 'run-it-again',
};

function payload(phase, extra = {}) {
  const p = {
    phase,
    code: 'ABC234',
    label: PQ.HOST_PHASE_LABEL[phase],
    primary: { action: NEXT[phase], label: PQ.host.actionLabel(NEXT[phase]) },
    present: 12,
  };
  if (phase !== 'idle') p.answered = 7;
  if (phase === 'work' || phase === 'reveal') {
    const at = phase === 'work' ? 1 : 5;
    p.step = {
      at, m: 6,
      label: PQ.fill(PQ.COPY.wall_trace_step, { N: at + 1, M: 6 }),
      note: `FIXTURE-STEP-NOTE-${at}`,
      back: { action: 'step-back', label: '←' },
      forward: { action: 'step-forward', label: '→' },
      can_back: true,
      can_forward: phase === 'work',
    };
  }
  if (phase === 'reveal') {
    p.read_aloud = {
      heading: PQ.COPY.host_reveal_read_aloud,
      what: { heading: PQ.COPY.host_reveal_beat_what, text: 'FIXTURE-WHAT' },
      middle: { heading: PQ.fill(PQ.COPY.host_reveal_beat_why, { n: 4, X: 'B' }), text: 'FIXTURE-WHY-B' },
      takeaway: { heading: PQ.COPY.host_reveal_beat_remember, text: 'FIXTURE-TAKEAWAY' },
    };
  }
  return Object.assign(p, extra);
}

const buttons = (html) => [...html.matchAll(/<button\b[^>]*>([^<]*)<\/button>/g)].map((m) => ({ tag: m[0], text: m[1] }));
const primaries = (html) => buttons(html).filter((b) => /data-primary=/.test(b.tag));
const unescape = (s) => s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&amp;/g, '&');

// PQ-34 (HC-0): every host screen is headed Pop Quiz Host; the phase is
// announced through the live region, not shown.
test('every phase is headed Pop Quiz Host and has exactly one primary action', () => {
  for (const phase of PQ.PHASES) {
    const html = PQ.host.render(payload(phase));
    assert.ok(html.includes(`>${PQ.COPY.host_title}</h1>`), `${phase}: the title`);
    assert.ok(!html.includes(`>${PQ.HOST_PHASE_LABEL[phase]}<`), `${phase}: no phase label shown`);
    const p = primaries(html);
    assert.equal(p.length, 1, `${phase}: one primary`);
    assert.ok(p[0].tag.includes(`data-primary="${NEXT[phase]}"`), `${phase}: the next legal action`);
    assert.equal(unescape(p[0].text), PQ.host.actionLabel(NEXT[phase]), `${phase}: §11 label`);
  }
});

test('AC-49: the room code shows wherever returning is possible, and not once released', () => {
  for (const phase of PQ.PHASES) {
    const html = PQ.host.render(payload(phase));
    assert.equal(html.includes('>ABC234<'), phase !== 'released', phase);
  }
});

test('AC-45: the actions across the screens are exactly the eight, plus ← → in work and reveal', () => {
  const seen = new Set([...primaries(PQ.host.renderCreate({})).map((b) => /data-primary="([^"]+)"/.exec(b.tag)[1])]);
  for (const phase of PQ.PHASES) {
    const html = PQ.host.render(payload(phase));
    for (const b of primaries(html)) seen.add(/data-primary="([^"]+)"/.exec(b.tag)[1]);
    const steps = buttons(html).filter((b) => /data-step=/.test(b.tag));
    const tracing = phase === 'work' || phase === 'reveal';
    assert.equal(steps.length, tracing ? 2 : 0, `${phase}: ← →`);
    // No button that is neither the primary nor a step.
    assert.equal(buttons(html).length, 1 + steps.length, `${phase}: nothing else offered`);
  }
  const labels = [...seen].map((s) => PQ.host.actionLabel(s)).sort();
  const eight = [...PQ.COPY_ROWS['Host, actions']].map((k) => PQ.COPY[k]).sort();
  assert.deepEqual(labels, eight);
});

test('AC-45: ← and → are disabled when the room cannot step that way', () => {
  const html = PQ.host.render(payload('reveal'));
  const fwd = buttons(html).find((b) => b.tag.includes('data-step="step-forward"'));
  const back = buttons(html).find((b) => b.tag.includes('data-step="step-back"'));
  assert.ok(/\bdisabled\b/.test(fwd.tag));
  assert.ok(!/\bdisabled\b/.test(back.tag));
});

test('the create screen: Pop Quiz Host and Create a room, nothing else', () => {
  const html = PQ.host.renderCreate({});
  assert.equal(primaries(html).length, 1);
  assert.ok(html.includes('data-primary="create"'));
  const t = unescape(html.replace(/<[^>]+>/g, ' ')).replace(/\s+/g, ' ').trim();
  assert.equal(t, `${PQ.COPY.host_title} ${PQ.COPY.host_action_create}`);
});

test('AC-46: one count line — Joined: n in idle, Answered: n from live on', () => {
  const idle = unescape(PQ.host.render(payload('idle', { present: 15 })));
  assert.ok(idle.includes(`<div class="host-count">Joined: 15</div>`));
  assert.ok(!idle.includes('Answered:'));
  for (const phase of PQ.PHASES.filter((p) => p !== 'idle')) {
    const html = unescape(PQ.host.render(payload(phase, { present: 15, answered: 9 })));
    assert.ok(html.includes(`<div class="host-count">Answered: 9</div>`), phase);
    assert.ok(!html.includes('Joined:'), phase);
  }
  // moves when the payload does
  assert.ok(unescape(PQ.host.render(payload('live', { answered: 3 }))).includes('Answered: 3'));
});

test('no host screen shows the old count sentence, a fit line, a resume line or first-screen prose', () => {
  for (const phase of PQ.PHASES) {
    const html = unescape(PQ.host.render(payload(phase, { present: 15, answered: 9, fit: PQ.COPY.host_fit_clipped_y })));
    assert.ok(!html.includes('9 of 15'), `${phase}: counts`);
    assert.ok(!html.includes(PQ.COPY.host_fit_clipped_y), `${phase}: fit line`);
    assert.ok(!html.includes('<a '), `${phase}: a link`);
  }
});

test('AC-47: no pre-reveal screen carries a ✓, the script, or a provenance marker', () => {
  for (const phase of ['idle', 'live', 'closed', 'split', 'work']) {
    const html = PQ.host.render(payload(phase));
    assert.ok(!html.includes('✓'), `${phase}: ✓`);
    assert.ok(!html.includes('data-provenance'), `${phase}: provenance`);
    assert.ok(!html.includes(PQ.COPY.host_reveal_read_aloud), `${phase}: Read it aloud`);
  }
});

test('AC-47: the page asks the room for the host projection and the host socket only', () => {
  const src = fs.readFileSync(HOST_JS, 'utf8');
  for (const other of ['/wall', '/buzzer', '/answer', '/join']) {
    assert.ok(!src.includes(`"${other}`) && !src.includes(`'${other}`) && !src.includes(`${other}"`), `host.js names ${other}`);
  }
  assert.ok(src.includes('"/host", {') || src.includes('/host"'), 'fetches the host projection');
  assert.ok(src.includes('/ws/host'), 'attaches the host socket');
});

test('AC-74: the three beats sit under the human provenance marker, in order', () => {
  const html = unescape(PQ.host.render(payload('reveal')));
  assert.ok(/<section class="panel by-human[^"]*" data-provenance="human">/.test(html));
  assert.ok(html.includes('<span class="provenance human">✎</span>'));
  assert.ok(!html.includes('by-machine'));
  const order = ['FIXTURE-WHAT', 'Why 4 of us said B', 'FIXTURE-WHY-B', PQ.COPY.host_reveal_beat_remember, 'FIXTURE-TAKEAWAY'];
  const at = order.map((s) => html.indexOf(s));
  assert.ok(at.every((i) => i >= 0), `all beats present: ${at}`);
  assert.deepEqual([...at].sort((x, y) => x - y), at, 'in order');
  assert.ok(html.includes(PQ.COPY.host_reveal_read_aloud));
});

test('AC-74: with no incorrect votes the middle beat is the heading alone', () => {
  const p = payload('reveal');
  p.read_aloud.middle = { heading: PQ.COPY.host_reveal_nobody_else };
  const html = unescape(PQ.host.render(p));
  assert.ok(html.includes(`<h3>${PQ.COPY.host_reveal_nobody_else}</h3></div>`));
});

test('work and reveal show the step and its words', () => {
  const html = unescape(PQ.host.render(payload('work')));
  assert.ok(html.includes(PQ.fill(PQ.COPY.wall_trace_step, { N: 2, M: 6 })));
  assert.ok(html.includes('FIXTURE-STEP-NOTE-1'));
});

test('room text is escaped before it reaches the page', () => {
  const p = payload('work');
  p.step.note = '<img src=x onerror=alert(1)>';
  assert.ok(!PQ.host.render(p).includes('<img'));
});

test('an unknown phase is a loud failure, not a blank screen', () => {
  assert.throws(() => PQ.host.render(payload('idle', { phase: 'lobby' })));
});

test('AC-50: the resume link is the room screen\'s own address', () => {
  assert.deepEqual({ ...PQ.host.parseLocation({ pathname: '/host/0123abcd', search: '', hash: '#feedface' }) },
    { mode: 'room', roomId: '0123abcd', session: 'feedface' });
  const r = new URL('https://popquiz.example/host/0123abcd#feedface');
  assert.deepEqual({ ...PQ.host.parseLocation(r) }, { mode: 'room', roomId: '0123abcd', session: 'feedface' });
  assert.equal(PQ.host.parseLocation({ pathname: '/host/abc', search: '', hash: '' }).session, null);
});

test('§8.2: the create screen reads the credential from the fragment and the question from the query', () => {
  assert.deepEqual({ ...PQ.host.parseLocation({ pathname: '/host', search: '?question=q3', hash: '#dev-token' }) },
    { mode: 'create', token: 'dev-token', question: 'q3' });
  assert.deepEqual({ ...PQ.host.parseLocation({ pathname: '/host/', search: '', hash: '' }) },
    { mode: 'create', token: null, question: null });
});

test('reconnect backs off 0.5 s doubling to 8 s, and stops on a refused credential or a gone room', () => {
  assert.deepEqual([0, 1, 2, 3, 4, 5, 50].map(PQ.host.backoff), [500, 1000, 2000, 4000, 8000, 8000, 8000]);
  assert.equal(PQ.host.isTerminalClose(4401), true);
  assert.equal(PQ.host.isTerminalClose(4404), true);
  assert.equal(PQ.host.isTerminalClose(1006), false);
  assert.equal(PQ.host.isTerminalClose(4408), false);
});

test('every action label comes from copy.js and an unknown slug throws', () => {
  for (const [slug, key] of Object.entries(PQ.host.ACTIONS)) assert.equal(PQ.host.actionLabel(slug), PQ.COPY[key]);
  assert.throws(() => PQ.host.actionLabel('publish-hint'));
});

test('the page loads the shared modules and the host script by absolute URL', () => {
  const html = fs.readFileSync(HOST_HTML, 'utf8');
  for (const src of ['/shared/dom.js', '/shared/phase.js', '/shared/copy.js', '/host/host.js',
                     '/shared/tokens.css', '/shared/fonts.css', '/shared/components.css', '/host/host.css']) {
    assert.ok(html.includes(`"${src}"`), src);
  }
});
