// Loading web/shared/*.js into a test.
//
// The shared modules are classic scripts that attach to window.PopQuiz — no
// "type": "module" anywhere in web/, because the prototype loads its JS through
// plain <script src> and the port stays that way (web/README.md). That means
// require() and import cannot reach them, so they are evaluated into a
// node:vm context that carries the window/document stubs a DOM-touching
// function wants. Reaching for a bundler or jsdom to avoid this would be a
// bigger mechanism than the problem needs.
//
// Not named *.test.js on purpose: `node --test 'web/test/*.test.js'` (justfile)
// picks up suites by that glob and must not try to run this file as one.

const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const SHARED = path.join(__dirname, '..', 'shared');

// Load order matters only in that dom.js supplies escapeHtml to the renderers
// and phase.js supplies the predicates. In a browser the <script> tags compose
// in any order because every module reads PQ.* at call time, not at load time;
// these lists are written in the order a page would sensibly write them.
const MODULES = {
  dom: ['dom.js'],
  phase: ['phase.js'],
  check: ['dom.js', 'check.js'],
  well: ['dom.js', 'phase.js', 'well.js'],
  trace: ['dom.js', 'phase.js', 'well.js', 'trace.js'],
  typemodel: ['dom.js', 'phase.js', 'well.js', 'typemodel.js'],
  copy: ['copy.js'],
  all: ['dom.js', 'phase.js', 'check.js', 'well.js', 'trace.js', 'typemodel.js', 'copy.js'],
};

// A document stub with just enough surface for what web/shared touches:
// createElement, body.appendChild, textContent/innerHTML, setAttribute, and a
// classList that add/remove/contains honestly. It does no layout — nothing here
// computes scrollWidth, and that is the point: refit() takes its measurer as a
// parameter precisely so the loop can be proven without a layout engine.
function makeElement(tagName) {
  const classes = new Set();
  return {
    tagName,
    className: '',
    textContent: '',
    innerHTML: '',
    attributes: {},
    children: [],
    setAttribute(k, v) { this.attributes[k] = String(v); },
    getAttribute(k) { return Object.prototype.hasOwnProperty.call(this.attributes, k) ? this.attributes[k] : null; },
    appendChild(child) { this.children.push(child); return child; },
    classList: {
      add(c) { classes.add(c); },
      remove(c) { classes.delete(c); },
      contains(c) { return classes.has(c); },
      get length() { return classes.size; },
      toArray() { return [...classes]; },
    },
  };
}

function makeDocument() {
  const doc = {
    createElement: (tag) => makeElement(tag),
  };
  doc.body = makeElement('body');
  return doc;
}

// Load the named module group and return its PopQuiz namespace.
// `timers: false` runs setTimeout callbacks synchronously, so announce()'s
// 40ms clear-then-set is observable in a test without waiting on a real timer.
function load(which, opts = {}) {
  const files = MODULES[which];
  if (!files) throw new Error(`no such module group: ${which}`);

  const sandbox = {};
  sandbox.document = makeDocument();
  sandbox.window = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.console = console;
  sandbox.setTimeout = opts.timers === false
    ? (fn) => { fn(); return 0; }
    : setTimeout;

  vm.createContext(sandbox);
  for (const f of files) {
    const code = fs.readFileSync(path.join(SHARED, f), 'utf8');
    vm.runInContext(code, sandbox, { filename: `web/shared/${f}` });
  }
  if (!sandbox.window.PopQuiz) {
    throw new Error(`loading ${which} produced no window.PopQuiz`);
  }
  return { PQ: sandbox.window.PopQuiz, document: sandbox.document, sandbox };
}

// Read a shared file as text, for assertions about the source itself rather
// than its behaviour (AC-40's "no code path yields the class without the
// glyph" is a claim about the file, not about one call).
function source(file) {
  return fs.readFileSync(path.join(SHARED, file), 'utf8');
}

// Round to one decimal — SPEC §5.2 states its worked examples to one decimal
// and the modules deliberately do not round internally, so the rounding lives
// at the assertion.
const to1 = (n) => Math.round(n * 10) / 10;

// A source of exactly `lines` lines whose widest line is `chars` characters.
// Built rather than copied: a fixture that pasted a real bank question would
// be writing down what a program prints (CLAUDE.md), and the type model only
// ever looks at the shape.
function shapedSource(lines, chars) {
  const wide = 'x'.repeat(chars);
  const rest = 'x'.repeat(Math.max(1, Math.min(chars, 8)));
  return [wide, ...Array.from({ length: lines - 1 }, () => rest)].join('\n');
}

// Copy a vm-created object into this realm's Object.prototype.
//
// Objects the shared modules construct are built inside the node:vm context, so
// they inherit that context's Object.prototype — a different one. deepStrictEqual
// compares prototypes, so it rejects two structurally identical objects with
// "Values have same structure but are not reference-equal", which reads as a
// value mismatch and is not one. Spreading into a host-realm literal settles it.
const plain = (o) => (o === null || o === undefined ? o : { ...o });

module.exports = { load, source, to1, shapedSource, makeElement, plain, SHARED, MODULES };
