//! Layout computation using Taffy.
//!
//! Builds a Taffy layout tree from the Element tree and computes layout.

use std::collections::HashMap;

use taffy::prelude::*;
use taffy::style::Style;

use crate::element::{
    self, AlignItems as ElemAlignItems, BoxStyle, Color, Dimension, Element,
    FlexDirection as ElemFlexDirection, FlexStyle, FlexWrap as ElemFlexWrap, ImageStyle,
    JustifyContent as ElemJustifyContent, Spacing, TextAlign, TextRendering, TextStyle,
};
use crate::text::TextLayoutEngine;

/// Context attached to Taffy leaf nodes that need measurement.
#[derive(Debug, Clone)]
pub enum NodeContext {
    /// Text node that needs Parley for measurement
    Text(TextContext),
    /// Image with intrinsic dimensions
    Image { width: f32, height: f32 },
}

/// Text measurement context.
#[derive(Debug, Clone)]
pub struct TextContext {
    pub content: String,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub font_weight: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for TextContext {
    fn default() -> Self {
        Self {
            content: String::new(),
            font_family: None,
            font_size: 16.0,
            font_weight: 400,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

/// Result of layout computation.
pub struct LayoutResult {
    /// The Taffy tree with computed layout
    pub taffy: TaffyTree<NodeContext>,
    /// Root node ID
    pub root: NodeId,
    /// Map from Taffy NodeId to original Element reference index
    pub node_data: HashMap<NodeId, NodeData>,
}

/// Data associated with each laid out node.
#[derive(Debug, Clone)]
pub struct NodeData {
    /// The element type for rendering
    pub element_type: ElementType,
    /// Visual style for rendering
    pub visual: VisualStyle,
}

#[derive(Debug, Clone)]
pub enum ElementType {
    Box,
    Flex,
    Text { content: String, style: TextStyleResolved },
    Image { src: String },
}

/// Resolved text style with defaults applied.
#[derive(Debug, Clone)]
pub struct TextStyleResolved {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub line_height: f32,
    pub text_align: TextAlign,
    pub color: Color,
    pub letter_spacing: f32,
    pub text_rendering: TextRendering,
}

impl Default for TextStyleResolved {
    fn default() -> Self {
        Self {
            font_family: "sans-serif".to_string(),
            font_size: 16.0,
            font_weight: 400,
            line_height: 1.2,
            text_align: TextAlign::Left,
            color: Color::BLACK,
            letter_spacing: 0.0,
            text_rendering: TextRendering::Text,
        }
    }
}

/// Visual properties for rendering.
#[derive(Debug, Clone, Default)]
pub struct VisualStyle {
    pub background_color: Option<Color>,
    pub border_width: f32,
    pub border_color: Option<Color>,
    pub border_radius: [f32; 4],
    pub opacity: f32,
}

/// Layout engine that builds and computes layout.
pub struct LayoutEngine {
    pub text_engine: TextLayoutEngine,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            text_engine: TextLayoutEngine::new(),
        }
    }

    /// Build layout tree from element tree and compute layout.
    pub fn compute_layout(
        &mut self,
        element: &Element,
        viewport_width: f32,
        viewport_height: Option<f32>,
    ) -> Result<LayoutResult, LayoutError> {
        let mut taffy: TaffyTree<NodeContext> = TaffyTree::new();
        let mut node_data = HashMap::new();

        // Build tree recursively
        let root = self.build_node(&mut taffy, &mut node_data, element)?;

        // Compute layout
        let available_space = Size {
            width: AvailableSpace::Definite(viewport_width),
            height: viewport_height
                .map(AvailableSpace::Definite)
                .unwrap_or(AvailableSpace::MaxContent),
        };

        // Create a reference to text_engine for the closure
        let text_engine = &mut self.text_engine;

        taffy.compute_layout_with_measure(
            root,
            available_space,
            |known_dimensions, available_space, _node_id, node_context, _style| {
                measure_function(known_dimensions, available_space, node_context, text_engine)
            },
        )?;

        Ok(LayoutResult {
            taffy,
            root,
            node_data,
        })
    }

    fn build_node(
        &self,
        taffy: &mut TaffyTree<NodeContext>,
        node_data: &mut HashMap<NodeId, NodeData>,
        element: &Element,
    ) -> Result<NodeId, LayoutError> {
        match element {
            Element::Box { style, children } => {
                let taffy_style = box_style_to_taffy(style);
                let child_ids = children
                    .iter()
                    .map(|child| self.build_node(taffy, node_data, child))
                    .collect::<Result<Vec<_>, _>>()?;

                let node_id = taffy.new_with_children(taffy_style, &child_ids)?;

                node_data.insert(
                    node_id,
                    NodeData {
                        element_type: ElementType::Box,
                        visual: VisualStyle {
                            background_color: style.background_color.clone(),
                            border_width: style.border_width.unwrap_or(0.0),
                            border_color: style.border_color.clone(),
                            border_radius: style
                                .border_radius
                                .as_ref()
                                .map(|r| r.to_corners())
                                .unwrap_or([0.0; 4]),
                            opacity: style.opacity.unwrap_or(1.0),
                        },
                    },
                );

                Ok(node_id)
            }

            Element::Flex { style, children } => {
                let taffy_style = flex_style_to_taffy(style);
                let child_ids = children
                    .iter()
                    .map(|child| self.build_node(taffy, node_data, child))
                    .collect::<Result<Vec<_>, _>>()?;

                let node_id = taffy.new_with_children(taffy_style, &child_ids)?;

                node_data.insert(
                    node_id,
                    NodeData {
                        element_type: ElementType::Flex,
                        visual: VisualStyle {
                            background_color: style.background_color.clone(),
                            border_width: style.border_width.unwrap_or(0.0),
                            border_color: style.border_color.clone(),
                            border_radius: style
                                .border_radius
                                .as_ref()
                                .map(|r| r.to_corners())
                                .unwrap_or([0.0; 4]),
                            opacity: style.opacity.unwrap_or(1.0),
                        },
                    },
                );

                Ok(node_id)
            }

            Element::Text { content, style } => {
                let text_context = NodeContext::Text(TextContext {
                    content: content.clone(),
                    font_family: style.font_family.clone(),
                    font_size: style.font_size.unwrap_or(16.0),
                    font_weight: style.font_weight.map(|w| w.0).unwrap_or(400),
                    line_height: style.line_height.unwrap_or(1.2),
                    letter_spacing: style.letter_spacing.unwrap_or(0.0),
                });

                let taffy_style = text_style_to_taffy(style);
                let node_id = taffy.new_leaf_with_context(taffy_style, text_context)?;

                node_data.insert(
                    node_id,
                    NodeData {
                        element_type: ElementType::Text {
                            content: content.clone(),
                            style: TextStyleResolved {
                                font_family: style
                                    .font_family
                                    .clone()
                                    .unwrap_or_else(|| "sans-serif".to_string()),
                                font_size: style.font_size.unwrap_or(16.0),
                                font_weight: style.font_weight.map(|w| w.0).unwrap_or(400),
                                line_height: style.line_height.unwrap_or(1.2),
                                text_align: style.text_align.unwrap_or_default(),
                                color: style.color.clone().unwrap_or(Color::BLACK),
                                letter_spacing: style.letter_spacing.unwrap_or(0.0),
                                text_rendering: style.text_rendering.unwrap_or_default(),
                            },
                        },
                        visual: VisualStyle {
                            opacity: 1.0,
                            ..Default::default()
                        },
                    },
                );

                Ok(node_id)
            }

            Element::Image {
                src,
                width,
                height,
                style,
            } => {
                let image_context = NodeContext::Image {
                    width: *width,
                    height: *height,
                };

                let taffy_style = image_style_to_taffy(style, *width, *height);
                let node_id = taffy.new_leaf_with_context(taffy_style, image_context)?;

                node_data.insert(
                    node_id,
                    NodeData {
                        element_type: ElementType::Image { src: src.clone() },
                        visual: VisualStyle {
                            border_radius: style
                                .border_radius
                                .as_ref()
                                .map(|r| r.to_corners())
                                .unwrap_or([0.0; 4]),
                            opacity: style.opacity.unwrap_or(1.0),
                            ..Default::default()
                        },
                    },
                );

                Ok(node_id)
            }
        }
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Measure function for leaf nodes.
fn measure_function(
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    node_context: Option<&mut NodeContext>,
    text_engine: &mut TextLayoutEngine,
) -> Size<f32> {
    match node_context {
        Some(NodeContext::Text(ctx)) => {
            // Use known dimensions if available
            if let (Some(width), Some(height)) = (known_dimensions.width, known_dimensions.height) {
                return Size { width, height };
            }

            // Determine available width for text wrapping
            let available_width = match available_space.width {
                AvailableSpace::Definite(w) => Some(w),
                AvailableSpace::MinContent => Some(0.0),
                AvailableSpace::MaxContent => None,
            };

            // Measure text using the text engine
            text_engine.measure(&ctx.content, ctx.font_size, available_width)
        }
        Some(NodeContext::Image { width, height }) => {
            // Use intrinsic dimensions, respecting any known constraints
            Size {
                width: known_dimensions.width.unwrap_or(*width),
                height: known_dimensions.height.unwrap_or(*height),
            }
        }
        None => Size::ZERO,
    }
}

// ============================================================================
// Style conversion functions
// ============================================================================

fn box_style_to_taffy(style: &BoxStyle) -> Style {
    Style {
        display: match style.display {
            Some(element::Display::None) => taffy::Display::None,
            _ => taffy::Display::Block,
        },
        size: Size {
            width: dimension_to_taffy(&style.width),
            height: dimension_to_taffy(&style.height),
        },
        min_size: Size {
            width: dimension_to_taffy(&style.min_width),
            height: dimension_to_taffy(&style.min_height),
        },
        max_size: Size {
            width: dimension_to_taffy(&style.max_width),
            height: dimension_to_taffy(&style.max_height),
        },
        margin: spacing_to_taffy_rect(&style.margin),
        padding: spacing_to_taffy_rect_lp(&style.padding),
        border: spacing_to_taffy_rect_lp(&style.border_width.map(Spacing::Uniform)),
        ..Default::default()
    }
}

fn flex_style_to_taffy(style: &FlexStyle) -> Style {
    Style {
        display: match style.display {
            Some(element::Display::None) => taffy::Display::None,
            _ => taffy::Display::Flex,
        },
        size: Size {
            width: dimension_to_taffy(&style.width),
            height: dimension_to_taffy(&style.height),
        },
        min_size: Size {
            width: dimension_to_taffy(&style.min_width),
            height: dimension_to_taffy(&style.min_height),
        },
        max_size: Size {
            width: dimension_to_taffy(&style.max_width),
            height: dimension_to_taffy(&style.max_height),
        },
        margin: spacing_to_taffy_rect(&style.margin),
        padding: spacing_to_taffy_rect_lp(&style.padding),
        border: spacing_to_taffy_rect_lp(&style.border_width.map(Spacing::Uniform)),
        flex_direction: match style.flex_direction {
            Some(ElemFlexDirection::Row) | None => taffy::FlexDirection::Row,
            Some(ElemFlexDirection::Column) => taffy::FlexDirection::Column,
            Some(ElemFlexDirection::RowReverse) => taffy::FlexDirection::RowReverse,
            Some(ElemFlexDirection::ColumnReverse) => taffy::FlexDirection::ColumnReverse,
        },
        justify_content: Some(match style.justify_content {
            Some(ElemJustifyContent::FlexStart) | None => taffy::JustifyContent::FlexStart,
            Some(ElemJustifyContent::FlexEnd) => taffy::JustifyContent::FlexEnd,
            Some(ElemJustifyContent::Center) => taffy::JustifyContent::Center,
            Some(ElemJustifyContent::SpaceBetween) => taffy::JustifyContent::SpaceBetween,
            Some(ElemJustifyContent::SpaceAround) => taffy::JustifyContent::SpaceAround,
            Some(ElemJustifyContent::SpaceEvenly) => taffy::JustifyContent::SpaceEvenly,
        }),
        align_items: Some(match style.align_items {
            Some(ElemAlignItems::FlexStart) => taffy::AlignItems::FlexStart,
            Some(ElemAlignItems::FlexEnd) => taffy::AlignItems::FlexEnd,
            Some(ElemAlignItems::Center) => taffy::AlignItems::Center,
            Some(ElemAlignItems::Stretch) | None => taffy::AlignItems::Stretch,
            Some(ElemAlignItems::Baseline) => taffy::AlignItems::Baseline,
        }),
        gap: Size {
            width: length(style.gap.unwrap_or(0.0)),
            height: length(style.gap.unwrap_or(0.0)),
        },
        flex_wrap: match style.flex_wrap {
            Some(ElemFlexWrap::Wrap) => taffy::FlexWrap::Wrap,
            _ => taffy::FlexWrap::NoWrap,
        },
        ..Default::default()
    }
}

fn text_style_to_taffy(style: &TextStyle) -> Style {
    Style {
        flex_grow: style.flex_grow.unwrap_or(0.0),
        flex_shrink: style.flex_shrink.unwrap_or(1.0),
        ..Default::default()
    }
}

fn image_style_to_taffy(style: &ImageStyle, intrinsic_width: f32, intrinsic_height: f32) -> Style {
    Style {
        size: Size {
            width: style
                .width
                .as_ref()
                .map(|d| dimension_to_taffy(&Some(d.clone())))
                .unwrap_or(length(intrinsic_width)),
            height: style
                .height
                .as_ref()
                .map(|d| dimension_to_taffy(&Some(d.clone())))
                .unwrap_or(length(intrinsic_height)),
        },
        min_size: Size {
            width: dimension_to_taffy(&style.min_width),
            height: dimension_to_taffy(&style.min_height),
        },
        max_size: Size {
            width: dimension_to_taffy(&style.max_width),
            height: dimension_to_taffy(&style.max_height),
        },
        margin: spacing_to_taffy_rect(&style.margin),
        flex_grow: style.flex_grow.unwrap_or(0.0),
        flex_shrink: style.flex_shrink.unwrap_or(1.0),
        ..Default::default()
    }
}

fn dimension_to_taffy(dim: &Option<Dimension>) -> taffy::Dimension {
    match dim {
        None => taffy::Dimension::auto(),
        Some(Dimension::Px(px)) => taffy::Dimension::length(*px),
        Some(Dimension::Percent(s)) => {
            let pct = s.trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
            taffy::Dimension::percent(pct / 100.0)
        }
    }
}

fn spacing_to_taffy_rect(spacing: &Option<Spacing>) -> Rect<LengthPercentageAuto> {
    match spacing {
        None => Rect {
            top: LengthPercentageAuto::length(0.0),
            right: LengthPercentageAuto::length(0.0),
            bottom: LengthPercentageAuto::length(0.0),
            left: LengthPercentageAuto::length(0.0),
        },
        Some(s) => {
            let [top, right, bottom, left] = s.to_edges();
            Rect {
                top: LengthPercentageAuto::length(top),
                right: LengthPercentageAuto::length(right),
                bottom: LengthPercentageAuto::length(bottom),
                left: LengthPercentageAuto::length(left),
            }
        }
    }
}

fn spacing_to_taffy_rect_lp(spacing: &Option<Spacing>) -> Rect<LengthPercentage> {
    match spacing {
        None => Rect {
            top: LengthPercentage::length(0.0),
            right: LengthPercentage::length(0.0),
            bottom: LengthPercentage::length(0.0),
            left: LengthPercentage::length(0.0),
        },
        Some(s) => {
            let [top, right, bottom, left] = s.to_edges();
            Rect {
                top: LengthPercentage::length(top),
                right: LengthPercentage::length(right),
                bottom: LengthPercentage::length(bottom),
                left: LengthPercentage::length(left),
            }
        }
    }
}

/// Layout error type.
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("Taffy error: {0}")]
    Taffy(#[from] taffy::TaffyError),
}
