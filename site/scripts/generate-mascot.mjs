import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const siteDirectory = resolve(scriptDirectory, '..');
const assetDirectory = resolve(siteDirectory, 'src', 'assets');
const publicDirectory = resolve(siteDirectory, 'public');

const PALETTE_LEGEND = Object.freeze({
  '.': null,
  o: '#3B1D0B',
  s: '#7C2D12',
  d: '#C2410C',
  m: '#EA580C',
  b: '#F97316',
  p: '#FF8A3D',
  w: '#FFD9B0',
  h: '#FFF3E4',
  k: '#1A120B',
  r: '#EA4335',
  g: '#34A853',
  y: '#FBBC04',
  u: '#4285F4',
  c: '#FFF8F0'
});

const xml = (value) => String(value)
  .replaceAll('&', '&amp;')
  .replaceAll('<', '&lt;')
  .replaceAll('>', '&gt;')
  .replaceAll('"', '&quot;');

const number = (value) => Number.isFinite(value) ? String(value) : '0';

const rect = (x, y, width, height, fill, extra = '') =>
  `<rect x="${number(x)}" y="${number(y)}" width="${number(width)}" height="${number(height)}" fill="${fill}"${extra ? ` ${extra}` : ''}/>`;

const polygon = (points, fill, extra = '') =>
  `<polygon points="${points.map(([x, y]) => `${number(x)},${number(y)}`).join(' ')}" fill="${fill}"${extra ? ` ${extra}` : ''}/>`;

const svgDocument = ({ width, height, viewBox = `0 0 ${width} ${height}`, title, description, body, idPrefix = null }) => {
  const titleId = idPrefix ? `${idPrefix}-title` : 'title';
  const descId = idPrefix ? `${idPrefix}-desc` : 'desc';
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${number(width)}" height="${number(height)}" viewBox="${viewBox}" role="img" aria-labelledby="${titleId} ${descId}"><title id="${titleId}">${xml(title)}</title><desc id="${descId}">${xml(description)}</desc>${body}\n</svg>\n`;
};

/**
 * Contact shadow, pixel-art style: a dithered strip on the same lattice as the
 * artwork around it. The previous version stacked three blurred ellipses at
 * low opacity, which was the one thing in the whole set that read as smooth
 * vector — and a blur also smears on fractional viewports.
 *
 * A strip, not an ellipse. A contact shadow under a flat sprite is two to four
 * pixels tall and forty wide, and a Bayer dissolve across an ellipse that flat
 * varies only along x — which renders as a row of vertical bars rather than as
 * a shadow. Fading the strip's ends gives the checker its second dimension,
 * which is what makes it read as a shadow at all.
 */
const ditherShadowStrip = (width, rows, { color = 's' } = {}) => {
  const map = grid(width, rows);
  const centre = (width - 1) / 2;
  const half = Math.max(centre, 1);
  for (let row = 0; row < rows; row += 1) {
    // Rows disperse downward: density falls off with depth as well as across,
    // so the strip reads as a shadow pooling under the object rather than as a
    // bar of solid pixels sitting on the page.
    const depth = 1 - (row / Math.max(rows - 1, 1)) * 0.6;
    for (let x = 0; x < width; x += 1) {
      const t = Math.abs(x - centre) / half;
      const keep = Math.round(8 * (1 - t) ** 1.5 * depth);
      if (keep <= 0) continue;
      const bayer = ((x & 3) + (row & 3) * 4) & 3;
      if (bayer * 2 < keep) plot(map, x, row, color);
    }
  }
  return map;
};

/** Size the strip from user-space radii and place it on the artwork's lattice,
 *  so the same four-argument call works on the 512px mascot, the 960px
 *  terminal, and the 480px illustrations without each restating its grid. */
const ditherShadowBox = (cx, cy, rx, ry, scale = 8) => {
  const width = Math.max(3, Math.round((rx * 2) / scale));
  const rows = Math.max(2, Math.round((ry * 2) / scale));
  const left = Math.round(cx / scale) - Math.floor(width / 2);
  const top = Math.round(cy / scale) - Math.floor(rows / 2);
  return pixels(ditherShadowStrip(width, rows), { x: left * scale, y: top * scale, scale });
};

const contactShadow = (cx, cy, rx, ry, scale = 8) =>
  `<g aria-hidden="true">${ditherShadowBox(cx, cy, rx, ry, scale)}</g>`;

const parseAscii = (name, source) => {
  const rows = source.trim().split(/\r?\n/).map((row) => [...row]);
  if (rows.length !== 64 || rows.some((row) => row.length !== 64)) {
    throw new Error(`${name} must be a 64x64 ASCII map`);
  }
  for (const row of rows) {
    for (const cell of row) {
      if (!(cell in PALETTE_LEGEND)) {
        throw new Error(`${name} contains unknown palette cell ${cell}`);
      }
    }
  }
  return rows;
};

const blankMap = () => Array.from({ length: 64 }, () => Array(64).fill('.'));

const setPixel = (map, x, y, color) => {
  const px = Math.round(x);
  const py = Math.round(y);
  if (px >= 0 && py >= 0 && px < 64 && py < 64) map[py][px] = color;
};

const fillRect = (map, x, y, width, height, color) => {
  for (let py = y; py < y + height; py += 1) {
    for (let px = x; px < x + width; px += 1) setPixel(map, px, py, color);
  }
};

const fillEllipse = (map, cx, cy, rx, ry, color) => {
  for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y += 1) {
    for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x += 1) {
      const dx = (x + 0.5 - cx) / (rx + 0.5);
      const dy = (y + 0.5 - cy) / (ry + 0.5);
      if (dx * dx + dy * dy <= 1) setPixel(map, x, y, color);
    }
  }
};

const pointInPolygon = (x, y, points) => {
  let inside = false;
  for (let index = 0, previous = points.length - 1; index < points.length; previous = index, index += 1) {
    const [xi, yi] = points[index];
    const [xj, yj] = points[previous];
    const intersects = ((yi > y) !== (yj > y)) && (x < (xj - xi) * (y - yi) / ((yj - yi) || 1) + xi);
    if (intersects) inside = !inside;
  }
  return inside;
};

const fillPolygon = (map, points, color) => {
  const xs = points.map(([x]) => x);
  const ys = points.map(([, y]) => y);
  for (let y = Math.floor(Math.min(...ys)); y <= Math.ceil(Math.max(...ys)); y += 1) {
    for (let x = Math.floor(Math.min(...xs)); x <= Math.ceil(Math.max(...xs)); x += 1) {
      if (pointInPolygon(x + 0.5, y + 0.5, points)) setPixel(map, x, y, color);
    }
  }
};

const outlinePolygon = (map, points, color = 'o') => {
  fillPolygon(map, points, color);
  const xs = points.map(([x]) => x);
  const ys = points.map(([, y]) => y);
  for (let y = Math.floor(Math.min(...ys)); y <= Math.ceil(Math.max(...ys)); y += 1) {
    for (let x = Math.floor(Math.min(...xs)); x <= Math.ceil(Math.max(...xs)); x += 1) {
      if (map[y]?.[x] !== color) continue;
      const neighbors = [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]];
      if (neighbors.some(([nx, ny]) => map[ny]?.[nx] !== color)) continue;
      setPixel(map, x, y, '.');
    }
  }
};

const drawLine = (map, x1, y1, x2, y2, width, color) => {
  const steps = Math.max(Math.abs(x2 - x1), Math.abs(y2 - y1));
  for (let index = 0; index <= steps; index += 1) {
    const x = Math.round(x1 + (x2 - x1) * index / steps);
    const y = Math.round(y1 + (y2 - y1) * index / steps);
    fillRect(map, x - Math.floor((width - 1) / 2), y - Math.floor((width - 1) / 2), width, width, color);
  }
};

const drawLimb = (map, points, fill = 'b') => {
  for (let index = 1; index < points.length; index += 1) {
    const [x1, y1] = points[index - 1];
    const [x2, y2] = points[index];
    drawLine(map, x1, y1, x2, y2, 3, 'o');
  }
  for (let index = 1; index < points.length; index += 1) {
    const [x1, y1] = points[index - 1];
    const [x2, y2] = points[index];
    drawLine(map, x1, y1, x2, y2, 1, fill);
  }
};

const drawFoot = (map, x, y, direction, fill = 'b') => {
  const width = direction < 0 ? 6 : 6;
  const start = direction < 0 ? x - width + 1 : x;
  fillRect(map, start, y, width, 3, 'o');
  fillRect(map, start + 1, y, width - 2, 1, 'w');
  fillRect(map, start + 1, y + 1, width - 2, 1, fill);
};

const drawClaw = (map, centerX, centerY, side = 'left', open = false) => {
  const direction = side === 'left' ? -1 : 1;
  const mirror = (points) => points.map(([x, y]) => [2 * centerX - x, y]);
  const rightPoints = open
    ? [
        [centerX - 5, centerY - 5], [centerX + 3, centerY - 7],
        [centerX + 7, centerY - 4], [centerX + 6, centerY],
        [centerX + 3, centerY + 1], [centerX + 5, centerY + 4],
        [centerX, centerY + 7], [centerX - 5, centerY + 4]
      ]
    : [
        [centerX - 5, centerY - 5], [centerX + 3, centerY - 6],
        [centerX + 6, centerY - 2], [centerX + 5, centerY + 3],
        [centerX, centerY + 6], [centerX - 5, centerY + 3]
      ];
  const innerRight = open
    ? [
        [centerX - 4, centerY - 4], [centerX + 2, centerY - 6],
        [centerX + 5, centerY - 3], [centerX + 4, centerY],
        [centerX + 2, centerY + 1], [centerX + 4, centerY + 3],
        [centerX, centerY + 5], [centerX - 4, centerY + 3]
      ]
    : [
        [centerX - 4, centerY - 4], [centerX + 2, centerY - 5],
        [centerX + 4, centerY - 2], [centerX + 4, centerY + 2],
        [centerX, centerY + 4], [centerX - 4, centerY + 2]
      ];
  const points = direction < 0 ? mirror(rightPoints) : rightPoints;
  const inner = direction < 0 ? mirror(innerRight) : innerRight;
  outlinePolygon(map, points, 'o');
  fillPolygon(map, inner, 'm');
  fillRect(map, centerX - 3, centerY - 4, 5, 3, 'p');
  fillRect(map, centerX - 2, centerY - 4, 3, 1, 'h');
  setPixel(map, centerX + direction * 3, centerY + 2, 'd');
  setPixel(map, centerX - direction * 3, centerY - 1, 'b');
  if (open) {
    fillRect(map, centerX + direction * 3, centerY - 1, direction > 0 ? 1 : 2, 2, 'o');
  }
};

const drawEye = (map, x, y) => {
  fillRect(map, x, y, 10, 12, 'o');
  fillRect(map, x + 1, y + 1, 8, 10, 'h');
  fillRect(map, x + 1, y + 1, 6, 3, 'w');
  fillRect(map, x + 3, y + 4, 4, 5, 'k');
  fillRect(map, x + 3, y + 4, 2, 2, 'h');
  setPixel(map, x + 6, y + 8, 'd');
  setPixel(map, x + 1, y + 9, 'p');
};

const drawSmile = (map, open = false) => {
  if (open) {
    fillRect(map, 30, 36, 5, 4, 'o');
    fillRect(map, 31, 37, 3, 2, 'r');
    setPixel(map, 30, 39, 'd');
    setPixel(map, 34, 39, 'd');
  } else {
    fillRect(map, 28, 36, 1, 1, 'k');
    fillRect(map, 29, 37, 2, 1, 'k');
    fillRect(map, 31, 37, 2, 1, 'k');
    fillRect(map, 33, 37, 1, 1, 'k');
    fillRect(map, 34, 36, 1, 1, 'k');
  }
};

const paintBody = (map, sleeping = false) => {
  if (sleeping) {
    fillEllipse(map, 32, 37, 23, 10, 'o');
    fillEllipse(map, 32, 37, 21.5, 8.5, 'm');
    fillEllipse(map, 29, 34, 18, 6, 'b');
    fillEllipse(map, 26, 31, 11, 4, 'p');
    fillEllipse(map, 22, 28, 5, 2, 'w');
    for (let y = 29; y < 47; y += 1) {
      for (let x = 10; x < 55; x += 1) {
        const color = map[y][x];
        if (!'mbpw'.includes(color)) continue;
        if ((x > 46 && y > 34) || (y > 42 && x > 34)) map[y][x] = 'd';
        if (x > 50 && y > 40) map[y][x] = 's';
      }
    }
    return;
  }
  fillEllipse(map, 32, 35, 18, 14, 'o');
  fillEllipse(map, 32, 35, 16.6, 12.6, 'm');
  fillEllipse(map, 30, 33, 15, 10, 'b');
  fillEllipse(map, 27, 29, 12, 7, 'p');
  fillEllipse(map, 24, 26, 7, 4, 'w');
  fillEllipse(map, 21.5, 24, 2.5, 1.5, 'h');
  for (let y = 22; y < 49; y += 1) {
    for (let x = 14; x < 51; x += 1) {
      const color = map[y][x];
      if (!'mbpwh'.includes(color)) continue;
      if (x > 43 && y > 31) map[y][x] = 'd';
      if (x > 47 && y > 39) map[y][x] = 's';
      if (y > 44 && x > 30) map[y][x] = 'd';
      if (x < 20 && y < 31) map[y][x] = color === 'w' ? 'w' : 'p';
    }
  }
  fillRect(map, 36, 26, 5, 2, 'w');
  fillRect(map, 40, 28, 2, 4, 'w');
  for (const [x, y, color] of [[23, 31, 'd'], [26, 32, 'm'], [39, 33, 'd'], [42, 31, 'm'], [20, 36, 'p'], [45, 38, 'd']]) {
    fillRect(map, x, y, 2, 1, color);
  }
};

const drawSleepMark = (map, x, y, scale, color = 'k') => {
  const glyph = [
    '11111',
    '00001',
    '00001',
    '00100',
    '01000',
    '10000',
    '11111'
  ];
  glyph.forEach((row, rowIndex) => [...row].forEach((cell, columnIndex) => {
    if (cell === '1') fillRect(map, x + columnIndex * scale, y + rowIndex * scale, scale, scale, color);
  }));
};

const buildIdleMap = () => {
  const map = blankMap();
  drawLimb(map, [[21, 44], [18, 48], [15, 52]], 'b');
  drawLimb(map, [[22, 44], [25, 48], [28, 52]], 'p');
  drawLimb(map, [[43, 44], [46, 48], [49, 52]], 'p');
  drawLimb(map, [[42, 44], [39, 48], [36, 52]], 'b');
  drawFoot(map, 15, 52, -1, 'b');
  drawFoot(map, 28, 52, 1, 'p');
  drawFoot(map, 49, 52, 1, 'p');
  drawFoot(map, 36, 52, -1, 'b');
  drawLimb(map, [[19, 34], [14, 34], [11, 30]], 'b');
  drawLimb(map, [[45, 34], [50, 34], [53, 30]], 'p');
  drawClaw(map, 8, 29, 'left');
  drawClaw(map, 56, 29, 'right');
  paintBody(map);
  fillRect(map, 23, 22, 7, 4, 'p');
  fillRect(map, 34, 22, 7, 4, 'p');
  drawEye(map, 21, 12);
  drawEye(map, 33, 12);
  drawSmile(map);
  fillRect(map, 20, 35, 4, 2, 'r');
  fillRect(map, 40, 35, 4, 2, 'r');
  fillRect(map, 20, 36, 2, 1, 'p');
  fillRect(map, 42, 36, 2, 1, 'd');
  return map;
};

const buildWaveMap = () => {
  const map = blankMap();
  drawLimb(map, [[21, 44], [18, 48], [15, 52]], 'b');
  drawLimb(map, [[22, 44], [25, 48], [28, 52]], 'p');
  drawLimb(map, [[43, 44], [46, 48], [49, 52]], 'p');
  drawLimb(map, [[42, 44], [39, 48], [36, 52]], 'b');
  drawFoot(map, 15, 52, -1, 'b');
  drawFoot(map, 28, 52, 1, 'p');
  drawFoot(map, 49, 52, 1, 'p');
  drawFoot(map, 36, 52, -1, 'b');
  drawLimb(map, [[19, 34], [14, 34], [11, 30]], 'b');
  drawClaw(map, 8, 29, 'left');
  drawLimb(map, [[45, 33], [50, 27], [52, 15]], 'p');
  drawClaw(map, 54, 10, 'right', true);
  paintBody(map);
  fillRect(map, 23, 22, 7, 4, 'p');
  fillRect(map, 34, 22, 7, 4, 'p');
  drawEye(map, 21, 12);
  drawEye(map, 33, 12);
  drawSmile(map);
  fillRect(map, 20, 35, 4, 2, 'r');
  fillRect(map, 40, 35, 4, 2, 'r');
  fillRect(map, 43, 30, 3, 1, 'w');
  return map;
};

const buildSleepMap = () => {
  const map = blankMap();
  drawLimb(map, [[15, 39], [10, 41], [6, 45]], 'b');
  drawLimb(map, [[49, 39], [54, 41], [58, 45]], 'p');
  drawFoot(map, 6, 45, -1, 'b');
  drawFoot(map, 58, 45, 1, 'p');
  paintBody(map, true);
  fillRect(map, 23, 31, 7, 1, 'k');
  fillRect(map, 25, 32, 3, 1, 'k');
  fillRect(map, 34, 31, 7, 1, 'k');
  fillRect(map, 36, 32, 3, 1, 'k');
  drawSmile(map);
  fillRect(map, 20, 35, 4, 2, 'r');
  fillRect(map, 40, 35, 4, 2, 'r');
  drawSleepMark(map, 43, 24, 1, 'k');
  drawSleepMark(map, 50, 17, 1, 'k');
  drawSleepMark(map, 56, 10, 1, 'k');
  return map;
};

const buildCelebrateMap = () => {
  const map = blankMap();
  drawLimb(map, [[21, 44], [18, 48], [15, 52]], 'b');
  drawLimb(map, [[22, 44], [25, 48], [28, 52]], 'p');
  drawLimb(map, [[43, 44], [46, 48], [49, 52]], 'p');
  drawLimb(map, [[42, 44], [39, 48], [36, 52]], 'b');
  drawFoot(map, 15, 52, -1, 'b');
  drawFoot(map, 28, 52, 1, 'p');
  drawFoot(map, 49, 52, 1, 'p');
  drawFoot(map, 36, 52, -1, 'b');
  drawLimb(map, [[19, 33], [15, 25], [13, 15]], 'b');
  drawLimb(map, [[45, 33], [49, 25], [51, 15]], 'p');
  drawClaw(map, 11, 10, 'left', true);
  drawClaw(map, 53, 10, 'right', true);
  paintBody(map);
  fillRect(map, 23, 22, 7, 4, 'p');
  fillRect(map, 34, 22, 7, 4, 'p');
  drawEye(map, 21, 12);
  drawEye(map, 33, 12);
  drawSmile(map, true);
  fillRect(map, 20, 35, 4, 2, 'r');
  fillRect(map, 40, 35, 4, 2, 'r');
  const confetti = [[4, 5, 'w'], [8, 3, 'h'], [17, 3, 'p'], [25, 1, 'd'], [32, 2, 'w'], [42, 3, 'p'], [55, 4, 'd'], [59, 11, 'h'], [4, 18, 'p'], [60, 22, 'w']];
  for (const [x, y, color] of confetti) {
    fillRect(map, x, y, 2, 2, 'o');
    fillRect(map, x, y, 1, 1, color);
  }
  return map;
};

const mapRuns = (map) => {
  const size = map.length;
  const rectangles = new Map();
  let previous = [];
  for (let y = 0; y < size; y += 1) {
    const current = [];
    let x = 0;
    while (x < size) {
      const color = map[y][x];
      if (color === '.') {
        x += 1;
        continue;
      }
      const start = x;
      while (x < size && map[y][x] === color) x += 1;
      const width = x - start;
      const prior = previous.find((run) => run.color === color && run.x === start && run.width === width);
      if (prior) {
        prior.height += 1;
        current.push(prior);
      } else {
        const run = { color, x: start, y, width, height: 1 };
        rectangles.set(run, run);
        current.push(run);
      }
    }
    previous = current;
  }
  return [...rectangles.values()];
};

const mapLayer = (map, { x = 0, y = 0, scale = 8 } = {}) => {
  const runs = mapRuns(map);
  return ['o', 's', 'd', 'm', 'b', 'p', 'w', 'h', 'k', 'r', 'g', 'y', 'u'].map((color) => {
    const paths = runs.filter((run) => run.color === color).map((run) => `M${number(run.x * scale + x)} ${number(run.y * scale + y)}h${number(run.width * scale)}v${number(run.height * scale)}h-${number(run.width * scale)}z`);
    return paths.length ? `<path fill="${PALETTE_LEGEND[color]}" d="${paths.join('')}"/>` : '';
  }).join('');
};

// ── Pixel scene engine ──────────────────────────────────────────────────────
// The mascot and the wordmark are authored on a character grid and emitted as
// hard-edged rectangles. The service illustrations and the glyphs were not:
// they were hand-composed polygons sitting on a blurred ellipse shadow, which
// reads as smooth vector illustration doing an impression of pixel art. The
// helpers below render any rectangular grid through the same path the mascot
// already uses, so the whole site speaks one visual language — hard edges, a
// fixed palette ramp, and dithered shadow instead of blur.

/** Blank grid of any size. '.' is transparent (see PALETTE_LEGEND). */
const grid = (width, height) =>
  Array.from({ length: height }, () => Array(width).fill('.'));

/** Bounds-checked single-pixel write. */
const plot = (map, x, y, color) => {
  const px = Math.round(x);
  const py = Math.round(y);
  if (py >= 0 && py >= 0 && px < map[0].length && py < map.length) map[py][px] = color;
};

/** Filled axis-aligned rectangle, in grid cells. */
const box = (map, x, y, width, height, color) => {
  for (let py = y; py < y + height; py += 1) {
    for (let px = x; px < x + width; px += 1) plot(map, px, py, color);
  }
};

/** Pixel line of the given thickness, stepped rather than interpolated so
 *  it never lands on a half-cell. */
const pixelLine = (map, x1, y1, x2, y2, color, thickness = 1) => {
  const steps = Math.max(Math.abs(x2 - x1), Math.abs(y2 - y1), 1);
  const half = Math.floor((thickness - 1) / 2);
  for (let index = 0; index <= steps; index += 1) {
    const x = Math.round(x1 + ((x2 - x1) * index) / steps);
    const y = Math.round(y1 + ((y2 - y1) * index) / steps);
    box(map, x - half, y - half, thickness, thickness, color);
  }
};

/** Filled pixel ellipse. */
const pixelEllipse = (map, cx, cy, rx, ry, color) => {
  for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y += 1) {
    for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x += 1) {
      const dx = (x + 0.5 - cx) / (rx + 0.5);
      const dy = (y + 0.5 - cy) / (ry + 0.5);
      if (dx * dx + dy * dy <= 1) plot(map, x, y, color);
    }
  }
};

/**
 * Emit a grid as crisp-edged paths, run-length merged per colour.
 * `scale` is the pixel size in user units, so a 60x45 grid at scale 8 is the
 * same 480x360 canvas the vector illustrations used to occupy.
 */
const pixels = (map, { x = 0, y = 0, scale = 8 } = {}) => {
  const width = map[0].length;
  const height = map.length;
  const rectangles = new Map();
  let previous = [];
  for (let row = 0; row < height; row += 1) {
    const current = [];
    let column = 0;
    while (column < width) {
      const color = map[row][column];
      if (color === '.') {
        column += 1;
        continue;
      }
      const start = column;
      while (column < width && map[row][column] === color) column += 1;
      const runWidth = column - start;
      const prior = previous.find(
        (candidate) => candidate.color === color && candidate.x === start && candidate.width === runWidth,
      );
      if (prior) {
        prior.height += 1;
        current.push(prior);
      } else {
        const run = { color, x: start, y: row, width: runWidth, height: 1 };
        rectangles.set(run, run);
        current.push(run);
      }
    }
    previous = current;
  }
  return ['o', 's', 'd', 'm', 'b', 'p', 'w', 'h', 'k', 'r', 'g', 'y', 'u']
    .map((color) => {
      const paths = [...rectangles.values()]
        .filter((run) => run.color === color)
        .map((run) => `M${number(run.x * scale + x)} ${number(run.y * scale + y)}h${number(run.width * scale)}v${number(run.height * scale)}h-${number(run.width * scale)}z`);
      return paths.length ? `<path fill="${PALETTE_LEGEND[color]}" d="${paths.join('')}"/>` : '';
    })
    .join('');
};


/** Filled polygon on any grid (the 64x64 helpers above are hard-bound). */
const fillPoly = (map, points, color) => {
  const ys = points.map(([, y]) => y);
  for (let y = Math.floor(Math.min(...ys)); y <= Math.ceil(Math.max(...ys)); y += 1) {
    for (let x = 0; x < map[0].length; x += 1) {
      let inside = false;
      for (let i = 0, j = points.length - 1; i < points.length; j = i, i += 1) {
        const [xi, yi] = points[i];
        const [xj, yj] = points[j];
        if ((yi > y) !== (yj > y) && x + 0.5 < ((xj - xi) * (y - yi)) / ((yj - yi) || 1) + xi) {
          inside = !inside;
        }
      }
      if (inside) plot(map, x, y, color);
    }
  }
};

/** Point-in-polygon (even-odd ray cast) at a cell centre. */
const insidePoly = (px, py, points) => {
  let inside = false;
  for (let i = 0, j = points.length - 1; i < points.length; j = i, i += 1) {
    const [xi, yi] = points[i];
    const [xj, yj] = points[j];
    if ((yi > py) !== (yj > py) && px < ((xj - xi) * (py - yi)) / ((yj - yi) || 1) + xi) {
      inside = !inside;
    }
  }
  return inside;
};

/**
 * Stroke a polygon's edge in `color` WITHOUT touching its interior.
 * Deliberately not "fill then hollow out": that destroys whatever is already
 * painted inside, which turns a rimmed shape into a black blob. A cell is on
 * the edge when it is inside the polygon and at least one of its four
 * neighbours is not.
 */
const strokePoly = (map, points, color) => {
  const xs = points.map(([x]) => x);
  const ys = points.map(([, y]) => y);
  for (let y = Math.floor(Math.min(...ys)); y <= Math.ceil(Math.max(...ys)); y += 1) {
    for (let x = Math.floor(Math.min(...xs)); x <= Math.ceil(Math.max(...xs)); x += 1) {
      if (!insidePoly(x + 0.5, y + 0.5, points)) continue;
      const exposed =
        !insidePoly(x + 0.5, y + 1.5, points) ||
        !insidePoly(x + 0.5, y - 0.5, points) ||
        !insidePoly(x + 1.5, y + 0.5, points) ||
        !insidePoly(x - 0.5, y + 0.5, points);
      if (exposed) plot(map, x, y, color);
    }
  }
};

/** Ordered-dither a shape's fill: a real pixel-art gradient, and the reason
 *  the illustrations no longer need a blur filter. Fill first, then dither —
 *  the pass is scoped to the polygon's interior so it cannot punch holes in
 *  neighbouring artwork that happens to share the colour. */
const ditherPoly = (map, points, color, density = 2) => {
  for (let y = 0; y < map.length; y += 1) {
    for (let x = 0; x < map[0].length; x += 1) {
      if (map[y][x] !== color) continue;
      if (!insidePoly(x + 0.5, y + 0.5, points)) continue;
      if ((x + y * density) % (density * 2) >= density) plot(map, x, y, '.');
    }
  }
};

const extractGroups = (map, groups) => {
  const base = map.map((row) => [...row]);
  const layers = groups.map(() => blankMap());
  groups.forEach((group, groupIndex) => {
    for (const [x, y, width, height] of group.rects) {
      for (let py = y; py < y + height; py += 1) {
        for (let px = x; px < x + width; px += 1) {
          const color = base[py]?.[px];
          if (!color || color === '.') continue;
          layers[groupIndex][py][px] = color;
          base[py][px] = '.';
        }
      }
    }
  });
  return { base, layers };
};

const CONFETTI_SPOTS = [[4, 5], [8, 3], [17, 3], [25, 1], [32, 2], [42, 3], [55, 4], [59, 11], [4, 18], [60, 22]];

const MASCOT_KEYFRAMES = [
  '@keyframes m-blink{0%,90.5%,93.5%,100%{transform:scaleY(1)}92%{transform:scaleY(.1)}}',
  '@keyframes m-lids{0%,100%{opacity:1}50%{opacity:.45}}',
  '@keyframes m-wave{0%,100%{transform:rotate(-9deg)}50%{transform:rotate(9deg)}}',
  '@keyframes m-twitch-l{0%,18%,100%{transform:rotate(0deg)}4%{transform:rotate(-8deg)}8%{transform:rotate(4deg)}13%{transform:rotate(-2deg)}}',
  '@keyframes m-twitch-r{0%,20%,100%{transform:rotate(0deg)}5%{transform:rotate(8deg)}10%{transform:rotate(-4deg)}15%{transform:rotate(2deg)}}',
  '@keyframes m-z-float{0%{opacity:0;transform:translateY(0)}22%{opacity:.95}60%{opacity:.95}100%{opacity:0;transform:translateY(-18px)}}',
  '@keyframes m-cf{0%,100%{opacity:1}50%{opacity:.25}}',
].join('');

const mascotStyle = (rules) =>
  `<style>@media (prefers-reduced-motion: no-preference){${rules}}${MASCOT_KEYFRAMES}</style>`;

const MASCOT_POSES = {
  idle: {
    name: 'idle',
    description: 'A friendly orange crab resting with chunky raised claws, glossy eyes, a small smile, and a separate soft contact shadow.',
    groups: [
      { cls: 'm-ant m-ant-l', rects: [[23, 22, 7, 4]] },
      { cls: 'm-ant m-ant-r', rects: [[34, 22, 7, 4]] },
      { cls: 'm-eyes', rects: [[21, 12, 10, 12], [33, 12, 10, 12]] },
    ],
    rules: '.m-ant-l{transform-box:fill-box;transform-origin:100% 100%;animation:m-twitch-l 5.2s infinite}.m-ant-r{transform-box:fill-box;transform-origin:0 100%;animation:m-twitch-r 4.6s infinite}.m-eyes{transform-box:fill-box;transform-origin:50% 58%;animation:m-blink 5.4s infinite}',
  },
  wave: {
    name: 'waving',
    description: 'A friendly orange crab lifting one tiny claw in a wave, shaded with a warm top light.',
    groups: [
      { cls: 'm-ant m-ant-l', rects: [[23, 22, 7, 4]] },
      { cls: 'm-ant m-ant-r', rects: [[34, 22, 7, 4]] },
      { cls: 'm-claw', rects: [[48, 2, 15, 17]] },
      { cls: 'm-eyes', rects: [[21, 12, 10, 12], [33, 12, 10, 12]] },
    ],
    rules: '.m-ant-l{transform-box:fill-box;transform-origin:100% 100%;animation:m-twitch-l 5.6s infinite}.m-ant-r{transform-box:fill-box;transform-origin:0 100%;animation:m-twitch-r 4.9s infinite}.m-claw{transform-box:fill-box;transform-origin:25% 86%;animation:m-wave 1.15s ease-in-out infinite}.m-eyes{transform-box:fill-box;transform-origin:50% 58%;animation:m-blink 4.6s infinite}',
  },
  sleep: {
    name: 'sleeping',
    description: 'A friendly orange crab lying down with closed eyes and tiny pixel sleep marks.',
    groups: [
      { cls: 'm-eyes', rects: [[23, 31, 7, 2], [34, 31, 7, 2]] },
      { cls: 'm-z m-z1', rects: [[43, 24, 5, 7]] },
      { cls: 'm-z m-z2', rects: [[50, 17, 5, 7]] },
      { cls: 'm-z m-z3', rects: [[56, 10, 5, 7]] },
    ],
    rules: '.m-eyes{animation:m-lids 6.5s ease-in-out infinite}.m-z1{animation:m-z-float 3.2s linear -.4s infinite}.m-z2{animation:m-z-float 3.2s linear -1.5s infinite}.m-z3{animation:m-z-float 3.2s linear -2.6s infinite}',
  },
  celebrate: {
    name: 'celebrating',
    description: 'A joyful orange crab with both claws raised, a wide smile, and small warm confetti pixels.',
    groups: [
      ...CONFETTI_SPOTS.map(([x, y], index) => ({
        cls: `m-cf m-cf${index}`,
        rects: [[x, y, 2, 2]],
      })),
      { cls: 'm-ant m-ant-l', rects: [[23, 22, 7, 4]] },
      { cls: 'm-ant m-ant-r', rects: [[34, 22, 7, 4]] },
      { cls: 'm-eyes', rects: [[21, 12, 10, 12], [33, 12, 10, 12]] },
    ],
    rules: [
      '.m-eyes{transform-box:fill-box;transform-origin:50% 58%;animation:m-blink 4.2s infinite}',
      '.m-ant-l{transform-box:fill-box;transform-origin:100% 100%;animation:m-twitch-l 5.1s infinite}',
      '.m-ant-r{transform-box:fill-box;transform-origin:0 100%;animation:m-twitch-r 4.4s infinite}',
      ...CONFETTI_SPOTS.map(([, ], index) => `.m-cf${index}{animation:m-cf ${(1.3 + (index % 5) * 0.22).toFixed(2)}s ease-in-out ${(-(index * 0.17)).toFixed(2)}s infinite}`),
    ].join(''),
  },
};

const mascotSvg = (pose) => {
  const config = MASCOT_POSES[pose];
  const { base, layers } = extractGroups(ASCII_MAPS[pose], config.groups);
  const groups = config.groups
    .map((group, index) => `<g class="${group.cls}" shape-rendering="crispEdges">${mapLayer(layers[index])}${group.smil ?? ''}</g>`)
    .join('');
  return svgDocument({
    width: 512,
    height: 512,
    viewBox: '0 0 512 512',
    idPrefix: `mascot-${pose}`,
    title: `grr mascot ${config.name}`,
    description: config.description,
    body: `${contactShadow(256, 480, 112, 14)}${mascotStyle(config.rules)}<g shape-rendering="crispEdges">${mapLayer(base)}</g>${groups}`,
  });
};

const terminalCrab = (map, x, y, scale) => `<g shape-rendering="crispEdges">${mapLayer(map, { x, y, scale })}</g>`;

const htBar = (x, y, width, fill, steps, delay) =>
  `<path class="ht-ln" d="M${number(x)} ${number(y + 8)}h${number(width)}" fill="none" stroke="${fill}" stroke-width="16" pathLength="100" style="--steps:${number(steps)};--ln-delay:${number(delay)}ms"/>`;

const heroTerminal = () => {
  const idle = ASCII_MAPS.idle;
  const frame = `${polygon([[96, 448], [184, 584], [832, 504], [856, 192], [760, 96], [128, 160]], PALETTE_LEGEND.o)}${polygon([[112, 448], [192, 568], [816, 496], [840, 200], [752, 112], [144, 168]], PALETTE_LEGEND.d)}${polygon([[128, 160], [760, 96], [864, 200], [232, 280]], PALETTE_LEGEND.m)}${polygon([[144, 160], [752, 104], [824, 176], [232, 248]], PALETTE_LEGEND.b)}${polygon([[160, 160], [744, 112], [768, 132], [208, 184]], PALETTE_LEGEND.p)}${polygon([[96, 448], [128, 160], [232, 280], [192, 568]], PALETTE_LEGEND.s)}${polygon([[112, 432], [144, 192], [208, 288], [184, 520]], PALETTE_LEGEND.d)}${polygon([[232, 280], [864, 200], [832, 504], [192, 584]], PALETTE_LEGEND.o)}${polygon([[248, 288], [840, 216], [812, 480], [216, 544]], PALETTE_LEGEND.c)}${polygon([[256, 300], [824, 228], [800, 468], [232, 532]], PALETTE_LEGEND.p, 'class="ht-glow" opacity=".14"')}${polygon([[248, 288], [840, 216], [832, 248], [256, 320]], PALETTE_LEGEND.h)}${polygon([[264, 312], [792, 248], [768, 448], [240, 512]], PALETTE_LEGEND.k)}${polygon([[280, 328], [776, 268], [760, 420], [264, 480]], PALETTE_LEGEND.k)}${polygon([[288, 336], [768, 280], [760, 332], [288, 388]], PALETTE_LEGEND.s, 'opacity=".5"')}${rect(224, 536, 48, 16, PALETTE_LEGEND.d)}${rect(712, 464, 64, 16, PALETTE_LEGEND.p)}${rect(624, 472, 56, 16, PALETTE_LEGEND.b)}${rect(128, 128, 64, 8, PALETTE_LEGEND.h, 'opacity=".8"')}`;
  const lines = `${rect(320, 352, 32, 24, PALETTE_LEGEND.p, `class="ht-prompt" pathLength="100" stroke="${PALETTE_LEGEND.p}" stroke-width="6"`)}${htBar(376, 344, 128, PALETTE_LEGEND.w, 8, 140)}${htBar(376, 376, 200, PALETTE_LEGEND.d, 13, 280)}${htBar(320, 408, 176, PALETTE_LEGEND.b, 11, 420)}${htBar(536, 400, 160, PALETTE_LEGEND.m, 10, 480)}${htBar(320, 440, 104, PALETTE_LEGEND.h, 7, 620)}${htBar(456, 432, 208, PALETTE_LEGEND.s, 13, 700)}${rect(696, 440, 32, 24, PALETTE_LEGEND.w, 'class="ht-caret"')}`;
  const body = `${contactShadow(480, 568, 348, 28)}<g class="ht-frame" shape-rendering="crispEdges">${frame}</g><g class="ht-lines" shape-rendering="crispEdges">${lines}</g><g class="ht-crab">${terminalCrab(idle, 664, 0, 3)}</g>`;
  return svgDocument({ width: 960, height: 640, viewBox: '0 0 960 640', idPrefix: 'ht', title: 'grr terminal', description: 'A dimensional cream and orange terminal with a glowing screen and a tiny crab perched on its top edge.', body });
};

// Service illustrations: 60x45 grids at scale 8, which is the same 480x360
// canvas the old hand-composed polygons occupied. Same subjects, same palette,
// but every edge is now on the pixel grid and every shadow is a dither — which
// is what makes them read as pixel art rather than vector illustration.
const ART_W = 60;
const ART_H = 45;
const ART_SCALE = 8;

const artCanvas = (title, description, object) =>
  svgDocument({
    width: ART_W * ART_SCALE,
    height: ART_H * ART_SCALE,
    viewBox: `0 0 ${ART_W * ART_SCALE} ${ART_H * ART_SCALE}`,
    title,
    description,
    body:
      '<g shape-rendering="crispEdges">' +
      // The shadow is emitted first so the sprite always paints over it, and
      // both are already-flat path runs — there is nothing to composite, so no
      // grid-merge layer is needed.
      ditherShadowBox(240, 328, 152, 16, ART_SCALE) +
      pixels(object, { scale: ART_SCALE }) +
      '</g>',
  });

const artMail = () => {
  const map = grid(ART_W, ART_H);
  // Blue paper plane lifting out of the top-right corner.
  fillPoly(map, [[37, 4], [52, 8], [42, 17]], 'u');
  strokePoly(map, [[37, 4], [52, 8], [42, 17]], 'o');
  pixelLine(map, 37, 4, 42, 17, 'h', 1);
  // Envelope: cream body, ink rim, folded-down orange flap.
  box(map, 11, 18, 38, 17, 'h');
  strokePoly(map, [[11, 18], [48, 18], [48, 34], [11, 34]], 'o');
  fillPoly(map, [[13, 20], [46, 20], [30, 30]], 'b');
  strokePoly(map, [[13, 20], [46, 20], [30, 30]], 'o');
  // A dithered band along the bottom reads as the envelope's thickness.
  fillPoly(map, [[12, 30], [47, 30], [47, 33], [12, 33]], 'w');
  ditherPoly(map, [[12, 30], [47, 30], [47, 33], [12, 33]], 'w', 2);
  return artCanvas(
    'Mail illustration',
    'Pixel art: a cream envelope with an orange folded flap and a blue paper plane lifting out of it.',
    map,
  );
};

const artCalendar = () => {
  const map = grid(ART_W, ART_H);
  // Two hanger tabs poking above the sheet.
  box(map, 17, 6, 3, 7, 'o');
  box(map, 40, 6, 3, 7, 'o');
  const sheet = [[11, 12], [48, 12], [48, 37], [11, 37]];
  fillPoly(map, sheet, 'h');
  strokePoly(map, sheet, 'o');
  // Orange header band under the rim.
  box(map, 12, 13, 36, 6, 'm');
  box(map, 12, 13, 36, 2, 'p');
  box(map, 12, 19, 36, 1, 'o');
  // Date grid: alternating filled cells, all on the pixel grid.
  for (let row = 0; row < 3; row += 1) {
    for (let column = 0; column < 4; column += 1) {
      const x = 15 + column * 8;
      const y = 23 + row * 4;
      const filled = (row + column) % 3 === 0;
      box(map, x, y, 5, 2, filled ? 'b' : 'w');
      if (filled) box(map, x, y, 5, 1, 'p');
    }
  }
  return artCanvas(
    'Calendar illustration',
    'Pixel art: a calendar sheet with an orange header band, two hanger tabs, and a grid of date cells.',
    map,
  );
};

const artDrive = () => {
  const map = grid(ART_W, ART_H);
  // A cream page peeking out from behind the folder.
  fillPoly(map, [[16, 10], [44, 10], [44, 30], [16, 30]], 'h');
  strokePoly(map, [[16, 10], [44, 10], [44, 30], [16, 30]], 'o');
  box(map, 19, 14, 22, 2, 'w');
  box(map, 19, 18, 16, 2, 'w');
  // Folder back with its tab.
  const back = [[9, 18], [24, 18], [27, 22], [51, 22], [51, 36], [9, 36]];
  fillPoly(map, back, 'd');
  strokePoly(map, back, 'o');
  // Front panel in the brighter orange, with a dithered lip.
  const front = [[11, 24], [50, 24], [50, 35], [11, 35]];
  fillPoly(map, front, 'm');
  strokePoly(map, front, 'o');
  fillPoly(map, [[12, 25], [49, 25], [49, 28], [12, 28]], 'p');
  fillPoly(map, [[12, 31], [49, 31], [49, 34], [12, 34]], 'b');
  ditherPoly(map, [[12, 31], [49, 31], [49, 34], [12, 34]], 'b', 3);
  return artCanvas(
    'Drive illustration',
    'Pixel art: an orange folder with a tab, a cream document peeking out behind it, and a dithered front lip.',
    map,
  );
};

const artContacts = () => {
  const map = grid(ART_W, ART_H);
  const card = [[10, 12], [50, 12], [50, 37], [10, 37]];
  fillPoly(map, card, 'h');
  strokePoly(map, card, 'o');
  // Avatar: an orange head over shoulders.
  pixelEllipse(map, 21, 21, 5, 5, 'b');
  fillPoly(map, [[14, 31], [28, 31], [28, 36], [14, 36]], 'm');
  pixelEllipse(map, 21, 20, 2, 2, 'o');
  box(map, 20, 19, 2, 1, 'h');
  box(map, 20, 21, 2, 1, 'h');
  // Two text lines to the right of the avatar.
  box(map, 30, 17, 16, 2, 'd');
  box(map, 30, 21, 11, 2, 'p');
  // A dithered detail row across the bottom.
  fillPoly(map, [[14, 33], [45, 33], [45, 35], [14, 35]], 'w');
  ditherPoly(map, [[14, 33], [45, 33], [45, 35], [14, 35]], 'w', 2);
  return artCanvas(
    'Contacts illustration',
    'Pixel art: a cream contact card with an orange avatar, two text lines, and a dithered detail row.',
    map,
  );
};

const artChat = () => {
  const map = grid(ART_W, ART_H);
  // Big orange bubble with a tail.
  const big = [[10, 11], [46, 11], [46, 29], [24, 29], [19, 35], [20, 29], [10, 29]];
  fillPoly(map, big, 'm');
  strokePoly(map, big, 'o');
  fillPoly(map, [[11, 12], [45, 12], [45, 15], [11, 15]], 'p');
  // Small cream bubble overlapping, offset to the lower right.
  const small = [[26, 24], [48, 24], [48, 35], [42, 35], [40, 39], [39, 35], [26, 35]];
  fillPoly(map, small, 'c');
  strokePoly(map, small, 'o');
  // Three dots in the big bubble, one reaction dot on the small one.
  box(map, 16, 20, 3, 3, 'h');
  box(map, 23, 20, 3, 3, 'h');
  box(map, 30, 20, 3, 3, 'h');
  box(map, 33, 28, 3, 3, 'g');
  return artCanvas(
    'Chat illustration',
    'Pixel art: a large orange speech bubble with three dots, a smaller cream bubble, and a green reaction dot.',
    map,
  );
};

const artForms = () => {
  const map = grid(ART_W, ART_H);
  // Cream sheet with a clipped top-right corner, rimmed in ink.
  const sheet = [[11, 8], [42, 8], [47, 13], [47, 38], [11, 38]];
  fillPoly(map, sheet, 'h');
  strokePoly(map, sheet, 'o');
  // Two checked boxes with orange rules beside them.
  [16, 26].forEach((y, index) => {
    box(map, 16, y, 5, 5, 'o');
    box(map, 17, y + 1, 3, 3, 'g');
    pixelLine(map, 17, y + 2, 18, y + 3, 'h', 1);
    pixelLine(map, 18, y + 3, 20, y, 'h', 1);
    box(map, 24, y, 16 - index * 3, 2, index === 0 ? 'd' : 'p');
    box(map, 24, y + 3, 10, 1, 'w');
  });
  // Pencil leaning against the bottom-right corner.
  fillPoly(map, [[44, 22], [47, 25], [37, 37], [33, 34]], 'b');
  strokePoly(map, [[44, 22], [47, 25], [37, 37], [33, 34]], 'o');
  fillPoly(map, [[33, 34], [37, 37], [32, 39]], 'h');
  strokePoly(map, [[33, 34], [37, 37], [32, 39]], 'o');
  return artCanvas(
    'Forms illustration',
    'Pixel art: a cream form sheet with two checked boxes, ruled lines, and a pencil leaning on the corner.',
    map,
  );
};

// Glyphs are authored as 16x16 character grids and emitted at scale 3, so the
// canvas stays the 48x48 the templates already size them to. Hand-placing the
// pixels is the whole point: the old rect/polygon glyphs were smooth vector
// shapes wearing a `shape-rendering="crispEdges"` hat.
const GLYPH_SCALE = 3;

const parseGrid = (name, rows) => {
  const cells = rows.map((row) => [...row]);
  const width = cells[0].length;
  if (cells.some((row) => row.length !== width)) {
    const widths = cells.map((row) => row.length).join(', ');
    throw new Error(`${name}: ragged grid — row widths are [${widths}]`);
  }
  for (const row of cells) {
    for (const cell of row) {
      if (!(cell in PALETTE_LEGEND)) {
        throw new Error(`${name}: unknown palette cell '${cell}'`);
      }
    }
  }
  return cells;
};

const glyph = (title, rows) => {
  const map = parseGrid(title, rows);
  return svgDocument({
    width: 48,
    height: 48,
    viewBox: '0 0 48 48',
    title,
    description: `${title} — a 16x16 pixel sprite on the grr palette`,
    body: `<g shape-rendering="crispEdges">${pixels(map, { scale: GLYPH_SCALE })}</g>`,
  });
};

const glyphMail = () => glyph('Mail glyph', [
  '................',
  '.oooooooooooooo.',
  '.ohhhhhhhhhhhho.',
  '.ohwwwwwwwwwwho.',
  '.ohwwwwwwwwwwho.',
  '.ohwwwbwwwwbwho.',
  '.ohwwwwwbwwwwho.',
  '.ohwwwwwwbwwwwo.',
  '.ohwwwwwwbwwwwo.',
  '.ohwwwwwbwwwwho.',
  '.ohwwwbwwwwbwho.',
  '.ohwwwwwwwwwwho.',
  '.ohhhhhhhhhhhho.',
  '.oooooooooooooo.',
  '................',
  '................',
]);

const glyphCalendar = () => glyph('Calendar glyph', [
  '................',
  '.oooooooooooooo.',
  '.oddddddddddddo.',
  '.oddddddddddddo.',
  '.ohhhhhhhhhhhho.',
  '.ohhhwwhhhhwwho.',
  '.ohhhhhhhhhhhho.',
  '.ohhhwwhhhhwwho.',
  '.ohhhhhhhhhhhho.',
  '.ohhhwwhhhhwwho.',
  '.ohhhhhhhhhhhho.',
  '.ohhhwwhhhhwwho.',
  '.ohhhhhhhhhhhho.',
  '.oooooooooooooo.',
  '................',
  '................',
]);

const glyphDrive = () => glyph('Drive glyph', [
  '................',
  '................',
  '..oooooooo......',
  '..obbbbbboo.....',
  '.obbbbbbbboo....',
  '.obbbbbbbbbbo...',
  '.obbbbbbbbbbbo..',
  '.obbbbbbbbbbbbo.',
  '.obbbbbbbbbbbbo.',
  '.obbbbbbbbbbbbo.',
  '.oppppppppppppo.',
  '.oppppppppppppo.',
  '.oooooooooooooo.',
  '................',
  '................',
  '................',
]);

const glyphContacts = () => glyph('Contacts glyph', [
  '................',
  '.oooooooooooooo.',
  '.ohhhhhhhhhhhho.',
  '.ohhwwwwwwwwhho.',
  '.ohwwbbwwwwwwho.',
  '.ohwwbbwwwwwwho.',
  '.ohwwwwwwwwwwho.',
  '.ohhhhhhhhhhhho.',
  '.ohwwwwwwwwwwho.',
  '.ohwwwwwwwwwwho.',
  '.ohwwwwwwwwwwho.',
  '.ohwwwwwwwwwwho.',
  '.ohhhhhhhhhhhho.',
  '.oooooooooooooo.',
  '................',
  '................',
]);

const glyphChat = () => glyph('Chat glyph', [
  '................',
  '..oooooooooooo..',
  '.ohhhhhhhhhhhho.',
  '.ohwwwwwwwwwwho.',
  '.ohwwwwwwwwwwho.',
  '.ohwwwwwwwwwwho.',
  '.ohhhhhhhhhhhho.',
  '.ohhhhhhhhhhhho.',
  '.ohhhhhhhhhhhho.',
  '..oooooooooooo..',
  '...obbbbbbo.....',
  '....obbbbo......',
  '.....obbo.......',
  '......oo........',
  '................',
  '................',
]);

const glyphForms = () => glyph('Forms glyph', [
  '................',
  '.oooooooooooooo.',
  '.ohhhhhhhhhhhho.',
  '.ohgghhhhhhhhho.',
  '.ohgghhhhhhhhho.',
  '.ohhhhhhhhhhhho.',
  '.ohwwwwwwwwwwho.',
  '.ohhhhhhhhhhhho.',
  '.ohgghhhhhhhhho.',
  '.ohgghhhhhhhhho.',
  '.ohhhhhhhhhhhho.',
  '.ohwwwwwwwwwwho.',
  '.ohhhhhhhhhhhho.',
  '.oooooooooooooo.',
  '................',
  '................',
]);

const makeHeadMap = () => {
  const map = Array.from({ length: 32 }, () => Array(32).fill('.'));
  const set = (x, y, color) => { if (x >= 0 && y >= 0 && x < 32 && y < 32) map[y][x] = color; };
  const ellipse = (cx, cy, rx, ry, color) => { for (let y = 0; y < 32; y += 1) for (let x = 0; x < 32; x += 1) if (((x + 0.5 - cx) / rx) ** 2 + ((y + 0.5 - cy) / ry) ** 2 <= 1) set(x, y, color); };
  ellipse(16, 19, 14, 10, 'o');
  ellipse(16, 19, 12.5, 8.5, 'm');
  ellipse(14, 17, 10, 6, 'b');
  ellipse(11, 14, 5, 3, 'p');
  ellipse(8, 12, 2, 1, 'h');
  for (const [x, y, color] of [[3, 15, 'o'], [2, 18, 'o'], [4, 21, 'o'], [28, 15, 'o'], [29, 18, 'o'], [27, 21, 'o'], [5, 16, 'b'], [26, 16, 'p'], [5, 20, 'd'], [26, 20, 'd']]) set(x, y, color);
  for (const [x, y] of [[10, 10], [11, 10], [10, 11], [20, 10], [21, 10], [20, 11]]) { set(x, y, 'o'); set(x + 1, y, 'h'); set(x, y + 1, 'h'); }
  for (const [x, y] of [[11, 12], [12, 12], [11, 13], [21, 12], [22, 12], [21, 13]]) set(x, y, 'k');
  set(12, 12, 'h'); set(22, 12, 'h');
  for (const [x, y] of [[13, 21], [14, 22], [15, 22], [16, 22], [17, 22], [18, 22], [19, 21]]) set(x, y, 'k');
  set(8, 20, 'r'); set(23, 20, 'r');
  set(14, 25, 'd'); set(18, 25, 'd');
  return map;
};

const HEAD_MAP = makeHeadMap();

const headLayer = (x, y, scale) => `<g shape-rendering="crispEdges">${mapRuns(HEAD_MAP).map((run) => `<path fill="${PALETTE_LEGEND[run.color]}" d="M${run.x * scale + x} ${run.y * scale + y}h${run.width * scale}v${run.height * scale}h-${run.width * scale}z"/>`).join('')}</g>`;

const FONT = Object.freeze({
  ' ': ['00000', '00000', '00000', '00000', '00000', '00000', '00000'],
  G: ['01110', '10001', '10000', '10111', '10001', '10001', '01111'],
  g: ['00000', '01111', '10001', '10001', '01111', '00001', '01110'],
  o: ['00000', '01110', '10001', '10001', '10001', '10001', '01110'],
  l: ['00100', '00100', '00100', '00100', '00100', '00100', '01110'],
  e: ['00000', '01110', '10001', '11111', '10000', '10000', '01111'],
  t: ['01000', '01000', '11110', '01000', '01000', '01001', '00110'],
  s: ['00000', '01111', '10000', '01110', '00001', '00001', '11110'],
  f: ['00110', '01001', '01000', '11110', '01000', '01000', '01000'],
  r: ['00000', '10110', '11001', '10000', '10000', '10000', '10000'],
  m: ['00000', '11010', '10101', '10101', '10101', '10101', '10101'],
  h: ['10000', '10000', '10110', '11001', '10001', '10001', '10001'],
  i: ['00100', '00000', '01100', '00100', '00100', '00100', '01110'],
  n: ['00000', '10110', '11001', '10001', '10001', '10001', '10001'],
  a: ['00000', '01110', '00001', '01111', '10001', '10001', '01111'],
  '-': ['00000', '00000', '00000', '11111', '00000', '00000', '00000'],
  '/': ['00001', '00010', '00010', '00100', '01000', '01000', '10000'],
  '.': ['00000', '00000', '00000', '00000', '00000', '00110', '00110'],
  ':': ['00000', '00110', '00110', '00000', '00110', '00110', '00000']
});

const pixelText = (text, x, y, scale, fill, extra = '') => {
  const paths = [];
  [...text].forEach((character, index) => {
    const glyph = FONT[character] ?? FONT[' '];
    glyph.forEach((row, rowIndex) => {
      [...row].forEach((cell, columnIndex) => {
        if (cell !== '1') return;
        const px = x + (index * 6 + columnIndex) * scale;
        const py = y + rowIndex * scale;
        paths.push(`M${px} ${py}h${scale}v${scale}h-${scale}z`);
      });
    });
  });
  return paths.length ? `<path fill="${fill}" d="${paths.join('')}"${extra ? ` ${extra}` : ''}/>` : '';
};

/**
 * Wrap the mark in a slow idle bob, so the tab icon and the header logo
 * breathe instead of sitting dead still.
 *
 * CSS rather than SMIL, deliberately: SMIL cannot be switched off by
 * `prefers-reduced-motion`, and an always-animating tab icon is exactly the
 * kind of motion that query exists to stop. `cell` is the user-unit size of one
 * pixel cell, so the travel is half a pixel whatever the render scale — without
 * it a 256px logo would bob eight times further than a 32px favicon.
 */
const idleMark = (cell, body) =>
  '<style>@media (prefers-reduced-motion: no-preference){' +
  '.grr-idle{animation:grr-idle 3.4s ease-in-out infinite}' +
  `@keyframes grr-idle{0%,100%{transform:translateY(0)}50%{transform:translateY(${number(-cell / 2)}px)}}` +
  '}</style><g class="grr-idle">' +
  body +
  '</g>';

const faviconSvg = (width) => svgDocument({
  width,
  height: width,
  viewBox: '0 0 32 32',
  title: 'grr',
  description: 'A compact orange crab head with dimensional pixel shading.',
  body: idleMark(1, headLayer(0, 0, 1)),
});

const logoSvg = () => svgDocument({
  width: 256,
  height: 256,
  viewBox: '0 0 256 256',
  title: 'Google Rust Rewrite',
  description: 'The dimensional orange grr crab mascot.',
  body: idleMark(8, headLayer(0, 0, 8)),
});

const logoWordmarkSvg = () => svgDocument({
  width: 640,
  height: 256,
  viewBox: '0 0 640 256',
  title: 'Google Rust Rewrite',
  description: 'The grr crab mascot with its pixel wordmark.',
  body: `${headLayer(0, 0, 8)}${pixelText('grr', 288, 72, 16, PALETTE_LEGEND.o)}${pixelText('grr', 280, 64, 16, PALETTE_LEGEND.p)}${rect(296, 80, 16, 8, PALETTE_LEGEND.h)}`
});

const ogSvg = () => {
  const body = `${rect(0, 0, 1200, 630, PALETTE_LEGEND.c)}${polygon([[0, 0], [1200, 0], [1200, 176], [0, 328]], PALETTE_LEGEND.h, 'opacity=".55"')}${polygon([[0, 502], [1200, 348], [1200, 630], [0, 630]], PALETTE_LEGEND.w, 'opacity=".6"')}${contactShadow(236, 496, 170, 22)}${terminalCrab(ASCII_MAPS.idle, 40, 168, 5)}${pixelText('grr', 560, 112, 24, PALETTE_LEGEND.s, 'opacity=".22"')}${pixelText('grr', 552, 104, 24, PALETTE_LEGEND.m)}${pixelText('grr', 544, 96, 24, PALETTE_LEGEND.p)}${rect(568, 112, 32, 16, PALETTE_LEGEND.h, 'opacity=".75"')}${pixelText('Google tools from the terminal', 480, 304, 4, PALETTE_LEGEND.k)}${polygon([[632, 424], [1096, 384], [1136, 448], [672, 488]], PALETTE_LEGEND.o)}${polygon([[648, 428], [1088, 392], [1116, 440], [676, 476]], PALETTE_LEGEND.d)}${polygon([[664, 432], [1080, 400], [1096, 428], [680, 460]], PALETTE_LEGEND.m)}${polygon([[680, 440], [1072, 412], [1080, 430], [688, 458]], PALETTE_LEGEND.p)}${rect(728, 430, 24, 16, PALETTE_LEGEND.h)}${rect(768, 426, 80, 12, PALETTE_LEGEND.w)}${rect(864, 422, 128, 12, PALETTE_LEGEND.b)}${rect(768, 450, 168, 12, PALETTE_LEGEND.s)}$`;
  return svgDocument({ width: 1200, height: 630, viewBox: '0 0 1200 630', title: 'grr — Google tools from the terminal', description: 'A warm paper product card with the dimensional grr crab, wordmark, tagline, and a small terminal slab.', body });
};

const ASCII_SOURCES = Object.freeze({
  idle: buildIdleMap().map((row) => row.join('')).join('\n'),
  wave: buildWaveMap().map((row) => row.join('')).join('\n'),
  sleep: buildSleepMap().map((row) => row.join('')).join('\n'),
  celebrate: buildCelebrateMap().map((row) => row.join('')).join('\n')
});

const ASCII_MAPS = Object.freeze(Object.fromEntries(
  Object.entries(ASCII_SOURCES).map(([name, source]) => [name, parseAscii(name, source)])
));

const files = new Map([
  [resolve(assetDirectory, 'mascot-idle.svg'), mascotSvg('idle')],
  [resolve(assetDirectory, 'mascot-wave.svg'), mascotSvg('wave')],
  [resolve(assetDirectory, 'mascot-sleep.svg'), mascotSvg('sleep')],
  [resolve(assetDirectory, 'mascot-celebrate.svg'), mascotSvg('celebrate')],
  [resolve(assetDirectory, 'hero-terminal.svg'), heroTerminal()],
  [resolve(assetDirectory, 'art-mail.svg'), artMail()],
  [resolve(assetDirectory, 'art-calendar.svg'), artCalendar()],
  [resolve(assetDirectory, 'art-drive.svg'), artDrive()],
  [resolve(assetDirectory, 'art-contacts.svg'), artContacts()],
  [resolve(assetDirectory, 'art-chat.svg'), artChat()],
  [resolve(assetDirectory, 'art-forms.svg'), artForms()],
  [resolve(assetDirectory, 'glyph-mail.svg'), glyphMail()],
  [resolve(assetDirectory, 'glyph-calendar.svg'), glyphCalendar()],
  [resolve(assetDirectory, 'glyph-drive.svg'), glyphDrive()],
  [resolve(assetDirectory, 'glyph-contacts.svg'), glyphContacts()],
  [resolve(assetDirectory, 'glyph-chat.svg'), glyphChat()],
  [resolve(assetDirectory, 'glyph-forms.svg'), glyphForms()],
  [resolve(publicDirectory, 'favicon.svg'), faviconSvg(32)],
  [resolve(publicDirectory, 'favicon-16.svg'), faviconSvg(16)],
  [resolve(publicDirectory, 'favicon-32.svg'), faviconSvg(32)],
  [resolve(publicDirectory, 'logo.svg'), logoSvg()],
  [resolve(publicDirectory, 'logo-wordmark.svg'), logoWordmarkSvg()],
  [resolve(publicDirectory, 'og.svg'), ogSvg()]
]);

const rejected = [
  resolve(publicDirectory, 'assets', 'patterns', 'dither-tile.svg'),
  resolve(publicDirectory, 'assets', 'patterns', 'halftone-tile.svg'),
  resolve(assetDirectory, 'corner-ornament.svg'),
  resolve(assetDirectory, 'terminal-frame.svg')
];

mkdirSync(assetDirectory, { recursive: true });
mkdirSync(publicDirectory, { recursive: true });

for (const [path, content] of files) {
  writeFileSync(path, content, 'utf8');
}

for (const path of rejected) {
  if (existsSync(path)) rmSync(path);
}

console.log(`Generated ${files.size} SVG assets with 64x64 mascot pixel maps.`);
