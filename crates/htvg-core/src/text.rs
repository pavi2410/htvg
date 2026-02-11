//! Text layout and measurement using Parley.
//!
//! Handles text shaping, line breaking, and glyph positioning.
//!
//! TODO: Support font rasterization — render text glyphs to path data so SVG
//! output is fully self-contained and doesn't depend on the viewer having the
//! font installed. This would use the glyph outlines from the font file to
//! emit `<path>` elements instead of `<text>` elements.

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

    /// Register a font from binary data. Returns registered family names.
    pub fn register_font(&mut self, data: Vec<u8>) -> Vec<String> {
        let families = self.font_cx.collection.register_fonts(data.into(), None);
        families
            .iter()
            .map(|(id, _info)| {
                self.font_cx
                    .collection
                    .family_name(*id)
                    .unwrap_or("unknown")
                    .to_string()
            })
            .collect()
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

        let width = layout.width();
        let height = layout.height();

        // Fallback: if Parley returns zero dimensions (no font available),
        // use approximate character-width estimation.
        if width == 0.0 || height == 0.0 {
            return fallback_measure(text, font_size, 1.2, max_width);
        }

        Size { width, height }
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

        // Fallback: if Parley produced lines but none have text content
        // (no font was available to produce glyph runs), use fallback layout.
        let has_content = lines.iter().any(|l| !l.text.is_empty());
        if !has_content {
            return fallback_layout(text, font_size, line_height, max_width);
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

// ============================================================================
// Fallback text measurement/layout (used when no font is available, e.g. WASM)
// ============================================================================

/// Approximate average character width as a fraction of font size.
const CHAR_WIDTH_RATIO: f32 = 0.55;
/// Approximate ascent as a fraction of font size.
const ASCENT_RATIO: f32 = 0.8;
/// Approximate descent as a fraction of font size.
const DESCENT_RATIO: f32 = 0.2;

/// Estimate text width for a string at a given font size.
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * CHAR_WIDTH_RATIO
}

/// Simple word-wrap: split text into lines that fit within max_width.
fn word_wrap(text: &str, font_size: f32, max_width: Option<f32>) -> Vec<String> {
    let max = max_width.unwrap_or(f32::MAX);
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width: f32 = 0.0;
    let space_width = font_size * CHAR_WIDTH_RATIO;

    for word in text.split_whitespace() {
        let word_width = estimate_text_width(word, font_size);

        if !current_line.is_empty() && current_width + space_width + word_width > max {
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += space_width;
            }
            current_line.push_str(word);
            current_width += word_width;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Fallback measurement when Parley returns zero dimensions.
fn fallback_measure(
    text: &str,
    font_size: f32,
    line_height: f32,
    max_width: Option<f32>,
) -> Size<f32> {
    let wrapped = word_wrap(text, font_size, max_width);
    let row_height = font_size * line_height;
    let width = wrapped
        .iter()
        .map(|l| estimate_text_width(l, font_size))
        .fold(0.0_f32, f32::max);
    let width = match max_width {
        Some(mw) => width.min(mw),
        None => width,
    };
    Size {
        width,
        height: row_height * wrapped.len() as f32,
    }
}

/// Fallback layout when Parley produces no glyph runs.
fn fallback_layout(
    text: &str,
    font_size: f32,
    line_height: f32,
    max_width: f32,
) -> TextLayoutResult {
    let wrapped = word_wrap(text, font_size, Some(max_width));
    let row_height = font_size * line_height;
    let ascent = font_size * ASCENT_RATIO;
    let descent = font_size * DESCENT_RATIO;

    let mut lines = Vec::new();
    for (i, line_text) in wrapped.iter().enumerate() {
        let baseline = row_height * i as f32 + ascent;
        lines.push(TextLine {
            text: line_text.clone(),
            baseline,
            ascent,
            descent,
            glyphs: vec![],
        });
    }

    let width = wrapped
        .iter()
        .map(|l| estimate_text_width(l, font_size))
        .fold(0.0_f32, f32::max)
        .min(max_width);

    TextLayoutResult {
        width,
        height: row_height * wrapped.len() as f32,
        lines,
    }
}
