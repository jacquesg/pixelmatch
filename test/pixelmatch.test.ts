import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import { PNG } from 'pngjs';
import match from '../src/pixelmatch.js';
import type { PixelmatchOptions } from '../src/pixelmatch.js';

const options: PixelmatchOptions = { threshold: 0.05 };

describe('pixelmatch', () => {
  diffTest('1a', '1b', '1diff', options, 109);
  diffTest('1a', '1b', '1diffdefaultthreshold', { threshold: undefined }, 81);
  diffTest('1a', '1b', '1diffmask', { threshold: 0.05, includeAA: false, diffMask: true }, 109);
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

  it('throws error if image sizes do not match', () => {
    expect(() => match(new Uint8Array(8), new Uint8Array(9), null, 2, 1)).toThrow('Image sizes do not match');
  });

  it('throws error if image sizes do not match width and height', () => {
    expect(() => match(new Uint8Array(9), new Uint8Array(9), null, 2, 1)).toThrow(
      'Image data size does not match width/height',
    );
  });

  it('throws error if provided wrong image data format', () => {
    const arr = new Uint8Array(4 * 20 * 20);
    const bad = new Array(arr.length).fill(0);
    expect(() => match(bad as never, arr, null, 20, 20)).toThrow(
      'Image data: Uint8Array, Uint8ClampedArray or Buffer expected',
    );
    expect(() => match(arr, bad as never, null, 20, 20)).toThrow(
      'Image data: Uint8Array, Uint8ClampedArray or Buffer expected',
    );
    expect(() => match(arr, arr, bad as never, 20, 20)).toThrow(
      'Image data: Uint8Array, Uint8ClampedArray or Buffer expected',
    );
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

    const mismatch = match(img1.data, img2.data, diff.data, width, height, opts);
    const mismatch2 = match(img1.data, img2.data, null, width, height, opts);

    if (process.env.UPDATE) {
      writeImage(diffPath, diff);
    } else {
      const expectedDiff = readImage(diffPath);
      expect(diff.data.equals(expectedDiff.data)).toBe(true);
    }
    expect(mismatch).toBe(expectedMismatch);
    expect(mismatch).toBe(mismatch2);
  });
}

function readImage(name: string): PNG {
  return PNG.sync.read(fs.readFileSync(new URL(`fixtures/${name}.png`, import.meta.url)));
}

function writeImage(name: string, image: PNG): void {
  fs.writeFileSync(new URL(`fixtures/${name}.png`, import.meta.url), PNG.sync.write(image));
}
