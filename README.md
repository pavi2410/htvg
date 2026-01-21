# HTVG - HyperText Vector Graphics

A static compiler that converts JSON element trees to pure SVG with correct multiline text layout.

## Features

- **JSON Element Tree Input** - Simple, typed element primitives matching JSX/VDOM structure
- **Correct Text Layout** - Powered by Parley for proper text shaping, line breaking, and glyph positioning
- **Flexbox Layout** - Full CSS Flexbox support via Taffy
- **WASM Compatible** - Runs in browsers, Node.js, and edge runtimes (Cloudflare Workers, etc.)
- **Pure SVG Output** - Static, self-contained SVG with no JavaScript

## Installation

```bash
cargo add htvg
```

## Usage

### Rust API

```rust
use htvg::{compile, CompileOptions};

let json = r###"{
    "type": "flex",
    "style": { "width": 400, "padding": 20, "backgroundColor": "#ffffff" },
    "children": [
        {
            "type": "text",
            "content": "Hello World",
            "style": { "fontSize": 24, "color": "#333333" }
        }
    ]
}"###;

let options = CompileOptions {
    width: 400.0,
    height: None,  // Auto-computed
    base_font_size: 16.0,
};

let result = compile(json, &options).unwrap();
println!("{}", result.svg);
```

### WASM API

```javascript
import { compile_to_svg } from 'htvg';

const element = {
  type: "flex",
  style: { flexDirection: "column", width: 400, padding: 20, backgroundColor: "#fff" },
  children: [
    {
      type: "text",
      content: "Hello World",
      style: { fontSize: 24, color: "#333" }
    }
  ]
};

const options = { width: 400 };

const result = compile_to_svg(JSON.stringify(element), JSON.stringify(options));
console.log(result.svg);
```

## Element Types

### `box`
Block container element.

```json
{
  "type": "box",
  "style": { "width": 200, "height": 100, "backgroundColor": "#ff0000" },
  "children": []
}
```

### `flex`
Flexbox container element.

```json
{
  "type": "flex",
  "style": {
    "flexDirection": "column",
    "justifyContent": "center",
    "alignItems": "center",
    "gap": 10
  },
  "children": []
}
```

### `text`
Text leaf element with automatic line wrapping.

```json
{
  "type": "text",
  "content": "Hello World",
  "style": {
    "fontFamily": "sans-serif",
    "fontSize": 16,
    "fontWeight": 700,
    "lineHeight": 1.5,
    "textAlign": "center",
    "color": "#000000"
  }
}
```

### `image`
Image element with intrinsic dimensions.

```json
{
  "type": "image",
  "src": "data:image/png;base64,...",
  "width": 100,
  "height": 100,
  "style": { "objectFit": "cover" }
}
```

## Style Properties

### Layout (Box/Flex)
- `width`, `height` - Dimensions (pixels or percentage string)
- `minWidth`, `maxWidth`, `minHeight`, `maxHeight` - Size constraints
- `margin`, `padding` - Spacing (single value or "top right bottom left")
- `display` - "block", "flex", or "none"

### Flex Container
- `flexDirection` - "row", "column", "row-reverse", "column-reverse"
- `justifyContent` - "flex-start", "flex-end", "center", "space-between", "space-around", "space-evenly"
- `alignItems` - "flex-start", "flex-end", "center", "stretch", "baseline"
- `flexWrap` - "nowrap", "wrap"
- `gap` - Gap between items (pixels)

### Visual
- `backgroundColor` - Background color (hex, rgb, rgba, or named)
- `borderWidth` - Border width (pixels)
- `borderColor` - Border color
- `borderRadius` - Corner radius (single value or "tl tr br bl")
- `opacity` - Opacity (0-1)

### Typography (Text)
- `fontFamily` - Font family name
- `fontSize` - Font size (pixels)
- `fontWeight` - Weight (100-900 or "normal"/"bold")
- `lineHeight` - Line height multiplier
- `textAlign` - "left", "center", "right", "justify"
- `color` - Text color
- `letterSpacing` - Letter spacing (pixels)

### Flex Child (Text/Image)
- `flexGrow` - Flex grow factor
- `flexShrink` - Flex shrink factor

## Building from Source

```bash
# Check
cargo check

# Test
cargo test

# Build release
cargo build --release

# Build WASM
wasm-pack build --target web
```

## Architecture

```
JSON Element Tree
       │
       ▼
┌──────────────────────┐
│ 1. DESERIALIZE       │  serde_json → Element tree
└──────────────────────┘
       │
       ▼
┌──────────────────────┐
│ 2. LAYOUT TREE       │  Map elements → Taffy tree
└──────────────────────┘
       │
       ▼
┌──────────────────────┐
│ 3. LAYOUT COMPUTE    │  Taffy + Parley measure
└──────────────────────┘
       │
       ▼
┌──────────────────────┐
│ 4. RENDER TREE       │  Final positions, boxes
└──────────────────────┘
       │
       ▼
┌──────────────────────┐
│ 5. SVG GENERATION    │  Emit <rect>, <text>, etc.
└──────────────────────┘
```

## Dependencies

- **[Taffy](https://github.com/DioxusLabs/taffy)** - CSS Flexbox layout engine
- **[Parley](https://github.com/linebender/parley)** - Text layout and shaping
- **[serde](https://serde.rs/)** - JSON deserialization
- **[wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/)** - WASM bindings

## License

MIT OR Apache-2.0
