import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import { PNG } from 'pngjs';
import match from '../src/pixelmatch.js';
import type { PixelmatchOptions } from '../src/types.js';

const options: PixelmatchOptions = { threshold: 0.05 };

describe('pixelmatch', () => {
  diffTest('1a', '1b', '1diff', options, 109);
  diffTest('1a', '1b', '1diffdefaultthreshold', { threshold: undefined }, 81);
  diffTest('1a', '1b', '1diffmask', { threshold: 0.05, diffMask: true }, 109);
  diffTest('1a', '1a', '1emptydiffmask', { threshold: 0, diffMask: true }, 0);
  diffTest(
    '2a',
    '2b',
    '2diff',
    {
      threshold: 0.05,
      alpha: 0.5,
      aaColor: [0, 192, 0],
      diffColor: [255, 0, 255],
    },
    9579,
  );
  diffTest('3a', '3b', '3diff', options, 112);
  diffTest('4a', '4b', '4diff', options, 31739);
  diffTest('5a', '5b', '5diff', options, 3);
  diffTest('6a', '6b', '6diff', options, 44);
  diffTest('6a', '6a', '6empty', { threshold: 0 }, 0);
  diffTest('7a', '7b', '7diff', { diffColorAlt: [0, 255, 0] }, 687);
  diffTest('8a', '5b', '8diff', options, 32896);

  it('returns a PixelmatchResult with all fields', () => {
    const img1 = readImage('1a');
    const img2 = readImage('1b');
    const result = match(img1, img2, { threshold: 0.05 });

    expect(result).toHaveProperty('diffCount');
    expect(result).toHaveProperty('diffPercentage');
    expect(result).toHaveProperty('totalPixels');
    expect(result).toHaveProperty('aaCount');
    expect(result).toHaveProperty('identical');
    expect(result.totalPixels).toBe(img1.width * img1.height);
    expect(result.diffPercentage).toBeCloseTo(result.diffCount / result.totalPixels, 10);
  });

  it('returns identical: true for identical images', () => {
    const img1 = readImage('1a');
    const result = match(img1, img1, { threshold: 0 });

    expect(result.identical).toBe(true);
    expect(result.diffCount).toBe(0);
    expect(result.aaCount).toBe(0);
    expect(result.diffPercentage).toBe(0);
  });

  it('returns identical: false for different images', () => {
    const img1 = readImage('1a');
    const img2 = readImage('1b');
    const result = match(img1, img2, { threshold: 0.05 });

    expect(result.identical).toBe(false);
  });

  it('tracks aaCount when detectAntiAliasing is true', () => {
    const img1 = readImage('1a');
    const img2 = readImage('1b');

    const withAA = match(img1, img2, { threshold: 0.05, detectAntiAliasing: true });
    const withoutAA = match(img1, img2, { threshold: 0.05, detectAntiAliasing: false });

    expect(withAA.aaCount).toBeGreaterThan(0);
    expect(withoutAA.aaCount).toBe(0);
    expect(withoutAA.diffCount).toBe(withAA.diffCount + withAA.aaCount);
  });

  it('throws error if image sizes do not match', () => {
    const img1 = { data: new Uint8Array(8), width: 2, height: 1 };
    const img2 = { data: new Uint8Array(9), width: 2, height: 1 };
    expect(() => match(img1, img2)).toThrow('Image sizes do not match');
  });

  it('throws error if image sizes do not match width and height', () => {
    const img1 = { data: new Uint8Array(9), width: 2, height: 1 };
    const img2 = { data: new Uint8Array(9), width: 2, height: 1 };
    expect(() => match(img1, img2)).toThrow('Image data size does not match width/height');
  });

  it('throws error if provided wrong image data format', () => {
    const arr = new Uint8Array(4 * 20 * 20);
    const bad = new Array(arr.length).fill(0);
    const good = { data: arr, width: 20, height: 20 };
    expect(() => match({ data: bad as never, width: 20, height: 20 }, good)).toThrow(
      'Image data: Uint8Array, Uint8ClampedArray or Buffer expected',
    );
    expect(() => match(good, { data: bad as never, width: 20, height: 20 })).toThrow(
      'Image data: Uint8Array, Uint8ClampedArray or Buffer expected',
    );
  });

  it('throws error if output buffer has wrong size', () => {
    const img = { data: new Uint8Array(4 * 20 * 20), width: 20, height: 20 };
    expect(() => match(img, img, { output: new Uint8Array(100) })).toThrow(
      'Output buffer size does not match',
    );
  });

  it('throws error if image dimensions do not match', () => {
    const img1 = { data: new Uint8Array(4 * 20 * 20), width: 20, height: 20 };
    const img2 = { data: new Uint8Array(4 * 10 * 40), width: 10, height: 40 };
    expect(() => match(img1, img2)).toThrow('Image dimensions do not match: 20x20 vs 10x40');
  });

  it('throws error if output buffer has wrong type', () => {
    const img = { data: new Uint8Array(4 * 20 * 20), width: 20, height: 20 };
    const bad = new Array(4 * 20 * 20).fill(0);
    expect(() => match(img, img, { output: bad as never })).toThrow(
      'Output data: Uint8Array, Uint8ClampedArray or Buffer expected',
    );
  });
});

describe('compat', () => {
  it('returns a number (diffCount) with the legacy API', async () => {
    const compatMatch = (await import('../src/compat-fallback.js')).default;
    const img1 = readImage('1a');
    const img2 = readImage('1b');
    const { width, height } = img1;
    const diff = new PNG({ width, height });

    const count = compatMatch(img1.data, img2.data, diff.data, width, height, { threshold: 0.05 });
    expect(typeof count).toBe('number');
    expect(count).toBe(109);
  });

  it('maps includeAA to detectAntiAliasing correctly', async () => {
    const compatMatch = (await import('../src/compat-fallback.js')).default;
    const img1 = readImage('1a');
    const img2 = readImage('1b');
    const { width, height } = img1;

    // includeAA: true means "include AA pixels in diff count" (skip detection)
    const withAA = compatMatch(img1.data, img2.data, null, width, height, { threshold: 0.05, includeAA: true });
    // includeAA: false (default) means "detect and exclude AA"
    const withoutAA = compatMatch(img1.data, img2.data, null, width, height, { threshold: 0.05, includeAA: false });

    expect(withAA).toBeGreaterThan(withoutAA);
    expect(withoutAA).toBe(109);
  });

  it('works via the Node.js compat entry point', async () => {
    const compatMatch = (await import('../src/compat.js')).default;
    const img1 = readImage('1a');
    const img2 = readImage('1b');
    const { width, height } = img1;

    const count = compatMatch(img1.data, img2.data, null, width, height, { threshold: 0.05 });
    expect(count).toBe(109);
  });

  it('handles null output', async () => {
    const compatMatch = (await import('../src/compat-fallback.js')).default;
    const img1 = readImage('1a');
    const img2 = readImage('1b');
    const { width, height } = img1;

    const count = compatMatch(img1.data, img2.data, null, width, height, { threshold: 0.05 });
    expect(count).toBe(109);
  });
});

function diffTest(
  imgPath1: string,
  imgPath2: string,
  diffPath: string,
  opts: PixelmatchOptions,
  expectedMismatch: number,
): void {
  const name = `comparing ${imgPath1} to ${imgPath2}, ${JSON.stringify(opts)}`;

  it(name, () => {
    const img1 = readImage(imgPath1);
    const img2 = readImage(imgPath2);
    const { width, height } = img1;
    const diff = new PNG({ width, height });

    const result = match(img1, img2, { ...opts, output: diff.data });
    const result2 = match(img1, img2, opts);

    if (process.env.UPDATE) {
      writeImage(diffPath, diff);
    } else {
      const expectedDiff = readImage(diffPath);
      expect(diff.data.equals(expectedDiff.data)).toBe(true);
    }
    expect(result.diffCount).toBe(expectedMismatch);
    expect(result.diffCount).toBe(result2.diffCount);
    expect(result.aaCount).toBe(result2.aaCount);
    expect(result.identical).toBe(result2.identical);
  });
}

function readImage(name: string): PNG {
  return PNG.sync.read(fs.readFileSync(new URL(`fixtures/${name}.png`, import.meta.url)));
}

function writeImage(name: string, image: PNG): void {
  fs.writeFileSync(new URL(`fixtures/${name}.png`, import.meta.url), PNG.sync.write(image));
}
