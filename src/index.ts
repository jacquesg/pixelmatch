import { createRequire } from 'node:module';
import type { ImageLike, PixelData, PixelmatchOptions, PixelmatchResult } from './types.js';
import { buildResult, validateInput } from './validate.js';
import jsFallback from './pixelmatch.js';

export type { ImageLike, PixelData, PixelmatchOptions, PixelmatchResult } from './types.js';

interface NativeMatchResult {
  diffCount: number;
  aaCount: number;
  identical: boolean;
}

interface NativeOptions {
  threshold?: number;
  detectAntiAliasing?: boolean;
  alpha?: number;
  aaColor?: number[];
  diffColor?: number[];
  diffColorAlt?: number[];
  diffMask?: boolean;
}

interface NativeBinding {
  pixelmatch: (
    img1: PixelData,
    img2: PixelData,
    output: PixelData,
    width: number,
    height: number,
    options: NativeOptions,
  ) => NativeMatchResult;
  pixelmatchCount: (
    img1: PixelData,
    img2: PixelData,
    width: number,
    height: number,
    options: NativeOptions,
  ) => NativeMatchResult;
}

type PixelmatchFunction = {
  (img1: ImageLike, img2: ImageLike, options?: PixelmatchOptions): PixelmatchResult;
  _backend: 'native' | 'js';
};

const triples: Record<string, string> = {
  'darwin-arm64': '@scaryterry/pixelmatch-darwin-arm64',
  'darwin-x64': '@scaryterry/pixelmatch-darwin-x64',
  'linux-x64': '@scaryterry/pixelmatch-linux-x64-gnu',
  'linux-arm64': '@scaryterry/pixelmatch-linux-arm64-gnu',
  'win32-x64': '@scaryterry/pixelmatch-win32-x64-msvc',
};

function isNativeBinding(value: unknown): value is NativeBinding {
  return (
    typeof value === 'object' &&
    value !== null &&
    'pixelmatch' in value &&
    typeof value.pixelmatch === 'function' &&
    'pixelmatchCount' in value &&
    typeof value.pixelmatchCount === 'function'
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
        binding = req('@scaryterry/pixelmatch-linux-x64-musl');
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

  const fn = (img1: ImageLike, img2: ImageLike, options: PixelmatchOptions = {}): PixelmatchResult => {
    const { output, ...rest } = options;
    validateInput(img1, img2, output);

    const { data: data1, width, height } = img1;
    const { data: data2 } = img2;
    const totalPixels = width * height;

    let raw: NativeMatchResult;
    if (output) {
      raw = nativeMatch(data1, data2, output, width, height, rest);
    } else {
      raw = pixelmatchCount(data1, data2, width, height, rest);
    }

    return buildResult(raw.diffCount, raw.aaCount, totalPixels, raw.identical);
  };
  fn._backend = 'native' as const;
  impl = fn;
} else {
  impl = Object.assign(
    (img1: ImageLike, img2: ImageLike, options?: PixelmatchOptions) => jsFallback(img1, img2, options),
    { _backend: 'js' as const },
  );
}

export default impl;
