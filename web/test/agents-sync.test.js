// AGENTS.md is a plain copy of CLAUDE.md, not a symlink: nothing verifies that
// every tool which reads AGENTS.md follows symlinks. This holds the two equal
// until that is established, so the copy cannot go stale again.
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..', '..');
const read = (name) => fs.readFileSync(path.join(root, name), 'utf8');

test('AGENTS.md is a regular file equal to CLAUDE.md', () => {
  assert.ok(!fs.lstatSync(path.join(root, 'AGENTS.md')).isSymbolicLink());
  assert.equal(read('AGENTS.md'), read('CLAUDE.md'));
});
