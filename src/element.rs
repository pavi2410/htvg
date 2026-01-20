//! Element types for HTVG input schema.
//!
//! Defines the JSON element tree structure that gets deserialized and rendered to SVG.

use serde::Deserialize;

/// Root element type - can be Box, Flex, Text, or Image.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    /// Block container element
    Box {
        #[serde(default)]
        style: BoxStyle,
        #[serde(default)]
        children: Vec<Element>,
    },
    /// Flex container element
    Flex {
        #[serde(default)]
        style: FlexStyle,
        #[serde(default)]
        children: Vec<Element>,
    },
    /// Text leaf element
    Text {
        content: String,
        #[serde(default)]
        style: TextStyle,
    },
    /// Image element with intrinsic dimensions
    Image {
        src: String,
        width: f32,
        height: f32,
        #[serde(default)]
        style: ImageStyle,
    },
}

// ============================================================================
// Dimension types
// ============================================================================

/// A dimension value that can be a number (pixels) or percentage string.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Dimension {
    Px(f32),
    Percent(String), // e.g., "50%"
}

impl Dimension {
    /// Convert to pixels given a container size for percentages.
    pub fn to_px(&self, container_size: f32) -> f32 {
        match self {
            Dimension::Px(px) => *px,
            Dimension::Percent(s) => {
                let percent = s.trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
                container_size * percent / 100.0
            }
        }
    }
}

/// Spacing value for margin/padding - can be a single number or space-separated string.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Spacing {
    Uniform(f32),
    Multi(String), // e.g., "10 20" or "10 20 30 40"
}

impl Default for Spacing {
    fn default() -> Self {
        Spacing::Uniform(0.0)
    }
}

impl Spacing {
    /// Parse into [top, right, bottom, left] values.
    pub fn to_edges(&self) -> [f32; 4] {
        match self {
            Spacing::Uniform(v) => [*v, *v, *v, *v],
            Spacing::Multi(s) => {
                let parts: Vec<f32> = s
                    .split_whitespace()
                    .filter_map(|p| p.parse().ok())
                    .collect();
                match parts.len() {
                    1 => [parts[0], parts[0], parts[0], parts[0]],
                    2 => [parts[0], parts[1], parts[0], parts[1]], // vertical, horizontal
                    3 => [parts[0], parts[1], parts[2], parts[1]], // top, horizontal, bottom
                    4 => [parts[0], parts[1], parts[2], parts[3]], // top, right, bottom, left
                    _ => [0.0, 0.0, 0.0, 0.0],
                }
            }
        }
    }
}

/// Border radius - single value or per-corner.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum BorderRadius {
    Uniform(f32),
    Multi(String), // e.g., "8 8 0 0" for top-left, top-right, bottom-right, bottom-left
}

impl Default for BorderRadius {
    fn default() -> Self {
        BorderRadius::Uniform(0.0)
    }
}

impl BorderRadius {
    /// Parse into [top-left, top-right, bottom-right, bottom-left] values.
    pub fn to_corners(&self) -> [f32; 4] {
        match self {
            BorderRadius::Uniform(v) => [*v, *v, *v, *v],
            BorderRadius::Multi(s) => {
                let parts: Vec<f32> = s
                    .split_whitespace()
                    .filter_map(|p| p.parse().ok())
                    .collect();
                match parts.len() {
                    1 => [parts[0], parts[0], parts[0], parts[0]],
                    2 => [parts[0], parts[1], parts[0], parts[1]],
                    4 => [parts[0], parts[1], parts[2], parts[3]],
                    _ => [0.0, 0.0, 0.0, 0.0],
                }
            }
        }
    }
}

// ============================================================================
// Color type
// ============================================================================

/// Color value - supports hex (#RGB, #RRGGBB, #RRGGBBAA) and rgb/rgba functions.
#[derive(Debug, Clone, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };

    /// Convert to CSS color string for SVG output.
    pub fn to_css(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else if self.a == 0 {
            "none".to_string()
        } else {
            format!(
                "rgba({},{},{},{:.3})",
                self.r,
                self.g,
                self.b,
                self.a as f32 / 255.0
            )
        }
    }

    /// Parse a color string.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();

        // Hex colors
        if s.starts_with('#') {
            let hex = &s[1..];
            return match hex.len() {
                3 => {
                    // #RGB
                    let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                    let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                    let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                    Some(Self { r, g, b, a: 255 })
                }
                6 => {
                    // #RRGGBB
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some(Self { r, g, b, a: 255 })
                }
                8 => {
                    // #RRGGBBAA
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    Some(Self { r, g, b, a })
                }
                _ => None,
            };
        }

        // rgb(r, g, b) or rgba(r, g, b, a)
        if s.starts_with("rgb") {
            let inner = s
                .trim_start_matches("rgba")
                .trim_start_matches("rgb")
                .trim_start_matches('(')
                .trim_end_matches(')');
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

            if parts.len() >= 3 {
                let r = parts[0].parse().ok()?;
                let g = parts[1].parse().ok()?;
                let b = parts[2].parse().ok()?;
                let a = if parts.len() >= 4 {
                    let a_float: f32 = parts[3].parse().ok()?;
                    (a_float * 255.0) as u8
                } else {
                    255
                };
                return Some(Self { r, g, b, a });
            }
        }

        // Named colors (basic support)
        match s.to_lowercase().as_str() {
            "transparent" => Some(Self::TRANSPARENT),
            "black" => Some(Self::BLACK),
            "white" => Some(Self::WHITE),
            "red" => Some(Self { r: 255, g: 0, b: 0, a: 255 }),
            "green" => Some(Self { r: 0, g: 128, b: 0, a: 255 }),
            "blue" => Some(Self { r: 0, g: 0, b: 255, a: 255 }),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {}", s)))
    }
}

// ============================================================================
// Enum types for flex properties
// ============================================================================

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Display {
    #[default]
    Block,
    Flex,
    None,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    #[default]
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlexWrap {
    #[default]
    Nowrap,
    Wrap,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextRendering {
    #[default]
    Text,
    Vector,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectFit {
    #[default]
    Contain,
    Cover,
    Fill,
}

/// Font weight - can be a number (100-900) or keyword.
#[derive(Debug, Clone, Copy)]
pub struct FontWeight(pub u16);

impl Default for FontWeight {
    fn default() -> Self {
        Self(400)
    }
}

impl<'de> Deserialize<'de> for FontWeight {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum FontWeightValue {
            Number(u16),
            String(String),
        }

        match FontWeightValue::deserialize(deserializer)? {
            FontWeightValue::Number(n) => Ok(FontWeight(n)),
            FontWeightValue::String(s) => match s.to_lowercase().as_str() {
                "normal" => Ok(FontWeight(400)),
                "bold" => Ok(FontWeight(700)),
                _ => s
                    .parse()
                    .map(FontWeight)
                    .map_err(|_| serde::de::Error::custom("invalid font weight")),
            },
        }
    }
}

// ============================================================================
// Style structs
// ============================================================================

/// Style for Box elements (block container).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BoxStyle {
    // Display
    pub display: Option<Display>,

    // Dimensions
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_height: Option<Dimension>,

    // Spacing
    pub margin: Option<Spacing>,
    pub padding: Option<Spacing>,

    // Visual
    pub background_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_color: Option<Color>,
    pub border_radius: Option<BorderRadius>,
    pub opacity: Option<f32>,
}

/// Style for Flex elements (flex container).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FlexStyle {
    // Display
    pub display: Option<Display>,

    // Dimensions
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_height: Option<Dimension>,

    // Spacing
    pub margin: Option<Spacing>,
    pub padding: Option<Spacing>,

    // Flex container
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub gap: Option<f32>,
    pub flex_wrap: Option<FlexWrap>,

    // Visual
    pub background_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_color: Option<Color>,
    pub border_radius: Option<BorderRadius>,
    pub opacity: Option<f32>,
}

/// Style for Text elements.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TextStyle {
    // Typography
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<FontWeight>,
    pub line_height: Option<f32>,
    pub text_align: Option<TextAlign>,
    pub color: Option<Color>,
    pub letter_spacing: Option<f32>,
    pub text_rendering: Option<TextRendering>,

    // Flex child properties
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
}

/// Style for Image elements.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ImageStyle {
    // Dimensions (overrides intrinsic if set)
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_height: Option<Dimension>,

    // Spacing
    pub margin: Option<Spacing>,

    // Image-specific
    pub object_fit: Option<ObjectFit>,

    // Visual
    pub border_radius: Option<BorderRadius>,
    pub opacity: Option<f32>,

    // Flex child properties
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_element() {
        let json = r###"{
            "type": "flex",
            "style": { "flexDirection": "column", "width": 400, "padding": 20, "backgroundColor": "#fff" },
            "children": [
                {
                    "type": "text",
                    "content": "Hello World",
                    "style": { "fontSize": 24, "color": "#333" }
                }
            ]
        }"###;

        let element: Element = serde_json::from_str(json).unwrap();
        match element {
            Element::Flex { style, children } => {
                assert!(matches!(style.flex_direction, Some(FlexDirection::Column)));
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Flex element"),
        }
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(Color::parse("#fff").unwrap().r, 255);
        assert_eq!(Color::parse("#000000").unwrap().r, 0);
        assert_eq!(Color::parse("rgb(255, 0, 0)").unwrap().r, 255);
        assert_eq!(Color::parse("rgba(0, 0, 255, 0.5)").unwrap().a, 127);
    }

    #[test]
    fn test_spacing_edges() {
        assert_eq!(Spacing::Uniform(10.0).to_edges(), [10.0, 10.0, 10.0, 10.0]);
        assert_eq!(
            Spacing::Multi("10 20".to_string()).to_edges(),
            [10.0, 20.0, 10.0, 20.0]
        );
        assert_eq!(
            Spacing::Multi("10 20 30 40".to_string()).to_edges(),
            [10.0, 20.0, 30.0, 40.0]
        );
    }
}
