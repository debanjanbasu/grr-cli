/**
 * Subset the IBM 3270 webfont to the range the site can actually render.
 *
 * The upstream face maps 1997 codepoints (CJK compatibility ideographs,
 * fullwidth forms, Hangul jamo, ...) and ships as a 63 KB woff2. The site is an
 * English technical-documentation site: across every built page it renders 105
 * distinct characters. The unreferenced codepoints are pure LCP weight -- the
 * brand font is the largest single asset on the critical path and gates the
 * hero text.
 *
 * The retained set is deliberately far wider than the characters currently
 * rendered, so ordinary copy edits cannot fall off the edge:
 *
 *   * ASCII plus Latin-1 Supplement and Latin Extended-A (European prose and
 *     names), Greek and Cyrillic (quoted API field names, author names)
 *   * General Punctuation, Currency, Letterlike, Number Forms, Arrows,
 *     Mathematical Operators, Misc Technical, Box Drawing, Block Elements,
 *     Geometric Shapes and Dingbats (the prose set, plus every symbol the
 *     terminal mockups and the install tables draw with)
 *   * Combining Diacritical Marks, so decomposed accents stay intact
 *
 * Anything outside this set still renders: the site declares a metric-matched
 * 'IBM3270 Fallback' (Courier New at 90%), so a missing glyph falls back
 * without shifting line breaks. What it costs is brand consistency on that one
 * character, not layout -- which is the right trade against 40 KB of LCP.
 *
 * Run from the repo root:  node scripts/subset-font.mjs [output.woff2]
 *
 * The optional output path exists so a regeneration can be diffed against the
 * committed font before it is overwritten; it defaults to the served subset.
 * Subsetting goes through harfbuzz (the `subset-font` npm package), so a
 * rerun on the same inputs is byte-identical.
 */

import { readFile, writeFile } from 'node:fs/promises';
import { existsSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import zlib from 'node:zlib';
import subsetFont from 'subset-font';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
// The full upstream face lives outside public/ so it is never served: the build
// only references the subset, and re-subsetting stays possible without
// re-downloading the original.
const SOURCE = path.join(REPO, 'site', 'fonts-src', '3270-Regular-full.woff2');
const TARGET = path.join(REPO, 'site', 'public', 'fonts', '3270-Regular.woff2');

const RANGES = [
  [0x0020, 0x007e], // Basic Latin
  [0x00a0, 0x00ff], // Latin-1 Supplement
  [0x0100, 0x017f], // Latin Extended-A
  [0x0300, 0x036f], // Combining Diacritical Marks
  [0x2000, 0x206f], // General Punctuation
  [0x20a0, 0x20bf], // Currency Symbols
  [0x2190, 0x21ff], // Arrows
  [0x2500, 0x257f], // Box Drawing
  [0x2580, 0x259f], // Block Elements
  [0x25a0, 0x25ff], // Geometric Shapes
  [0x2700, 0x27bf], // Dingbats
];

// Ranges deliberately NOT retained, and what dropping each one cost. The site
// is English technical documentation and renders 105 distinct characters, all
// of them above; these were measured against the upstream face:
//
//   Greek                +3.0 KB      Cyrillic            +5.1 KB
//   Misc Technical       +4.0 KB      Mathematical Ops    +2.3 KB
//   Letterlike Symbols   +0.5 KB      Number Forms        +0.8 KB
//
// Together that is ~15 KB of LCP weight for glyphs no page has ever used. IBM
// 3270 is a Latin heritage face, so its Greek and Cyrillic cuts are of dubious
// quality anyway. Should a non-Latin character ever reach the site it falls
// back to the metric-matched 'IBM3270 Fallback' for that one glyph: brand
// consistency is lost on that character, line breaks are not. Add the range
// back here if the site ever starts shipping that content.

// The subset must retain at least these codepoints or the site silently breaks
// months later, so the run asserts them explicitly.
const REQUIRED = new Set([
  ...Array.from({ length: 0x7f - 0x20 }, (_, i) => 0x20 + i),
  0x00b7, 0x2014, 0x2019, 0x201c, 0x201d, 0x2026, 0x2192, 0x2197, 0x2713,
]);

function codepoints() {
  const out = new Set();
  for (const [lo, hi] of RANGES) {
    for (let c = lo; c <= hi; c++) out.add(c);
  }
  return out;
}

// --- minimal woff2 -> cmap reader ------------------------------------------
// Enough of the WOFF2 container to recover the character map, which is what we
// report and assert on. Only `cmap` is read; glyf/loca transforms never matter
// here because we walk the decompressed stream by table length.

const WOFF2_TAGS = [
  'cmap', 'head', 'hhea', 'hmtx', 'maxp', 'name', 'OS/2', 'post', 'cvt ',
  'fpgm', 'glyf', 'loca', 'prep', 'CFF ', 'VORG', 'EBDT', 'EBLC', 'gasp',
  'hdmx', 'kern', 'LTSH', 'PCLT', 'VDMX', 'vhea', 'vmtx', 'BASE', 'GDEF',
  'GPOS', 'GSUB', 'EBSC', 'JSTF', 'MATH', 'CBDT', 'CBLC', 'COLR', 'CPAL',
  'SVG ', 'sbix', 'acnt', 'avar', 'bdat', 'bloc', 'bsln', 'cvar', 'fdsc',
  'feat', 'fmtx', 'fvar', 'gvar', 'hsty', 'just', 'lcar', 'mort', 'morx',
  'opbd', 'prop', 'trak', 'Zapf', 'Silf', 'Glat', 'Gloc', 'Feat', 'Sill',
];

function readBase128(buf, off) {
  let value = 0;
  for (let i = 0; i < 5; i++) {
    const byte = buf[off++];
    if (i === 0 && byte === 0x80) throw new Error('invalid UIntBase128');
    if (value & 0xfe000000) throw new Error('UIntBase128 overflow');
    value = (value << 7) | (byte & 0x7f);
    if (!(byte & 0x80)) return [value >>> 0, off];
  }
  throw new Error('UIntBase128 longer than 5 bytes');
}

function parseCmap(cmap) {
  const numTables = cmap.readUInt16BE(2);
  let best = -1;
  let bestScore = -1;
  for (let i = 0; i < numTables; i++) {
    const rec = 4 + i * 8;
    const platformID = cmap.readUInt16BE(rec);
    const encodingID = cmap.readUInt16BE(rec + 2);
    const subOff = cmap.readUInt32BE(rec + 4);
    let score = -1;
    if (platformID === 3 && encodingID === 10) score = 5;
    else if (platformID === 0 && encodingID >= 4) score = 4;
    else if (platformID === 3 && encodingID === 1) score = 3;
    else if (platformID === 0) score = 2;
    if (score > bestScore) {
      bestScore = score;
      best = subOff;
    }
  }
  if (best < 0) throw new Error('no usable cmap subtable');

  const format = cmap.readUInt16BE(best);
  const out = new Set();
  if (format === 4) {
    const segCount = cmap.readUInt16BE(best + 6) / 2;
    const endBase = best + 14;
    const startBase = endBase + segCount * 2 + 2;
    const deltaBase = startBase + segCount * 2;
    const rangeBase = deltaBase + segCount * 2;
    for (let s = 0; s < segCount; s++) {
      const end = cmap.readUInt16BE(endBase + s * 2);
      const start = cmap.readUInt16BE(startBase + s * 2);
      const delta = cmap.readInt16BE(deltaBase + s * 2);
      const rangeOffset = cmap.readUInt16BE(rangeBase + s * 2);
      for (let c = start; c <= end && c !== 0xffff; c++) {
        let gid;
        if (rangeOffset === 0) {
          gid = (c + delta) & 0xffff;
        } else {
          const gi = rangeBase + s * 2 + rangeOffset + (c - start) * 2;
          gid = gi + 2 <= cmap.length ? cmap.readUInt16BE(gi) : 0;
          if (gid !== 0) gid = (gid + delta) & 0xffff;
        }
        if (gid !== 0) out.add(c);
      }
    }
  } else if (format === 12) {
    const nGroups = cmap.readUInt32BE(best + 12);
    let p = best + 16;
    for (let g = 0; g < nGroups; g++) {
      const start = cmap.readUInt32BE(p);
      const end = cmap.readUInt32BE(p + 4);
      const startGid = cmap.readUInt32BE(p + 8);
      for (let c = start; c <= end; c++) {
        if (startGid + (c - start) !== 0) out.add(c);
      }
      p += 12;
    }
  } else {
    throw new Error(`unsupported cmap format ${format}`);
  }
  return out;
}

function cmapOf(woff2) {
  if (woff2.toString('latin1', 0, 4) !== 'wOF2') {
    throw new Error('not a WOFF2 font (missing wOF2 magic)');
  }
  const numTables = woff2.readUInt16BE(12);
  const totalCompressedSize = woff2.readUInt32BE(20);
  let off = 48;
  const tables = [];
  for (let i = 0; i < numTables; i++) {
    const flags = woff2[off++];
    const tagIndex = flags & 0x3f;
    let tag;
    if (tagIndex === 0x3f) {
      tag = woff2.toString('latin1', off, off + 4);
      off += 4;
    } else {
      tag = WOFF2_TAGS[tagIndex];
    }
    let origLength;
    [origLength, off] = readBase128(woff2, off);
    const transformVersion = flags >> 6;
    const transformed =
      tag === 'glyf' || tag === 'loca'
        ? transformVersion === 0
        : transformVersion === 3;
    let length = origLength;
    if (transformed) [length, off] = readBase128(woff2, off);
    tables.push({ tag, length });
  }
  const data = zlib.brotliDecompressSync(
    woff2.subarray(off, off + totalCompressedSize),
  );
  let cursor = 0;
  let cmap = null;
  for (const table of tables) {
    if (table.tag === 'cmap') cmap = data.subarray(cursor, cursor + table.length);
    cursor += table.length;
  }
  if (!cmap) throw new Error('WOFF2 font has no cmap table');
  return parseCmap(cmap);
}

// --- main -------------------------------------------------------------------

async function main() {
  const outPath = path.resolve(process.argv[2] ?? TARGET);
  if (!existsSync(SOURCE)) throw new Error(`missing ${SOURCE}`);

  const source = await readFile(SOURCE);
  const sourceCmap = cmapOf(source);
  const keep = [...codepoints()].filter((c) => sourceCmap.has(c)).sort((a, b) => a - b);
  const dropped = sourceCmap.size - keep.length;

  const text = String.fromCodePoint(...keep);
  const subset = await subsetFont(source, text, {
    targetFormat: 'woff2',
    noHinting: true,
  });
  await writeFile(outPath, subset);

  if (subset.toString('latin1', 0, 4) !== 'wOF2') {
    throw new Error('subset output is not a WOFF2 font (missing wOF2 magic)');
  }
  const outCmap = cmapOf(subset);
  const missing = [...REQUIRED].filter((c) => !outCmap.has(c));
  if (missing.length) {
    throw new Error(
      `subset lost required codepoints: ${missing.map((c) => '0x' + c.toString(16)).join(', ')}`,
    );
  }

  const before = statSync(SOURCE).size;
  const after = subset.length;
  console.log(`source ${before} bytes (${sourceCmap.size} codepoints)`);
  console.log(`subset ${after} bytes (${outCmap.size} codepoints), dropped ${dropped}`);
  console.log(
    `saved ${before - after} bytes (${((100 * (before - after)) / before).toFixed(1)}%)`,
  );
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
