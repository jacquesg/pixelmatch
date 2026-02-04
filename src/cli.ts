import { PNG } from 'pngjs';
import fs from 'node:fs';
import match from './index.js';
import type { PixelmatchOptions } from './types.js';

if (process.argv.length < 4) {
  console.log('Usage: pixelmatch image1.png image2.png [diff.png] [threshold] [detectAntiAliasing]');
  process.exit(64);
}

const [, , img1Path, img2Path, diffPath, threshold, detectAA] = process.argv;
const options: PixelmatchOptions = {};
if (threshold !== undefined) options.threshold = +threshold;
if (detectAA !== undefined) options.detectAntiAliasing = detectAA !== 'false';

const img1 = PNG.sync.read(fs.readFileSync(img1Path));
const img2 = PNG.sync.read(fs.readFileSync(img2Path));

const { width, height } = img1;

if (img2.width !== width || img2.height !== height) {
  console.log(`Image dimensions do not match: ${width}x${height} vs ${img2.width}x${img2.height}`);
  process.exit(65);
}

const diff = diffPath ? new PNG({ width, height }) : null;
if (diff) {
  options.output = diff.data;
}

console.time('matched in');
const result = match(img1, img2, options);
console.timeEnd('matched in');

console.log(`different pixels: ${result.diffCount}`);
console.log(`error: ${Math.round(result.diffPercentage * 100 * 100) / 100}%`);
if (result.aaCount > 0) {
  console.log(`anti-aliased pixels: ${result.aaCount}`);
}

if (diff) {
  fs.writeFileSync(diffPath, PNG.sync.write(diff));
}
process.exit(result.diffCount ? 66 : 0);
