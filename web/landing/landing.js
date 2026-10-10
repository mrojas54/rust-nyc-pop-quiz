/* ===========================================================================
   The front door — GET /.

   One headline, a way into a room and a way
   to last meetup's question. No room state, no fetch: both exits are links, so
   the page works the moment its own HTML and this script arrive.

   Every string is a web/shared/copy.js key (SPEC §11); none is typed here.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  function view() {
    var t = PQ.t;
    return (
      '<p class="land-mark">' + t("title_wordmark") + "</p>" +
      '<h1 class="land-title">' + t("wall_idle_title") + "</h1>" +
      '<nav class="land-go">' +
      '<a class="land-btn" href="/join">' + t("landing_join") + "</a>" +
      '<a class="land-last" href="/last">' + t("landing_last") + "</a>" +
      "</nav>" +
      '<p class="land-foot"><a href="/host">' + t("landing_host") + "</a></p>"
    );
  }

  function mount(doc) {
    var el = doc.getElementById("landing-copy");
    if (el) el.innerHTML = view();
    return el;
  }

  PQ.Landing = { view: view, mount: mount };
  if (root.document && !root.__POPQUIZ_NO_MOUNT__) mount(root.document);
})(typeof window !== "undefined" ? window : globalThis);
