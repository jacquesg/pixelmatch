import type { PixelData, LegacyPixelmatchOptions } from './types.js';
import impl from './index.js';

export type { PixelData };
export type { LegacyPixelmatchOptions as PixelmatchOptions };

/**
 * Compat wrapper: mapbox/pixelmatch-compatible API for Node.js.
 * Uses the native backend when available, falls back to JS.
 */
export default function pixelmatch(
  img1: PixelData,
  img2: PixelData,
  output: PixelData | null | undefined,
  width: number,
  height: number,
  options: LegacyPixelmatchOptions = {},
): number {
  const { includeAA, ...rest } = options;
  const result = impl(
    { data: img1, width, height },
    { data: img2, width, height },
    {
      ...rest,
      detectAntiAliasing: includeAA === undefined ? undefined : !includeAA,
      output: output ?? undefined,
    },
  );
  return result.diffCount;
}
