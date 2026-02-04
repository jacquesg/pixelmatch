import match from './src/pixelmatch.js';
import { PNG } from 'pngjs';
import fs from 'node:fs';

console.log(`backend: js`);

const data = [1, 2, 3, 4, 5, 6, 7].map((i) => [readImage(`${i}a`), readImage(`${i}b`)]);

console.time('match');
let sum = 0;
for (let i = 0; i < 100; i++) {
  for (const [img1, img2] of data) {
    sum += match(img1, img2).diffCount;
  }
}
console.timeEnd('match');
console.log(sum);

function readImage(name: string): PNG {
  return PNG.sync.read(fs.readFileSync(new URL(`test/fixtures/${name}.png`, import.meta.url)));
}
