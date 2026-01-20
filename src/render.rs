//! Render tree generation from layout results.
//!
//! Converts the laid-out Taffy tree into a flat list of render commands
//! that can be converted to SVG.

use taffy::NodeId;

use crate::element::Color;
use crate::layout::{ElementType, LayoutResult};
use crate::text::TextLayoutEngine;

/// A rectangle in pixel coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Render commands that can be converted to SVG.
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Draw a filled rectangle
    FillRect {
        rect: Rect,
        color: Color,
        border_radius: [f32; 4],
    },

    /// Draw a stroked rectangle (border)
    StrokeRect {
        rect: Rect,
        color: Color,
        width: f32,
        border_radius: [f32; 4],
    },

    /// Draw text as <text> element (selectable)
    Text {
        x: f32,
        y: f32,
        content: String,
        font_family: String,
        font_size: f32,
        font_weight: u16,
        color: Color,
        lines: Vec<TextLineRender>,
    },

    /// Draw text as <path> (vector mode)
    TextPath {
        path_data: String,
        color: Color,
    },

    /// Draw an image
    Image {
        rect: Rect,
        src: String,
        border_radius: [f32; 4],
    },

    /// Begin a clipping region
    PushClip {
        rect: Rect,
        border_radius: [f32; 4],
    },

    /// End clipping region
    PopClip,

    /// Begin opacity group
    PushOpacity {
        opacity: f32,
    },

    /// End opacity group
    PopOpacity,
}

/// A line of text for rendering.
#[derive(Debug, Clone)]
pub struct TextLineRender {
    pub x: f32,
    pub y: f32,
    pub text: String,
}

/// The render tree - a flat list of commands in draw order.
#[derive(Debug)]
pub struct RenderTree {
    pub commands: Vec<RenderCommand>,
    pub width: f32,
    pub height: f32,
}

/// Build render tree from layout result.
pub fn build_render_tree(
    layout: &LayoutResult,
    text_engine: &mut TextLayoutEngine,
) -> RenderTree {
    let mut commands = Vec::new();

    // Get root layout to determine dimensions
    let root_layout = layout.taffy.layout(layout.root).unwrap();
    let width = root_layout.size.width;
    let height = root_layout.size.height;

    // Traverse tree in depth-first order
    render_node(
        &layout,
        layout.root,
        0.0,
        0.0,
        &mut commands,
        text_engine,
    );

    RenderTree {
        commands,
        width,
        height,
    }
}

fn render_node(
    layout: &LayoutResult,
    node_id: NodeId,
    parent_x: f32,
    parent_y: f32,
    commands: &mut Vec<RenderCommand>,
    text_engine: &mut TextLayoutEngine,
) {
    let node_layout = layout.taffy.layout(node_id).unwrap();
    let node_data = layout.node_data.get(&node_id);

    let x = parent_x + node_layout.location.x;
    let y = parent_y + node_layout.location.y;
    let width = node_layout.size.width;
    let height = node_layout.size.height;

    let rect = Rect {
        x,
        y,
        width,
        height,
    };

    if let Some(data) = node_data {
        let visual = &data.visual;

        // Handle opacity
        let needs_opacity = visual.opacity < 1.0;
        if needs_opacity {
            commands.push(RenderCommand::PushOpacity {
                opacity: visual.opacity,
            });
        }

        // Handle clipping for border-radius
        let has_radius = visual.border_radius.iter().any(|&r| r > 0.0);
        if has_radius && matches!(data.element_type, ElementType::Box | ElementType::Flex) {
            // For now, skip clipping - just apply border-radius to the rect
        }

        // Draw background
        if let Some(ref bg_color) = visual.background_color {
            if bg_color.a > 0 {
                commands.push(RenderCommand::FillRect {
                    rect,
                    color: bg_color.clone(),
                    border_radius: visual.border_radius,
                });
            }
        }

        // Draw border
        if visual.border_width > 0.0 {
            if let Some(ref border_color) = visual.border_color {
                if border_color.a > 0 {
                    commands.push(RenderCommand::StrokeRect {
                        rect,
                        color: border_color.clone(),
                        width: visual.border_width,
                        border_radius: visual.border_radius,
                    });
                }
            }
        }

        // Handle element-specific rendering
        match &data.element_type {
            ElementType::Box | ElementType::Flex => {
                // Container - render children
                if let Ok(children) = layout.taffy.children(node_id) {
                    // Account for padding
                    let padding = node_layout.padding;
                    let _content_x = x + padding.left;
                    let _content_y = y + padding.top;

                    for child_id in children {
                        render_node(layout, child_id, x, y, commands, text_engine);
                    }
                }
            }

            ElementType::Text { content, style } => {
                // Render text
                let text_layout = text_engine.layout(
                    content,
                    &style.font_family,
                    style.font_size,
                    style.font_weight,
                    style.line_height,
                    style.text_align,
                    width,
                );

                // Convert to render command
                let mut lines = Vec::new();
                let mut current_y = y;

                for line in &text_layout.lines {
                    // For simplicity, we'll output the original text
                    // In a more complete implementation, we'd track which glyphs
                    // correspond to which characters
                    lines.push(TextLineRender {
                        x,
                        y: current_y + line.baseline,
                        text: content.clone(), // Simplified - should be per-line
                    });
                    current_y += line.ascent + line.descent;
                }

                // For now, output as simple text command
                commands.push(RenderCommand::Text {
                    x,
                    y: y + text_layout.lines.first().map(|l| l.baseline).unwrap_or(style.font_size),
                    content: content.clone(),
                    font_family: style.font_family.clone(),
                    font_size: style.font_size,
                    font_weight: style.font_weight,
                    color: style.color.clone(),
                    lines,
                });
            }

            ElementType::Image { src } => {
                commands.push(RenderCommand::Image {
                    rect,
                    src: src.clone(),
                    border_radius: visual.border_radius,
                });
            }
        }

        // Close opacity group
        if needs_opacity {
            commands.push(RenderCommand::PopOpacity);
        }
    }
}
