// The web suite, run by Node's built-in test runner: `node --test web/test/`.
//
// No package.json, no node_modules, no network. The prototype's JS is classic
// scripts with globals (C-projector-first.html loads _shared/data.js and
// _shared/proto.js through plain <script src>), so web/ stays classic too and
// nothing here declares "type": "module". T-02 ports proto.js into shared/.
//
// Today this asserts the layout the later tickets are written against — that
// each surface has its directory and that nothing has quietly merged two of
// them. It is a real assertion, not a placeholder that always passes: delete
// web/buzzer/ and this suite goes red.

const test = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');

const WEB = path.join(__dirname, '..');

// Each surface, and the ticket that fills it.
const SURFACES = {
  shared: 'T-02 — tokens, fonts, the source well, colour, trace, type model',
  wall: 'T-05 — seven phases on the projector; T-26 adds the static fallback',
  buzzer: 'T-06 — join, answer, the private hint, the released line',
  host: 'T-07 — one screen per phase, one primary action',
  home: 'T-12 — take it home, rebuilt at release',
};

test('every surface has a directory for the ticket that fills it', () => {
  for (const [name, owner] of Object.entries(SURFACES)) {
    const dir = path.join(WEB, name);
    assert.ok(
      fs.existsSync(dir) && fs.statSync(dir).isDirectory(),
      `web/${name}/ is missing — ${owner}`
    );
  }
});

test('no surface directory has been added without being written down here', () => {
  const found = fs
    .readdirSync(WEB, { withFileTypes: true })
    .filter((e) => e.isDirectory() && e.name !== 'test')
    .map((e) => e.name)
    .sort();

  assert.deepStrictEqual(
    found,
    Object.keys(SURFACES).sort(),
    'web/ has a directory this suite does not know about. Add it to SURFACES ' +
      'with the ticket that owns it, so the layout stays readable.'
  );
});
