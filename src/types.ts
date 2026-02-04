export type PixelData = Uint8Array | Uint8ClampedArray;

export interface ImageLike {
  readonly data: PixelData;
  readonly width: number;
  readonly height: number;
}

export interface PixelmatchOptions {
  /** Matching threshold (0 to 1); smaller is more sensitive. Default: 0.1 */
  threshold?: number;
  /** Whether to detect and exclude anti-aliased pixels from the diff count. Default: true */
  detectAntiAliasing?: boolean;
  /** Output buffer to write the diff image into. Must be the same size as the input images (width * height * 4). */
  output?: PixelData;
  /** Draw the diff over a transparent background (a mask), without the original image. Default: false */
  diffMask?: boolean;
  /** Blending factor of the original image in the diff output (0 to 1). Default: 0.1 */
  alpha?: number;
  /** Colour of anti-aliased pixels in the diff output [R, G, B]. Default: [255, 255, 0] (yellow) */
  aaColor?: [number, number, number];
  /** Colour of different pixels in the diff output [R, G, B]. Default: [255, 0, 0] (red) */
  diffColor?: [number, number, number];
  /** Alternative diff colour for pixels that are darker in img2 [R, G, B]. Default: same as diffColor */
  diffColorAlt?: [number, number, number];
}

export interface PixelmatchResult {
  /** Number of mismatched pixels. */
  readonly diffCount: number;
  /** diffCount / totalPixels (0 to 1). */
  readonly diffPercentage: number;
  /** Total number of pixels (width * height). */
  readonly totalPixels: number;
  /** Number of anti-aliased pixels detected. */
  readonly aaCount: number;
  /** Whether the two images are byte-identical. */
  readonly identical: boolean;
}

/** Legacy options matching mapbox/pixelmatch for the compat entry point. */
export interface LegacyPixelmatchOptions {
  threshold?: number;
  includeAA?: boolean;
  alpha?: number;
  aaColor?: [number, number, number];
  diffColor?: [number, number, number];
  diffColorAlt?: [number, number, number];
  diffMask?: boolean;
}
