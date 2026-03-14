# Project Guidelines

## Code Style
- Follow existing Rust and TypeScript style in place; keep changes minimal and avoid broad refactors.
- Preserve JSON-facing field names in camelCase and Rust serde attributes that enforce schema compatibility.
- Prefer extending existing types/functions over introducing parallel APIs.
- Keep examples and docs aligned with the public API in `htvg-core` and `packages/htvg`.

## Architecture
- `crates/htvg-core`: source of truth for schema, layout, and SVG generation.
- `crates/htvg-cli`: CLI wrapper around core compile APIs.
- `crates/htvg-wasm`: wasm-bindgen bridge exposing core compile APIs to JS.
- `packages/htvg`: TypeScript wrapper and distributed WASM artifacts.
- `packages/htvg-demo`: Cloudflare Workers demo using the npm package.
- Main pipeline: JSON element tree -> deserialize -> layout (Taffy + text measure) -> render tree -> SVG.

## Build and Test
- Rust workspace:
  - `cargo check`
  - `cargo test`
  - `cargo build --release`
- CLI example:
  - `cargo run -p htvg-cli -- compile examples/hello.json`
- npm package build (from `packages/htvg`):
  - `npm run build` (runs wasm-pack + TypeScript build)
- Demo app (from `packages/htvg-demo`):
  - `npm run dev`
  - `npm run deploy`

## Conventions
- Input schema uses tagged union elements with `type`: `box`, `flex`, `text`, `image`.
- Dimensions/spacings may be numeric or string forms (for example, percentages and multi-value spacing).
- Maintain compatibility between Rust types in `crates/htvg-core/src/element.rs` and TypeScript types in `packages/htvg/src/types.ts`.
- Prefer self-contained document format `{ meta, content }` when adding examples and tests.
- In JS/TS usage, `init()` must run before `compile()` or `compileDocument()`.
- Avoid adding assumptions of browser-only runtime behavior; package supports browsers, Node, and edge runtimes.

## Pitfalls
- `wasm-pack` is required for npm WASM builds.
- Keep WASM export names and TS wrapper calls in sync (`compile`, `compileDocument`, `version`).
- Changes to font handling should account for both embedded `data` and `url` usage paths.