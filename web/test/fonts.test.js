// Font loading is the UI's largest static payload. Check the browser-selected
// files, rather than the originals retained for provenance and old URLs.
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const shared = path.join(__dirname, '..', 'shared');

test('each face prefers a valid WOFF2 payload at least 50% smaller than its TTF', () => {
  const css = fs.readFileSync(path.join(shared, 'fonts.css'), 'utf8');
  const faces = css.match(/@font-face\s*\{[^}]+\}/g);
  assert.equal(faces.length, 4);
  for (const face of faces) {
    const sources = [...face.matchAll(/url\("([^"]+)"\)\s*format\("([^"]+)"\)/g)];
    assert.equal(sources.length, 2, 'WOFF2 first, original TTF fallback');
    assert.equal(sources[0][2], 'woff2');
    assert.equal(sources[1][2], 'truetype');
    const compressed = fs.readFileSync(path.join(shared, sources[0][1]));
    const original = fs.readFileSync(path.join(shared, sources[1][1]));
    assert.equal(compressed.toString('ascii', 0, 4), 'wOF2');
    assert.equal(compressed.readUInt32BE(8), compressed.length);
    assert.ok(compressed.length < original.length / 2, sources[0][1]);
    assert.match(face, /font-display:\s*swap/);
  }
});
