//! HTVG - HyperText Vector Graphics
//!
//! A static compiler that converts JSON element trees to SVG with correct text layout.
//!
//! # Example
//!
//! ```ignore
//! use htvg::{compile, CompileOptions};
//!
//! let json = r#"{
//!     "type": "flex",
//!     "style": { "width": 400, "padding": 20, "backgroundColor": "#fff" },
//!     "children": [
//!         { "type": "text", "content": "Hello World", "style": { "fontSize": 24 } }
//!     ]
//! }"#;
//!
//! let result = compile(json, &CompileOptions::default())?;
//! println!("{}", result.svg);
//! ```

pub mod element;
pub mod layout;
pub mod render;
pub mod svg;
pub mod text;

use serde::{Deserialize, Serialize};

pub use element::Element;
pub use layout::LayoutEngine;
pub use render::RenderTree;
pub use svg::SvgOptions;

/// A font source to register before rendering.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontSource {
    /// Font family name (used in SVG @font-face and as identifier).
    pub family: String,
    /// URL to the font file — emitted as @font-face src in the SVG.
    #[serde(default)]
    pub url: Option<String>,
    /// Font weight (default: 400).
    #[serde(default = "default_font_weight")]
    pub weight: u16,
    /// Base64-encoded font data (TTF/OTF/WOFF2) — used for text layout.
    #[serde(default)]
    pub data: Option<String>,
}

fn default_font_weight() -> u16 {
    400
}

/// Compilation options.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileOptions {
    /// Output width in pixels
    pub width: f32,
    /// Output height in pixels (auto-computed if not specified)
    pub height: Option<f32>,
    /// Default font size (default: 16)
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Font family applied to text elements without an explicit fontFamily
    #[serde(default)]
    pub font_family: Option<String>,
    /// Fonts to register.
    #[serde(default)]
    pub fonts: Vec<FontSource>,
}

fn default_font_size() -> f32 {
    16.0
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: None,
            font_size: 16.0,
            font_family: None,
            fonts: Vec::new(),
        }
    }
}

/// Compilation result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileResult {
    /// Generated SVG string
    pub svg: String,
    /// Computed width
    pub width: f32,
    /// Computed height
    pub height: f32,
    /// Any warnings during compilation
    pub warnings: Vec<String>,
}

/// Compilation error.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileError {
    pub message: String,
    pub kind: String,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for CompileError {}

/// A self-contained HTVG document with metadata and content.
///
/// ```json
/// {
///   "meta": { "width": 800 },
///   "content": { "type": "flex", "children": [...] }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtvgDocument {
    /// Compilation options (width, height, etc.)
    #[serde(default)]
    pub meta: CompileOptions,
    /// The element tree to render
    pub content: Element,
}

/// Compile a self-contained HTVG document (meta + content) to SVG.
pub fn compile_document(doc_json: &str) -> Result<CompileResult, CompileError> {
    let doc: HtvgDocument = serde_json::from_str(doc_json).map_err(|e| CompileError {
        message: e.to_string(),
        kind: "parse_error".to_string(),
    })?;

    compile_element(&doc.content, &doc.meta)
}

/// Compile an element tree to SVG.
///
/// # Arguments
///
/// * `element_json` - JSON string representing the element tree
/// * `options` - Compilation options
///
/// # Returns
///
/// Result containing the SVG string and metadata, or an error.
pub fn compile(element_json: &str, options: &CompileOptions) -> Result<CompileResult, CompileError> {
    // Parse element tree
    let element: Element = serde_json::from_str(element_json).map_err(|e| CompileError {
        message: e.to_string(),
        kind: "parse_error".to_string(),
    })?;

    compile_element(&element, options)
}

/// Compile a parsed element tree to SVG.
pub fn compile_element(
    element: &Element,
    options: &CompileOptions,
) -> Result<CompileResult, CompileError> {
    let warnings = Vec::new();

    // Create layout engine
    let mut layout_engine = LayoutEngine::new();

    // Decode and register base64 font data (used by CLI; WASM falls back to approximate layout)
    for font in &options.fonts {
        if let Some(data) = &font.data {
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .map_err(|e| CompileError {
                    message: format!("Invalid base64 font data: {}", e),
                    kind: "font_error".to_string(),
                })?;
            layout_engine.text_engine.register_font(bytes);
        }
    }

    // Compute layout
    let layout_result = layout_engine
        .compute_layout(element, options.width, options.height, options.font_family.as_deref())
        .map_err(|e| CompileError {
            message: e.to_string(),
            kind: "layout_error".to_string(),
        })?;

    // Build render tree
    let render_tree = render::build_render_tree(&layout_result, &mut layout_engine.text_engine);

    // Generate SVG
    let svg_options = SvgOptions::default();
    let svg = svg::generate_svg(&render_tree, &svg_options, &options.fonts);

    Ok(CompileResult {
        svg,
        width: render_tree.width,
        height: render_tree.height,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
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
            ..CompileOptions::default()
        };

        let result = compile(json, &options).unwrap();
        assert!(result.svg.contains("<svg"));
        assert!(result.svg.contains("</svg>"));
        assert!(result.svg.contains("Hello World"));
    }

    #[test]
    fn test_compile_box_element() {
        let json = r###"{
            "type": "box",
            "style": { "width": 200, "height": 100, "backgroundColor": "#ff0000" }
        }"###;

        let options = CompileOptions::default();
        let result = compile(json, &options).unwrap();

        assert!(result.svg.contains("rect"));
        assert!(result.svg.contains("#ff0000"));
    }

    #[test]
    fn test_invalid_json() {
        let json = "not valid json";
        let options = CompileOptions::default();
        let result = compile(json, &options);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, "parse_error");
    }
}
