// WCAG 2.1 contrast, written out, and the computed values it needs.
//
// Relative luminance and contrast ratio are WCAG 2.1's own definitions
// (Understanding SC 1.4.3, "relative luminance"; the 0.03928 threshold is the
// sRGB one WCAG 2.1 prints). AA: 4.5:1 for text, 3:1 for large text (24 px, or
// 18.66 px bold) and for the non-text contrast of 1.4.11 — focus indicators,
// the edges that identify a control, and graphics needed to understand the
// content.

const { winner } = require('./dom.js');

function channel(c8) {
  const c = c8 / 255;
  return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

function luminance([r, g, b]) {
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrast(a, b) {
  const la = luminance(a), lb = luminance(b);
  const [hi, lo] = la > lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

// `#abc`, `#aabbcc`, `rgb()`/`rgba()` (alpha dropped: none of ours carries one
// where text sits), `white`/`black`. Anything else in a colour position
// (`transparent`, `inherit`, `none`) is null: not a colour of its own.
function parseColor(v) {
  if (!v) return null;
  const s = v.trim().toLowerCase();
  let m;
  if ((m = /#([0-9a-f]{3})\b(?![0-9a-f])/.exec(s))) return [...m[1]].map((h) => parseInt(h + h, 16));
  if ((m = /#([0-9a-f]{6})\b/.exec(s))) return [0, 2, 4].map((i) => parseInt(m[1].slice(i, i + 2), 16));
  if ((m = /rgba?\(\s*(\d+)[\s,]+(\d+)[\s,]+(\d+)/.exec(s))) return [+m[1], +m[2], +m[3]];
  if (/\bwhite\b/.test(s)) return [255, 255, 255];
  if (/\bblack\b/.test(s)) return [0, 0, 0];
  return null;
}

const hex = (c) => '#' + c.map((x) => Math.round(x).toString(16).padStart(2, '0')).join('');
const blend = (fg, bg, alpha) => fg.map((x, i) => x * alpha + bg[i] * (1 - alpha));

// --- Computed values, walking up the tree ------------------------------------

const BODY_FONT_PX = 16;

function color(css, n) {
  for (let x = n; x; x = x.parent) {
    const w = winner(css, x, 'color');
    if (w && w.value !== 'inherit') {
      const c = parseColor(w.value);
      if (c) return c;
    }
  }
  return [0, 0, 0];
}

function ownBackground(css, n, opts) {
  for (const prop of ['background-color', 'background']) {
    const w = winner(css, n, prop, opts);
    if (w) {
      const c = parseColor(w.value);
      if (c) return c;
    }
  }
  return null;
}

// The colour showing behind `n`: its own background or the nearest ancestor's.
function background(css, n, opts) {
  for (let x = n; x; x = x.parent) {
    const c = ownBackground(css, x, opts);
    if (c) return { color: c, from: x };
  }
  return { color: [255, 255, 255], from: null };
}

function fontPx(css, n) {
  const w = winner(css, n, 'font-size');
  const parent = () => (n.parent ? fontPx(css, n.parent) : BODY_FONT_PX);
  if (!w || w.value === 'inherit') {
    // `font: inherit` sets the size to the parent's too.
    return parent();
  }
  let m;
  if ((m = /^([\d.]+)px$/.exec(w.value))) return +m[1];
  if ((m = /^([\d.]+)em$/.exec(w.value))) return +m[1] * parent();
  throw new Error(`font-size not understood: ${w.value} (${w.selector})`);
}

function bold(css, n) {
  for (let x = n; x; x = x.parent) {
    const w = winner(css, x, 'font-weight');
    if (w && w.value !== 'inherit') return w.value === 'bold' || Number(w.value) >= 700;
    if (x.tag === 'b' || x.tag === 'strong') return true;
  }
  return false;
}

// The product of opacities from `n` up — `.dim` lines, disabled controls.
function opacity(css, n) {
  let o = 1;
  for (let x = n; x; x = x.parent) {
    const w = winner(css, x, 'opacity');
    if (w) o *= Number(w.value);
  }
  return o;
}

const isLarge = (px, isBold) => px >= 24 || (isBold && px >= 18.66);

// The focus ring an element would draw, from the `outline` shorthand and any
// `outline-color` / `outline-offset` that outrank it.
function outline(css, n, bodyColor) {
  const opts = { focused: n };
  const short = winner(css, n, 'outline', opts);
  const col = winner(css, n, 'outline-color', opts);
  const off = winner(css, n, 'outline-offset', opts);
  if (!short) return null;
  const width = /([\d.]+)px/.exec(short.value);
  let c = parseColor(short.value);
  let from = short.selector;
  if (col && (col.important || rankAbove(col, short))) { c = parseColor(col.value); from = col.selector; }
  return {
    width: width ? +width[1] : 0,
    color: c || bodyColor,
    offset: off ? parseFloat(off.value) : 0,
    selector: from,
    none: /^(none|0)$/.test(short.value.trim()),
  };
}

function rankAbove(a, b) {
  if (!!a.important !== !!b.important) return !!a.important;
  for (let i = 0; i < 3; i++) if (a.spec[i] !== b.spec[i]) return a.spec[i] > b.spec[i];
  return a.order > b.order;
}

module.exports = {
  luminance, contrast, parseColor, hex, blend,
  color, background, ownBackground, fontPx, bold, opacity, isLarge, outline,
};
