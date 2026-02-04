# pixelmatch

[![CI](https://github.com/jacquesg/pixelmatch/actions/workflows/ci.yml/badge.svg)](https://github.com/jacquesg/pixelmatch/actions/workflows/ci.yml)

A pixel-level image comparison library with accurate **anti-aliased pixel detection**
and **perceptual colour difference metrics**. Originally designed for comparing
screenshots in tests.

This is a TypeScript/Rust rewrite of [mapbox/pixelmatch](https://github.com/mapbox/pixelmatch),
with multiple backend options for different environments and performance requirements.

## Features

- **TypeScript** — fully typed, ships its own declarations
- **Native backend** — Rust-based napi-rs binding for significantly faster comparisons
- **WASM backend** — for browser and edge runtime environments
- **Improved anti-aliasing detection** — two-pass approach and relaxed
  sibling check for thin strokes, text, and small features
  (ref: [mapbox/pixelmatch#74](https://github.com/mapbox/pixelmatch/issues/74))

## Installation

```bash
pnpm add @scaryterry/pixelmatch
```

The native binding is distributed as optional platform-specific packages that
install automatically:

| Platform          | Package                                  |
| ----------------- | ---------------------------------------- |
| macOS arm64       | `@scaryterry/pixelmatch-darwin-arm64`    |
| macOS x64         | `@scaryterry/pixelmatch-darwin-x64`      |
| Linux x64 (glibc) | `@scaryterry/pixelmatch-linux-x64-gnu`   |
| Linux x64 (musl)  | `@scaryterry/pixelmatch-linux-x64-musl`  |
| Linux arm64       | `@scaryterry/pixelmatch-linux-arm64-gnu` |
| Windows x64       | `@scaryterry/pixelmatch-win32-x64-msvc`  |

If the native binding is unavailable for your platform, the JS fallback is
used automatically.

## Backends

| Backend                   | Entry point                        | API    | Environment             | Speed    |
| ------------------------- | ---------------------------------- | ------ | ----------------------- | -------- |
| **Native** (Rust/napi-rs) | `@scaryterry/pixelmatch` (Node.js) | New    | Node.js                 | Fastest  |
| **JS fallback**           | `@scaryterry/pixelmatch/fallback`  | New    | Node.js, browsers       | Baseline |
| **WASM**                  | `@scaryterry/pixelmatch/wasm`      | New    | Browsers, edge runtimes | Fast     |
| **Compat**                | `@scaryterry/pixelmatch/compat`    | Legacy | Node.js, browsers       | Varies   |

The default entry point (`@scaryterry/pixelmatch`) automatically loads the native
binding when available and falls back to the pure JS implementation. You can
check which backend is active via the `_backend` property:

```ts
import pixelmatch from '@scaryterry/pixelmatch';

console.log(pixelmatch._backend); // 'native' or 'js'
```

The `./compat` entry point provides a drop-in replacement for the
[mapbox/pixelmatch](https://github.com/mapbox/pixelmatch) API (see
[Migration from mapbox/pixelmatch](#migration-from-mapboxpixelmatch)).

## Example output

| expected                  | actual                    | diff                         |
| ------------------------- | ------------------------- | ---------------------------- |
| ![](test/fixtures/4a.png) | ![](test/fixtures/4b.png) | ![](test/fixtures/4diff.png) |
| ![](test/fixtures/3a.png) | ![](test/fixtures/3b.png) | ![](test/fixtures/3diff.png) |
| ![](test/fixtures/6a.png) | ![](test/fixtures/6b.png) | ![](test/fixtures/6diff.png) |

## API

### `pixelmatch(img1, img2[, options])`

Compares two equally sized images pixel by pixel.

- **`img1`**, **`img2`** — `ImageLike` objects with `data`, `width`, and `height`
  properties. `data` must be a `Uint8Array` or `Uint8ClampedArray` of length
  `width * height * 4` (RGBA). Objects from `pngjs`, Canvas `getImageData()`,
  and similar libraries satisfy this interface directly.
- **`options`** — Optional `PixelmatchOptions` object (see below).

Returns a `PixelmatchResult`:

| Property         | Type      | Description                                |
| ---------------- | --------- | ------------------------------------------ |
| `diffCount`      | `number`  | Number of mismatched pixels.               |
| `diffPercentage` | `number`  | `diffCount / totalPixels` (0 to 1).        |
| `totalPixels`    | `number`  | Total number of pixels (`width * height`). |
| `aaCount`        | `number`  | Number of anti-aliased pixels detected.    |
| `identical`      | `boolean` | Whether the two images are byte-identical. |

**`options`**:

| Option               | Type        | Default         | Description                                                                                              |
| -------------------- | ----------- | --------------- | -------------------------------------------------------------------------------------------------------- |
| `threshold`          | `number`    | `0.1`           | Matching threshold (`0` to `1`). Smaller values make the comparison more sensitive.                      |
| `detectAntiAliasing` | `boolean`   | `true`          | Detect anti-aliased pixels and exclude them from the diff count.                                         |
| `output`             | `PixelData` | `undefined`     | Buffer to write the diff image into. Must be `width * height * 4` bytes.                                 |
| `alpha`              | `number`    | `0.1`           | Blending factor of unchanged pixels in the diff output. `0` for pure white, `1` for original brightness. |
| `aaColor`            | `[R, G, B]` | `[255, 255, 0]` | Colour of anti-aliased pixels in the diff output.                                                        |
| `diffColor`          | `[R, G, B]` | `[255, 0, 0]`   | Colour of differing pixels in the diff output.                                                           |
| `diffColorAlt`       | `[R, G, B]` | `undefined`     | Alternative colour for dark-on-light differences. If not set, all differing pixels use `diffColor`.      |
| `diffMask`           | `boolean`   | `false`         | Draw the diff over a transparent background (a mask), rather than over the original image.               |

## Usage

### Node.js

```ts
import fs from 'node:fs';
import { PNG } from 'pngjs';
import pixelmatch from '@scaryterry/pixelmatch';

const img1 = PNG.sync.read(fs.readFileSync('img1.png'));
const img2 = PNG.sync.read(fs.readFileSync('img2.png'));
const { width, height } = img1;
const diff = new PNG({ width, height });

const result = pixelmatch(img1, img2, { threshold: 0.1, output: diff.data });

console.log(`${result.diffCount} pixels differ (${(result.diffPercentage * 100).toFixed(2)}%)`);
console.log(`anti-aliased pixels: ${result.aaCount}`);
console.log(`identical: ${result.identical}`);

fs.writeFileSync('diff.png', PNG.sync.write(diff));
```

> PNG objects from `pngjs` have `data`, `width`, and `height` properties, so
> they satisfy the `ImageLike` interface directly — no need to destructure.

### Pure JS fallback (Node.js or browsers)

```ts
import pixelmatch from '@scaryterry/pixelmatch/fallback';

const result = pixelmatch(img1, img2, { threshold: 0.1 });
console.log(result.diffCount);
```

### WASM (browsers / edge runtimes)

```ts
import pixelmatch, { initialize } from '@scaryterry/pixelmatch/wasm';

// Initialise the WASM module (call once)
await initialize();

const result = pixelmatch(img1, img2, { threshold: 0.1 });
console.log(result.diffCount);
```

### Browser (Canvas API)

```ts
import pixelmatch from '@scaryterry/pixelmatch/fallback';

const img1 = img1Context.getImageData(0, 0, width, height);
const img2 = img2Context.getImageData(0, 0, width, height);
const diff = diffContext.createImageData(width, height);

const result = pixelmatch(img1, img2, { threshold: 0.1, output: diff.data });

diffContext.putImageData(diff, 0, 0);
```

## Migration from mapbox/pixelmatch

The `./compat` entry point is a drop-in replacement for
[mapbox/pixelmatch](https://github.com/mapbox/pixelmatch), preserving the
original positional-parameter signature:

```ts
import pixelmatch from '@scaryterry/pixelmatch/compat';

// Same API as mapbox/pixelmatch — returns a number (diff count)
const numDiffPixels = pixelmatch(img1.data, img2.data, diff.data, width, height, {
  threshold: 0.1,
  includeAA: false,
});
```

On Node.js, `./compat` uses the native backend (with JS fallback). In browsers,
it uses the pure JS backend. The compat layer maps the legacy options to the new
API internally.

| Legacy option    | New option                 | Notes                                                     |
| ---------------- | -------------------------- | --------------------------------------------------------- |
| `includeAA`      | `detectAntiAliasing`       | Inverted: `includeAA: false` = `detectAntiAliasing: true` |
| `output` (param) | `options.output`           | Moved from positional parameter to options                |
| Returns `number` | Returns `PixelmatchResult` | Compat wrapper returns `.diffCount`                       |

## Command line

```bash
pixelmatch image1.png image2.png [diff.png] [threshold] [detectAntiAliasing]
```

**Exit codes:**

| Code | Meaning                       |
| ---- | ----------------------------- |
| `0`  | Images are identical          |
| `64` | Invalid arguments             |
| `65` | Image dimensions do not match |
| `66` | Images have differences       |

## Algorithm

This library implements ideas from the following papers:

- [Measuring perceived colour difference using YIQ NTSC transmission colour space in mobile applications](https://www.spiedigitallibrary.org/conference-proceedings-of-spie/8011/80119D/Simple-perceptual-color-space-for-color-specification-and-real-time/10.1117/12.901997.full) (2010, Yuriy Kotsarenko, Fernando Ramos)
- [Anti-aliased pixel and intensity slope detector](https://www.researchgate.net/publication/234126755_Anti-aliased_Pixel_and_Intensity_Slope_Detector) (2009, Vytautas Vyšniauskas)

### Anti-aliasing detection improvements

This implementation includes two changes to the anti-aliasing detection
algorithm to improve accuracy for thin strokes, text, and small features:

1. **Two-pass approach** — The original algorithm only checks `hasManySiblings`
   on the last neighbour found with the min/max delta, missing tied candidates.
   This made results depend on loop iteration order. We now find min/max in
   pass 1, then check all matching neighbours in pass 2.

2. **Relaxed sibling check** — Changed from requiring `hasManySiblings` in
   both images to requiring it in either image. For 1px-wide strokes, the
   stroke-side extreme never has 3+ identical siblings because the feature is
   too narrow. Requiring siblings in just one image is sufficient — the
   gradient requirement already confirms we are at an edge.

## Attribution

This project is a TypeScript/Rust rewrite based on [mapbox/pixelmatch](https://github.com/mapbox/pixelmatch)
by [Mapbox](https://www.mapbox.com/). The original JavaScript implementation,
algorithm design, and test fixtures are from that project.

## Licence

ISC © [Mapbox](https://www.mapbox.com/) (original implementation), [Jacques Germishuys](https://github.com/jacquesg) (this fork)
