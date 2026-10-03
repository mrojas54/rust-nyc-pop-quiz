# web

The three surfaces and the public page: the wall on the projector, the buzzer in
everyone's hand, the host's phone, and take-it-home afterwards.

Plain HTML, CSS and JS, ported one-to-one from `prototypes/`. No framework and
no build step — the prototype *is* the design, and it is already plain JS
(BUILDPLAN D-B).

## Layout

| Directory | What goes in it | Ticket |
|---|---|---|
| `shared/` | Tokens, fonts, the source well, syntax colour, the trace renderer, the type model | T-02 |
| `wall/` | Seven phases on the 1120×630 canvas | T-05, and T-26's static fallback |
| `buzzer/` | Join, answer, the private hint, the released line | T-06 |
| `host/` | One screen per phase, one primary action | T-07 |
| `home/` | Take it home, rebuilt at each release | T-12 |

`shared/` is built **first and alone**, because three tickets consume it.

## Tests

    just test          # runs this suite along with the room's and the pipeline's
    node --test web/test/

Node's built-in test runner, which means **no `package.json`, no
`node_modules`, and nothing to install**. Keeping the web suite free of a
dependency tree is what lets `just test` stay hermetic without a lockfile for a
third ecosystem.

### Buzzer join recovery

The join form ignores repeated submits while a request is pending. Each join
targets a ten-second deadline, including response-body reading. Background
tabs may delay that timer until JavaScript resumes. Network failures,
unrecognized responses and timeouts show recovery guidance linked to the room
code field; the submitted code stays available for retry. The browser aborts a
timed-out fetch when AbortController is available. Late responses are ignored
so they cannot attach an old room after a retry. An unconfirmed response can
still leave a server session consuming capacity, and retrying can create another;
join is not idempotent. The recovery message recommends waiting before retry but
does not enforce server Retry-After.

For a local browser check, open
`/web/test/a11y/browser.html?surface=buzzer` from a server at the repository root.
Use **fail next join** or **stall next join**, enter a code, and submit. The stall
should disable the join button until the deadline, then allow another attempt.
These controls belong to the fixture harness, outside the product interface.
[Verification evidence](test/verification/buzzer-join.md) records the scope and
limits of the checks.

### A note for T-02

There is deliberately no `"type": "module"` anywhere here. The prototype loads
`_shared/data.js` and `_shared/proto.js` through plain `<script src>` and they
communicate through globals, so the port stays classic scripts and the browser
needs no module graph.

That does mean `node --test` cannot `require()` or `import` a ported file to get
at its functions. Load it with `node:vm` into a context you build, which also
gives you somewhere to put the `window`/`document` stubs a DOM-touching function
will want. Reaching for a bundler or jsdom to avoid that is a bigger mechanism
than the problem needs.

## Accessibility

SPEC §10, AC-82…AC-86 (with AC-40, AC-48, AC-102). Two halves: a suite for
what the markup and the stylesheets decide, and one pass in a browser for what
only a layout engine and a screen reader can show.

### `just a11y` — what it proves

    just a11y          # the suite, then the surface × screen × criterion matrix and every contrast ratio

`web/test/a11y.test.js`, on node's runner with no dependency, like every suite
here. It is hermetic and takes about a second, so `just test` runs it too
(through the `test-web` glob, silently); `just test-full` runs `just a11y`.

It renders every surface at every phase from fixtures: the wall at every phase
and step (and the unanimous reveal), the static fallback at every `{phase,
step}` its keys reach, every screen the buzzer's reducer reaches (join, the six
refusals, each phase, saving / saved / failed, the hint, paused), the host's
seven phases plus sign-in and create, and take-it-home at every step of three
snapshots. `web/test/a11y/dom.js` builds the markup into a tree and runs the
real stylesheets' cascade over it, so each colour, size and outline is the one
the CSS gives that element; `a11y/wcag.js` is WCAG 2.1's relative luminance and
contrast ratio, written out.

| Criterion | What the suite asserts |
|---|---|
| AC-82 | Every element that acts on a click is a native control; none is out of the Tab order or reordered; each has a name; each draws a `:focus-visible` outline of 2 px or more at 3:1 against what is behind it. The wall has no control (AC-79): its one Tab stop is the reading region. The static fallback reaches every phase and step from the keyboard. The buzzer, host and take-it-home keep the focus on the same control across a repaint (driven through their own `mount` / `boot`). |
| AC-83 | Each state change says its §11 string through `PopQuiz.announce` and nothing else: the wall's six phase entries (and the static fallback's, identical), the buzzer's ten strings, the host's seven labels (the first on opening the room), take-it-home's trace steps. `idle` and trace steps on the wall say nothing. The hint is said by the buzzer alone and makes no request (AC-48). |
| AC-84 | Every element with visible text, on every screen: 4.5:1, or 3:1 at 24 px / 18.66 px bold. Plus the ✓, the join field's edge and the trace highlight's edge at 3:1. Dimmed trace lines and the split's bars are printed but not held: the dim *is* the trace's signal, and `n · p%` carries each bar's value. |
| AC-85 | Every buzzer, host and take-it-home control asks for `min-height` and `min-width` of at least `--touch-target` (44 px). |
| AC-86 | No stylesheet animates or transitions outside a `prefers-reduced-motion: reduce` block; each surface carries that block; no script animates. Each state's static cue (the words, the glyph, the frame) is present. |

AC-84's *dim-room* half has no built presentation (no dark mode in v1,
DESIGN.md; touchpoint T-16). The suite proves the one there is.

### The browser pass — what it proves, and how to repeat it

A tree with no layout cannot show the real Tab order, the ring as drawn, the
rendered size of a target, the region actually speaking, or the OS
reduced-motion setting. `web/test/a11y/browser.html` puts each surface on
screen with its real markup, stylesheets and scripts. A scripted stand-in plays
the room: the buzzer's socket and answer route, and the host's. The wall is the
real static fallback on q3's bake, and take-it-home the real page on q3's
snapshot.

    python3 -m http.server 8766 --bind 127.0.0.1      # from the repository root
    open http://127.0.0.1:8766/web/test/a11y/browser.html?surface=buzzer   (wall, host, home)

`window.a11yProbe()` in the page returns what the engine rendered: every
focusable element's box, the focused one's computed outline, the live region's
text, whether the page scrolls sideways, and the motion setting.

1. **Buzzer.** Type a code and join. Press *attach (idle)* then *next phase* in
   the harness bar. Tab to a letter and press Enter or Space. The focus stays
   on that letter through *saving…* and *saved*, and the region reads
   *Saving.* then *Saved, ‹X›.* Press *fail next save*, pick another letter,
   then Tab to *Try again* and press it. The focus lands on the letter it
   retries. *Show me a hint* hands its focus to the hint, and the region says
   *Hint shown, only to you.* Step through every phase with *next phase*: each
   one's §11 string, no sideways scroll, and every target at least 44 × 44.
2. **Host.** Tab to the primary action and press it at each phase. The focus
   moves on to the next action, and the region says each phase label,
   starting with *before the question* on load. In `work`, press → at the last
   step and the focus moves to ←.
3. **Wall.** Space through the phases, then Esc and the arrows. The region
   says the same six strings as the live wall, and a step says nothing. The
   reading region is the only Tab stop.
4. **Take it home.** Tab to →, then press it. The focus stays, and the region
   says *Step N of M.* with the step's words. At the last step the focus moves
   to ←. The `-Vv` block takes the focus and scrolls with the arrow keys.
5. **By hand, in a desktop browser.** Press real Tab: the ring is visible on
   every stop, in the dark amber. Run each page once with VoiceOver
   (⌘F5) and hear the strings above. Turn on *Reduce motion* (System Settings
   → Accessibility → Display): nothing on any surface animates either way,
   and `a11yProbe().animations` is 0.

The suite reads the default state of each rule. It does not check `:hover` or
`:active` colours, or any `@media` variant. Those are this pass's too: on
hover, the primary buttons turn white on the hover amber.

The c11 browser drives steps 1–4 (`c11 browser <surface> eval "JSON.stringify(a11yProbe())"`).
Its `press` sends a synthetic key, which moves no focus in WKWebView, so the
real Tab walk and VoiceOver are step 5.
