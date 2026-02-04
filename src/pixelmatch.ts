import type { ImageLike, PixelData, PixelmatchOptions, PixelmatchResult } from './types.js';
import { buildResult, validateInput } from './validate.js';

export type { ImageLike, PixelData, PixelmatchOptions, PixelmatchResult } from './types.js';

/**
 * Compare two equally sized images, pixel by pixel.
 *
 * @return A PixelmatchResult with diffCount, diffPercentage, totalPixels, aaCount, and identical.
 */
export default function pixelmatch(img1: ImageLike, img2: ImageLike, options: PixelmatchOptions = {}): PixelmatchResult {
  const {
    threshold = 0.1,
    alpha = 0.1,
    aaColor = [255, 255, 0],
    diffColor = [255, 0, 0],
    detectAntiAliasing = true,
    diffColorAlt,
    diffMask,
    output,
  } = options;

  validateInput(img1, img2, output);

  const { data: data1, width, height } = img1;
  const { data: data2 } = img2;

  // check if images are identical
  const len = width * height;
  const a32 = new Uint32Array(data1.buffer, data1.byteOffset, len);
  const b32 = new Uint32Array(data2.buffer, data2.byteOffset, len);
  let identical = true;

  for (let i = 0; i < len; i++) {
    if (a32[i] !== b32[i]) {
      identical = false;
      break;
    }
  }
  if (identical) {
    // fast path if identical
    if (output && !diffMask) {
      for (let i = 0; i < len; i++) drawGrayPixel(data1, 4 * i, alpha, output);
    }
    return buildResult(0, 0, len, true);
  }

  // maximum acceptable square distance between two colours;
  // 35215 is the maximum possible value for the YIQ difference metric
  const maxDelta = 35215 * threshold * threshold;
  const [aaR, aaG, aaB] = aaColor;
  const [diffR, diffG, diffB] = diffColor;
  const [altR, altG, altB] = diffColorAlt || diffColor;
  let diff = 0;
  let aaCount = 0;

  // compare each pixel of one image against the other one
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = y * width + x;
      const pos = i * 4;

      // squared YUV distance between colours at this pixel position, negative if the img2 pixel is darker
      const delta = a32[i] === b32[i] ? 0 : colorDelta(data1, data2, pos, pos, false);

      // the colour difference is above the threshold
      if (Math.abs(delta) > maxDelta) {
        // check it's a real rendering difference or just anti-aliasing
        const isAA =
          detectAntiAliasing &&
          (antialiased(data1, x, y, width, height, a32, b32) ||
            antialiased(data2, x, y, width, height, b32, a32));
        if (isAA) {
          aaCount++;
          // one of the pixels is anti-aliasing; draw as yellow and do not count as difference
          // note that we do not include such pixels in a mask
          if (output && !diffMask) drawPixel(output, pos, aaR, aaG, aaB);
        } else {
          // found substantial difference not caused by anti-aliasing; draw it as such
          if (output) {
            if (delta < 0) {
              drawPixel(output, pos, altR, altG, altB);
            } else {
              drawPixel(output, pos, diffR, diffG, diffB);
            }
          }
          diff++;
        }
      } else if (output && !diffMask) {
        // pixels are similar; draw background as grayscale image blended with white
        drawGrayPixel(data1, pos, alpha, output);
      }
    }
  }

  return buildResult(diff, aaCount, len, false);
}

/**
 * Check if a pixel is likely a part of anti-aliasing;
 * based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009
 *
 * DEVIATION FROM UPSTREAM (mapbox/pixelmatch):
 * Two changes to improve detection for thin strokes, text, and small features
 * (ref: https://github.com/mapbox/pixelmatch/issues/74):
 *
 * 1. Two-pass approach: the original algorithm only checks hasManySiblings on the
 *    last neighbour found with the min/max delta, missing tied candidates. This made
 *    results depend on loop iteration order. We now find min/max in pass 1, then check
 *    ALL matching neighbours in pass 2.
 *
 * 2. Relaxed sibling check: changed from requiring hasManySiblings in both images
 *    (a32 AND b32) to either image (a32 OR b32). For 1px-wide strokes, the stroke-side
 *    extreme never has 3+ identical siblings because the feature is too narrow. Requiring
 *    siblings in just one image is sufficient — the gradient requirement already confirms
 *    we are at an edge.
 */
function antialiased(
  img: PixelData,
  x1: number,
  y1: number,
  width: number,
  height: number,
  a32: Uint32Array,
  b32: Uint32Array,
): boolean {
  const x0 = Math.max(x1 - 1, 0);
  const y0 = Math.max(y1 - 1, 0);
  const x2 = Math.min(x1 + 1, width - 1);
  const y2 = Math.min(y1 + 1, height - 1);
  const pos = (y1 * width + x1) * 4;
  let zeroes = x1 === x0 || x1 === x2 || y1 === y0 || y1 === y2 ? 1 : 0;
  let min = 0;
  let max = 0;

  // pass 1: find min/max brightness deltas and count equal neighbours
  for (let x = x0; x <= x2; x++) {
    for (let y = y0; y <= y2; y++) {
      if (x === x1 && y === y1) continue;
      const delta = colorDelta(img, img, pos, (y * width + x) * 4, true);
      if (delta === 0) {
        zeroes++;
        if (zeroes > 2) return false;
      } else if (delta < min) {
        min = delta;
      } else if (delta > max) {
        max = delta;
      }
    }
  }

  // if there are no both darker and brighter pixels among siblings, it's not anti-aliasing
  if (min === 0 || max === 0) return false;

  // pass 2: check all neighbours matching min or max delta for flat-region siblings
  // in either image (relaxed from both images — see deviation note above)
  for (let x = x0; x <= x2; x++) {
    for (let y = y0; y <= y2; y++) {
      if (x === x1 && y === y1) continue;
      const delta = colorDelta(img, img, pos, (y * width + x) * 4, true);
      if (delta === min || delta === max) {
        if (hasManySiblings(a32, x, y, width, height) || hasManySiblings(b32, x, y, width, height)) return true;
      }
    }
  }
  return false;
}

/** Check if a pixel has 3+ adjacent pixels of the same colour. */
function hasManySiblings(img: Uint32Array, x1: number, y1: number, width: number, height: number): boolean {
  const x0 = Math.max(x1 - 1, 0);
  const y0 = Math.max(y1 - 1, 0);
  const x2 = Math.min(x1 + 1, width - 1);
  const y2 = Math.min(y1 + 1, height - 1);
  const val = img[y1 * width + x1];
  let zeroes = x1 === x0 || x1 === x2 || y1 === y0 || y1 === y2 ? 1 : 0;

  // go through 8 adjacent pixels
  for (let x = x0; x <= x2; x++) {
    for (let y = y0; y <= y2; y++) {
      if (x === x1 && y === y1) continue;
      zeroes += +(val === img[y * width + x]);
      if (zeroes > 2) return true;
    }
  }
  return false;
}

/**
 * Calculate colour difference according to the paper "Measuring perceived colour difference
 * using YIQ NTSC transmission colour space in mobile applications" by Y. Kotsarenko and F. Ramos
 */
function colorDelta(img1: PixelData, img2: PixelData, k: number, m: number, yOnly: boolean): number {
  const r1 = img1[k];
  const g1 = img1[k + 1];
  const b1 = img1[k + 2];
  const a1 = img1[k + 3];
  const r2 = img2[m];
  const g2 = img2[m + 1];
  const b2 = img2[m + 2];
  const a2 = img2[m + 3];

  let dr = r1 - r2;
  let dg = g1 - g2;
  let db = b1 - b2;
  const da = a1 - a2;

  if (!dr && !dg && !db && !da) return 0;

  if (a1 < 255 || a2 < 255) {
    // blend pixels with background
    const rb = 48 + 159 * (k % 2);
    const gb = 48 + 159 * (((k / 1.618033988749895) | 0) % 2);
    const bb = 48 + 159 * (((k / 2.618033988749895) | 0) % 2);
    dr = (r1 * a1 - r2 * a2 - rb * da) / 255;
    dg = (g1 * a1 - g2 * a2 - gb * da) / 255;
    db = (b1 * a1 - b2 * a2 - bb * da) / 255;
  }

  const y = dr * 0.29889531 + dg * 0.58662247 + db * 0.11448223;

  if (yOnly) return y; // brightness difference only

  const i = dr * 0.59597799 - dg * 0.2741761 - db * 0.32180189;
  const q = dr * 0.21147017 - dg * 0.52261711 + db * 0.31114694;

  const delta = 0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q;

  // encode whether the pixel lightens or darkens in the sign
  return y > 0 ? -delta : delta;
}

function drawPixel(output: PixelData, pos: number, r: number, g: number, b: number): void {
  output[pos + 0] = r;
  output[pos + 1] = g;
  output[pos + 2] = b;
  output[pos + 3] = 255;
}

function drawGrayPixel(img: PixelData, i: number, alpha: number, output: PixelData): void {
  const val =
    255 + ((img[i] * 0.29889531 + img[i + 1] * 0.58662247 + img[i + 2] * 0.11448223 - 255) * alpha * img[i + 3]) / 255;
  drawPixel(output, i, val, val, val);
}
