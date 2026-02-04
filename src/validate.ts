import type { ImageLike, PixelData, PixelmatchResult } from './types.js';

export function isPixelData(arr: ArrayBufferView | null): arr is PixelData {
  return ArrayBuffer.isView(arr) && 'BYTES_PER_ELEMENT' in arr && arr.BYTES_PER_ELEMENT === 1;
}

export function validateInput(img1: ImageLike, img2: ImageLike, output?: PixelData): void {
  if (!isPixelData(img1.data) || !isPixelData(img2.data))
    throw new Error('Image data: Uint8Array, Uint8ClampedArray or Buffer expected.');
  if (output && !isPixelData(output))
    throw new Error('Output data: Uint8Array, Uint8ClampedArray or Buffer expected.');
  if (img1.width !== img2.width || img1.height !== img2.height)
    throw new Error(
      `Image dimensions do not match: ${img1.width}x${img1.height} vs ${img2.width}x${img2.height}`,
    );
  if (img1.data.length !== img2.data.length)
    throw new Error(
      `Image sizes do not match. Image 1 size: ${img1.data.length}, image 2 size: ${img2.data.length}`,
    );
  const expected = img1.width * img1.height * 4;
  if (img1.data.length !== expected)
    throw new Error(`Image data size does not match width/height. Expecting ${expected}. Got ${img1.data.length}`);
  if (output && output.length !== expected)
    throw new Error(`Output buffer size does not match. Expecting ${expected}. Got ${output.length}`);
}

export function buildResult(
  diffCount: number,
  aaCount: number,
  totalPixels: number,
  identical: boolean,
): PixelmatchResult {
  return {
    diffCount,
    diffPercentage: totalPixels > 0 ? diffCount / totalPixels : 0,
    totalPixels,
    aaCount,
    identical,
  };
}
