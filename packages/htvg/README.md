# htvg

A JSON element tree to SVG compiler powered by WASM. Supports flexbox layout, multiline text, images, and custom fonts.

[GitHub](https://github.com/pavi2410/htvg) · [Demo](https://htvg-demo.pavi2410.workers.dev)

## Installation

```bash
npm install htvg
```

## Quick Start

```js
import { init, compileDocument } from "htvg";

await init();

const result = compileDocument({
  meta: { width: 400 },
  content: {
    type: "flex",
    style: { width: 400, padding: 20, backgroundColor: "#ffffff", flexDirection: "column", gap: 12 },
    children: [
      { type: "text", content: "Hello, HTVG!", style: { fontSize: 24, fontWeight: "bold", color: "#1a1a1a" } },
      { type: "text", content: "JSON to SVG, simple as that.", style: { fontSize: 14, color: "#666" } },
    ],
  },
});

console.log(result.svg);    // SVG string
console.log(result.width);  // 400
console.log(result.height); // auto-computed
```

## API

### `init(input?)`

Initialize the WASM module. Must be called once before any compilation.

```js
// Browser — auto-fetches the .wasm file
await init();

// Node.js
import fs from "node:fs";
await init(fs.readFileSync("node_modules/htvg/dist/wasm/htvg_bg.wasm"));

// Cloudflare Workers
import wasmModule from "htvg/htvg.wasm";
await init(wasmModule);
```

### `compileDocument(doc)`

Compile a self-contained `HtvgDocument` to SVG.

```js
const result = compileDocument({
  meta: { width: 600, fonts: [{ family: "Inter", url: "https://fonts.gstatic.com/..." }] },
  content: { type: "text", content: "Hello", style: { fontSize: 32 } },
});
// result.svg — SVG string
// result.width / result.height — computed dimensions
// result.warnings — any warnings
```

The `doc` parameter can be an `HtvgDocument` object or a JSON string.

### `compile(element, options)`

Compile an element tree with separate options.

```js
const element = {
  type: "flex",
  style: { flexDirection: "column", gap: 8 },
  children: [
    { type: "text", content: "Title", style: { fontSize: 20, fontWeight: 700 } },
  ],
};

const result = compile(element, { width: 400 });
```

### `version()`

Returns the HTVG version string.

## Element Types

### `box`
Block container with optional children.

### `flex`
Flexbox container — supports `flexDirection`, `justifyContent`, `alignItems`, `gap`, `flexWrap`.

### `text`
Text leaf with automatic line wrapping. Supports `fontSize`, `fontWeight`, `fontFamily`, `lineHeight`, `textAlign`, `color`, `letterSpacing`.

### `image`
Image element with intrinsic dimensions. Supports `src` (data URI or URL), `width`, `height`, and `objectFit`.

## Document Format

```jsonc
{
  "meta": {
    "width": 600,           // output width (required)
    "height": 400,          // output height (optional, auto-computed)
    "fonts": [              // custom fonts (optional)
      { "family": "Inter", "url": "https://..." }
    ]
  },
  "content": {              // root element tree
    "type": "flex",
    "style": { ... },
    "children": [ ... ]
  }
}
```

## Style Properties

| Category | Properties |
|---|---|
| **Layout** | `width`, `height`, `minWidth`, `maxWidth`, `minHeight`, `maxHeight`, `margin`, `padding` |
| **Flex** | `flexDirection`, `justifyContent`, `alignItems`, `gap`, `flexWrap`, `flexGrow`, `flexShrink` |
| **Visual** | `backgroundColor`, `borderWidth`, `borderColor`, `borderRadius`, `opacity` |
| **Typography** | `fontFamily`, `fontSize`, `fontWeight`, `lineHeight`, `textAlign`, `color`, `letterSpacing` |
| **Image** | `objectFit` (`contain`, `cover`, `fill`) |

Dimensions accept pixels (`number`) or percentages (`"50%"`). Spacing accepts a single value or `"top right bottom left"`.

## License

MIT OR Apache-2.0
