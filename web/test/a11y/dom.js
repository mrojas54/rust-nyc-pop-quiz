// The a11y suite's reader: markup into a tree, stylesheets into rules, and the
// cascade over the two — just enough to ask what colour, size and outline an
// element on one of our pages ends up with.
//
// Why not jsdom: web/README.md — no package.json, no dependency. And why this
// is enough: every page here is our own generated markup, well-formed, with no
// script-inserted styles, and every stylesheet is plain CSS with no nesting.
// What it does NOT do is layout: no box is measured, no line is wrapped, no
// pixel is rendered. Anything that needs those is the browser pass
// (web/README.md, Accessibility).
//
// Not named *.test.js: the `test-web` glob must not run it as a suite.

const fs = require('node:fs');
const path = require('node:path');

const WEB = path.join(__dirname, '..', '..');

// --- Markup ------------------------------------------------------------------

const VOID = new Set(['input', 'br', 'img', 'meta', 'link', 'hr', 'source', 'col', 'wbr']);

function decode(s) {
  return s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'").replace(/&amp;/g, '&');
}

function parseAttrs(src) {
  const attrs = {};
  const re = /([^\s=/]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/g;
  let m;
  while ((m = re.exec(src))) {
    const v = m[2] !== undefined ? m[2] : m[3] !== undefined ? m[3] : m[4] !== undefined ? m[4] : '';
    attrs[m[1].toLowerCase()] = decode(v);
  }
  return attrs;
}

// A node that is also a small DOM element: enough for the surfaces' own focus
// code (querySelector, contains, focus, disabled) to run against it.
class Node {
  constructor(tag, attrs, parent, doc) {
    this.tag = tag;
    this.tagName = tag.toUpperCase();
    this.attrs = attrs || {};
    this.parent = parent || null;
    this.children = [];
    this.text = '';          // this element's own text, children excluded
    this.ownerDocument = doc || (parent && parent.ownerDocument) || null;
  }
  get classes() { return (this.attrs.class || '').split(/\s+/).filter(Boolean); }
  get id() { return this.attrs.id || ''; }
  get disabled() { return Object.prototype.hasOwnProperty.call(this.attrs, 'disabled'); }
  getAttribute(k) { return Object.prototype.hasOwnProperty.call(this.attrs, k) ? this.attrs[k] : null; }
  setAttribute(k, v) { this.attrs[k] = String(v); }
  contains(n) { for (let x = n; x; x = x.parent) if (x === this) return true; return false; }
  // Records what each call asked for: a restore that scrolls the page would
  // pull a reader off what they were reading (`preventScroll`, PQ-40).
  focus(opts) {
    const doc = this.ownerDocument;
    if (!doc) return;
    doc.activeElement = this;
    (doc.focusCalls = doc.focusCalls || []).push({ node: this, opts: opts === undefined ? null : opts });
  }
  *walk() { for (const c of this.children) { yield c; yield* c.walk(); } }
  querySelectorAll(sel) { return [...this.walk()].filter((n) => matches(n, sel)); }
  querySelector(sel) { return this.querySelectorAll(sel)[0] || null; }
  get textContent() { return this.text + this.children.map((c) => c.textContent).join(''); }
  closest(sel) { for (let x = this; x && x.tag !== 'html'; x = x.parent) if (matches(x, sel)) return x; return null; }
  // As a browser does: the old children leave the tree, and if the focus was
  // among them it falls back to <body>.
  set innerHTML(html) {
    const doc = this.ownerDocument;
    const lost = doc && doc.activeElement && doc.activeElement !== this && this.contains(doc.activeElement);
    for (const c of this.children) c.parent = null;
    this.children = [];
    this.text = '';
    parseInto(this, String(html));
    if (lost) doc.activeElement = doc.body;
  }
}

function parseInto(root, html) {
  let cur = root;
  const re = /<!--[\s\S]*?-->|<\/([a-zA-Z][\w-]*)\s*>|<([a-zA-Z][\w-]*)([^>]*?)(\/?)>|([^<]+)/g;
  let m;
  while ((m = re.exec(html))) {
    if (m[0].startsWith('<!--')) continue;
    if (m[1]) {
      const tag = m[1].toLowerCase();
      for (let x = cur; x && x !== root; x = x.parent) {
        if (x.tag === tag) { cur = x.parent; break; }
      }
    } else if (m[2]) {
      const tag = m[2].toLowerCase();
      const node = new Node(tag, parseAttrs(m[3]), cur);
      cur.children.push(node);
      if (!m[4] && !VOID.has(tag)) cur = node;
    } else if (m[5]) {
      cur.text += decode(m[5]);
    }
  }
  return root;
}

// A document around `html`, wrapped in the page's own body and root element
// so the surface's scoped selectors (`.buzzer .btn`) find their ancestors.
//   page: [{tag, attrs}] outermost first, e.g. body.wall-page > div.wall-wrap
function page(html, chain) {
  const doc = { activeElement: null };
  const htmlEl = new Node('html', {}, null, doc);
  doc.documentElement = htmlEl;
  let cur = htmlEl;
  for (const { tag, attrs } of chain) {
    const n = new Node(tag, attrs, cur, doc);
    cur.children.push(n);
    if (tag === 'body') doc.body = n;
    cur = n;
  }
  if (!doc.body) throw new Error('page chain has no body');
  doc.activeElement = doc.body;
  doc.getElementById = (id) => htmlEl.querySelector('#' + id);
  parseInto(cur, html);
  doc.root = cur;
  return doc;
}

// --- Selectors ---------------------------------------------------------------
//
// Compound selectors joined by descendant or child combinators (both read as
// descendant — nothing here depends on the difference), and inside a compound:
// tag, #id, .class, [attr], [attr="v"], and the pseudo-classes our sheets use.
// :focus-visible matches only when the caller asks about the focused state.

function splitTop(s, sep) {
  const out = [];
  let depth = 0, buf = '', quote = null;
  for (const ch of s) {
    if (quote) { buf += ch; if (ch === quote) quote = null; continue; }
    if (ch === '"' || ch === "'") { quote = ch; buf += ch; continue; }
    if (ch === '(' || ch === '[') depth++;
    if (ch === ')' || ch === ']') depth--;
    if (depth === 0 && sep.test(ch)) { out.push(buf); buf = ''; continue; }
    buf += ch;
  }
  out.push(buf);
  return out.map((x) => x.trim()).filter((x) => x && x !== '>');
}

function compounds(selector) {
  return splitTop(selector.replace(/\s*>\s*/g, ' '), /\s/);
}

const STATE_PSEUDO = /^(hover|active|focus|focus-within|visited|link|checked|target)$/;

// Parse a compound into parts, or null if it names something we never match
// (a pseudo-element, a user-action state).
function parseCompound(c) {
  const parts = [];
  const re = /(\*)|([a-zA-Z][\w-]*)|#([\w-]+)|\.([\w-]+)|\[([\w-]+)(?:([~|^$*]?=)"?([^"\]]*)"?)?\]|::?([\w-]+)(?:\(((?:[^()]|\([^()]*\))*)\))?/g;
  let m, at = 0;
  while ((m = re.exec(c))) {
    if (m.index !== at) throw new Error(`selector part not understood: ${c}`);
    at = re.lastIndex;
    if (m[1]) continue;
    if (m[2]) parts.push({ tag: m[2].toLowerCase() });
    else if (m[3]) parts.push({ id: m[3] });
    else if (m[4]) parts.push({ cls: m[4] });
    else if (m[5]) parts.push({ attr: m[5], op: m[6], val: m[7] });
    else if (m[8]) {
      if (m[0].startsWith('::')) return null;
      const name = m[8];
      if (name === 'before' || name === 'after' || name === 'placeholder') return null;
      if (STATE_PSEUDO.test(name)) return null;
      parts.push({ pseudo: name, arg: m[9] });
    }
  }
  if (at !== c.length) throw new Error(`selector part not understood: ${c}`);
  return parts;
}

function matchPart(n, p, opts) {
  if (p.tag) return n.tag === p.tag;
  if (p.id) return n.id === p.id;
  if (p.cls) return n.classes.includes(p.cls);
  if (p.attr) {
    const v = n.getAttribute(p.attr);
    if (v === null) return false;
    if (!p.op) return true;
    if (p.op === '=') return v === p.val;
    if (p.op === '~=') return v.split(/\s+/).includes(p.val);
    return false;
  }
  switch (p.pseudo) {
    case 'focus-visible': return !!(opts && opts.focused === n);
    case 'disabled': return n.disabled;
    case 'root': return n.tag === 'html';
    case 'empty': return n.children.length === 0 && n.text === '';
    case 'not': return !matchCompound(n, p.arg.trim(), opts);
    case 'has': return [...n.walk()].some((d) => matches(d, p.arg.trim(), opts));
    default: throw new Error(`pseudo-class not supported by the a11y reader: :${p.pseudo}`);
  }
}

function matchCompound(n, c, opts) {
  const parts = parseCompound(c);
  if (!parts) return false;
  return parts.every((p) => matchPart(n, p, opts));
}

// Does node `n` match one complex selector (no commas)?
function matchesOne(n, selector, opts) {
  const cs = compounds(selector);
  if (!matchCompound(n, cs[cs.length - 1], opts)) return false;
  let anc = n.parent;
  for (let i = cs.length - 2; i >= 0; i--) {
    while (anc && !matchCompound(anc, cs[i], opts)) anc = anc.parent;
    if (!anc) return false;
    anc = anc.parent;
  }
  return true;
}

function matches(n, selectorList, opts) {
  return splitTop(selectorList, /,/).some((s) => matchesOne(n, s, opts));
}

function specificity(selector) {
  let a = 0, b = 0, c = 0;
  for (const comp of compounds(selector)) {
    const parts = parseCompound(comp.replace(/::(before|after|placeholder)/, '')) || [];
    for (const p of parts) {
      if (p.id) a++;
      else if (p.cls || p.attr) b++;
      else if (p.pseudo === 'not' || p.pseudo === 'has') {
        const inner = specificity(p.arg);
        a += inner[0]; b += inner[1]; c += inner[2];
      } else if (p.pseudo) b++;
      else if (p.tag) c++;
    }
  }
  return [a, b, c];
}

// --- Stylesheets ---------------------------------------------------------------

// Rules in source order, each with the @media it sits in (or null).
function parseCss(text, file) {
  const src = text.replace(/\/\*[\s\S]*?\*\//g, '');
  const rules = [];
  let i = 0;
  function block(media) {
    while (i < src.length) {
      const open = src.indexOf('{', i);
      const close = src.indexOf('}', i);
      if (close !== -1 && (open === -1 || close < open)) { i = close + 1; return; }
      if (open === -1) return;
      let head = src.slice(i, open).trim();
      // @import and friends end in ';' and may sit before the next rule.
      head = head.replace(/^(@[^;{]*;\s*)+/, '').trim();
      i = open + 1;
      if (head.startsWith('@media')) { block(head.slice(6).trim()); continue; }
      const end = src.indexOf('}', i);
      if (head.startsWith('@')) { i = end + 1; continue; }   // @font-face: no selector
      const body = src.slice(i, end);
      i = end + 1;
      const decls = [];
      for (const d of splitTop(body, /;/)) {
        const k = d.indexOf(':');
        if (k < 0) continue;
        let value = d.slice(k + 1).trim();
        const important = /!important$/.test(value);
        value = value.replace(/\s*!important$/, '');
        decls.push({ prop: d.slice(0, k).trim().toLowerCase(), value, important });
      }
      rules.push({ selector: head.replace(/\s+/g, ' '), decls, media, file });
    }
  }
  block(null);
  return rules;
}

function readSheet(rel) {
  return parseCss(fs.readFileSync(path.join(WEB, rel), 'utf8'), rel);
}

// A surface's stylesheets, in the order its page links them.
function sheets(files) {
  const rules = [];
  files.forEach((f) => rules.push(...readSheet(f)));
  const vars = {};
  for (const r of rules) {
    if (r.media || r.selector !== ':root') continue;
    for (const d of r.decls) if (d.prop.startsWith('--')) vars[d.prop] = d.value;
  }
  return { files, rules, vars };
}

function resolve(css, value, depth = 0) {
  if (depth > 20) throw new Error(`var() cycle resolving ${value}`);
  return value.replace(/var\((--[\w-]+)(?:,\s*([^)]*))?\)/g, (_, name, fallback) => {
    const v = css.vars[name] !== undefined ? css.vars[name] : fallback;
    if (v === undefined) throw new Error(`undefined custom property ${name}`);
    return resolve(css, v, depth + 1);
  });
}

// Every declaration of `prop` that applies to `n`, winner last.
function cascade(css, n, prop, opts) {
  const hits = [];
  css.rules.forEach((r, order) => {
    if (r.media) return;
    for (const d of r.decls) {
      if (d.prop !== prop) continue;
      for (const s of splitTop(r.selector, /,/)) {
        if (s.startsWith(':root') || s.startsWith('html:has')) continue;
        if (!matchesOne(n, s, opts)) continue;
        hits.push({ value: d.value, important: d.important, spec: specificity(s), order, selector: s, file: r.file });
      }
    }
  });
  const inline = n.getAttribute('style');
  if (inline) {
    for (const d of splitTop(inline, /;/)) {
      const k = d.indexOf(':');
      if (k < 0 || d.slice(0, k).trim().toLowerCase() !== prop) continue;
      hits.push({ value: d.slice(k + 1).trim(), important: false, spec: [1, 0, 0, 0], inline: true, order: Infinity, selector: 'style=""' });
    }
  }
  const rank = (h) => [h.important ? 1 : 0, h.inline ? 1 : 0, ...h.spec.slice(-3), h.order];
  hits.sort((x, y) => {
    const a = rank(x), b = rank(y);
    for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return a[i] - b[i];
    return 0;
  });
  return hits;
}

function winner(css, n, prop, opts) {
  const h = cascade(css, n, prop, opts);
  return h.length ? { ...h[h.length - 1], value: resolve(css, h[h.length - 1].value) } : null;
}

module.exports = { Node, page, parseInto, parseCss, sheets, resolve, cascade, winner, matches, specificity, decode, WEB };
