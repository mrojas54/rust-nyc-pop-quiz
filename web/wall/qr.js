/* ===========================================================================
   A QR code for the released wall — SPEC §5.5, §4 `released`.

   "The link in 40px monospace; the QR encoding the same link." The prototype
   drew a stand-in (real finder squares, random modules); the build needs a code
   a phone can actually scan, and AC-77 runs the segment with outbound network
   blocked, so it is generated here, in the page, with no library and no CDN.

   What it does and nothing more: BYTE mode, error-correction level M, versions
   1-10 (up to 213 bytes at M — a take-it-home URL is a few dozen), all eight
   masks scored with the standard penalty and the lowest chosen. Output is an
   <svg> of <rect>s with a four-module quiet zone.

   The algorithm's structure — the function-pattern drawing, the zig-zag
   codeword placement, the Reed-Solomon divisor/remainder, the penalty scoring —
   follows Project Nayuki's "QR Code generator library", which is released
   under the MIT License, reproduced here as that licence requires:

     Copyright (c) Project Nayuki. (MIT License)
     https://www.nayuki.io/page/qr-code-generator-library

     Permission is hereby granted, free of charge, to any person obtaining a
     copy of this software and associated documentation files (the "Software"),
     to deal in the Software without restriction, including without limitation
     the rights to use, copy, modify, merge, publish, distribute, sublicense,
     and/or sell copies of the Software, and to permit persons to whom the
     Software is furnished to do so, subject to the following conditions:
     - The above copyright notice and this permission notice shall be included
       in all copies or substantial portions of the Software.
     - The Software is provided "as is", without warranty of any kind, express
       or implied, including but not limited to the warranties of
       merchantability, fitness for a particular purpose and noninfringement.
       In no event shall the authors or copyright holders be liable for any
       claim, damages or other liability, whether in an action of contract,
       tort or otherwise, arising from, out of or in connection with the
       Software or the use or other dealings in the Software.

   Proven two ways: web/test/wall.test.js pins a golden matrix and the
   structural invariants (finders, timing, BCH-valid format bits); the golden
   was decoded by an independent reader (macOS CoreImage) at validation, which
   is the only proof that matters — a QR that this file's own logic agrees with
   and no phone can read is worth nothing.
   =========================================================================== */
(function (root) {
  "use strict";
  var PQ = (root.PopQuiz = root.PopQuiz || {});

  /* ISO/IEC 18004 Table 9, level M, versions 1-10 (index 0 unused). */
  var ECC_PER_BLOCK = [-1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26];
  var NUM_BLOCKS = [-1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5];
  var MAX_VERSION = 10;
  var FORMAT_LEVEL_M = 0; /* the two format bits for level M */

  function bit(x, i) { return ((x >>> i) & 1) !== 0; }

  /* Data modules available in a version, after every function pattern. */
  function rawDataModules(ver) {
    var result = (16 * ver + 128) * ver + 64;
    if (ver >= 2) {
      var align = Math.floor(ver / 7) + 2;
      result -= (25 * align - 10) * align - 55;
      if (ver >= 7) result -= 36;
    }
    return result;
  }

  function dataCodewords(ver) {
    return Math.floor(rawDataModules(ver) / 8) - ECC_PER_BLOCK[ver] * NUM_BLOCKS[ver];
  }

  /* UTF-8 bytes of a string, without TextEncoder so it runs in node:vm too. */
  function utf8(s) {
    var out = [];
    var str = unescape(encodeURIComponent(String(s)));
    for (var i = 0; i < str.length; i++) out.push(str.charCodeAt(i));
    return out;
  }

  /* --- GF(256), polynomial 0x11D ------------------------------------- */
  function gfMul(x, y) {
    var z = 0;
    for (var i = 7; i >= 0; i--) {
      z = (z << 1) ^ ((z >>> 7) * 0x11D);
      z ^= ((y >>> i) & 1) * x;
    }
    return z;
  }

  function rsDivisor(degree) {
    var result = [];
    for (var i = 0; i < degree - 1; i++) result.push(0);
    result.push(1);
    var r = 1;
    for (var n = 0; n < degree; n++) {
      for (var j = 0; j < result.length; j++) {
        result[j] = gfMul(result[j], r);
        if (j + 1 < result.length) result[j] ^= result[j + 1];
      }
      r = gfMul(r, 0x02);
    }
    return result;
  }

  function rsRemainder(data, divisor) {
    var result = divisor.map(function () { return 0; });
    data.forEach(function (b) {
      var factor = b ^ result.shift();
      result.push(0);
      divisor.forEach(function (coef, i) { result[i] ^= gfMul(coef, factor); });
    });
    return result;
  }

  /* --- The codewords --------------------------------------------------- */

  function chooseVersion(nBytes) {
    for (var ver = 1; ver <= MAX_VERSION; ver++) {
      var ccBits = ver <= 9 ? 8 : 16;
      if (4 + ccBits + 8 * nBytes <= dataCodewords(ver) * 8) return ver;
    }
    throw new Error("qr: " + nBytes + " bytes do not fit version " + MAX_VERSION + " at level M");
  }

  function dataBytes(bytes, ver) {
    var bits = [];
    function put(val, len) { for (var i = len - 1; i >= 0; i--) bits.push((val >>> i) & 1); }
    put(0x4, 4);                              /* byte mode */
    put(bytes.length, ver <= 9 ? 8 : 16);     /* character count */
    bytes.forEach(function (b) { put(b, 8); });
    var cap = dataCodewords(ver) * 8;
    put(0, Math.min(4, cap - bits.length));   /* terminator */
    put(0, (8 - bits.length % 8) % 8);        /* to a byte boundary */
    for (var pad = 0xEC; bits.length < cap; pad ^= 0xEC ^ 0x11) put(pad, 8);
    var out = [];
    for (var i = 0; i < bits.length; i += 8) {
      var v = 0;
      for (var j = 0; j < 8; j++) v = (v << 1) | bits[i + j];
      out.push(v);
    }
    return out;
  }

  function withEcc(data, ver) {
    var numBlocks = NUM_BLOCKS[ver];
    var eccLen = ECC_PER_BLOCK[ver];
    var raw = Math.floor(rawDataModules(ver) / 8);
    var numShort = numBlocks - raw % numBlocks;
    var shortLen = Math.floor(raw / numBlocks);
    var divisor = rsDivisor(eccLen);
    var blocks = [];
    for (var i = 0, k = 0; i < numBlocks; i++) {
      var dat = data.slice(k, k + shortLen - eccLen + (i < numShort ? 0 : 1));
      k += dat.length;
      var ecc = rsRemainder(dat, divisor);
      if (i < numShort) dat.push(0);
      blocks.push(dat.concat(ecc));
    }
    var result = [];
    for (var c = 0; c < blocks[0].length; c++) {
      blocks.forEach(function (b, j) {
        if (c !== shortLen - eccLen || j >= numShort) result.push(b[c]);
      });
    }
    return result;
  }

  /* --- The matrix ------------------------------------------------------ */

  function alignmentPositions(ver, size) {
    if (ver === 1) return [];
    var n = Math.floor(ver / 7) + 2;
    var step = Math.ceil((ver * 4 + 4) / (n * 2 - 2)) * 2;
    var result = [6];
    for (var pos = size - 7; result.length < n; pos -= step) result.splice(1, 0, pos);
    return result;
  }

  function Matrix(ver) {
    var size = ver * 4 + 17;
    this.ver = ver;
    this.size = size;
    this.dark = [];
    this.func = [];
    for (var y = 0; y < size; y++) {
      this.dark.push(new Array(size).fill(false));
      this.func.push(new Array(size).fill(false));
    }
  }

  Matrix.prototype.setFunc = function (x, y, dark) {
    this.dark[y][x] = dark;
    this.func[y][x] = true;
  };

  Matrix.prototype.drawFunctionPatterns = function () {
    var size = this.size;
    var i;
    for (i = 0; i < size; i++) {
      this.setFunc(6, i, i % 2 === 0);
      this.setFunc(i, 6, i % 2 === 0);
    }
    this.drawFinder(3, 3);
    this.drawFinder(size - 4, 3);
    this.drawFinder(3, size - 4);
    var pos = alignmentPositions(this.ver, size);
    var last = pos.length - 1;
    for (i = 0; i < pos.length; i++) {
      for (var j = 0; j < pos.length; j++) {
        if ((i === 0 && j === 0) || (i === 0 && j === last) || (i === last && j === 0)) continue;
        this.drawAlignment(pos[i], pos[j]);
      }
    }
    this.drawFormatBits(0); /* reserve; overwritten once the mask is chosen */
    this.drawVersion();
  };

  Matrix.prototype.drawFinder = function (x, y) {
    for (var dy = -4; dy <= 4; dy++) {
      for (var dx = -4; dx <= 4; dx++) {
        var d = Math.max(Math.abs(dx), Math.abs(dy));
        var xx = x + dx, yy = y + dy;
        if (xx >= 0 && xx < this.size && yy >= 0 && yy < this.size) {
          this.setFunc(xx, yy, d !== 2 && d !== 4);
        }
      }
    }
  };

  Matrix.prototype.drawAlignment = function (x, y) {
    for (var dy = -2; dy <= 2; dy++) {
      for (var dx = -2; dx <= 2; dx++) {
        this.setFunc(x + dx, y + dy, Math.max(Math.abs(dx), Math.abs(dy)) !== 1);
      }
    }
  };

  /* 15 format bits: level and mask, BCH(15,5), XOR 0x5412. */
  function formatBits(mask) {
    var data = (FORMAT_LEVEL_M << 3) | mask;
    var rem = data;
    for (var i = 0; i < 10; i++) rem = (rem << 1) ^ ((rem >>> 9) * 0x537);
    return ((data << 10) | rem) ^ 0x5412;
  }

  Matrix.prototype.drawFormatBits = function (mask) {
    var bits = formatBits(mask);
    var size = this.size;
    var i;
    for (i = 0; i <= 5; i++) this.setFunc(8, i, bit(bits, i));
    this.setFunc(8, 7, bit(bits, 6));
    this.setFunc(8, 8, bit(bits, 7));
    this.setFunc(7, 8, bit(bits, 8));
    for (i = 9; i < 15; i++) this.setFunc(14 - i, 8, bit(bits, i));
    for (i = 0; i < 8; i++) this.setFunc(size - 1 - i, 8, bit(bits, i));
    for (i = 8; i < 15; i++) this.setFunc(8, size - 15 + i, bit(bits, i));
    this.setFunc(8, size - 8, true); /* the dark module */
  };

  Matrix.prototype.drawVersion = function () {
    if (this.ver < 7) return;
    var rem = this.ver;
    for (var i = 0; i < 12; i++) rem = (rem << 1) ^ ((rem >>> 11) * 0x1F25);
    var bits = (this.ver << 12) | rem;
    for (var n = 0; n < 18; n++) {
      var a = this.size - 11 + n % 3;
      var b = Math.floor(n / 3);
      this.setFunc(a, b, bit(bits, n));
      this.setFunc(b, a, bit(bits, n));
    }
  };

  Matrix.prototype.drawCodewords = function (codewords) {
    var size = this.size;
    var i = 0;
    for (var right = size - 1; right >= 1; right -= 2) {
      if (right === 6) right = 5;
      for (var vert = 0; vert < size; vert++) {
        for (var j = 0; j < 2; j++) {
          var x = right - j;
          var upward = ((right + 1) & 2) === 0;
          var y = upward ? size - 1 - vert : vert;
          if (!this.func[y][x] && i < codewords.length * 8) {
            this.dark[y][x] = bit(codewords[i >>> 3], 7 - (i & 7));
            i++;
          }
        }
      }
    }
  };

  var MASKS = [
    function (x, y) { return (x + y) % 2 === 0; },
    function (x, y) { return y % 2 === 0; },
    function (x, y) { return x % 3 === 0; },
    function (x, y) { return (x + y) % 3 === 0; },
    function (x, y) { return (Math.floor(x / 3) + Math.floor(y / 2)) % 2 === 0; },
    function (x, y) { return x * y % 2 + x * y % 3 === 0; },
    function (x, y) { return (x * y % 2 + x * y % 3) % 2 === 0; },
    function (x, y) { return ((x + y) % 2 + x * y % 3) % 2 === 0; }
  ];

  Matrix.prototype.applyMask = function (mask) {
    var f = MASKS[mask];
    for (var y = 0; y < this.size; y++) {
      for (var x = 0; x < this.size; x++) {
        if (!this.func[y][x] && f(x, y)) this.dark[y][x] = !this.dark[y][x];
      }
    }
  };

  /* --- Penalty (ISO/IEC 18004 §7.8.3) ---------------------------------- */
  var N1 = 3, N2 = 3, N3 = 40, N4 = 10;

  Matrix.prototype.penalty = function () {
    var size = this.size;
    var self = this;
    var result = 0;

    function addHistory(run, h) {
      if (h[0] === 0) run += size; /* light border before the first run */
      h.pop();
      h.unshift(run);
    }
    function countPatterns(h) {
      var n = h[1];
      var core = n > 0 && h[2] === n && h[3] === n * 3 && h[4] === n && h[5] === n;
      return (core && h[0] >= n * 4 && h[6] >= n ? 1 : 0) +
             (core && h[6] >= n * 4 && h[0] >= n ? 1 : 0);
    }
    function terminate(color, run, h) {
      if (color) { addHistory(run, h); run = 0; }
      run += size;
      addHistory(run, h);
      return countPatterns(h);
    }
    function line(get) {
      var color = false, run = 0, h = [0, 0, 0, 0, 0, 0, 0];
      for (var i = 0; i < size; i++) {
        if (get(i) === color) {
          run++;
          if (run === 5) result += N1;
          else if (run > 5) result++;
        } else {
          addHistory(run, h);
          if (!color) result += countPatterns(h) * N3;
          color = get(i);
          run = 1;
        }
      }
      result += terminate(color, run, h) * N3;
    }

    var y, x;
    for (y = 0; y < size; y++) line(function (i) { return self.dark[y][i]; });
    for (x = 0; x < size; x++) line(function (i) { return self.dark[i][x]; });

    for (y = 0; y < size - 1; y++) {
      for (x = 0; x < size - 1; x++) {
        var c = this.dark[y][x];
        if (c === this.dark[y][x + 1] && c === this.dark[y + 1][x] && c === this.dark[y + 1][x + 1]) {
          result += N2;
        }
      }
    }

    var darkCount = 0;
    for (y = 0; y < size; y++) for (x = 0; x < size; x++) if (this.dark[y][x]) darkCount++;
    var total = size * size;
    var k = Math.ceil(Math.abs(darkCount * 20 - total * 10) / total) - 1;
    result += k * N4;
    return result;
  };

  /* --- Public ---------------------------------------------------------- */

  /* The module matrix for `text`: rows of booleans, true = dark. */
  function qrMatrix(text) {
    var bytes = utf8(text);
    var ver = chooseVersion(bytes.length);
    var m = new Matrix(ver);
    m.drawFunctionPatterns();
    m.drawCodewords(withEcc(dataBytes(bytes, ver), ver));

    var best = 0, bestScore = Infinity;
    for (var mask = 0; mask < 8; mask++) {
      m.applyMask(mask);
      m.drawFormatBits(mask);
      var score = m.penalty();
      if (score < bestScore) { best = mask; bestScore = score; }
      m.applyMask(mask); /* XOR again: undone */
    }
    m.applyMask(best);
    m.drawFormatBits(best);
    return { version: ver, mask: best, size: m.size, modules: m.dark };
  }

  /* The QR as an <svg>, four-module quiet zone, dark on white. `label` is the
     accessible name; it is escaped because it carries the link, which comes
     from the payload. */
  function qrSvg(text, opts) {
    opts = opts || {};
    var q = qrMatrix(text);
    var quiet = 4;
    var n = q.size + quiet * 2;
    var esc = PQ.escapeAttr || function (s) { return String(s); };
    var rects = [];
    for (var y = 0; y < q.size; y++) {
      for (var x = 0; x < q.size; x++) {
        if (q.modules[y][x]) {
          rects.push('<rect x="' + (x + quiet) + '" y="' + (y + quiet) + '" width="1" height="1"/>');
        }
      }
    }
    return '<svg class="' + (opts.className || "endqr") + '" viewBox="0 0 ' + n + " " + n +
      '" shape-rendering="crispEdges" role="img" aria-label="' + esc(opts.label || text) + '">' +
      '<rect width="' + n + '" height="' + n + '" fill="#ffffff"/>' +
      '<g fill="#1a202c">' + rects.join("") + "</g></svg>";
  }

  PQ.qrMatrix = qrMatrix;
  PQ.qrSvg = qrSvg;
  PQ._qrFormatBits = formatBits;
})(typeof window !== "undefined" ? window : globalThis);
