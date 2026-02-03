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
├── pixelmatch.ts   # Core algorithm (JS fallback) - anti-aliasing detection, YIQ colour delta
├── index.ts        # Node.js entry - auto-loads native binding, falls back to JS
├── wasm.ts         # WASM entry - wraps wasm-bindgen bindings
└── cli.ts          # CLI tool

crate/
├── lib.rs          # Core algorithm (Rust) - parallelised with rayon
├── aa.rs           # Anti-aliasing detection
├── color.rs        # YIQ colour delta calculation
├── napi_bindings.rs # napi-rs bindings for Node.js
└── wasm_bindings.rs # wasm-bindgen bindings

test/
├── pixelmatch.test.ts  # Vitest tests
└── fixtures/           # PNG test images (pairs + expected diffs)

tests/
└── integration.rs      # Rust integration tests
```

## Entry Points

| Import Path                       | Backend     | Use Case                      |
| --------------------------------- | ----------- | ----------------------------- |
| `@scaryterry/pixelmatch`          | Native → JS | Node.js (auto-selects native) |
| `@scaryterry/pixelmatch/fallback` | JS          | Browser, explicit fallback    |
| `@scaryterry/pixelmatch/wasm`     | WASM        | Browser, edge runtimes        |

## Algorithm Notes

The anti-aliasing detection includes two improvements over the original mapbox/pixelmatch:

1. **Two-pass approach**: Finds min/max brightness deltas in pass 1, then checks all matching neighbours in pass 2 (fixes iteration-order dependency)

2. **Relaxed sibling check**: Requires `hasManySiblings` in _either_ image instead of _both_ (improves detection for 1px strokes and thin text)

These changes are documented in `src/pixelmatch.ts:120-136` and `crate/aa.rs`.

## Platform Binaries

Native bindings are distributed as optional platform-specific packages:

- `@scaryterry/pixelmatch-darwin-arm64` / `darwin-x64`
- `@scaryterry/pixelmatch-linux-x64-gnu` / `linux-x64-musl` / `linux-arm64-gnu`
- `@scaryterry/pixelmatch-win32-x64-msvc`

The `napi artifacts` command collects built binaries into the `npm/` directory for publishing.
