//! Text layout and measurement using Parley.
//!
//! Handles text shaping, line breaking, and glyph positioning.

use parley::layout::{Alignment, Layout, PositionedLayoutItem};
use parley::style::{FontStack, FontWeight, LineHeight, StyleProperty};
use parley::{AlignmentOptions, FontContext, LayoutContext};
use std::borrow::Cow;
use taffy::Size;

use crate::element::TextAlign;

/// Text layout engine wrapping Parley.
pub struct TextLayoutEngine {
    font_cx: FontContext,
    layout_cx: LayoutContext<[u8; 4]>,
}

impl TextLayoutEngine {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
        }
    }

    /// Register a font from binary data.
    pub fn register_font(&mut self, data: Vec<u8>) {
        self.font_cx.collection.register_fonts(data.into(), None);
    }

    /// Measure text and return (width, height).
    pub fn measure(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> Size<f32> {
        if text.is_empty() {
            return Size {
                width: 0.0,
                height: font_size * 1.2,
            };
        }

        let mut builder = self
            .layout_cx
            .ranged_builder(&mut self.font_cx, text, 1.0, false);

        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(1.2)));

        let mut layout: Layout<[u8; 4]> = builder.build(text);
        layout.break_all_lines(max_width);

        Size {
            width: layout.width(),
            height: layout.height(),
        }
    }

    /// Lay out text with full glyph positions.
    pub fn layout(
        &mut self,
        text: &str,
        font_family: &str,
        font_size: f32,
        font_weight: u16,
        line_height: f32,
        text_align: TextAlign,
        max_width: f32,
    ) -> TextLayoutResult {
        if text.is_empty() {
            return TextLayoutResult {
                width: 0.0,
                height: font_size * line_height,
                lines: vec![],
            };
        }

        let mut builder = self
            .layout_cx
            .ranged_builder(&mut self.font_cx, text, 1.0, false);

        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(font_weight as f32)));
        builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
            line_height,
        )));
        builder.push_default(StyleProperty::FontStack(FontStack::Source(Cow::Owned(
            font_family.to_string(),
        ))));

        let mut layout: Layout<[u8; 4]> = builder.build(text);
        layout.break_all_lines(Some(max_width));

        let alignment = match text_align {
            TextAlign::Left => Alignment::Start,
            TextAlign::Center => Alignment::Center,
            TextAlign::Right => Alignment::End,
            TextAlign::Justify => Alignment::Justify,
        };
        layout.align(Some(max_width), alignment, AlignmentOptions::default());

        // Extract lines
        let mut lines = Vec::new();

        for line in layout.lines() {
            let metrics = line.metrics();
            let mut line_glyphs = Vec::new();
            let mut line_start: Option<usize> = None;
            let mut line_end: usize = 0;

            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(positioned_run) = item {
                    let run_x = positioned_run.offset();
                    let run = positioned_run.run();
                    let range = run.text_range();
                    if line_start.is_none() || range.start < line_start.unwrap() {
                        line_start = Some(range.start);
                    }
                    if range.end > line_end {
                        line_end = range.end;
                    }

                    // Get glyphs from the positioned run directly
                    for glyph in positioned_run.glyphs() {
                        line_glyphs.push(PositionedGlyph {
                            glyph_id: glyph.id,
                            x: run_x + glyph.x,
                            y: glyph.y,
                            advance: glyph.advance,
                        });
                    }
                }
            }

            let line_text = if let Some(start) = line_start {
                text[start..line_end].trim_end().to_string()
            } else {
                String::new()
            };

            lines.push(TextLine {
                text: line_text,
                baseline: metrics.baseline,
                ascent: metrics.ascent,
                descent: metrics.descent,
                glyphs: line_glyphs,
            });
        }

        TextLayoutResult {
            width: layout.width(),
            height: layout.height(),
            lines,
        }
    }
}

impl Default for TextLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of text layout.
#[derive(Debug, Clone)]
pub struct TextLayoutResult {
    pub width: f32,
    pub height: f32,
    pub lines: Vec<TextLine>,
}

/// A line of laid out text.
#[derive(Debug, Clone)]
pub struct TextLine {
    pub text: String,
    pub baseline: f32,
    pub ascent: f32,
    pub descent: f32,
    pub glyphs: Vec<PositionedGlyph>,
}

/// A positioned glyph.
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
}
