/* ===========================================================================
   The front door — GET /.

   A header, one headline beside Ferris, a way into a room and a way to
   last meetup's question, and the host's way in pinned to the foot. No room state, no fetch: both exits are links, so
   the page works the moment its own HTML and this script arrive.

   Every string is a web/shared/copy.js key (SPEC §11); none is typed here.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* The three places the page's words go: the header, the copy column, Ferris's
     speech bubble and the footer. Ferris, between them, is drawn in index.html. */
  function parts() {
    var t = PQ.t;
    return {
      head: '<p class="land-mark">' + t("title_wordmark") + "</p>",
      copy:
        '<h1 class="land-title">' + t("wall_idle_title") + "</h1>" +
        '<nav class="land-go">' +
        '<a class="land-btn" href="/join">' + t("landing_join") + "</a>" +
        '<a class="land-last" href="/last">' + t("landing_last") + "</a>" +
        "</nav>",
      /* TODO: regenerate the koan automatically (see landing_koan in copy.js). */
      koan: t("landing_koan"),
      foot: '<a href="/host">' + t("landing_host") + "</a>"
    };
  }

  function mount(doc) {
    var p = parts();
    var ids = { "land-head": p.head, "land-copy": p.copy, "land-koan": p.koan, "land-foot": p.foot };
    var el = null;
    Object.keys(ids).forEach(function (id) {
      el = doc.getElementById(id);
      if (el) el.innerHTML = ids[id];
    });
    return el;
  }

  PQ.Landing = { parts: parts, mount: mount };
  if (root.document && !root.__POPQUIZ_NO_MOUNT__) mount(root.document);
})(typeof window !== "undefined" ? window : globalThis);
