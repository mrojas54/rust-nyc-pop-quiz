/* ===========================================================================
   The buzzer (T-06) — the phone in everyone's hand.

   Five letters, a private hint, and at the reveal the answer's letter. No
   source, no trace, no option text, ever (G-8):
   nothing in this file reads a payload's `source`, `trace` or `options`.

   Two halves:

     - The CORE is pure. `reduce(state, event)` returns the next state and a
       list of effects to run; `view(state)` returns the screen as an HTML
       string. No DOM, no fetch, no socket, no timer — so web/test runs it in
       node:vm and room/tests replays a real server's frames through it.
     - The SHELL (`mount`) owns the I/O: it runs the effects with the fetch,
       WebSocket, storage and announce it is handed, and feeds what comes back
       into `reduce` as events.

   Every participant-facing string comes from web/shared/copy.js by key; none
   is typed here (SPEC §11: authored there and only there).

   What the server promises, and this file relies on (room/README.md):
     POST /join {code}          201 {room_id, token, buzzer} | {refusal, reason}
     PUT /rooms/{id}/answer     200 {saved} | 409 {reason, phase, saved} | 401 | 4xx | 5xx
     GET /rooms/{id}/ws/buzzer  first message {"t":"attach","token"}; the first
                                frame is the current state plus
                                `session.saved`; close 4401 / 4404 / 4000
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  var LETTERS = ["A", "B", "C", "D", "E"];

  /* The six refusals `POST /join` names (room/README.md, *Joining*), each to
     its own §11 sentence and next step (AC-29). */
  var REFUSAL_KEY = {
    malformed: "join_fail_malformed",
    unknown: "join_fail_unknown",
    not_yet_open: "join_fail_not_yet_open",
    already_ended: "join_fail_already_ended",
    closed_for_inactivity: "join_fail_closed_inactivity",
    full: "join_fail_full"
  };

  /* AC-83: what the live region says on entering each phase. `idle` has no
     line in SPEC §11's list, so entering it announces nothing. */
  var PHASE_ANNOUNCE = {
    live: "live_question_on_screen",
    closed: "live_answers_closed",
    split: "live_split_on_screen",
    work: "live_walking_through",
    reveal: "live_revealed",
    released: "live_released"
  };

  function t(key, values) { return PQ.t(key, values); }
  function esc(s) { return PQ.escapeHtml(s); }
  function attr(s) { return PQ.escapeAttr(s); }

  function normalizeCode(raw) {
    return String(raw === undefined || raw === null ? "" : raw).trim().toUpperCase();
  }

  /* `?code=ABC234` from a location.search string. */
  function codeFromSearch(search) {
    var m = /[?&]code=([^&#]*)/.exec(String(search || ""));
    if (!m) return "";
    try { return normalizeCode(decodeURIComponent(m[1].replace(/\+/g, " "))); }
    catch (e) { return normalizeCode(m[1]); }
  }

  function refusalMessage(slug) {
    return Object.prototype.hasOwnProperty.call(REFUSAL_KEY, slug) ? t(REFUSAL_KEY[slug]) : null;
  }

  function isLetter(x) { return LETTERS.indexOf(x) !== -1; }

  // --------------------------------------------------------------------------
  // State
  // --------------------------------------------------------------------------

  function initial() {
    return {
      screen: "join",      // join | room
      code: "",            // the room code, as typed or from the link
      autoJoin: false,     // one automatic join from ?code= (or after a dead token)
      joining: false,
      refusal: null,       // a refusal slug from POST /join
      roomId: null,
      token: null,
      frame: null,         // the last state frame, whole
      conn: "paused",      // paused until a frame arrives on this socket (AC-37)
      reconnect: true,     // false once another tab replaced this one (4000)
      saved: null,         // the letter the server last confirmed
      submit: "idle",      // idle | saving | saved | failed — while live
      inFlight: null,      // the letter a PUT is carrying now
      queued: null,        // a tap made while a PUT was in flight
      lastTried: null,     // what Try again re-sends
      hintShown: false,
      phase: null          // the phase last announced
    };
  }

  function copyState(s) {
    var n = {};
    for (var k in s) if (Object.prototype.hasOwnProperty.call(s, k)) n[k] = s[k];
    return n;
  }

  function phaseOf(s) { return s.frame ? s.frame.phase : null; }

  function canAnswer(s) {
    return s.screen === "room" && s.conn === "attached" && phaseOf(s) === "live";
  }

  // --------------------------------------------------------------------------
  // The reducer. Pure: (state, event) -> {state, effects}.
  // --------------------------------------------------------------------------

  function put(s, letter, fx) {
    s.inFlight = letter;
    s.lastTried = letter;
    s.submit = "saving";
    fx.push({ do: "put", letter: letter });
    fx.push({ do: "announce", text: t("live_saving") });
  }

  function toJoinScreen(s, fx) {
    s.screen = "join";
    s.roomId = null;
    s.token = null;
    s.frame = null;
    s.conn = "paused";
    s.saved = null;
    s.submit = "idle";
    s.inFlight = null;
    s.queued = null;
    s.hintShown = false;
    s.phase = null;
    fx.push({ do: "store", value: null });
    fx.push({ do: "disconnect" });
  }

  function reduce(state, event) {
    var s = copyState(state);
    var fx = [];
    switch (event.type) {
      case "boot": {
        var code = codeFromSearch(event.search);
        var stored = event.stored;
        s.code = code;
        if (stored && stored.token && stored.room_id && (!code || stored.code === code)) {
          // A reload in the same tab: the same session, not a second one.
          s.code = stored.code || code;
          s.screen = "room";
          s.roomId = stored.room_id;
          s.token = stored.token;
          s.conn = "paused";
          s.autoJoin = !!code;
          fx.push({ do: "attach" });
        } else if (code) {
          s.joining = true;
          fx.push({ do: "join", code: code });
        }
        break;
      }

      case "submit": {
        s.code = normalizeCode(event.code);
        s.refusal = null;
        s.joining = true;
        fx.push({ do: "join", code: s.code });
        break;
      }

      case "joined": {
        s.joining = false;
        var body = event.body || {};
        if (event.status === 201 && body.token && body.room_id) {
          s.screen = "room";
          s.refusal = null;
          s.roomId = body.room_id;
          s.token = body.token;
          s.frame = null;
          s.conn = "paused";
          s.saved = null;
          s.submit = "idle";
          fx.push({ do: "store", value: { code: s.code, room_id: s.roomId, token: s.token } });
          fx.push({ do: "attach" });
        } else if (body.refusal && REFUSAL_KEY[body.refusal]) {
          s.refusal = body.refusal;
        } else {
          // The server did not answer with a refusal it names — a network
          // failure or a 5xx. Nothing was created; the field stays for a retry.
          s.refusal = "unknown_error";
        }
        break;
      }

      case "frame": {
        if (s.screen !== "room") break;
        var f = event.frame || {};
        var wasPhase = s.phase;
        var firstOnSocket = s.conn !== "attached";
        s.frame = f;
        s.conn = "attached";
        if (f.session && Object.prototype.hasOwnProperty.call(f.session, "saved")) {
          // Only the attach frame carries it: this session's own answer, from
          // the server's map (AC-37). It is the truth after any reconnect.
          s.saved = isLetter(f.session.saved) ? f.session.saved : null;
          if (!s.inFlight && s.submit === "failed" && s.saved === s.lastTried) s.submit = "saved";
        }
        if (f.phase !== "live") {
          s.hintShown = false;
          s.queued = null;
          if (!s.inFlight) s.submit = "idle";
        } else if (firstOnSocket && !s.inFlight && s.submit !== "failed") {
          s.submit = s.saved ? "saved" : "idle";
        }
        if (f.phase !== wasPhase) {
          s.phase = f.phase;
          var key = PHASE_ANNOUNCE[f.phase];
          if (key) {
            fx.push({ do: "announce", text: f.phase === "reveal" ? t(key, { Y: f.correct }) : t(key) });
          }
        }
        if (f.phase === "released") {
          // The room's sessions are gone with it; nothing to re-attach to.
          fx.push({ do: "store", value: null });
        }
        break;
      }

      case "closed": {
        if (s.screen !== "room") break;
        var codeNum = event.code;
        if (codeNum === 4401) {
          // The token names no session: never joined, or the room released.
          var again = s.code && s.autoJoin !== "spent";
          toJoinScreen(s, fx);
          if (again) {
            s.autoJoin = "spent";
            s.joining = true;
            fx.push({ do: "join", code: s.code });
          }
        } else if (codeNum === 4404) {
          toJoinScreen(s, fx);
          s.refusal = "already_ended";
        } else if (codeNum === 4000) {
          // Another tab attached for this session and took it over.
          s.conn = "paused";
          s.reconnect = false;
        } else {
          s.conn = "paused";
          if (s.reconnect) fx.push({ do: "reconnect" });
        }
        break;
      }

      case "tap": {
        var letter = event.letter;
        if (!isLetter(letter) || !canAnswer(s)) break;
        if (s.inFlight) {
          s.queued = letter;          // last write wins; sent when this one lands
          s.lastTried = letter;
          s.submit = "saving";
        } else {
          put(s, letter, fx);
        }
        break;
      }

      case "retry": {
        if (!canAnswer(s) || s.inFlight || !s.lastTried) break;
        put(s, s.lastTried, fx);
        break;
      }

      case "answered": {
        var status = event.status;
        var b = event.body || {};
        s.inFlight = null;
        if (status === 200 && isLetter(b.saved)) {
          s.saved = b.saved;
          s.submit = "saved";
          fx.push({ do: "announce", text: t("live_saved", { X: b.saved }) });
        } else if (status === 409) {
          // Not live any more. Render from `phase` and `saved`, never `reason`.
          s.saved = isLetter(b.saved) ? b.saved : null;
          s.submit = "idle";
          s.queued = null;
          break;
        } else if (status === 401) {
          toJoinScreen(s, fx);
          s.refusal = "already_ended";
          break;
        } else {
          // 400, any other 4xx, a 5xx, or no response: nothing was stored and
          // the stored answer is as it was (AC-36).
          s.submit = "failed";
          fx.push({ do: "announce", text: t("live_save_failed") });
        }
        if (s.queued && canAnswer(s)) {
          var q = s.queued;
          s.queued = null;
          put(s, q, fx);
        } else {
          s.queued = null;
        }
        break;
      }

      case "hint": {
        if (phaseOf(s) !== "live" || s.hintShown || !s.frame.hint) break;
        // Read from the frame already held. No request: nothing can be
        // recorded and nothing can reach the host or the wall (AC-48, D-8).
        s.hintShown = true;
        fx.push({ do: "announce", text: t("live_hint_shown") });
        break;
      }

      default:
        break;
    }
    return { state: s, effects: fx };
  }

  // --------------------------------------------------------------------------
  // The view. Pure: state -> HTML.
  // --------------------------------------------------------------------------

  /* Exactly one of these while the question is live (AC-35):
     Vote / saving… / saved — X / couldn't save. */
  function submission(s) {
    if (s.submit === "saving") return { kind: "saving", text: t("buzzer_saving") };
    if (s.submit === "failed") {
      // §11 has no sentence for a first write that failed with nothing saved;
      // the dash stands where the letter would be (the prototype's own mark
      // for "no pick").
      return { kind: "failed", text: t("buzzer_save_failed", { X: s.saved || "—" }) };
    }
    if (s.saved) return { kind: "saved", text: t("buzzer_saved", { X: s.saved }) };
    return { kind: "none", text: t("buzzer_no_answer_yet") };
  }

  /* Every buzzer screen, the join form included: the wordmark, Guest beneath. */
  function title() {
    return '<div class="buzz-title"><h1 class="buzz-wordmark">' + esc(t("title_wordmark")) + '</h1>' +
      '<p class="buzz-role">' + esc(t("buzzer_title_role")) + '</p></div>';
  }

  function head(s, lookup) {
    var code = s.frame && s.frame.code ? s.frame.code : s.code;
    return title() + '<header class="buzz-head">' +
      '<div class="buzz-code"><span class="meta">' + esc(t("buzzer_join_label")) + '</span> ' +
      '<b>' + esc(code) + '</b></div>' +
      (lookup ? '<div class="lookup">' + lookup + '</div>' : '') +
      '</header>';
  }

  function joinScreen(s) {
    var msg = null;
    if (s.refusal === "unknown_error") msg = null;
    else if (s.refusal) msg = refusalMessage(s.refusal);
    return title() + '<form class="buzz-join" data-act="join" novalidate>' +
      '<label class="buzz-join-label" for="pq-code">' + esc(t("buzzer_join_label")) + '</label>' +
      '<input id="pq-code" name="code" class="buzz-input" type="text" inputmode="text"' +
      ' autocomplete="off" autocapitalize="characters" spellcheck="false"' +
      ' value="' + attr(s.code) + '"' + (msg ? ' aria-describedby="pq-refusal"' : '') + '>' +
      '<button class="btn btn-primary buzz-join-go" type="submit"' + (s.joining ? ' disabled' : '') + '>' +
      esc(t("buzzer_join_button")) + '</button>' +
      (msg ? '<p class="buzz-refusal" id="pq-refusal" role="status" data-refusal="' + attr(s.refusal) + '">' +
        esc(msg) + '</p>' : '') +
      '</form>';
  }

  function paused(s) {
    var line = s.saved
      ? t("buzzer_reconnecting_with_answer", { X: s.saved })
      : t("buzzer_reconnecting");
    return '<p class="buzz-paused" data-state="paused">' + esc(line) + '</p>';
  }

  function letters(s, locked) {
    return '<div class="buzz-grid" role="group">' + LETTERS.map(function (l) {
      var pressed = s.saved === l;
      var pending = !locked && s.submit === "saving" && (s.inFlight === l || s.queued === l) && !pressed;
      return '<button class="buzz' + (pending ? ' is-pending' : '') + '" type="button"' +
        ' data-letter="' + l + '" aria-pressed="' + (pressed ? "true" : "false") + '"' +
        (locked ? ' disabled' : '') + '>' + l + '</button>';
    }).join("") + '</div>';
  }

  function hint(s) {
    var h = s.frame && s.frame.hint;
    if (!h) return "";
    if (!s.hintShown) {
      return '<button class="btn buzz-hint-go" type="button" data-act="hint">' +
        esc(t("buzzer_hint_action")) + '</button>';
    }
    /* tabindex -1: not a stop in the Tab order, but somewhere for the focus the
       hint button had to land when the button goes (mount, render; AC-82). */
    return '<div class="panel buzz-hint" data-state="hint-shown" tabindex="-1">' + esc(h.text) + '</div>';
  }

  function liveScreen(s) {
    var paused_ = s.conn !== "attached";
    var sub = submission(s);
    return head(s, null) +
      letters(s, paused_) +
      (paused_
        ? paused(s)
        : '<p class="buzz-submit" data-state="' + sub.kind + '">' + esc(sub.text) +
          (sub.kind === "failed"
            ? ' <button class="btn buzz-retry" type="button" data-act="retry">' + esc(t("buzzer_save_retry")) + '</button>'
            : '') +
          '</p>') +
      (paused_ ? '' : hint(s));
  }

  function closedScreen(s) {
    return head(s, esc(t("buzzer_closed"))) +
      letters(s, true) +
      (s.conn !== "attached" ? paused(s) :
        '<p class="buzz-said">' + (s.saved
          ? bold(PQ.COPY.buzzer_closed_you_said, "X", s.saved)
          : esc(t("buzzer_closed_you_didnt"))) + '</p>');
  }

  function bold(template, name, value) {
    // Fill one placeholder with bold text: the string stays copy.js's own.
    var marker = "\u0000";
    var values = {};
    values[name] = marker;
    return esc(PQ.fill(template, values)).replace(marker, "<b>" + esc(value) + "</b>");
  }

  function itWas(correct) {
    // "✓ It was ‹Y›." with the glyph and the colour applied together (AC-40):
    // the copy's own ✓ is taken off and check.js puts it back with the class.
    var line = t("buzzer_reveal_it_was", { Y: correct }).replace(/^✓\s*/, "");
    return '<p class="buzz-itwas">' + PQ.correctHtml(line, { srLabel: "" }) + '</p>';
  }

  /* The glyph that marks the reader's own row, keyed by the *you said ‹X›*
     line beneath the bars. A glyph and a label, no colour (AC-40); never ✓,
     which is the answer's, and never a mark against anyone (AC-94). */
  var YOURS = "●";

  /* PQ-37 (HC-0): the wall's five bars, scaled to the phone. `split` is the
     wall's `split.bars` as a bare array, filled by the same server function,
     so the phone and the wall show one split. The markup is wall.js's
     barsHtml, class for class; at reveal the correct letter goes through
     check.js exactly as the wall's does. The reader's own letter is known
     here and never sent (AC-58). */
  function barsHtml(bars, correct, yours) {
    return '<div class="bars">' + bars.map(function (b) {
      var mine = b.letter === yours;
      var letter = b.letter === correct
        ? PQ.correctHtml(b.letter, { className: "letter" })
        : '<span class="letter">' + esc(b.letter) + "</span>";
      return '<div class="bar-row' + (mine ? " is-yours" : "") + '" data-bar="' + attr(b.letter) + '">' +
        '<span class="bar-yours" aria-hidden="true">' + (mine ? YOURS : "") + "</span>" + letter +
        '<span class="bar-track"><span class="bar-fill" style="width:' + Number(b.percent) + '%"></span></span>' +
        '<span class="bar-n">' + Number(b.count) + " · " + Number(b.percent) + "%</span></div>";
    }).join("") + "</div>" +
      (yours ? '<p class="buzz-said buzz-yours"><span aria-hidden="true">' + YOURS + "</span> " +
        bold(PQ.COPY.buzzer_closed_you_said, "X", yours) + "</p>" : "");
  }

  /* split, work, reveal: at reveal, ✓ It was ‹Y›, and *You didn't answer.*
     above it for a phone that holds no answer; then, from the split on, the
     room's five bars beneath whatever lines the phase has. */
  function countScreen(s) {
    var body = "";
    var reveal = s.frame.phase === "reveal";
    if (reveal) {
      body = (s.saved ? "" : '<p class="buzz-said">' + esc(t("buzzer_noanswer_count_one")) + '</p>') +
        itWas(s.frame.correct);
    }
    var bars = Array.isArray(s.frame.split) && s.frame.split.length
      ? '<div class="buzz-split">' + barsHtml(s.frame.split, reveal ? s.frame.correct : null, s.saved) + '</div>'
      : '';
    return head(s, null) +
      (s.conn !== "attached" ? paused(s) : '') +
      '<div class="buzz-verdict"><div>' + body + bars + '</div></div>';
  }

  function idleScreen(s) {
    return head(s, null) +
      (s.conn !== "attached" ? paused(s) : '') +
      '<div class="buzz-verdict"><div><div class="vb" aria-hidden="true">↑</div>' +
      '<p class="meta buzz-after">' + esc(t("buzzer_idle")) + '</p></div></div>';
  }

  function releasedScreen(s) {
    return head(s, null) + '<div class="buzz-verdict"></div>';
  }

  function view(s) {
    if (s.screen === "join") return joinScreen(s);
    if (!s.frame) return head(s, null) + paused(s);
    switch (s.frame.phase) {
      case "idle": return idleScreen(s);
      case "live": return liveScreen(s);
      case "closed": return closedScreen(s);
      case "split": case "work": case "reveal": return countScreen(s);
      case "released": return releasedScreen(s);
      default: return head(s, null) + paused(s);
    }
  }

  // --------------------------------------------------------------------------
  // The shell. Everything with a side effect lives below this line.
  // --------------------------------------------------------------------------

  var STORE_KEY = "popquiz.buzzer";

  function readStore(storage) {
    try {
      var raw = storage && storage.getItem(STORE_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch (e) { return null; }
  }

  function writeStore(storage, value) {
    try {
      if (!storage) return;
      if (value) storage.setItem(STORE_KEY, JSON.stringify(value));
      else storage.removeItem(STORE_KEY);
    } catch (e) { /* private mode: the tab just forgets on reload */ }
  }

  /* io: {fetch, WebSocket, storage, location, setTimeout, announce} — each
     injectable so the whole loop runs under test. */
  function mount(el, io) {
    io = io || {};
    var state = initial();
    var socket = null;
    var backoff = 500;

    /* AC-82: every event repaints the whole screen, and the control that had
       the focus goes with it — a keyboard user who pressed a letter would be
       left on <body>, the next Tab back at the top. So the focus is remembered
       by what the control does and given back to its twin on the new screen.
       A control that no longer exists hands it on: retry to the letter it was
       retrying, the hint button to the hint it showed. The key outlives a
       paint only when that paint could not honour it (the control was
       disabled or gone); a reader who clicked elsewhere is left there. */
    var focusKey = null;
    var owed = false;

    function keyOf(node) {
      if (!node || !node.getAttribute) return null;
      if (node.id === "pq-code") return "code";
      var letter = node.getAttribute("data-letter");
      if (letter) return "letter:" + letter;
      var act = node.getAttribute("data-act");
      return act ? "act:" + act : null;
    }

    function focusTarget(key) {
      if (!key || !el.querySelector) return null;
      var found = key === "code" ? el.querySelector("#pq-code")
        : el.querySelector("[data-" + key.replace(":", '="') + '"]');
      if (found && !found.disabled) return found;
      if (key === "act:hint") return el.querySelector(".buzz-hint");
      if (key === "act:retry" && state.lastTried) return focusTarget("letter:" + state.lastTried);
      return null;
    }

    function render() {
      var doc = el.ownerDocument;
      var active = doc && doc.activeElement;
      if (active && el.contains && el.contains(active)) focusKey = keyOf(active);
      else if (!owed || (active && active !== doc.body)) focusKey = null;   // the reader moved it
      el.innerHTML = view(state);
      var input = el.querySelector && el.querySelector("#pq-code");
      if (input && state.refusal && typeof input.focus === "function") { input.focus({ preventScroll: true }); owed = false; return; }
      var target = focusTarget(focusKey);
      if (target && target !== doc.activeElement && typeof target.focus === "function") target.focus({ preventScroll: true });
      owed = !!focusKey && !target;
    }

    function dispatch(event) {
      var r = reduce(state, event);
      state = r.state;
      render();
      r.effects.forEach(run);
    }

    function wsUrl() {
      var loc = io.location;
      var scheme = loc.protocol === "https:" ? "wss:" : "ws:";
      return scheme + "//" + loc.host + "/rooms/" + encodeURIComponent(state.roomId) + "/ws/buzzer";
    }

    function json(res) {
      return res.text().then(function (text) {
        var body = null;
        try { body = text ? JSON.parse(text) : null; } catch (e) { body = null; }
        return { status: res.status, body: body };
      });
    }

    function run(effect) {
      switch (effect.do) {
        case "join":
          io.fetch("/join", {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ code: effect.code })
          }).then(json).then(function (r) {
            dispatch({ type: "joined", status: r.status, body: r.body });
          }, function () {
            dispatch({ type: "joined", status: 0, body: null });
          });
          break;
        case "put":
          io.fetch("/rooms/" + encodeURIComponent(state.roomId) + "/answer", {
            method: "PUT",
            headers: { "content-type": "application/json", authorization: "Bearer " + state.token },
            body: JSON.stringify({ letter: effect.letter })
          }).then(json).then(function (r) {
            dispatch({ type: "answered", status: r.status, body: r.body });
          }, function () {
            dispatch({ type: "answered", status: 0, body: null });
          });
          break;
        case "attach": {
          if (socket) { socket.onclose = null; try { socket.close(); } catch (e) {} }
          var ws = new io.WebSocket(wsUrl());
          var token = state.token;
          socket = ws;
          ws.onopen = function () { ws.send(JSON.stringify({ t: "attach", token: token })); };
          ws.onmessage = function (m) {
            var frame;
            try { frame = JSON.parse(m.data); } catch (e) { return; }
            if (frame && frame.t === "state") { backoff = 500; dispatch({ type: "frame", frame: frame }); }
          };
          ws.onclose = function (e) {
            if (socket !== ws) return;
            socket = null;
            dispatch({ type: "closed", code: e && e.code });
          };
          break;
        }
        case "disconnect":
          if (socket) { var s = socket; socket = null; s.onclose = null; try { s.close(); } catch (e) {} }
          break;
        case "reconnect":
          io.setTimeout(function () {
            if (state.screen === "room" && !socket) run({ do: "attach" });
          }, backoff);
          backoff = Math.min(backoff * 2, 5000);
          break;
        case "store":
          writeStore(io.storage, effect.value);
          break;
        case "announce":
          if (io.announce) io.announce(effect.text);
          break;
      }
    }

    el.addEventListener("click", function (e) {
      var target = e.target && e.target.closest ? e.target.closest("[data-letter],[data-act]") : null;
      if (!target) return;
      var letter = target.getAttribute("data-letter");
      if (letter) return dispatch({ type: "tap", letter: letter });
      var act = target.getAttribute("data-act");
      if (act === "hint") dispatch({ type: "hint" });
      else if (act === "retry") dispatch({ type: "retry" });
    });
    el.addEventListener("submit", function (e) {
      e.preventDefault();
      var input = el.querySelector("#pq-code");
      dispatch({ type: "submit", code: input ? input.value : "" });
    });

    render();
    dispatch({ type: "boot", search: io.location && io.location.search, stored: readStore(io.storage) });
    return { dispatch: dispatch, state: function () { return state; } };
  }

  PQ.Buzzer = {
    LETTERS: LETTERS,
    REFUSAL_KEY: REFUSAL_KEY,
    PHASE_ANNOUNCE: PHASE_ANNOUNCE,
    codeFromSearch: codeFromSearch,
    refusalMessage: refusalMessage,
    initial: initial,
    reduce: reduce,
    submission: submission,
    view: view,
    mount: mount
  };

  if (root.document && root.document.getElementById && !root.__POPQUIZ_NO_MOUNT__) {
    var boot = function () {
      var el = root.document.getElementById("buzzer");
      if (!el) return;
      mount(el, {
        fetch: root.fetch && root.fetch.bind(root),
        WebSocket: root.WebSocket,
        storage: (function () { try { return root.sessionStorage; } catch (e) { return null; } })(),
        location: root.location,
        setTimeout: root.setTimeout.bind(root),
        announce: PQ.announce
      });
    };
    if (root.document.readyState === "loading") root.document.addEventListener("DOMContentLoaded", boot);
    else boot();
  }
})(typeof window !== "undefined" ? window : globalThis);
