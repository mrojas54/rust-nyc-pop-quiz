/* ===========================================================================
   The host phone (T-07) — SPEC §4's host column, §6, §8.1, §8.2, §11.

   One page serves two addresses:

     /host?question=<id>                the first screen, signed out: *Sign in
                                        with Discord*, a link to
                                        /auth/discord?question=<id> (T-10).
     /host?question=<id>#<organizer     the first screen, signed in: Discord's
      session>                          callback lands here. *Create a room*
                                        presents the organizer session as a
                                        bearer; the room asks Discord whether
                                        its owner holds the host role (§8). A
                                        401 means the session is gone: the page
                                        forgets it and offers sign-in again.
                                        403 shows the room's reason (wrong
                                        server, wrong role, AC-70).
     /host/<room_id>#<host session>     one screen per phase. This address IS
                                        the resume link (rooms.rs builds
                                        host_resume_url this way): a refresh
                                        keeps the fragment, and opening it on a
                                        second device attaches that device to
                                        the same room (AC-50). The session never
                                        rotates, and a fragment never reaches a
                                        server log.

   What the page reads: the host projection (GET /rooms/{id}/host) and the host
   socket (/rooms/{id}/ws/host), nothing else. It never asks for the wall's or
   the buzzer's payload, so it cannot preview an answer the room has not
   revealed (AC-47, G-3).

   Every string on the screen is copy.js's (SPEC §11) or text the room sent
   (the step's words, the three beats). Nothing
   is authored here. Errors show the server's `reason` or the HTTP status line.

   Classic script on window.PopQuiz, like web/shared (web/README.md). The pure
   parts — parseLocation, render, renderCreate, backoff — are what
   web/test/host.test.js exercises; boot() wires them to the browser.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* The eight host actions (AC-45), by the route slug the room uses
     (room/src/phase.rs HostAction::slug), to their §11 string. */
  var ACTIONS = {
    "create": "host_action_create",
    "put-on-screen": "host_action_put_on_screen",
    "close-answers": "host_action_close",
    "show-split": "host_action_show_split",
    "walk-it": "host_action_walk",
    "reveal": "host_action_reveal",
    "release": "host_action_release",
    "run-it-again": "host_action_run_again"
  };

  /* The two trace steps, `←` and `→`: in `work` and `reveal` only, and not
     phase transitions (SPEC §4). */
  var STEPS = { "step-back": "←", "step-forward": "→" };

  /* Close codes after which reconnecting cannot help (room/src/ws.rs): the
     credential was refused, or the room is gone. */
  var TERMINAL_CLOSES = [4401, 4404];

  function actionLabel(slug) {
    if (!Object.prototype.hasOwnProperty.call(ACTIONS, slug)) {
      throw new Error("host: no host action " + JSON.stringify(slug) + " (AC-45)");
    }
    return PQ.t(ACTIONS[slug]);
  }

  function decode(s) {
    try { return decodeURIComponent(s); } catch (e) { return s; }
  }

  /* Where this page is. `loc` is anything with pathname, search and hash. */
  function parseLocation(loc) {
    var hash = String(loc.hash || "").replace(/^#/, "");
    var secret = hash ? decode(hash) : null;
    var path = String(loc.pathname || "").replace(/\/+$/, "");
    var m = /^\/host\/([^\/]+)$/.exec(path);
    if (m) return { mode: "room", roomId: decode(m[1]), session: secret };
    var q = /(?:^\?|&)question=([^&]*)/.exec(String(loc.search || ""));
    return { mode: "create", token: secret, question: q ? decode(q[1]) : null };
  }

  /* Reconnect delay in ms after `attempt` failures: 500, 1000, 2000, 4000,
     then 8000 for as long as it takes. */
  function backoff(attempt) {
    var n = Math.max(0, attempt | 0);
    return Math.min(8000, 500 * Math.pow(2, Math.min(n, 4)));
  }

  function isTerminalClose(code) {
    return TERMINAL_CLOSES.indexOf(code) >= 0;
  }

  var esc = function (s) { return PQ.escapeHtml(s); };

  function statusHtml(ui) {
    return '<p class="host-status meta" role="status">' + (ui && ui.status ? esc(ui.status) : "") + "</p>";
  }

  function primaryHtml(slug, ui) {
    return '<button type="button" class="btn btn-primary host-primary" data-primary="' + PQ.escapeAttr(slug) + '"' +
      (ui && ui.busy ? " disabled" : "") + ">" + esc(actionLabel(slug)) + "</button>";
  }

  /* Every host screen's heading, whatever the phase: the design system's
     wordmark, "Host" beneath. The phase itself is announced through the live
     region (HOST_PHASE_LABEL), not shown. */
  function titleHtml(phase) {
    return '<h1 class="host-h" data-phase="' + phase + '">' + esc(PQ.t("title_wordmark")) + "</h1>" +
      '<p class="host-role">' + esc(PQ.t("host_title_role")) + "</p>";
  }

  function codeHtml(code) {
    return '<div class="panel host-code-panel"><div class="meta">' + esc(PQ.t("buzzer_join_label")) + "</div>" +
      '<div class="host-code">' + esc(code) + "</div></div>";
  }

  function stepHtml(step, ui) {
    var dots = "";
    for (var i = 0; i < step.m; i++) {
      dots += "<i" + (i < step.at ? ' class="past"' : i === step.at ? ' class="on"' : "") + "></i>";
    }
    var btn = function (slug, enabled) {
      return '<button type="button" class="btn host-step-btn" data-step="' + slug + '" aria-label="' +
        PQ.escapeAttr(STEPS[slug]) + '"' + (enabled && !(ui && ui.busy) ? "" : " disabled") + ">" +
        esc(STEPS[slug]) + "</button>";
    };
    return '<div class="trace host-trace">' +
      '<div class="trace-note"><span class="step-n">' +
      esc(PQ.t("wall_trace_step", { N: step.at + 1, M: step.m })) + "</span>" + esc(step.note) + "</div>" +
      '<div class="trace-nav">' + btn("step-back", step.can_back) + btn("step-forward", step.can_forward) +
      '<span class="trace-dots" aria-hidden="true">' + dots + "</span></div></div>";
  }

  /* The three beats (§4.5): human-reviewed prose, so the human provenance
     marker, distinct from the receipt's machine one (AC-74). The middle beat's
     heading is the room's: *Why ‹n› of us said ‹X›* for the most-chosen
     incorrect option, or *Why nobody said anything else* with no text. */
  function readAloudHtml(r) {
    var beat = function (cls, heading, text) {
      return '<div class="beat' + cls + '"><h3>' + esc(heading) + "</h3>" +
        (text ? "<p>" + esc(text) + "</p>" : "") + "</div>";
    };
    return '<section class="panel by-human host-read-aloud" data-provenance="human">' +
      '<span class="provenance human">✎</span>' +
      '<h2 class="host-read-h">' + esc(PQ.t("host_reveal_read_aloud")) + "</h2>" +
      beat("", PQ.t("host_reveal_beat_what"), r.what && r.what.text) +
      beat(" beat-company", r.middle.heading, r.middle.text) +
      beat("", PQ.t("host_reveal_beat_remember"), r.takeaway && r.takeaway.text) +
      "</section>";
  }

  /* AC-46, in the client's form: one word, a colon, a number. How many are
     in the room before the question is live; how many answered after. */
  function countHtml(p) {
    var line = p.phase === "idle"
      ? PQ.t("count_joined", { n: p.present })
      : PQ.t("count_answered", { n: typeof p.answered === "number" ? p.answered : 0 });
    return '<div class="host-count">' + esc(line) + "</div>";
  }

  /* One screen for one host payload (view::HostPayload). `ui`: {status, busy}. */
  function render(p, ui) {
    var phase = PQ.assertPhase(p.phase);
    var html = titleHtml(phase) + '<div class="stack">';
    // The code while joining is still possible: every phase but `released`.
    if (phase !== "released") html += codeHtml(p.code);
    if ((phase === "work" || phase === "reveal") && p.step) html += stepHtml(p.step, ui);
    if (phase === "reveal" && p.read_aloud) html += readAloudHtml(p.read_aloud);
    html += primaryHtml(p.primary.action, ui) + statusHtml(ui) + "</div>" + countHtml(p);
    return html;
  }

  /* The first screen before a room exists: *Create a room*. */
  function renderCreate(ui) {
    return titleHtml("idle") +
      '<div class="stack">' + primaryHtml("create", ui) + statusHtml(ui) + "</div>";
  }

  /* The first screen signed out (T-10): one way on, to Discord's consent
     screen through the room's /auth/discord, carrying the question. A link,
     not a button: it is a navigation, and it works with no script after this. */
  function signInHref(question) {
    return "/auth/discord?question=" + encodeURIComponent(question || "");
  }

  function renderSignIn(ui, question) {
    return titleHtml("idle") +
      '<div class="stack"><a class="btn btn-primary host-primary" data-sign-in href="' +
      PQ.escapeAttr(signInHref(question)) + '">' + esc(PQ.t("host_action_sign_in")) + "</a>" +
      statusHtml(ui) + "</div>";
  }

  /* ------------------------------------------------------------------------
     The browser half.
     ------------------------------------------------------------------------ */

  var STORE_KEY = "popquiz.host.create";

  function storeGet(win) {
    try { return JSON.parse(win.sessionStorage.getItem(STORE_KEY) || "null"); } catch (e) { return null; }
  }

  function storeSet(win, v) {
    try { win.sessionStorage.setItem(STORE_KEY, JSON.stringify(v)); } catch (e) { /* private mode */ }
  }

  function statusLine(res, body) {
    if (body && typeof body.reason === "string") return body.reason;
    return res.status + (res.statusText ? " " + res.statusText : "");
  }

  function roomPath(id, session) {
    return "/host/" + encodeURIComponent(id) + "#" + session;
  }

  function boot(win) {
    var el = win.document.getElementById("host");
    var where = parseLocation(win.location);
    var ui = { status: null, busy: false };

    function post(url, bearer, body) {
      var headers = {};
      if (bearer) headers.Authorization = "Bearer " + bearer;
      if (body) headers["Content-Type"] = "application/json";
      return win.fetch(url, { method: "POST", headers: headers, body: body ? JSON.stringify(body) : undefined })
        .then(function (res) {
          return res.text().then(function (t) {
            var json = null;
            try { json = t ? JSON.parse(t) : null; } catch (e) { json = null; }
            return { res: res, body: json };
          });
        });
    }

    /* AC-82: every paint replaces the screen, and the control that had the
       focus with it. The focus is remembered by role — the primary action,
       or a step button — and given back on the new screen. The primary
       action changes with the phase and is disabled while it is in flight,
       so the key outlives the paint that could not honour it: pressing
       *Close answers* from the keyboard lands on *Show the room its split*. */
    var focusKey = null;
    var owed = false;   // a paint could not honour the key; the next one may

    function keyOf(node) {
      if (!node || !node.getAttribute) return null;
      if (node.getAttribute("data-primary") !== null) return "primary";
      var step = node.getAttribute("data-step");
      return step ? "step:" + step : null;
    }

    function focusTarget(key) {
      if (!key || !el.querySelector) return null;
      if (key === "primary") return el.querySelector("[data-primary]:not([disabled])");
      var slug = key.slice(5);
      var other = slug === "step-back" ? "step-forward" : "step-back";
      return el.querySelector('[data-step="' + slug + '"]:not([disabled])') ||
        el.querySelector('[data-step="' + other + '"]:not([disabled])');
    }

    function repaint(html) {
      var active = win.document.activeElement;
      if (active && el.contains && el.contains(active)) focusKey = keyOf(active);
      else if (!owed || (active && active !== win.document.body)) focusKey = null;   // the reader moved it
      el.innerHTML = html;
      var target = focusTarget(focusKey);
      if (target && target !== win.document.activeElement && typeof target.focus === "function") target.focus();
      owed = !!focusKey && !target;
    }

    if (where.mode === "create") {
      var paintCreate = function () {
        repaint(where.token ? renderCreate(ui) : renderSignIn(ui, where.question));
      };
      el.addEventListener("click", function (ev) {
        var b = ev.target.closest && ev.target.closest("[data-primary]");
        if (!b || ui.busy) return;
        ui.busy = true; ui.status = null; paintCreate();
        post("/rooms", where.token, { question_id: where.question || "" }).then(function (r) {
          if (r.res.status === 201 && r.body) {
            storeSet(win, { token: where.token, question: where.question });
            win.location.replace(r.body.host_resume_url || roomPath(r.body.id, r.body.host_session));
            return;
          }
          if (r.res.status === 401) {
            // The organizer session is unknown or expired (T-10): sign in again.
            where.token = null;
            win.history.replaceState(null, "", "/host?question=" + encodeURIComponent(where.question || ""));
            ui.busy = false; ui.status = null; paintCreate();
            return;
          }
          ui.busy = false; ui.status = statusLine(r.res, r.body); paintCreate();
        }, function (e) {
          ui.busy = false; ui.status = String(e && e.message || e); paintCreate();
        });
      });
      paintCreate();
      return;
    }

    var id = where.roomId;
    var session = where.session;
    var payload = null;
    var revision = -1;
    var attempt = 0;
    var socketOpen = false;

    function paint() {
      repaint(payload ? render(payload, ui) : statusHtml(ui));
    }

    function take(next, rev) {
      if (typeof rev === "number") {
        if (rev < revision) return;
        revision = rev;
      }
      var before = payload && payload.phase;
      payload = next;
      /* The first payload too (T-13): a host who opens or resumes a room hears
         which phase it is in, or `before the question` is never said at all. */
      var label = PQ.HOST_PHASE_LABEL[next.phase];
      if (before !== next.phase && label) PQ.announce(label);
      paint();
    }

    function act(slug) {
      ui.busy = true; ui.status = null; paint();
      var done = function (status) { ui.busy = false; ui.status = status; paint(); };
      if (slug === "run-it-again") {
        var stored = storeGet(win);
        if (!stored || !stored.token) { win.location.assign("/host"); return; }
        post("/rooms/" + encodeURIComponent(id) + "/run-it-again", stored.token, { question_id: stored.question || "" })
          .then(function (r) {
            if (r.res.status === 201 && r.body) {
              win.location.assign(r.body.host_resume_url || roomPath(r.body.id, r.body.host_session));
              return;
            }
            // T-10: the organizer session lapsed; sign in again, then create.
            if (r.res.status === 401) { win.location.assign(signInHref(stored.question)); return; }
            done(statusLine(r.res, r.body));
          }, function (e) { done(String(e && e.message || e)); });
        return;
      }
      post("/rooms/" + encodeURIComponent(id) + "/" + slug, session).then(function (r) {
        if (r.res.ok && r.body) {
          // The response carries no revision, so while the socket is up it is
          // the socket's frame for this change (pushed right after it) that
          // paints; taking the body could paint over a newer frame from the
          // other device. With no socket, the body is all there is.
          ui.busy = false;
          if (socketOpen) paint(); else take(r.body);
          return;
        }
        done(statusLine(r.res, r.body));
      }, function (e) { done(String(e && e.message || e)); });
    }

    el.addEventListener("click", function (ev) {
      if (!ev.target.closest || ui.busy) return;
      var b = ev.target.closest("[data-primary],[data-step]");
      if (!b || b.disabled) return;
      act(b.getAttribute("data-primary") || b.getAttribute("data-step"));
    });

    function connect() {
      var scheme = win.location.protocol === "https:" ? "wss:" : "ws:";
      var ws = new win.WebSocket(scheme + "//" + win.location.host + "/rooms/" + encodeURIComponent(id) + "/ws/host");
      ws.onopen = function () { ws.send(JSON.stringify({ t: "attach", token: session })); };
      ws.onmessage = function (m) {
        var f;
        try { f = JSON.parse(m.data); } catch (e) { return; }
        if (f.t !== "state") return;
        attempt = 0;
        socketOpen = true;
        if (ui.status === PQ.t("buzzer_reconnecting")) ui.status = null;
        var rev = f.revision;
        delete f.t; delete f.revision;
        take(f, rev);
      };
      ws.onclose = function (ev) {
        socketOpen = false;
        if (isTerminalClose(ev.code)) { ui.status = String(ev.code); paint(); return; }
        ui.status = PQ.t("buzzer_reconnecting"); paint();
        win.setTimeout(connect, backoff(attempt++));
      };
    }

    if (!session) { ui.status = "401"; paint(); return; }
    win.fetch("/rooms/" + encodeURIComponent(id) + "/host", { headers: { Authorization: "Bearer " + session } })
      .then(function (res) {
        return res.text().then(function (t) {
          var body = null;
          try { body = t ? JSON.parse(t) : null; } catch (e) { body = null; }
          if (res.ok && body) { take(body); connect(); }
          else { ui.status = statusLine(res, body); paint(); }
        });
      }, function (e) { ui.status = String(e && e.message || e); paint(); connect(); });
  }

  PQ.host = {
    ACTIONS: ACTIONS,
    STEPS: STEPS,
    actionLabel: actionLabel,
    parseLocation: parseLocation,
    backoff: backoff,
    isTerminalClose: isTerminalClose,
    render: render,
    renderCreate: renderCreate,
    renderSignIn: renderSignIn,
    signInHref: signInHref,
    boot: boot
  };
})(typeof window !== "undefined" ? window : globalThis);
