use wasm_bindgen::prelude::*;

use htvg::{compile, compile_document, CompileOptions};

/// Initialize the WASM module (sets up panic hook for better error messages).
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Compile a self-contained HTVG document JSON to SVG.
///
/// Input: `{ "meta": { "width": 800 }, "content": { "type": "flex", ... } }`
/// Returns: `{ "svg": "...", "width": 800, "height": 600, "warnings": [] }`
#[wasm_bindgen(js_name = "compileDocument")]
pub fn compile_document_wasm(doc_json: &str) -> Result<JsValue, JsValue> {
    let result = compile_document(doc_json).map_err(|e| {
        JsValue::from_str(&format!("{}: {}", e.kind, e.message))
    })?;

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Compile an element tree JSON with separate options JSON.
///
/// `element_json`: `{ "type": "flex", "children": [...] }`
/// `options_json`: `{ "width": 800 }`
/// Returns: `{ "svg": "...", "width": 800, "height": 600, "warnings": [] }`
#[wasm_bindgen(js_name = "compile")]
pub fn compile_wasm(element_json: &str, options_json: &str) -> Result<JsValue, JsValue> {
    let options: CompileOptions = serde_json::from_str(options_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid options: {}", e)))?;

    let result = compile(element_json, &options).map_err(|e| {
        JsValue::from_str(&format!("{}: {}", e.kind, e.message))
    })?;

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get version information.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
