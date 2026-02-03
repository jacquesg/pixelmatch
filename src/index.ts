import { createRequire } from 'node:module';
import jsFallback from './pixelmatch.js';
import type { PixelData, PixelmatchOptions } from './pixelmatch.js';

export type { PixelData, PixelmatchOptions } from './pixelmatch.js';

interface NativeBinding {
  pixelmatch: (
    img1: PixelData,
    img2: PixelData,
    output: PixelData,
    width: number,
    height: number,
    options: PixelmatchOptions,
  ) => number;
  pixelmatchCount: (
    img1: PixelData,
    img2: PixelData,
    width: number,
    height: number,
    options: PixelmatchOptions,
  ) => number;
}

type PixelmatchFunction = {
  (
    img1: PixelData,
    img2: PixelData,
    output: PixelData | null | undefined,
    width: number,
    height: number,
    options?: PixelmatchOptions,
  ): number;
  _backend: 'native' | 'js';
};

const triples: Record<string, string> = {
  'darwin-arm64': '@jacquesg/pixelmatch-darwin-arm64',
  'darwin-x64': '@jacquesg/pixelmatch-darwin-x64',
  'linux-x64': '@jacquesg/pixelmatch-linux-x64-gnu',
  'linux-arm64': '@jacquesg/pixelmatch-linux-arm64-gnu',
  'win32-x64': '@jacquesg/pixelmatch-win32-x64-msvc',
};

function isPixelData(arr: ArrayBufferView | null): arr is PixelData {
  return ArrayBuffer.isView(arr) && (arr as Uint8Array).BYTES_PER_ELEMENT === 1;
}

function isNativeBinding(value: unknown): value is NativeBinding {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as Record<string, unknown>).pixelmatch === 'function' &&
    typeof (value as Record<string, unknown>).pixelmatchCount === 'function'
  );
}

function loadNativeBinding(): NativeBinding | null {
  try {
    const req = createRequire(import.meta.url);
    const key = `${process.platform}-${process.arch}`;
    let binding: unknown;
    try {
      binding = req(triples[key]);
    } catch {
      if (process.platform === 'linux') {
        binding = req('@jacquesg/pixelmatch-linux-x64-musl');
      } else {
        return null;
      }
    }
    return isNativeBinding(binding) ? binding : null;
  } catch {
    return null;
  }
}

let impl: PixelmatchFunction;

const native = loadNativeBinding();
if (native) {
  const { pixelmatch: nativeMatch, pixelmatchCount } = native;

  const fn = (
    img1: PixelData,
    img2: PixelData,
    output: PixelData | null | undefined,
    width: number,
    height: number,
    options: PixelmatchOptions = {},
  ): number => {
    if (!isPixelData(img1) || !isPixelData(img2) || (output && !isPixelData(output)))
      throw new Error('Image data: Uint8Array, Uint8ClampedArray or Buffer expected.');
    if (img1.length !== img2.length || (output && output.length !== img1.length))
      throw new Error(`Image sizes do not match. Image 1 size: ${img1.length}, image 2 size: ${img2.length}`);
    if (img1.length !== width * height * 4)
      throw new Error(
        `Image data size does not match width/height. Expecting ${width * height * 4}. Got ${img1.length}`,
      );

    if (output) {
      return nativeMatch(img1, img2, output, width, height, options);
    }
    return pixelmatchCount(img1, img2, width, height, options);
  };
  fn._backend = 'native' as const;
  impl = fn;
} else {
  const fn = jsFallback as PixelmatchFunction;
  fn._backend = 'js' as const;
  impl = fn;
}

export default impl;
