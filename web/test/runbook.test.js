// The organizer runbook, docs/RUNBOOK.md (T-24).
//
// AC-62 and AC-63 are proven here: EVALUATION.md asks that `test` assert the
// two statements are present in the organizer docs. PQ-34 took them off the host
// screen (copy.test.js, REMOVED), so the runbook is now their only carrier, and
// the expected strings are copied from SPEC.md §8.1 rather than imported from
// copy.js, where they no longer exist.
//
// AC-90's documentary half: the host script says the segment is one question,
// scheduled last. The sentence is pinned, so a rewording is a deliberate change
// to this file.
//
// Then two rot guards, because a runbook that names a file or recipe that is
// gone sends an organizer looking for it on the night: every repository path in
// a code span exists, and every `just <recipe>` is a recipe in the justfile.
// And the Forbidden row (SPEC §11, AC-98) over the words the host says.

const test = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const { load } = require('./_load.js');

const REPO = path.join(__dirname, '..', '..');
const RUNBOOK = fs.readFileSync(path.join(REPO, 'docs', 'RUNBOOK.md'), 'utf8');
const JUSTFILE = fs.readFileSync(path.join(REPO, 'justfile'), 'utf8');

// Blockquote markers are dropped and whitespace collapsed, so a re-wrapped line
// still matches; every character of the sentence still has to be there.
const flat = (s) => s.replace(/^[ \t]*>[ \t]?/gm, '').replace(/\s+/g, ' ');

// SPEC.md §8.1, character for character.
const OPTIONS_PUBLIC =
  'Option text is public — the correct answer is always one of the five visible options.';
const HOST_CAN_INFER =
  "A host who reads Rust can work out the answer from the source; the host's not being shown it keeps the host honest, it is not a security guarantee.";

const LAST =
  'The Pop Quiz is one question, and it is scheduled last: after the last talk, with nothing after it, and the wrap-up releases the room.';

// The text under one `## ` heading, up to the next.
function section(heading) {
  const lines = RUNBOOK.split('\n');
  const start = lines.findIndex((l) => l === `## ${heading}`);
  assert.ok(start >= 0, `docs/RUNBOOK.md has no "## ${heading}" section`);
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((l) => l.startsWith('## '));
  return (end < 0 ? rest : rest.slice(0, end)).join('\n');
}

// Inline code spans, and the bodies of fenced blocks.
function codeSpans(text) {
  const fenced = [...text.matchAll(/^```[^\n]*\n([\s\S]*?)^```/gm)].map((m) => m[1]);
  const unfenced = text.replace(/^```[^\n]*\n[\s\S]*?^```/gm, '');
  const inline = [...unfenced.matchAll(/`([^`\n]+)`/g)].map((m) => m[1]);
  return { fenced, inline };
}

test('AC-62: the runbook says option text is public, verbatim from SPEC §8.1', () => {
  assert.ok(flat(RUNBOOK).includes(OPTIONS_PUBLIC),
    'docs/RUNBOOK.md must carry SPEC §8.1\'s first sentence exactly');
});

test('AC-63: the runbook says a Rust-reading host can infer the answer, verbatim from SPEC §8.1', () => {
  assert.ok(flat(RUNBOOK).includes(HOST_CAN_INFER),
    'docs/RUNBOOK.md must carry SPEC §8.1\'s second sentence exactly');
});

test('AC-90: the host script says one question, scheduled last', () => {
  assert.ok(flat(section('The host script')).includes(LAST),
    'the host script must open with the pinned one-question, scheduled-last sentence');
});

test('every repository path the runbook names exists', () => {
  const { inline } = codeSpans(RUNBOOK);
  const looksLikePath = (s) =>
    /\.(md|json|py|rs|js|html)$/.test(s) || /^(mvp|docs|room|pipeline|web|bank)\//.test(s);
  const checked = [];
  for (const span of inline) {
    if (/\s|:\/\/|^[/~]/.test(span) || !looksLikePath(span)) continue;
    // A placeholder (`mvp/<YYYY-MM-DD>/field-notes.md`, `<id>.html`) names a
    // file that is made later; what must exist is the directory it goes in.
    const fixed = span.split('<')[0];
    if (span.includes('<') && !fixed.includes('/')) continue;
    const target = span.includes('<') ? fixed.slice(0, fixed.lastIndexOf('/') + 1) : span;
    assert.ok(fs.existsSync(path.join(REPO, target)),
      `docs/RUNBOOK.md names \`${span}\`, and ${target} does not exist`);
    checked.push(span);
  }
  assert.ok(checked.length >= 8, `only ${checked.length} paths checked; the guard has gone slack`);
});

test('every just recipe the runbook names is in the justfile', () => {
  const recipes = new Set(
    [...JUSTFILE.matchAll(/^([a-z][a-z0-9_-]*)(?: [^:\n]*)?:(?!=)/gm)].map((m) => m[1]));
  const { fenced, inline } = codeSpans(RUNBOOK);
  const named = [];
  for (const code of [...fenced, ...inline]) {
    for (const m of code.matchAll(/(?:^|[\s(])just ([a-z][a-z0-9_-]*)/g)) named.push(m[1]);
  }
  assert.ok(named.length >= 5, `only ${named.length} recipes named; the guard has gone slack`);
  for (const name of named) {
    assert.ok(recipes.has(name), `docs/RUNBOOK.md names \`just ${name}\`, which the justfile does not have`);
  }
});

test('every button the host script says to tap is a SPEC §11 host action', () => {
  const { PQ } = load('copy');
  const actions = new Set(Object.keys(PQ.COPY)
    .filter((k) => k.startsWith('host_action_')).map((k) => PQ.COPY[k]));
  const taps = [...section('The host script').matchAll(/\*\*Tap \*([^*]+)\*\.\*\*/g)].map((m) => m[1]);
  assert.ok(taps.length >= 6, `only ${taps.length} taps in the host script; the guard has gone slack`);
  for (const label of taps) {
    assert.ok(actions.has(label), `the host script says to tap *${label}*, which is no host action in copy.js`);
  }
});

test('nothing the host is told to say matches SPEC §11\'s Forbidden row (AC-98)', () => {
  const { PQ } = load('copylint');
  // A Say line and any `>` lines that continue it.
  const says = [];
  let open = null;
  for (const l of RUNBOOK.split('\n')) {
    if (l.startsWith('> **Say:**')) { open = l.slice('> **Say:**'.length); says.push(open); }
    else if (open !== null && /^>\s*\S/.test(l)) { says[says.length - 1] += ' ' + l.replace(/^>\s*/, ''); }
    else { open = null; }
  }
  says.forEach((s, i) => { says[i] = s.trim(); });
  assert.ok(says.length >= 5, `only ${says.length} "Say:" lines; the guard has gone slack`);
  for (const line of says) {
    // Array.from: the lint runs in a node:vm context, and its arrays carry that
    // realm's prototype (_load.js, plain()).
    assert.deepStrictEqual(Array.from(PQ.COPYLINT.forbidden(line), (h) => h.match), [],
      `the host would say "${line}"`);
  }
});
