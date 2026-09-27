// T-08's page driver: the served pages' own scripts, under node.
//
// Not a test suite (no *.test.js), and not read by `node --test`. The Rust
// canary (room/tests/canary.rs, canary_full.rs) spawns it in one of two modes:
//
//   render   stdin: {scripts:[{name,text}], frames:[…]} — the wall page's
//            scripts as the room served them, and every wall frame the canary
//            received. stdout: one PQ.Wall.html string per frame.
//
//   buzzer   the real buzzer.js, as served, mounted with node's own fetch and
//            WebSocket pointed at the running room — so the page joins,
//            attaches and answers exactly as a phone would. Every request it
//            makes, every socket message it sends and every frame it receives
//            is logged. Commands arrive one JSON line at a time on stdin and
//            each gets one JSON line back:
//              {cmd:"mount", base, search, scripts}
//              {cmd:"wait", phase}         until that phase's frame has landed
//              {cmd:"tap", letter}         a tap, then until nothing is in flight
//              {cmd:"hint"}                Show me a hint
//              {cmd:"drop"}                the socket dies; wait for re-attach
//              {cmd:"snap"}                the page now
//              {cmd:"quit"}
//            Every reply to a page command is a snapshot: {phase, conn, saved,
//            submit, hintShown, token, html, log:[…since the last snapshot]}.
//
// Nothing here decides pass or fail; the Rust side scans what comes back.

'use strict';

const vm = require('node:vm');
const readline = require('node:readline');

function sandbox() {
  const el = (tag) => ({
    tagName: tag, className: '', textContent: '', innerHTML: '', style: {}, attributes: {}, children: [],
    setAttribute(k, v) { this.attributes[k] = String(v); },
    getAttribute(k) { return Object.prototype.hasOwnProperty.call(this.attributes, k) ? this.attributes[k] : null; },
    appendChild(c) { this.children.push(c); return c; },
    removeChild() {}, addEventListener() {}, querySelector() { return null; }, querySelectorAll() { return []; },
    classList: { add() {}, remove() {}, contains() { return false; }, toggle() {} },
  });
  const s = { console, __POPQUIZ_NO_MOUNT__: true, setTimeout, clearTimeout };
  s.window = s;
  s.globalThis = s;
  s.document = {
    createElement: el, body: el('body'), readyState: 'complete',
    getElementById() { return null; }, addEventListener() {}, querySelector() { return null; },
  };
  vm.createContext(s);
  return s;
}

function evaluate(s, scripts) {
  for (const { name, text } of scripts) vm.runInContext(text, s, { filename: name });
  return s.PopQuiz;
}

function readAll() {
  return new Promise((resolve) => {
    let data = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (d) => { data += d; });
    process.stdin.on('end', () => resolve(data));
  });
}

async function render() {
  const input = JSON.parse(await readAll());
  const PQ = evaluate(sandbox(), input.scripts);
  const out = input.frames.map((f) => PQ.Wall.html(f, {}));
  process.stdout.write(JSON.stringify(out));
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function until(pred, what, ms = 5000) {
  const end = Date.now() + ms;
  while (!pred()) {
    if (Date.now() > end) throw new Error('timed out waiting for ' + what);
    await sleep(10);
  }
}

async function buzzer() {
  let handle = null;
  let el = null;
  let log = [];
  let sockets = [];
  let base = '';

  function snapshot() {
    const st = handle.state();
    const out = {
      phase: st.frame ? st.frame.phase : null,
      conn: st.conn, saved: st.saved, submit: st.submit, hintShown: st.hintShown,
      token: st.token, roomId: st.roomId, screen: st.screen, refusal: st.refusal,
      html: el.innerHTML, log,
    };
    log = [];
    return out;
  }

  function mount(msg) {
    base = msg.base;
    const s = sandbox();
    const PQ = evaluate(s, msg.scripts);
    const host = new URL(base).host;
    const listeners = {};
    el = { innerHTML: '', addEventListener: (k, fn) => { listeners[k] = fn; }, querySelector: () => null };
    // The page's own fetch, logged. Relative URLs resolve against the room.
    const pageFetch = (url, init) => {
      init = init || {};
      log.push({ kind: 'fetch', method: init.method || 'GET', url, headers: init.headers || {}, body: init.body || null });
      return fetch(new URL(url, base), init);
    };
    class LoggedSocket {
      constructor(url) {
        log.push({ kind: 'ws-open', url });
        const ws = new WebSocket(url);
        this.ws = ws;
        sockets.push(this);
        ws.onopen = () => this.onopen && this.onopen();
        ws.onmessage = (m) => {
          log.push({ kind: 'ws-frame', data: String(m.data) });
          if (this.onmessage) this.onmessage({ data: String(m.data) });
        };
        ws.onclose = (e) => {
          log.push({ kind: 'ws-close', code: e.code });
          if (this.onclose) this.onclose({ code: e.code });
        };
      }
      send(m) { log.push({ kind: 'ws-send', data: String(m) }); this.ws.send(m); }
      close() { try { this.ws.close(); } catch (e) { /* already closing */ } }
    }
    const store = new Map();
    handle = PQ.Buzzer.mount(el, {
      fetch: pageFetch,
      WebSocket: LoggedSocket,
      storage: { getItem: (k) => (store.has(k) ? store.get(k) : null), setItem: (k, v) => store.set(k, String(v)), removeItem: (k) => store.delete(k) },
      location: { protocol: 'http:', host, search: msg.search },
      setTimeout,
      announce: () => true,
    });
    return { ok: true };
  }

  const state = () => handle.state();
  const commands = {
    async wait(msg) {
      await until(() => state().frame && state().frame.phase === msg.phase && state().conn === 'attached', 'phase ' + msg.phase);
      return snapshot();
    },
    async tap(msg) {
      handle.dispatch({ type: 'tap', letter: msg.letter });
      await until(() => !state().inFlight, 'the answer to land');
      await sleep(50);
      return snapshot();
    },
    async hint() {
      handle.dispatch({ type: 'hint' });
      await sleep(100);
      return snapshot();
    },
    async drop() {
      const before = sockets.length;
      const current = sockets[sockets.length - 1];
      // The network goes away under the page: the socket closes as a dropped
      // connection would, and the page's own reconnect runs.
      current.ws.onclose = (e) => {
        log.push({ kind: 'ws-close', code: e.code });
        if (current.onclose) current.onclose({ code: 1006 });
      };
      current.ws.close();
      await until(() => sockets.length > before && state().conn === 'attached', 'the page to re-attach');
      return snapshot();
    },
    async snap() { await sleep(50); return snapshot(); },
  };

  const rl = readline.createInterface({ input: process.stdin });
  for await (const line of rl) {
    if (!line.trim()) continue;
    const msg = JSON.parse(line);
    if (msg.cmd === 'quit') break;
    let reply;
    try {
      reply = msg.cmd === 'mount' ? mount(msg) : await commands[msg.cmd](msg);
    } catch (e) {
      reply = { error: String(e && e.stack || e), log };
    }
    process.stdout.write(JSON.stringify(reply) + '\n');
  }
  for (const s of sockets) s.close();
  process.exit(0);
}

const mode = process.argv[2];
(mode === 'render' ? render() : mode === 'buzzer' ? buzzer() : Promise.reject(new Error('mode: render | buzzer')))
  .catch((e) => { process.stderr.write(String(e && e.stack || e) + '\n'); process.exit(1); });
