//! Snapshot tests for PlayBadges recreation using htvg.
//!
//! These tests verify that htvg can recreate PlayBadges-style app badges
//! (originally built with satori) using the JSON element tree format.
//!
//! ## Missing features for full PlayBadges parity
//!
//! The following features would be needed for a pixel-perfect recreation:
//!
//! 1. **boxShadow**: PlayBadges uses `shadow-lg` on the app icon. htvg does not
//!    yet support box shadows (drop shadows on elements).
//!
//! 2. **Inline/mixed text styling**: PlayBadges renders the download count, star
//!    rating, and rating count as differently-styled spans within the same line
//!    (e.g., "4.4" in white + "★" in yellow + "(1.2K)" in gray). htvg's text
//!    element only supports a single style per text node. Rich text / inline
//!    spans would be needed.
//!
//! 3. **Icon fonts (Material Symbols)**: PlayBadges uses Material Symbols Outlined
//!    for the download icon (&#xf090;). htvg would need font loading and icon
//!    glyph rendering support.
//!
//! 4. **Font loading from URLs at compile time**: PlayBadges loads fonts from
//!    Google Fonts CDN at render time. htvg supports @font-face URL references
//!    in SVG output but doesn't fetch and use font metrics from remote URLs
//!    during layout computation.
//!
//! 5. **Percentage-based heights**: While htvg supports percentage dimensions,
//!    using `height: "100%"` on flex children for full-height content areas
//!    could be improved.

use htvg_core::{compile_document, CompileOptions, compile};

/// Helper to load and compile a PlayBadges example JSON file.
fn compile_example(filename: &str) -> htvg_core::CompileResult {
    let json = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples")
            .join(filename),
    )
    .unwrap_or_else(|e| panic!("Failed to read example {}: {}", filename, e));

    compile_document(&json).unwrap_or_else(|e| panic!("Failed to compile {}: {}", filename, e))
}

#[test]
fn test_playbadge_dark_compiles() {
    let result = compile_example("playbadge-dark.json");

    // Verify basic SVG structure
    assert!(result.svg.contains("<svg"));
    assert!(result.svg.contains("</svg>"));

    // Verify dimensions
    assert_eq!(result.width, 350.0);
    assert_eq!(result.height, 150.0);

    // Verify dark theme colors
    assert!(result.svg.contains("#000000")); // background
    assert!(result.svg.contains("#1f2937")); // border

    // Verify text content (text may be word-wrapped into separate tspan elements)
    assert!(result.svg.contains("VR"));
    assert!(result.svg.contains("Checker"));
    assert!(result.svg.contains("pavi2410"));
    assert!(result.svg.contains("150K"));

    // Snapshot the full SVG output
    insta::assert_snapshot!("playbadge_dark", result.svg);
}

#[test]
fn test_playbadge_light_compiles() {
    let result = compile_example("playbadge-light.json");

    // Verify basic SVG structure
    assert!(result.svg.contains("<svg"));
    assert!(result.svg.contains("</svg>"));

    // Verify dimensions
    assert_eq!(result.width, 350.0);
    assert_eq!(result.height, 150.0);

    // Verify light theme colors
    assert!(result.svg.contains("#ffffff")); // background
    assert!(result.svg.contains("#f3f4f6")); // border

    // Verify text content (text may be word-wrapped into separate tspan elements)
    assert!(result.svg.contains("VR"));
    assert!(result.svg.contains("Checker"));
    assert!(result.svg.contains("pavi2410"));
    assert!(result.svg.contains("150K"));

    // Snapshot the full SVG output
    insta::assert_snapshot!("playbadge_light", result.svg);
}

#[test]
fn test_playbadge_dark_norating_compiles() {
    let result = compile_example("playbadge-dark-norating.json");

    // Verify basic SVG structure
    assert!(result.svg.contains("<svg"));
    assert!(result.svg.contains("</svg>"));

    // Verify dimensions
    assert_eq!(result.width, 350.0);
    assert_eq!(result.height, 150.0);

    // Verify content for app with no rating (text may be word-wrapped)
    assert!(result.svg.contains("Folo:"));
    assert!(result.svg.contains("pavi2410"));
    assert!(result.svg.contains("N/A"));

    // Snapshot the full SVG output
    insta::assert_snapshot!("playbadge_dark_norating", result.svg);
}

#[test]
fn test_playbadge_has_rounded_corners() {
    let result = compile_example("playbadge-dark.json");

    // The outer container has borderRadius: 24 - should generate a rounded rect
    // Border radius 24 is uniform, so it uses rx attribute
    assert!(result.svg.contains("rx=\"24"));
}

#[test]
fn test_playbadge_has_image() {
    let result = compile_example("playbadge-dark.json");

    // Should contain an image element with the icon URL
    assert!(result.svg.contains("<image"));
    assert!(result.svg.contains("example-icon"));

    // Image should have rounded corners (borderRadius: 16)
    assert!(result.svg.contains("clip-path"));
}

#[test]
fn test_playbadge_has_border() {
    let result = compile_example("playbadge-dark.json");

    // Should have a border stroke with the gray color
    assert!(result.svg.contains("stroke=\"#1f2937\""));
    assert!(result.svg.contains("stroke-width"));
}

#[test]
fn test_playbadge_flex_grow() {
    // Test that flexGrow works on flex containers - the content area
    // should fill remaining space after the icon
    let json = r###"{
        "type": "flex",
        "style": {
            "width": 300,
            "height": 100,
            "flexDirection": "row",
            "backgroundColor": "#000000"
        },
        "children": [
            {
                "type": "box",
                "style": { "width": 50, "height": 50, "backgroundColor": "#ff0000" }
            },
            {
                "type": "flex",
                "style": {
                    "flexGrow": 1,
                    "flexDirection": "column",
                    "backgroundColor": "#00ff00"
                },
                "children": [
                    {
                        "type": "text",
                        "content": "Expanded",
                        "style": { "color": "#ffffff" }
                    }
                ]
            }
        ]
    }"###;

    let options = CompileOptions {
        width: 300.0,
        height: Some(100.0),
        ..CompileOptions::default()
    };

    let result = compile(json, &options).unwrap();
    assert!(result.svg.contains("Expanded"));

    // The green flex container should be wider than 50px (it should grow)
    // We can verify this by checking that both rects exist with different widths
    assert!(result.svg.contains("#00ff00"));
    assert!(result.svg.contains("#ff0000"));

    insta::assert_snapshot!("flex_grow_layout", result.svg);
}

/// Test to verify the OG image layout pattern (similar to pavi2410/website og-image.tsx)
#[test]
fn test_og_image_layout() {
    let json = r###"{
        "meta": {
            "width": 1200,
            "height": 630
        },
        "content": {
            "type": "flex",
            "style": {
                "width": 1200,
                "height": 630,
                "flexDirection": "column",
                "justifyContent": "space-between",
                "backgroundColor": "#1a1a1a",
                "padding": 48
            },
            "children": [
                {
                    "type": "text",
                    "content": "Building a CSS-in-Rust SVG Generator",
                    "style": {
                        "fontSize": 72,
                        "fontWeight": 800,
                        "color": "#ffffff"
                    }
                },
                {
                    "type": "flex",
                    "style": {
                        "flexDirection": "column",
                        "gap": 24
                    },
                    "children": [
                        {
                            "type": "text",
                            "content": "Pavitra Golchha",
                            "style": {
                                "fontSize": 28,
                                "color": "#ffffff"
                            }
                        },
                        {
                            "type": "flex",
                            "style": {
                                "flexDirection": "row",
                                "gap": 12,
                                "flexWrap": "wrap"
                            },
                            "children": [
                                {
                                    "type": "flex",
                                    "style": {
                                        "backgroundColor": "#4ade80",
                                        "borderRadius": 6,
                                        "padding": "8 16"
                                    },
                                    "children": [
                                        {
                                            "type": "text",
                                            "content": "css",
                                            "style": { "fontSize": 24, "color": "#000000" }
                                        }
                                    ]
                                },
                                {
                                    "type": "flex",
                                    "style": {
                                        "backgroundColor": "#22d3ee",
                                        "borderRadius": 6,
                                        "padding": "8 16"
                                    },
                                    "children": [
                                        {
                                            "type": "text",
                                            "content": "web",
                                            "style": { "fontSize": 24, "color": "#000000" }
                                        }
                                    ]
                                },
                                {
                                    "type": "flex",
                                    "style": {
                                        "backgroundColor": "#fb7185",
                                        "borderRadius": 6,
                                        "padding": "8 16"
                                    },
                                    "children": [
                                        {
                                            "type": "text",
                                            "content": "performance",
                                            "style": { "fontSize": 24, "color": "#000000" }
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    }"###;

    let result = compile_document(json).unwrap();

    assert_eq!(result.width, 1200.0);
    assert_eq!(result.height, 630.0);
    // Text may be word-wrapped into separate tspan elements
    assert!(result.svg.contains("Building"));
    assert!(result.svg.contains("Generator"));
    assert!(result.svg.contains("Pavitra"));
    assert!(result.svg.contains("css"));
    assert!(result.svg.contains("web"));
    assert!(result.svg.contains("performance"));

    insta::assert_snapshot!("og_image_layout", result.svg);
}
