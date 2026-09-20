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
