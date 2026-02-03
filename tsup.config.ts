import { defineConfig } from 'tsup';

export default defineConfig([
  {
    entry: {
      pixelmatch: 'src/pixelmatch.ts',
      index: 'src/index.ts',
      wasm: 'src/wasm.ts',
    },
    format: ['esm', 'cjs'],
    dts: true,
    outDir: 'dist',
    clean: true,
    splitting: false,
    platform: 'node',
    target: 'node18',
    shims: true,
    external: [/pixelmatch_bg/],
    outExtension: ({ format }) => ({
      js: format === 'esm' ? '.mjs' : '.cjs',
    }),
  },
  {
    entry: { cli: 'src/cli.ts' },
    format: ['esm'],
    outDir: 'dist',
    platform: 'node',
    target: 'node18',
    banner: { js: '#!/usr/bin/env node' },
    external: ['pngjs'],
    outExtension: () => ({ js: '.mjs' }),
  },
]);
