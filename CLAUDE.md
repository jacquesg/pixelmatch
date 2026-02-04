# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A pixel-level image comparison library with anti-aliased pixel detection and perceptual colour difference metrics. TypeScript/Rust rewrite of [mapbox/pixelmatch](https://github.com/mapbox/pixelmatch) with multiple backend options:

- **Native** (Rust/napi-rs): Fastest, for Node.js
- **WASM**: For browsers and edge runtimes
- **JS fallback**: Pure TypeScript, works everywhere

## Commands

```bash
# Build everything (TS + native + WASM)
pnpm build

# Build individual targets
pnpm build:ts          # TypeScript only (tsup)
pnpm build:native      # Rust native binding (napi-rs)
pnpm build:wasm        # WASM binding (wasm-pack)

# Testing
pnpm test              # Run vitest tests (TS)
pnpm test:rust         # Run cargo tests (Rust)
cargo test             # Rust unit + integration tests
cargo test --release   # Rust tests with release optimisations

# Code quality
pnpm lint              # ESLint
pnpm lint:fix          # ESLint with auto-fix
pnpm typecheck         # TypeScript type checking
pnpm format            # Prettier

# Benchmarks
pnpm bench             # JS benchmark (tsx bench.ts)
cargo bench            # Rust criterion benchmarks

# Update test fixtures (regenerate expected diff images)
UPDATE=1 pnpm test
```

## Architecture

```
src/
├── types.ts        # Shared types: ImageLike, PixelmatchOptions, PixelmatchResult
├── validate.ts     # Input validation: isPixelData, validateInput
├── pixelmatch.ts   # Core algorithm (JS fallback) - anti-aliasing detection, YIQ colour delta
├── index.ts        # Node.js entry - auto-loads native binding, falls back to JS
├── wasm.ts         # WASM entry - wraps wasm-bindgen bindings
├── compat.ts       # Compat entry (Node.js) - mapbox/pixelmatch-compatible API
├── compat-fallback.ts # Compat entry (browser) - mapbox/pixelmatch-compatible API
└── cli.ts          # CLI tool

crate/
├── lib.rs          # Core algorithm (Rust) - parallelised with rayon, returns MatchResult
├── aa.rs           # Anti-aliasing detection
├── color.rs        # YIQ colour delta calculation
├── napi_bindings.rs # napi-rs bindings for Node.js (returns NapiMatchResult)
└── wasm_bindings.rs # wasm-bindgen bindings (returns WasmMatchResult)

test/
├── pixelmatch.test.ts  # Vitest tests
└── fixtures/           # PNG test images (pairs + expected diffs)

tests/
└── integration.rs      # Rust integration tests
```

## API

All backends share the same signature:

```typescript
pixelmatch(img1: ImageLike, img2: ImageLike, options?: PixelmatchOptions): PixelmatchResult
```

- `ImageLike`: `{ data: Uint8Array | Uint8ClampedArray, width: number, height: number }`
- `PixelmatchResult`: `{ diffCount, diffPercentage, totalPixels, aaCount, identical }`
- `output` buffer is now an option (`options.output`) instead of a positional parameter
- `detectAntiAliasing` (default: `true`) replaces the inverted `includeAA` (default: `false`)

The `./compat` entry preserves the old mapbox/pixelmatch-compatible flat API:

```typescript
pixelmatch(img1, img2, output, width, height, options?): number
```

## Entry Points

| Import Path                       | API    | Backend     | Use Case                              |
| --------------------------------- | ------ | ----------- | ------------------------------------- |
| `@scaryterry/pixelmatch`          | New    | Native → JS | Node.js (auto-selects native)         |
| `@scaryterry/pixelmatch/fallback` | New    | JS          | Browser, explicit fallback            |
| `@scaryterry/pixelmatch/wasm`     | New    | WASM        | Browser, edge runtimes                |
| `@scaryterry/pixelmatch/compat`   | Compat | Native → JS | Drop-in mapbox/pixelmatch replacement |

## Algorithm Notes

The anti-aliasing detection includes two improvements over the original mapbox/pixelmatch:

1. **Two-pass approach**: Finds min/max brightness deltas in pass 1, then checks all matching neighbours in pass 2 (fixes iteration-order dependency)

2. **Relaxed sibling check**: Requires `hasManySiblings` in _either_ image instead of _both_ (improves detection for 1px strokes and thin text)

These changes are documented in `src/pixelmatch.ts:105-123` and `crate/aa.rs`.

## Platform Binaries

Native bindings are distributed as optional platform-specific packages:

- `@scaryterry/pixelmatch-darwin-arm64` / `darwin-x64`
- `@scaryterry/pixelmatch-linux-x64-gnu` / `linux-x64-musl` / `linux-arm64-gnu`
- `@scaryterry/pixelmatch-win32-x64-msvc`

The `napi artifacts` command collects built binaries into the `npm/` directory for publishing.
